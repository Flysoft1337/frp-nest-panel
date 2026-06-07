use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Config {
    pub database_url: String,
    pub app_bind: SocketAddr,
    pub session_secret: String,
    pub initial_admin_username: String,
    pub initial_admin_password: String,
    pub frps_server_addr: String,
    pub frps_bind_port: u16,
    pub frps_auth_token: String,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
    pub user_max_tunnels: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let app_bind = env_or("APP_BIND", "0.0.0.0:8080")
            .parse()
            .context("APP_BIND must be a socket address")?;

        let remote_port_min = env_or("FRP_REMOTE_PORT_MIN", "20000")
            .parse()
            .context("FRP_REMOTE_PORT_MIN must be a number")?;
        let remote_port_max = env_or("FRP_REMOTE_PORT_MAX", "30000")
            .parse()
            .context("FRP_REMOTE_PORT_MAX must be a number")?;

        anyhow::ensure!(remote_port_min > 0, "FRP_REMOTE_PORT_MIN must be positive");
        anyhow::ensure!(
            remote_port_max <= 65535,
            "FRP_REMOTE_PORT_MAX must not exceed 65535"
        );
        anyhow::ensure!(
            remote_port_min <= remote_port_max,
            "remote port range is invalid"
        );

        let session_secret = env_required("SESSION_SECRET")?;
        anyhow::ensure!(
            session_secret.len() >= 32,
            "SESSION_SECRET must be at least 32 characters"
        );
        let initial_admin_password = env_required("INITIAL_ADMIN_PASSWORD")?;
        anyhow::ensure!(
            initial_admin_password.len() >= 8,
            "INITIAL_ADMIN_PASSWORD must be at least 8 characters"
        );
        let frps_bind_port = env_or("FRPS_BIND_PORT", "7000")
            .parse()
            .context("FRPS_BIND_PORT must be a number")?;
        let user_max_tunnels = env_or("USER_MAX_TUNNELS", "3")
            .parse()
            .context("USER_MAX_TUNNELS must be a number")?;
        anyhow::ensure!(user_max_tunnels > 0, "USER_MAX_TUNNELS must be positive");

        Ok(Self {
            database_url: env_required("DATABASE_URL")?,
            app_bind,
            session_secret,
            initial_admin_username: env_or("INITIAL_ADMIN_USERNAME", "admin"),
            initial_admin_password,
            frps_server_addr: env_or("FRPS_SERVER_ADDR", "127.0.0.1"),
            frps_bind_port,
            frps_auth_token: env_required("FRPS_AUTH_TOKEN")?,
            remote_port_min,
            remote_port_max,
            user_max_tunnels,
        })
    }
}

fn env_required(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("{key} is required"))
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}
