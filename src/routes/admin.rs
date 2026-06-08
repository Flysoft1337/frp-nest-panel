use std::{collections::HashMap, sync::atomic::Ordering};

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    entities::{invite_codes, tunnels, users},
    error::{AppError, AppResult},
    routes::types::{
        AdminSummaryResponse, AdminTrafficSummaryResponse, AdminTunnelResponse,
        AdminTunnelTrafficResponse, CaddyResponse, ConfigResponse, FrpsStatusResponse,
        InviteResponse, OkResponse, PageResponse, PanelTlsResponse, PublicUser, TunnelResponse,
        UserRowResponse,
    },
    services::{caddy, frps, invite, password, validation},
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateInvitesForm {
    count: u32,
    expires_days: Option<i64>,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    new_password: String,
}

#[derive(Deserialize)]
pub struct UserQuotaForm {
    max_tunnels: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    q: Option<String>,
    status: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct PanelTlsForm {
    enabled: bool,
    bind: String,
    domain: String,
    certificate_pem: Option<String>,
    private_key_pem: Option<String>,
}

#[derive(Deserialize)]
pub struct CaddyForm {
    enabled: bool,
    domain: String,
}

#[derive(Deserialize)]
pub struct FrpsUpdateForm {
    server_addr: String,
    bind_port: u16,
    auth_token: String,
    remote_port_min: i32,
    remote_port_max: i32,
    dashboard_addr: String,
    dashboard_port: Option<u16>,
    dashboard_user: String,
    dashboard_password: String,
    vhost_http_port: Option<u16>,
    vhost_https_port: Option<u16>,
}

pub async fn config(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let frps = state.frps.read().await;
    Ok(Json(ConfigResponse {
        frps_server_addr: frps.server_addr.clone(),
        frps_bind_port: frps.bind_port,
        remote_port_min: frps.remote_port_min,
        remote_port_max: frps.remote_port_max,
        user_max_tunnels: state.config.user_max_tunnels,
    }))
}

pub async fn summary(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let user_count = users::Entity::find().count(&state.db).await?;
    let disabled_user_count = users::Entity::find()
        .filter(users::Column::Disabled.eq(true))
        .count(&state.db)
        .await?;
    let tunnel_count = tunnels::Entity::find().count(&state.db).await?;
    let invite_count = invite_codes::Entity::find().count(&state.db).await?;
    let unused_invite_count = invite_codes::Entity::find()
        .filter(invite_codes::Column::UsedBy.is_null())
        .count(&state.db)
        .await?;
    let frps = state.frps.read().await;
    let remote_port_capacity = (frps.remote_port_max - frps.remote_port_min + 1).max(0) as u64;

    Ok(Json(AdminSummaryResponse {
        user_count,
        disabled_user_count,
        tunnel_count,
        invite_count,
        unused_invite_count,
        used_remote_port_count: tunnel_count,
        remote_port_capacity,
    }))
}

pub async fn invites(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    let q = normalized_query(&query.q);
    let now = Utc::now().fixed_offset();
    let items = invite_codes::Entity::find()
        .order_by_desc(invite_codes::Column::CreatedAt)
        .all(&state.db)
        .await?
        .into_iter()
        .filter(|item| {
            q.as_ref()
                .map(|q| item.code.to_ascii_lowercase().contains(q))
                .unwrap_or(true)
        })
        .filter(|item| match query.status.as_deref() {
            Some("unused") => {
                item.used_by.is_none() && item.expires_at.map(|time| time > now).unwrap_or(true)
            }
            Some("used") => item.used_by.is_some(),
            Some("expired") => {
                item.used_by.is_none() && item.expires_at.map(|time| time <= now).unwrap_or(false)
            }
            Some(_) => false,
            None => true,
        })
        .map(InviteResponse::from)
        .collect::<Vec<_>>();
    Ok(Json(page_response(items, &query)))
}

pub async fn create_invite(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Json(form): Json<CreateInvitesForm>,
) -> AppResult<impl IntoResponse> {
    if !(1..=100).contains(&form.count) {
        return Err(AppError::BadRequest(
            "一次只能生成 1-100 个邀请码".to_owned(),
        ));
    }
    let expires_at = match form.expires_days {
        Some(days) if days > 0 => Some((Utc::now() + Duration::days(days)).fixed_offset()),
        _ => None,
    };

    let mut created = Vec::with_capacity(form.count as usize);
    for _ in 0..form.count {
        let invite = invite_codes::ActiveModel {
            id: Set(Uuid::new_v4()),
            code: Set(invite::generate_invite_code()),
            created_by: Set(user.id),
            used_by: Set(None),
            used_at: Set(None),
            expires_at: Set(expires_at),
            created_at: Set(Utc::now().fixed_offset()),
        }
        .insert(&state.db)
        .await?;
        created.push(InviteResponse::from(invite));
    }
    Ok(Json(created))
}

pub async fn delete_invite(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let Some(invite) = invite_codes::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    if invite.used_by.is_some() {
        return Err(AppError::BadRequest("已使用的邀请码不能作废".to_owned()));
    }
    invite_codes::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn users(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    let q = normalized_query(&query.q);
    let users = users::Entity::find()
        .order_by_asc(users::Column::CreatedAt)
        .all(&state.db)
        .await?
        .into_iter()
        .filter(|item| {
            q.as_ref()
                .map(|q| item.username.to_ascii_lowercase().contains(q))
                .unwrap_or(true)
        })
        .filter(|item| match query.status.as_deref() {
            Some("enabled") => !item.disabled,
            Some("disabled") => item.disabled,
            Some(_) => false,
            None => true,
        })
        .collect::<Vec<_>>();

    let mut user_rows = Vec::with_capacity(users.len());
    for item in users {
        let tunnel_count = tunnels::Entity::find()
            .filter(tunnels::Column::UserId.eq(item.id))
            .count(&state.db)
            .await?;
        let effective_max_tunnels = effective_max_tunnels(&item, state.config.user_max_tunnels);
        user_rows.push(UserRowResponse {
            user: PublicUser::from(item),
            tunnel_count,
            effective_max_tunnels,
        });
    }
    Ok(Json(page_response(user_rows, &query)))
}

pub async fn disable_user(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    if id == admin.id {
        return Err(AppError::BadRequest("不能禁用自己".to_owned()));
    }
    set_user_disabled(&state, id, true).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn enable_user(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    set_user_disabled(&state, id, false).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn reset_user_password(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
    Json(form): Json<ResetPasswordForm>,
) -> AppResult<impl IntoResponse> {
    validation::password(&form.new_password)?;
    let Some(user) = users::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(password::hash_password(&form.new_password)?);
    active.update(&state.db).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn update_user_quota(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
    Json(form): Json<UserQuotaForm>,
) -> AppResult<impl IntoResponse> {
    if let Some(max_tunnels) = form.max_tunnels {
        if max_tunnels <= 0 {
            return Err(AppError::BadRequest("用户隧道配额必须大于 0".to_owned()));
        }
        let tunnel_count = tunnels::Entity::find()
            .filter(tunnels::Column::UserId.eq(id))
            .count(&state.db)
            .await?;
        if (max_tunnels as u64) < tunnel_count {
            return Err(AppError::BadRequest(
                "用户隧道配额不能小于当前隧道数".to_owned(),
            ));
        }
    }

    let Some(user) = users::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    let mut active: users::ActiveModel = user.into();
    active.max_tunnels = Set(form.max_tunnels);
    active.update(&state.db).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn delete_tunnel(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    tunnels::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn all_tunnels(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    let users = users::Entity::find().all(&state.db).await?;
    let usernames = users
        .into_iter()
        .map(|user| (user.id, user.username))
        .collect::<HashMap<_, _>>();
    let q = normalized_query(&query.q);
    let items = tunnels::Entity::find()
        .order_by_asc(tunnels::Column::RemotePort)
        .all(&state.db)
        .await?
        .into_iter()
        .filter(|item| match query.status.as_deref() {
            Some("tcp") => item.protocol == "tcp",
            Some("udp") => item.protocol == "udp",
            Some("http") => item.protocol == "http",
            Some("https") => item.protocol == "https",
            Some(_) => false,
            None => true,
        })
        .filter(|item| {
            q.as_ref()
                .map(|q| {
                    let username = usernames
                        .get(&item.user_id)
                        .map(String::as_str)
                        .unwrap_or("");
                    item.name.to_ascii_lowercase().contains(q)
                        || item.local_host.to_ascii_lowercase().contains(q)
                        || item
                            .remote_port
                            .map(|port| port.to_string().contains(q))
                            .unwrap_or(false)
                        || item
                            .custom_domain
                            .as_deref()
                            .map(|domain| domain.contains(q))
                            .unwrap_or(false)
                        || username.to_ascii_lowercase().contains(q)
                })
                .unwrap_or(true)
        })
        .map(|item| AdminTunnelResponse {
            username: usernames
                .get(&item.user_id)
                .cloned()
                .unwrap_or_else(|| "未知用户".to_owned()),
            tunnel: TunnelResponse::from(item),
        })
        .collect::<Vec<_>>();
    Ok(Json(page_response(items, &query)))
}

pub async fn traffic_summary(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let frps = state.frps.read().await.clone();
    let snapshot = frps::traffic_snapshot(&frps).await;
    if !snapshot.available {
        return Ok(Json(AdminTrafficSummaryResponse {
            available: false,
            total_traffic_in: 0,
            total_traffic_out: 0,
            tunnels: Vec::new(),
        }));
    }

    let users = users::Entity::find().all(&state.db).await?;
    let usernames = users
        .into_iter()
        .map(|user| (user.id, user.username))
        .collect::<HashMap<_, _>>();
    let traffic = snapshot
        .proxies
        .into_iter()
        .map(|proxy| {
            (
                (proxy.protocol, proxy.name),
                (proxy.traffic_in, proxy.traffic_out),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut total_traffic_in = 0;
    let mut total_traffic_out = 0;
    let mut rows = Vec::new();
    for tunnel in tunnels::Entity::find().all(&state.db).await? {
        let key = (tunnel.protocol.clone(), tunnel.name.clone());
        let (traffic_in, traffic_out) = traffic.get(&key).copied().unwrap_or((0, 0));
        total_traffic_in += traffic_in;
        total_traffic_out += traffic_out;
        rows.push(AdminTunnelTrafficResponse {
            username: usernames
                .get(&tunnel.user_id)
                .cloned()
                .unwrap_or_else(|| "未知用户".to_owned()),
            tunnel: TunnelResponse::from(tunnel),
            traffic_in,
            traffic_out,
        });
    }

    Ok(Json(AdminTrafficSummaryResponse {
        available: true,
        total_traffic_in,
        total_traffic_out,
        tunnels: rows,
    }))
}

pub async fn panel_tls_status(AdminUser(_user): AdminUser) -> AppResult<impl IntoResponse> {
    let config = crate::services::panel_tls::load_config()
        .await
        .unwrap_or_default();
    Ok(Json(PanelTlsResponse {
        enabled: config.enabled,
        bind: config.bind,
        domain: config.domain,
        domains: config.domains,
        not_after: config.not_after,
        fingerprint_sha256: config.fingerprint_sha256,
    }))
}

pub async fn update_panel_tls(
    AdminUser(_user): AdminUser,
    Json(form): Json<PanelTlsForm>,
) -> AppResult<impl IntoResponse> {
    let config = crate::services::panel_tls::save_config(
        form.enabled,
        form.bind,
        form.domain,
        form.certificate_pem
            .filter(|value| !value.trim().is_empty()),
        form.private_key_pem
            .filter(|value| !value.trim().is_empty()),
    )
    .await?;
    Ok(Json(PanelTlsResponse {
        enabled: config.enabled,
        bind: config.bind,
        domain: config.domain,
        domains: config.domains,
        not_after: config.not_after,
        fingerprint_sha256: config.fingerprint_sha256,
    }))
}

pub async fn restart_panel(AdminUser(_user): AdminUser) -> AppResult<impl IntoResponse> {
    tokio::spawn(async move {
        if let Err(error) = crate::services::panel_tls::restart_panel().await {
            tracing::error!(%error, "failed to restart panel");
        }
    });
    Ok(Json(OkResponse { ok: true }))
}

pub async fn caddy_status(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let config = caddy::load_config().await.unwrap_or_default();
    Ok(Json(caddy_response(config, state.config.app_bind).await))
}

pub async fn update_caddy(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
    Json(form): Json<CaddyForm>,
) -> AppResult<impl IntoResponse> {
    let config = caddy::save_config(form.enabled, form.domain, state.config.app_bind).await?;
    Ok(Json(caddy_response(config, state.config.app_bind).await))
}

pub async fn reload_caddy(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let config = caddy::reload_config(state.config.app_bind).await?;
    Ok(Json(caddy_response(config, state.config.app_bind).await))
}

pub async fn frps_status(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let frps = state.frps.read().await.clone();
    let restarting = state.frps_restarting.load(Ordering::SeqCst);
    let runtime_status = frps::runtime_status(&frps, restarting).await;
    let dashboard_available = frps::dashboard_available(&frps).await;
    Ok(Json(FrpsStatusResponse {
        server_addr: frps.server_addr.clone(),
        bind_port: frps.bind_port,
        token_set: !frps.auth_token.is_empty(),
        remote_port_min: frps.remote_port_min,
        remote_port_max: frps.remote_port_max,
        config_path: crate::services::frps::FRPS_CONFIG_PATH.to_owned(),
        status: runtime_status.display_status.clone(),
        state: runtime_status.state,
        version: runtime_status.version,
        display_status: runtime_status.display_status,
        restarting,
        restart_command_configured: true,
        dashboard_addr: frps.dashboard_addr,
        dashboard_port: frps.dashboard_port,
        dashboard_user: frps.dashboard_user,
        dashboard_configured: frps.dashboard_port.is_some(),
        dashboard_available,
        vhost_http_port: frps.vhost_http_port,
        vhost_https_port: frps.vhost_https_port,
    }))
}

pub async fn update_frps(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
    Json(form): Json<FrpsUpdateForm>,
) -> AppResult<impl IntoResponse> {
    if form.server_addr.trim().is_empty() {
        return Err(AppError::BadRequest("frps 地址不能为空".to_owned()));
    }
    if form.bind_port == 0 {
        return Err(AppError::BadRequest(
            "frps bindPort 必须在 1-65535 内".to_owned(),
        ));
    }
    if !form.auth_token.is_empty() && form.auth_token.len() < 8 {
        return Err(AppError::BadRequest("frps token 至少 8 位".to_owned()));
    }
    if form.remote_port_min <= 0 || form.remote_port_max > 65535 {
        return Err(AppError::BadRequest(
            "远程端口范围必须在 1-65535 内".to_owned(),
        ));
    }
    if form.remote_port_min > form.remote_port_max {
        return Err(AppError::BadRequest("远程端口范围无效".to_owned()));
    }
    if form.dashboard_port.is_some() && form.dashboard_addr.trim().is_empty() {
        return Err(AppError::BadRequest("dashboard 地址不能为空".to_owned()));
    }
    if form.vhost_http_port == Some(form.bind_port) || form.vhost_https_port == Some(form.bind_port)
    {
        return Err(AppError::BadRequest(
            "域名入口端口不能和 bindPort 相同".to_owned(),
        ));
    }

    let allocated_ports = tunnels::Entity::find()
        .all(&state.db)
        .await?
        .into_iter()
        .filter_map(|item| item.remote_port)
        .collect::<Vec<_>>();
    if allocated_ports
        .iter()
        .any(|port| *port < form.remote_port_min || *port > form.remote_port_max)
    {
        return Err(AppError::BadRequest(
            "新端口范围不能排除已分配的远程端口".to_owned(),
        ));
    }

    let mut current = state.frps.read().await.clone();
    current.server_addr = form.server_addr.trim().to_owned();
    current.bind_port = form.bind_port;
    if !form.auth_token.is_empty() {
        current.auth_token = form.auth_token;
    }
    current.remote_port_min = form.remote_port_min;
    current.remote_port_max = form.remote_port_max;
    current.dashboard_addr = form.dashboard_addr.trim().to_owned();
    current.dashboard_port = form.dashboard_port;
    current.dashboard_user = form.dashboard_user.trim().to_owned();
    if !form.dashboard_password.is_empty() {
        current.dashboard_password = form.dashboard_password;
    }
    current.vhost_http_port = form.vhost_http_port;
    current.vhost_https_port = form.vhost_https_port;

    frps::write_runtime_config(&current)
        .await
        .map_err(|error| AppError::BadRequest(format!("保存 frps 配置失败: {error}")))?;
    *state.frps.write().await = current;

    Ok(Json(OkResponse { ok: true }))
}

pub async fn restart_frps(
    State(state): State<AppState>,
    AdminUser(_user): AdminUser,
) -> AppResult<impl IntoResponse> {
    if state
        .frps_restarting
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::BadRequest("frps 正在重启".to_owned()));
    }

    let restarting = state.frps_restarting.clone();
    tokio::spawn(async move {
        if let Err(error) = frps::restart_frps().await {
            tracing::error!(%error, "failed to restart frps");
        }
        restarting.store(false, Ordering::SeqCst);
    });

    Ok(Json(OkResponse { ok: true }))
}

async fn set_user_disabled(state: &AppState, id: Uuid, disabled: bool) -> AppResult<()> {
    let Some(user) = users::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    let mut active: users::ActiveModel = user.into();
    active.disabled = Set(disabled);
    active.update(&state.db).await?;
    Ok(())
}

async fn caddy_response(
    config: caddy::CaddyPanelConfig,
    upstream: std::net::SocketAddr,
) -> CaddyResponse {
    CaddyResponse {
        enabled: config.enabled,
        domain: config.domain,
        config_path: caddy::CADDY_CONFIG_PATH.to_owned(),
        available: caddy::caddy_available().await,
        upstream: caddy::upstream_from_bind(upstream),
    }
}

fn effective_max_tunnels(user: &users::Model, default_max_tunnels: u64) -> u64 {
    user.max_tunnels
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or(default_max_tunnels)
}

fn normalized_query(q: &Option<String>) -> Option<String> {
    q.as_ref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn page_response<T>(items: Vec<T>, query: &ListQuery) -> PageResponse<T> {
    let total = items.len() as u64;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let start = ((page - 1) * page_size) as usize;
    let paged_items = items
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();

    PageResponse {
        items: paged_items,
        total,
        page,
        page_size,
    }
}
