use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    config::Config,
    entities::{invite_codes, tunnels, users},
};

#[derive(Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub user: PublicUser,
}

#[derive(Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub disabled: bool,
    pub created_at: DateTime<FixedOffset>,
}

impl From<users::Model> for PublicUser {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
            disabled: user.disabled,
            created_at: user.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct TunnelResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub protocol: String,
    pub local_host: String,
    pub local_port: i32,
    pub remote_port: i32,
    pub created_at: DateTime<FixedOffset>,
}

impl From<tunnels::Model> for TunnelResponse {
    fn from(tunnel: tunnels::Model) -> Self {
        Self {
            id: tunnel.id,
            user_id: tunnel.user_id,
            name: tunnel.name,
            protocol: tunnel.protocol,
            local_host: tunnel.local_host,
            local_port: tunnel.local_port,
            remote_port: tunnel.remote_port,
            created_at: tunnel.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct FrpcResponse {
    pub tunnel: TunnelResponse,
    pub frpc_toml: String,
}

#[derive(Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub used_by: Option<Uuid>,
    pub used_at: Option<DateTime<FixedOffset>>,
    pub expires_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}

impl From<invite_codes::Model> for InviteResponse {
    fn from(invite: invite_codes::Model) -> Self {
        Self {
            id: invite.id,
            code: invite.code,
            created_by: invite.created_by,
            used_by: invite.used_by,
            used_at: invite.used_at,
            expires_at: invite.expires_at,
            created_at: invite.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct UserRowResponse {
    pub user: PublicUser,
    pub tunnel_count: u64,
}

#[derive(Serialize)]
pub struct AdminTunnelResponse {
    pub tunnel: TunnelResponse,
    pub username: String,
}

#[derive(Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Serialize)]
pub struct AdminSummaryResponse {
    pub user_count: u64,
    pub disabled_user_count: u64,
    pub tunnel_count: u64,
    pub invite_count: u64,
    pub unused_invite_count: u64,
    pub used_remote_port_count: u64,
    pub remote_port_capacity: u64,
}

#[derive(Serialize)]
pub struct FrpsStatusResponse {
    pub server_addr: String,
    pub bind_port: u16,
    pub token_set: bool,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
    pub config_path: String,
    pub status: String,
    pub state: String,
    pub version: String,
    pub display_status: String,
    pub restarting: bool,
    pub restart_command_configured: bool,
    pub upgrade_supported: bool,
    pub available_versions: Vec<String>,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub frps_server_addr: String,
    pub frps_bind_port: u16,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
    pub user_max_tunnels: u64,
}

#[derive(Serialize)]
pub struct DashboardSummaryResponse {
    pub tunnel_count: u64,
    pub user_max_tunnels: u64,
    pub remaining_tunnels: u64,
    pub frps_server_addr: String,
    pub frps_bind_port: u16,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
}

impl From<&Config> for ConfigResponse {
    fn from(config: &Config) -> Self {
        Self {
            frps_server_addr: config.frps_server_addr.clone(),
            frps_bind_port: config.frps_bind_port,
            remote_port_min: config.remote_port_min,
            remote_port_max: config.remote_port_max,
            user_max_tunnels: config.user_max_tunnels,
        }
    }
}
