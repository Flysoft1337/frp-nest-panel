use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{net::TcpStream, process::Command};

use crate::config::Config;

pub const FRPS_CONFIG_PATH: &str = "frps/frps.toml";
pub const FRPS_PANEL_CONFIG_PATH: &str = "frps/panel.toml";
pub const FRPS_IMAGE: &str = "snowdreamtech/frps:0.62.1";

#[derive(Clone, Debug)]
pub struct FrpsRuntimeConfig {
    pub server_addr: String,
    pub bind_port: u16,
    pub auth_token: String,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
    pub dashboard_addr: String,
    pub dashboard_port: Option<u16>,
    pub dashboard_user: String,
    pub dashboard_password: String,
}

pub struct FrpsRuntimeStatus {
    pub state: String,
    pub version: String,
    pub display_status: String,
}

pub struct FrpsTrafficSnapshot {
    pub available: bool,
    pub proxies: Vec<FrpsProxyTraffic>,
}

pub struct FrpsProxyTraffic {
    pub name: String,
    pub protocol: String,
    pub traffic_in: u64,
    pub traffic_out: u64,
}

#[derive(Deserialize)]
struct FrpsProxyList {
    proxies: Vec<Value>,
}

#[derive(Serialize)]
struct FrpsPanelToml<'a> {
    #[serde(rename = "serverAddr")]
    server_addr: &'a str,
    #[serde(rename = "remotePortMin")]
    remote_port_min: i32,
    #[serde(rename = "remotePortMax")]
    remote_port_max: i32,
    #[serde(rename = "dashboardAddr")]
    dashboard_addr: &'a str,
    #[serde(rename = "dashboardPort")]
    dashboard_port: Option<u16>,
    #[serde(rename = "dashboardUser")]
    dashboard_user: &'a str,
    #[serde(rename = "dashboardPassword")]
    dashboard_password: &'a str,
}

#[derive(Serialize)]
struct FrpsToml<'a> {
    #[serde(rename = "bindPort")]
    bind_port: u16,
    auth: FrpsTomlAuth<'a>,
    #[serde(rename = "webServer", skip_serializing_if = "Option::is_none")]
    web_server: Option<FrpsTomlWebServer<'a>>,
}

#[derive(Serialize)]
struct FrpsTomlAuth<'a> {
    method: &'a str,
    token: &'a str,
}

#[derive(Serialize)]
struct FrpsTomlWebServer<'a> {
    addr: &'a str,
    port: u16,
    user: &'a str,
    password: &'a str,
}

pub async fn load_runtime_config(config: &Config) -> Result<FrpsRuntimeConfig> {
    let mut runtime = FrpsRuntimeConfig {
        server_addr: config.frps_server_addr.clone(),
        bind_port: config.frps_bind_port,
        auth_token: config.frps_auth_token.clone(),
        remote_port_min: config.remote_port_min,
        remote_port_max: config.remote_port_max,
        dashboard_addr: "127.0.0.1".to_owned(),
        dashboard_port: None,
        dashboard_user: String::new(),
        dashboard_password: String::new(),
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
        if let Some(web_server) = value.get("webServer") {
            if let Some(addr) = web_server.get("addr").and_then(toml::Value::as_str) {
                runtime.dashboard_addr = addr.to_owned();
            }
            if let Some(port) = web_server.get("port").and_then(toml::Value::as_integer) {
                runtime.dashboard_port =
                    Some(u16::try_from(port).context("webServer port is invalid")?);
            }
            if let Some(user) = web_server.get("user").and_then(toml::Value::as_str) {
                runtime.dashboard_user = user.to_owned();
            }
            if let Some(password) = web_server.get("password").and_then(toml::Value::as_str) {
                runtime.dashboard_password = password.to_owned();
            }
        }
    } else {
        write_frps_config(&runtime).await?;
    }

    if Path::new(FRPS_PANEL_CONFIG_PATH).exists() {
        let content = tokio::fs::read_to_string(FRPS_PANEL_CONFIG_PATH)
            .await
            .with_context(|| format!("failed to read {FRPS_PANEL_CONFIG_PATH}"))?;
        let value: toml::Value = toml::from_str(&content)
            .with_context(|| format!("failed to parse {FRPS_PANEL_CONFIG_PATH}"))?;

        if let Some(server_addr) = value.get("serverAddr").and_then(toml::Value::as_str) {
            runtime.server_addr = server_addr.to_owned();
        }
        if let Some(remote_port_min) = value.get("remotePortMin").and_then(toml::Value::as_integer)
        {
            runtime.remote_port_min =
                i32::try_from(remote_port_min).context("remotePortMin is invalid")?;
        }
        if let Some(remote_port_max) = value.get("remotePortMax").and_then(toml::Value::as_integer)
        {
            runtime.remote_port_max =
                i32::try_from(remote_port_max).context("remotePortMax is invalid")?;
        }
        if let Some(dashboard_addr) = value.get("dashboardAddr").and_then(toml::Value::as_str) {
            runtime.dashboard_addr = dashboard_addr.to_owned();
        }
        if let Some(dashboard_port) = value.get("dashboardPort").and_then(toml::Value::as_integer) {
            runtime.dashboard_port =
                Some(u16::try_from(dashboard_port).context("dashboardPort is invalid")?);
        }
        if let Some(dashboard_user) = value.get("dashboardUser").and_then(toml::Value::as_str) {
            runtime.dashboard_user = dashboard_user.to_owned();
        }
        if let Some(dashboard_password) =
            value.get("dashboardPassword").and_then(toml::Value::as_str)
        {
            runtime.dashboard_password = dashboard_password.to_owned();
        }
    } else {
        write_panel_config(&runtime).await?;
    }

    Ok(runtime)
}

