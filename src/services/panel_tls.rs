use std::{net::SocketAddr, path::Path};

use tokio::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    services::{certificates, validation},
};

pub const PANEL_TLS_CONFIG_PATH: &str = "data/panel_tls/config.toml";
pub const PANEL_TLS_CERT_PATH: &str = "data/panel_tls/cert.pem";
pub const PANEL_TLS_KEY_PATH: &str = "data/panel_tls/key.pem";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PanelTlsConfig {
    pub enabled: bool,
    pub bind: String,
    #[serde(default)]
    pub domain: String,
    pub domains: Vec<String>,
    pub not_after: Option<String>,
    pub fingerprint_sha256: String,
}

impl Default for PanelTlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "0.0.0.0:8443".to_owned(),
            domain: String::new(),
            domains: Vec::new(),
            not_after: None,
            fingerprint_sha256: String::new(),
        }
    }
}

pub async fn load_config() -> Result<PanelTlsConfig> {
    if !Path::new(PANEL_TLS_CONFIG_PATH).exists() {
        return Ok(PanelTlsConfig::default());
    }
    let content = tokio::fs::read_to_string(PANEL_TLS_CONFIG_PATH)
        .await
        .with_context(|| format!("failed to read {PANEL_TLS_CONFIG_PATH}"))?;
    Ok(toml::from_str(&content)
        .with_context(|| format!("failed to parse {PANEL_TLS_CONFIG_PATH}"))?)
}

pub async fn save_config(
    enabled: bool,
    bind: String,
    domain: String,
    certificate_pem: Option<String>,
    private_key_pem: Option<String>,
) -> AppResult<PanelTlsConfig> {
    bind.parse::<SocketAddr>().map_err(|_| {
        AppError::BadRequest("HTTPS 监听地址必须是 IP:端口，例如 0.0.0.0:443".to_owned())
    })?;
    let domain = domain.trim();
    let domain = if domain.is_empty() {
        String::new()
    } else {
        validation::domain(domain)?
    };
    let mut current = load_config().await.unwrap_or_default();
    current.enabled = enabled;
    current.bind = bind;
    current.domain = domain;

    if let (Some(cert), Some(key)) = (certificate_pem, private_key_pem) {
        let parsed = certificates::parse_and_validate(&cert, &key)?;
        if let Some(parent) = Path::new(PANEL_TLS_CERT_PATH).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| AppError::BadRequest(format!("创建 TLS 目录失败: {error}")))?;
        }
        tokio::fs::write(PANEL_TLS_CERT_PATH, cert)
            .await
            .map_err(|error| AppError::BadRequest(format!("保存面板证书失败: {error}")))?;
        tokio::fs::write(PANEL_TLS_KEY_PATH, key)
            .await
            .map_err(|error| AppError::BadRequest(format!("保存面板私钥失败: {error}")))?;
        current.domains = parsed.domains;
        current.not_after = Some(parsed.not_after.to_rfc3339());
        current.fingerprint_sha256 = parsed.fingerprint_sha256;
    }

    if current.enabled {
        if current.domain.is_empty() {
            return Err(AppError::BadRequest(
                "启用 HTTPS 前请填写面板域名".to_owned(),
            ));
        }
        if !certificates::certificate_covers_domain(&current.domains, &current.domain) {
            return Err(AppError::BadRequest("证书未覆盖面板域名".to_owned()));
        }
        if !Path::new(PANEL_TLS_CERT_PATH).exists() || !Path::new(PANEL_TLS_KEY_PATH).exists() {
            return Err(AppError::BadRequest(
                "启用 HTTPS 前请先上传证书和私钥".to_owned(),
            ));
        }
    }

    if let Some(parent) = Path::new(PANEL_TLS_CONFIG_PATH).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| AppError::BadRequest(format!("创建 TLS 配置目录失败: {error}")))?;
    }
    let content = toml::to_string_pretty(&current)
        .map_err(|error| AppError::BadRequest(format!("序列化 TLS 配置失败: {error}")))?;
    tokio::fs::write(PANEL_TLS_CONFIG_PATH, content)
        .await
        .map_err(|error| AppError::BadRequest(format!("保存 TLS 配置失败: {error}")))?;
    Ok(current)
}

pub async fn restart_panel() -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg("nohup sh -c 'sleep 1; systemctl restart frp-nest-panel.service' >/dev/null 2>&1 &")
        .status()
        .await
        .context("failed to schedule frp-nest-panel.service restart")?;
    if !status.success() {
        anyhow::bail!("frp-nest-panel.service restart schedule failed");
    }
    Ok(())
}
