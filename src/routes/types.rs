use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    config::Config,
    entities::{certificates, invite_codes, tunnels, users},
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
    pub max_tunnels: Option<i32>,
}

impl From<users::Model> for PublicUser {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
            disabled: user.disabled,
            created_at: user.created_at,
            max_tunnels: user.max_tunnels,
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
    pub remote_port: Option<i32>,
    pub custom_domain: Option<String>,
    pub tls_mode: Option<String>,
    pub certificate_id: Option<Uuid>,
    pub use_encryption: bool,
    pub use_compression: bool,
    pub bandwidth_limit: Option<String>,
    pub bandwidth_limit_mode: Option<String>,
    pub proxy_protocol_version: Option<String>,
    pub locations: Option<String>,
    pub host_header_rewrite: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Serialize)]
pub struct TunnelWithTrafficResponse {
    pub tunnel: TunnelResponse,
    pub traffic_available: bool,
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub runtime_status: String,
    pub current_connections: u64,
    pub matched_proxy_name: Option<String>,
    pub persistent_traffic_available: bool,
    pub persistent_traffic_in: u64,
    pub persistent_traffic_out: u64,
    pub last_sampled_at: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
pub struct TrafficHistoryPointResponse {
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub sampled_at: DateTime<FixedOffset>,
}

#[derive(Serialize)]
pub struct TrafficHistoryResponse {
    pub points: Vec<TrafficHistoryPointResponse>,
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
            custom_domain: tunnel.custom_domain,
            tls_mode: tunnel.tls_mode,
            certificate_id: tunnel.certificate_id,
            use_encryption: tunnel.use_encryption,
            use_compression: tunnel.use_compression,
            bandwidth_limit: tunnel.bandwidth_limit,
            bandwidth_limit_mode: tunnel.bandwidth_limit_mode,
            proxy_protocol_version: tunnel.proxy_protocol_version,
            locations: tunnel.locations,
            host_header_rewrite: tunnel.host_header_rewrite,
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
pub struct CertificateResponse {
    pub id: Uuid,
    pub name: String,
    pub domains: Vec<String>,
    pub not_before: DateTime<FixedOffset>,
    pub not_after: DateTime<FixedOffset>,
    pub fingerprint_sha256: String,
    pub created_at: DateTime<FixedOffset>,
}

impl From<certificates::Model> for CertificateResponse {
    fn from(cert: certificates::Model) -> Self {
        Self {
            id: cert.id,
            name: cert.name,
            domains: crate::services::certificates::domains_from_json(&cert.domains_json),
            not_before: cert.not_before,
            not_after: cert.not_after,
            fingerprint_sha256: cert.fingerprint_sha256,
            created_at: cert.created_at,
        }
    }
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
    pub effective_max_tunnels: u64,
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
pub struct AdminTrafficSummaryResponse {
    pub available: bool,
    pub persistent_available: bool,
    pub total_traffic_in: u64,
    pub total_traffic_out: u64,
    pub persistent_total_traffic_in: u64,
    pub persistent_total_traffic_out: u64,
    pub last_sampled_at: Option<DateTime<FixedOffset>>,
    pub tunnels: Vec<AdminTunnelTrafficResponse>,
}

#[derive(Serialize)]
pub struct AdminTunnelTrafficResponse {
    pub tunnel: TunnelResponse,
    pub username: String,
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub runtime_status: String,
    pub current_connections: u64,
    pub matched_proxy_name: Option<String>,
    pub persistent_traffic_available: bool,
    pub persistent_traffic_in: u64,
    pub persistent_traffic_out: u64,
    pub last_sampled_at: Option<DateTime<FixedOffset>>,
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
    pub dashboard_addr: String,
    pub dashboard_port: Option<u16>,
    pub dashboard_user: String,
    pub dashboard_configured: bool,
    pub dashboard_available: bool,
    pub enable_prometheus: bool,
    pub prometheus_configured: bool,
    pub vhost_http_port: Option<u16>,
    pub vhost_https_port: Option<u16>,
}

#[derive(Serialize)]
pub struct PanelTlsResponse {
    pub enabled: bool,
    pub bind: String,
    pub domain: String,
    pub domains: Vec<String>,
    pub not_after: Option<String>,
    pub fingerprint_sha256: String,
}

#[derive(Serialize)]
pub struct CaddyResponse {
    pub enabled: bool,
    pub domain: String,
    pub config_path: String,
    pub available: bool,
    pub upstream: String,
    pub app_bind: String,
    pub app_bind_local: bool,
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
    pub username: String,
    pub role: String,
    pub disabled: bool,
    pub created_at: DateTime<FixedOffset>,
    pub effective_max_tunnels: u64,
    pub frps_server_addr: String,
    pub frps_bind_port: u16,
    pub remote_port_min: i32,
    pub remote_port_max: i32,
    pub vhost_http_port: Option<u16>,
    pub vhost_https_port: Option<u16>,
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