pub async fn write_runtime_config(config: &FrpsRuntimeConfig) -> Result<()> {
    write_frps_config(config).await?;
    write_panel_config(config).await?;
    Ok(())
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
        web_server: config.dashboard_port.map(|port| FrpsTomlWebServer {
            addr: &config.dashboard_addr,
            port,
            user: &config.dashboard_user,
            password: &config.dashboard_password,
        }),
    })?;
    tokio::fs::write(FRPS_CONFIG_PATH, content)
        .await
        .with_context(|| format!("failed to write {FRPS_CONFIG_PATH}"))?;
    Ok(())
}

pub async fn write_panel_config(config: &FrpsRuntimeConfig) -> Result<()> {
    if let Some(parent) = Path::new(FRPS_PANEL_CONFIG_PATH).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(&FrpsPanelToml {
        server_addr: &config.server_addr,
        remote_port_min: config.remote_port_min,
        remote_port_max: config.remote_port_max,
        dashboard_addr: &config.dashboard_addr,
        dashboard_port: config.dashboard_port,
        dashboard_user: &config.dashboard_user,
        dashboard_password: &config.dashboard_password,
    })?;
    tokio::fs::write(FRPS_PANEL_CONFIG_PATH, content)
        .await
        .with_context(|| format!("failed to write {FRPS_PANEL_CONFIG_PATH}"))?;
    Ok(())
}

pub async fn dashboard_available(config: &FrpsRuntimeConfig) -> bool {
    get_dashboard(config, "/api/serverinfo")
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

pub async fn traffic_snapshot(config: &FrpsRuntimeConfig) -> FrpsTrafficSnapshot {
    if config.dashboard_port.is_none() {
        return FrpsTrafficSnapshot {
            available: false,
            proxies: Vec::new(),
        };
    }

    let mut proxies = Vec::new();
    for protocol in ["tcp", "udp"] {
        let Ok(response) = get_dashboard(config, &format!("/api/proxy/{protocol}")).await else {
            return FrpsTrafficSnapshot {
                available: false,
                proxies: Vec::new(),
            };
        };
        if !response.status().is_success() {
            return FrpsTrafficSnapshot {
                available: false,
                proxies: Vec::new(),
            };
        }
        let Ok(list) = response.json::<FrpsProxyList>().await else {
            return FrpsTrafficSnapshot {
                available: false,
                proxies: Vec::new(),
            };
        };
        proxies.extend(list.proxies.into_iter().map(|proxy| FrpsProxyTraffic {
            name: proxy_string(&proxy, &["name", "proxyName"]),
            protocol: protocol.to_owned(),
            traffic_in: proxy_u64(&proxy, &["trafficIn", "todayTrafficIn", "curConns"]),
            traffic_out: proxy_u64(&proxy, &["trafficOut", "todayTrafficOut"]),
        }));
    }

    FrpsTrafficSnapshot {
        available: true,
        proxies,
    }
}

async fn get_dashboard(config: &FrpsRuntimeConfig, path: &str) -> Result<reqwest::Response> {
    let Some(port) = config.dashboard_port else {
        anyhow::bail!("frps dashboard is not configured");
    };
    let url = format!("http://{}:{port}{path}", config.dashboard_addr);
    let client = reqwest::Client::new();
    let mut request = client.get(url).timeout(Duration::from_secs(2));
    if !config.dashboard_user.is_empty() {
        request = request.basic_auth(&config.dashboard_user, Some(&config.dashboard_password));
    }
    Ok(request.send().await?)
}

fn proxy_string(proxy: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| proxy.get(*key).and_then(Value::as_str))
        .unwrap_or("")
        .to_owned()
}

fn proxy_u64(proxy: &Value, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|key| proxy.get(*key).and_then(Value::as_u64))
        .unwrap_or(0)
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
    let display_status = state.clone();

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
