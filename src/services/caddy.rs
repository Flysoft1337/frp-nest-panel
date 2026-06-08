use std::{net::SocketAddr, path::Path, time::Duration};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    error::{AppError, AppResult},
    services::validation,
};

pub const CADDY_PANEL_CONFIG_PATH: &str = "data/caddy/panel.toml";
pub const CADDY_CONFIG_PATH: &str = "/etc/caddy/frp-nest-panel.caddy";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaddyPanelConfig {
    pub enabled: bool,
    pub domain: String,
}

impl Default for CaddyPanelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain: String::new(),
        }
    }
}

pub async fn load_config() -> Result<CaddyPanelConfig> {
    if !Path::new(CADDY_PANEL_CONFIG_PATH).exists() {
        return Ok(CaddyPanelConfig::default());
    }
    let content = tokio::fs::read_to_string(CADDY_PANEL_CONFIG_PATH)
        .await
        .with_context(|| format!("failed to read {CADDY_PANEL_CONFIG_PATH}"))?;
    Ok(toml::from_str(&content)
        .with_context(|| format!("failed to parse {CADDY_PANEL_CONFIG_PATH}"))?)
}

pub async fn save_config(
    enabled: bool,
    domain: String,
    upstream: SocketAddr,
) -> AppResult<CaddyPanelConfig> {
    let domain = domain.trim();
    let domain = if domain.is_empty() {
        String::new()
    } else {
        validation::domain(domain)?
    };
    if enabled && domain.is_empty() {
        return Err(AppError::BadRequest(
            "启用 Caddy 前请填写面板访问域名".to_owned(),
        ));
    }

    let config = CaddyPanelConfig { enabled, domain };
    apply_config(&config, upstream).await?;
    write_panel_config(&config).await?;
    Ok(config)
}

pub async fn reload_config(upstream: SocketAddr) -> AppResult<CaddyPanelConfig> {
    let config = load_config().await.unwrap_or_default();
    apply_config(&config, upstream).await?;
    Ok(config)
}

pub async fn caddy_available() -> bool {
    tokio::time::timeout(
        Duration::from_secs(3),
        Command::new("caddy").arg("version").status(),
    )
    .await
    .ok()
    .and_then(|result| result.ok())
    .map(|status| status.success())
    .unwrap_or(false)
}

pub fn upstream_from_bind(bind: SocketAddr) -> String {
    let host = if bind.ip().is_unspecified() {
        "127.0.0.1".to_owned()
    } else {
        bind.ip().to_string()
    };
    format!("{host}:{}", bind.port())
}

fn render_caddyfile(config: &CaddyPanelConfig, upstream: SocketAddr) -> String {
    let fallback =
        ":80 {\n    respond \"not found\" 404\n}\n\n:443 {\n    respond \"not found\" 404\n}\n";
    if !config.enabled {
        return fallback.to_owned();
    }
    format!(
        "https://{} {{\n    reverse_proxy {}\n}}\n\nhttp://{} {{\n    respond \"not found\" 404\n}}\n\n{}",
        config.domain,
        upstream_from_bind(upstream),
        config.domain,
        fallback
    )
}

async fn write_panel_config(config: &CaddyPanelConfig) -> AppResult<()> {
    if let Some(parent) = Path::new(CADDY_PANEL_CONFIG_PATH).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| AppError::BadRequest(format!("创建 Caddy 配置目录失败: {error}")))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|error| AppError::BadRequest(format!("序列化 Caddy 配置失败: {error}")))?;
    tokio::fs::write(CADDY_PANEL_CONFIG_PATH, content)
        .await
        .map_err(|error| AppError::BadRequest(format!("保存 Caddy 配置失败: {error}")))?;
    Ok(())
}

async fn apply_config(config: &CaddyPanelConfig, upstream: SocketAddr) -> AppResult<()> {
    let path = Path::new(CADDY_CONFIG_PATH);
    let parent = path
        .parent()
        .ok_or_else(|| AppError::BadRequest("Caddy 配置路径不合法".to_owned()))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| AppError::BadRequest(format!("创建 Caddy 系统配置目录失败: {error}")))?;

    let temp_path = parent.join(format!(".frp-nest-panel.caddy.tmp.{}", std::process::id()));
    tokio::fs::write(&temp_path, render_caddyfile(config, upstream))
        .await
        .map_err(|error| AppError::BadRequest(format!("写入 Caddy 临时配置失败: {error}")))?;

    if let Err(error) = validate_caddyfile(&temp_path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(error);
    }

    tokio::fs::rename(&temp_path, path)
        .await
        .map_err(|error| AppError::BadRequest(format!("替换 Caddy 配置失败: {error}")))?;
    reload_caddy().await
}

async fn validate_caddyfile(path: &Path) -> AppResult<()> {
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        Command::new("caddy")
            .arg("validate")
            .arg("--config")
            .arg(path)
            .output(),
    )
    .await
    .map_err(|_| AppError::BadRequest("Caddy 配置校验超时".to_owned()))?
    .map_err(|error| AppError::BadRequest(format!("执行 caddy validate 失败: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(AppError::BadRequest(format!(
            "Caddy 配置校验失败: {stderr}"
        )));
    }
    Ok(())
}

async fn reload_caddy() -> AppResult<()> {
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        Command::new("systemctl")
            .arg("reload")
            .arg("caddy")
            .output(),
    )
    .await
    .map_err(|_| AppError::BadRequest("Caddy reload 超时".to_owned()))?
    .map_err(|error| AppError::BadRequest(format!("执行 systemctl reload caddy 失败: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(AppError::BadRequest(format!("Caddy reload 失败: {stderr}")));
    }
    Ok(())
}
