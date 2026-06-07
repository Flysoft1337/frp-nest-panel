use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::{net::TcpStream, process::Command};

use crate::config::Config;

pub const FRPS_CONFIG_PATH: &str = "frps/frps.toml";
pub const FRPS_IMAGE: &str = "snowdreamtech/frps:0.62.1";

#[derive(Clone, Debug)]
pub struct FrpsRuntimeConfig {
    pub server_addr: String,
    pub bind_port: u16,
    pub auth_token: String,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
}

pub struct FrpsRuntimeStatus {
    pub state: String,
    pub version: String,
    pub display_status: String,
}

#[derive(Serialize)]
struct FrpsToml<'a> {
    #[serde(rename = "bindPort")]
    bind_port: u16,
    auth: FrpsTomlAuth<'a>,
}

#[derive(Serialize)]
struct FrpsTomlAuth<'a> {
    method: &'a str,
    token: &'a str,
}

pub async fn load_runtime_config(config: &Config) -> Result<FrpsRuntimeConfig> {
    let mut runtime = FrpsRuntimeConfig {
        server_addr: config.frps_server_addr.clone(),
        bind_port: config.frps_bind_port,
        auth_token: config.frps_auth_token.clone(),
        remote_port_min: config.remote_port_min,
        remote_port_max: config.remote_port_max,
    };

    if Path::new(FRPS_CONFIG_PATH).exists() {
        let content = tokio::fs::read_to_string(FRPS_CONFIG_PATH)
            .await
            .with_context(|| format!("failed to read {FRPS_CONFIG_PATH}"))?;
        let value: toml::Value = toml::from_str(&content)
            .with_context(|| format!("failed to parse {FRPS_CONFIG_PATH}"))?;

        if let Some(bind_port) = value.get("bindPort").and_then(toml::Value::as_integer) {
            runtime.bind_port = u16::try_from(bind_port).context("frps bindPort is invalid")?;
        }
        if let Some(token) = value
            .get("auth")
            .and_then(|auth| auth.get("token"))
            .and_then(toml::Value::as_str)
        {
            runtime.auth_token = token.to_owned();
        }
    } else {
        write_frps_config(&runtime).await?;
    }

    Ok(runtime)
}

pub async fn write_frps_config(config: &FrpsRuntimeConfig) -> Result<()> {
    if let Some(parent) = Path::new(FRPS_CONFIG_PATH).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(&FrpsToml {
        bind_port: config.bind_port,
        auth: FrpsTomlAuth {
            method: "token",
            token: &config.auth_token,
        },
    })?;
    tokio::fs::write(FRPS_CONFIG_PATH, content)
        .await
        .with_context(|| format!("failed to write {FRPS_CONFIG_PATH}"))?;
    Ok(())
}

pub async fn runtime_status(config: &FrpsRuntimeConfig, restarting: bool) -> FrpsRuntimeStatus {
    let version = frps_version();
    let state = if restarting {
        "restarting".to_owned()
    } else {
        match tokio::time::timeout(
            Duration::from_millis(800),
            TcpStream::connect(("127.0.0.1", config.bind_port)),
        )
        .await
        {
            Ok(Ok(_)) => "running".to_owned(),
            Ok(Err(_)) => "stopped".to_owned(),
            Err(_) => "unknown".to_owned(),
        }
    };
    let display_status = format!("{state}-{version}");

    FrpsRuntimeStatus {
        state,
        version,
        display_status,
    }
}

fn frps_version() -> String {
    let tag = FRPS_IMAGE
        .rsplit_once(':')
        .map(|(_, tag)| tag)
        .unwrap_or("unknown");
    if tag.starts_with('v') || tag == "unknown" {
        tag.to_owned()
    } else {
        format!("v{tag}")
    }
}

pub async fn restart_frps() -> Result<()> {
    let output = tokio::time::timeout(
        Duration::from_secs(30),
        Command::new("docker")
            .args(["compose", "up", "-d", "frps"])
            .current_dir(".")
            .output(),
    )
    .await
    .context("frps restart timed out")?
    .context("failed to run docker compose")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("docker compose failed: {stderr}");
    }

    Ok(())
}
