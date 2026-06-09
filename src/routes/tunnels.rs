use std::io::{Cursor, Write};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    entities::{certificates, tunnels},
    error::{AppError, AppResult},
    routes::types::{
        FrpcResponse, OkResponse, TrafficHistoryPointResponse, TrafficHistoryResponse,
        TunnelResponse,
    },
    services::{frpc, ports, traffic, validation},
    state::AppState,
};

#[derive(Deserialize)]
pub struct TunnelForm {
    name: String,
    protocol: String,
    local_host: String,
    local_port: i32,
    remote_port: Option<i32>,
    custom_domain: Option<String>,
    tls_mode: Option<String>,
    certificate_id: Option<Uuid>,
    #[serde(default)]
    use_encryption: bool,
    #[serde(default)]
    use_compression: bool,
    bandwidth_limit: Option<String>,
    bandwidth_limit_mode: Option<String>,
    proxy_protocol_version: Option<String>,
    locations: Option<String>,
    host_header_rewrite: Option<String>,
}

#[derive(Deserialize)]
pub struct TrafficHistoryQuery {
    range: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(form): Json<TunnelForm>,
) -> AppResult<impl IntoResponse> {
    let count = tunnels::Entity::find()
        .filter(tunnels::Column::UserId.eq(user.id))
        .count(&state.db)
        .await?;
    let user_max_tunnels = user
        .max_tunnels
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or(state.config.user_max_tunnels);
    if count >= user_max_tunnels {
        return Err(AppError::BadRequest("隧道数量已达上限".to_owned()));
    }

    let input = validate_tunnel_form(&state, user.id, None, form).await?;

    let frps = state.frps.read().await.clone();
    if matches!(input.protocol.as_str(), "tcp" | "udp") {
        let attempts = if input.requested_remote_port.is_some() {
            1
        } else {
            5
        };
        for _ in 0..attempts {
            let remote_port = match input.requested_remote_port {
                Some(remote_port) => {
                    ports::validate_remote_port_available(
                        &state.db,
                        remote_port,
                        frps.remote_port_min,
                        frps.remote_port_max,
                        None,
                    )
                    .await?
                }
                None => {
                    ports::allocate_remote_port(
                        &state.db,
                        frps.remote_port_min,
                        frps.remote_port_max,
                    )
                    .await?
                }
            };

            let result = insert_tunnel(&state, user.id, &input, Some(remote_port)).await;
            match result {
                Ok(tunnel) => return Ok(Json(TunnelResponse::from(tunnel))),
                Err(sea_orm::DbErr::Exec(error)) if error.to_string().contains("remote_port") => {
                    continue
                }
                Err(error) => return Err(error.into()),
            }
        }
        return Err(AppError::BadRequest("远程端口分配冲突，请重试".to_owned()));
    }

    let tunnel = insert_tunnel(&state, user.id, &input, None).await?;
    Ok(Json(TunnelResponse::from(tunnel)))
}

pub async fn get(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let tunnel = get_owned_tunnel(&state, user.id, id).await?;
    Ok(Json(TunnelResponse::from(tunnel)))
}

pub async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(form): Json<TunnelForm>,
) -> AppResult<impl IntoResponse> {
    let tunnel = get_owned_tunnel(&state, user.id, id).await?;
    let input = validate_tunnel_form(&state, user.id, Some(id), form).await?;
    let remote_port = update_remote_port(&state, &tunnel, &input).await?;

    let mut active: tunnels::ActiveModel = tunnel.into();
    active.name = Set(input.name);
    active.protocol = Set(input.protocol);
    active.local_host = Set(input.local_host);
    active.local_port = Set(input.local_port);
    active.remote_port = Set(remote_port);
    active.custom_domain = Set(input.custom_domain);
    active.tls_mode = Set(input.tls_mode);
    active.certificate_id = Set(input.certificate_id);
    active.use_encryption = Set(input.use_encryption);
    active.use_compression = Set(input.use_compression);
    active.bandwidth_limit = Set(input.bandwidth_limit);
    active.bandwidth_limit_mode = Set(input.bandwidth_limit_mode);
    active.proxy_protocol_version = Set(input.proxy_protocol_version);
    active.locations = Set(input.locations);
    active.host_header_rewrite = Set(input.host_header_rewrite);
    let tunnel = active.update(&state.db).await?;

    Ok(Json(TunnelResponse::from(tunnel)))
}

pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    get_owned_tunnel(&state, user.id, id).await?;
    tunnels::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn preview_frpc(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let tunnel = get_owned_tunnel(&state, user.id, id).await?;

    let frps = state.frps.read().await;
    let frpc_toml = frpc::render_frpc_toml(&frps, &user, &tunnel);
    Ok(Json(FrpcResponse {
        tunnel: TunnelResponse::from(tunnel),
        frpc_toml,
    }))
}

pub async fn traffic_history(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<TrafficHistoryQuery>,
) -> AppResult<impl IntoResponse> {
    get_owned_tunnel(&state, user.id, id).await?;
    let Some(range) = traffic::TrafficHistoryRange::parse(query.range.as_deref()) else {
        return Err(AppError::BadRequest("流量历史时间范围不合法".to_owned()));
    };
    let points = traffic::history_by_tunnel(&state.db, id, range)
        .await?
        .into_iter()
        .map(|point| TrafficHistoryPointResponse {
            traffic_in: point.traffic_in,
            traffic_out: point.traffic_out,
            sampled_at: point.sampled_at,
        })
        .collect();
    Ok(Json(TrafficHistoryResponse { points }))
}

pub async fn download_frpc(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let tunnel = get_owned_tunnel(&state, user.id, id).await?;

    let frps = state.frps.read().await;
    let body = frpc::render_frpc_toml(&frps, &user, &tunnel);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/toml; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=frpc.toml"),
    );
    Ok((headers, body))
}

pub async fn download_frpc_bundle(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let tunnel = get_owned_tunnel(&state, user.id, id).await?;
    let frps = state.frps.read().await;
    let frpc_toml = frpc::render_frpc_toml(&frps, &user, &tunnel);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options: zip::write::SimpleFileOptions = zip::write::FileOptions::default();
    zip.start_file("frpc.toml", options)
        .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;
    zip.write_all(frpc_toml.as_bytes())
        .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;

    if tunnel.protocol == "https" && tunnel.tls_mode.as_deref() == Some("uploaded_cert") {
        let Some(certificate_id) = tunnel.certificate_id else {
            return Err(AppError::BadRequest("隧道未绑定证书".to_owned()));
        };
        let cert = get_owned_certificate(&state, user.id, certificate_id).await?;
        let (cert_pem, key_pem) =
            crate::services::certificates::read_certificate_bundle(&cert.cert_path, &cert.key_path)
                .await
                .map_err(|error| AppError::BadRequest(format!("读取证书失败: {error}")))?;
        zip.start_file("cert.pem", options)
            .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;
        zip.write_all(&cert_pem)
            .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;
        zip.start_file("key.pem", options)
            .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;
        zip.write_all(&key_pem)
            .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?;
    }

    let body = zip
        .finish()
        .map_err(|error| AppError::BadRequest(format!("生成配置包失败: {error}")))?
        .into_inner();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=frpc-bundle.zip"),
    );
    Ok((headers, body))
}

struct ValidTunnelInput {
    name: String,
    protocol: String,
    local_host: String,
    local_port: i32,
    requested_remote_port: Option<i32>,
    custom_domain: Option<String>,
    tls_mode: Option<String>,
    certificate_id: Option<Uuid>,
    use_encryption: bool,
    use_compression: bool,
    bandwidth_limit: Option<String>,
    bandwidth_limit_mode: Option<String>,
    proxy_protocol_version: Option<String>,
    locations: Option<String>,
    host_header_rewrite: Option<String>,
}

async fn validate_tunnel_form(
    state: &AppState,
    user_id: Uuid,
    tunnel_id: Option<Uuid>,
    form: TunnelForm,
) -> AppResult<ValidTunnelInput> {
    let name = validation::tunnel_name(&form.name)?;
    let protocol = validation::tunnel_protocol(&form.protocol)?;
    let local_host = validation::local_host(&form.local_host)?;
    let local_port = validation::local_port(form.local_port)?;
    let custom_domains = normalize_custom_domains(form.custom_domain.as_deref())?;
    let custom_domain = if custom_domains.is_empty() {
        None
    } else {
        let value = custom_domains.join(",");
        if value.len() > 253 {
            return Err(AppError::BadRequest("绑定域名总长度过长".to_owned()));
        }
        Some(value)
    };
    let tls_mode = form
        .tls_mode
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let mut certificate_id = form.certificate_id;
    let bandwidth_limit = normalize_bandwidth_limit(form.bandwidth_limit.as_deref())?;
    let bandwidth_limit_mode = if bandwidth_limit.is_some() {
        normalize_bandwidth_limit_mode(form.bandwidth_limit_mode.as_deref())?
    } else {
        None
    };
    let proxy_protocol_version =
        normalize_proxy_protocol_version(form.proxy_protocol_version.as_deref())?;
    let locations = normalize_locations(form.locations.as_deref())?;
    let host_header_rewrite = normalize_host_header_rewrite(form.host_header_rewrite.as_deref())?;

    match protocol.as_str() {
        "tcp" | "udp" => {
            if custom_domain.is_some() || tls_mode.is_some() || certificate_id.is_some() {
                return Err(AppError::BadRequest(
                    "TCP/UDP 隧道不能绑定域名或证书".to_owned(),
                ));
            }
            if locations.is_some() || host_header_rewrite.is_some() {
                return Err(AppError::BadRequest(
                    "TCP/UDP 隧道不能设置 HTTP 路径或 Host Header Rewrite".to_owned(),
                ));
            }
        }
        "http" => {
            if form.remote_port.is_some() || tls_mode.is_some() || certificate_id.is_some() {
                return Err(AppError::BadRequest(
                    "HTTP 隧道不能设置远程端口或证书".to_owned(),
                ));
            }
            if proxy_protocol_version.is_some() {
                return Err(AppError::BadRequest(
                    "HTTP 隧道不能设置 Proxy Protocol".to_owned(),
                ));
            }
            if custom_domain.is_none() {
                return Err(AppError::BadRequest("HTTP 隧道必须绑定域名".to_owned()));
            }
            if state.frps.read().await.vhost_http_port.is_none() {
                return Err(AppError::BadRequest(
                    "管理员尚未启用 HTTP 域名入口".to_owned(),
                ));
            }
        }
        "https" => {
            if form.remote_port.is_some() {
                return Err(AppError::BadRequest(
                    "HTTPS 隧道不能设置远程端口".to_owned(),
                ));
            }
            if proxy_protocol_version.is_some() {
                return Err(AppError::BadRequest(
                    "HTTPS 隧道不能设置 Proxy Protocol".to_owned(),
                ));
            }
            if custom_domains.is_empty() {
                return Err(AppError::BadRequest("HTTPS 隧道必须绑定域名".to_owned()));
            }
            if state.frps.read().await.vhost_https_port.is_none() {
                return Err(AppError::BadRequest(
                    "管理员尚未启用 HTTPS 域名入口".to_owned(),
                ));
            }
            match tls_mode.as_deref() {
                Some("https_passthrough") => certificate_id = None,
                Some("uploaded_cert") => {
                    let Some(cert_id) = certificate_id else {
                        return Err(AppError::BadRequest("请选择证书".to_owned()));
                    };
                    let cert = get_owned_certificate(state, user_id, cert_id).await?;
                    let domains =
                        crate::services::certificates::domains_from_json(&cert.domains_json);
                    if custom_domains.iter().any(|domain| {
                        !crate::services::certificates::certificate_covers_domain(&domains, domain)
                    }) {
                        return Err(AppError::BadRequest("证书未覆盖全部绑定域名".to_owned()));
                    }
                }
                _ => return Err(AppError::BadRequest("HTTPS TLS 模式不合法".to_owned())),
            }
        }
        _ => unreachable!(),
    }

    for domain in &custom_domains {
        ensure_domain_available(state, &protocol, domain, tunnel_id).await?;
    }

    Ok(ValidTunnelInput {
        name,
        protocol,
        local_host,
        local_port,
        requested_remote_port: form.remote_port,
        custom_domain,
        tls_mode,
        certificate_id,
        use_encryption: form.use_encryption,
        use_compression: form.use_compression,
        bandwidth_limit,
        bandwidth_limit_mode,
        proxy_protocol_version,
        locations,
        host_header_rewrite,
    })
}

fn normalize_custom_domains(value: Option<&str>) -> AppResult<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let mut domains = Vec::new();
    for domain in value.split([',', '\n']) {
        let domain = domain.trim();
        if domain.is_empty() {
            continue;
        }
        let domain = validation::domain(domain)?;
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    if domains.len() > 8 {
        return Err(AppError::BadRequest("最多绑定 8 个域名".to_owned()));
    }
    Ok(domains)
}

fn normalize_bandwidth_limit(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = validation::bandwidth_limit(value)?;
    Ok((!value.is_empty()).then_some(value))
}

fn normalize_bandwidth_limit_mode(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(Some("client".to_owned()));
    };
    validation::bandwidth_limit_mode(value).map(Some)
}

fn normalize_proxy_protocol_version(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    validation::proxy_protocol_version(value).map(Some)
}

fn normalize_locations(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut locations = Vec::new();
    for location in value.split([',', '\n']) {
        let location = location.trim();
        if location.is_empty() {
            continue;
        }
        let location = validation::http_location(location)?;
        if !locations.contains(&location) {
            locations.push(location);
        }
    }
    if locations.len() > 16 {
        return Err(AppError::BadRequest("最多设置 16 条路径匹配".to_owned()));
    }
    Ok((!locations.is_empty()).then_some(locations.join(",")))
}

fn normalize_host_header_rewrite(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    validation::host_header(value).map(Some)
}

async fn update_remote_port(
    state: &AppState,
    tunnel: &tunnels::Model,
    input: &ValidTunnelInput,
) -> AppResult<Option<i32>> {
    if !matches!(input.protocol.as_str(), "tcp" | "udp") {
        return Ok(None);
    }

    let Some(remote_port) = input.requested_remote_port.or(tunnel.remote_port) else {
        let frps = state.frps.read().await.clone();
        return ports::allocate_remote_port(&state.db, frps.remote_port_min, frps.remote_port_max)
            .await
            .map(Some);
    };

    let frps = state.frps.read().await.clone();
    ports::validate_remote_port_available(
        &state.db,
        remote_port,
        frps.remote_port_min,
        frps.remote_port_max,
        Some(tunnel.id),
    )
    .await
    .map(Some)
}

async fn insert_tunnel(
    state: &AppState,
    user_id: Uuid,
    input: &ValidTunnelInput,
    remote_port: Option<i32>,
) -> Result<tunnels::Model, sea_orm::DbErr> {
    tunnels::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(input.name.clone()),
        protocol: Set(input.protocol.clone()),
        local_host: Set(input.local_host.clone()),
        local_port: Set(input.local_port),
        remote_port: Set(remote_port),
        custom_domain: Set(input.custom_domain.clone()),
        tls_mode: Set(input.tls_mode.clone()),
        certificate_id: Set(input.certificate_id),
        use_encryption: Set(input.use_encryption),
        use_compression: Set(input.use_compression),
        bandwidth_limit: Set(input.bandwidth_limit.clone()),
        bandwidth_limit_mode: Set(input.bandwidth_limit_mode.clone()),
        proxy_protocol_version: Set(input.proxy_protocol_version.clone()),
        locations: Set(input.locations.clone()),
        host_header_rewrite: Set(input.host_header_rewrite.clone()),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(&state.db)
    .await
}

async fn ensure_domain_available(
    state: &AppState,
    protocol: &str,
    domain: &str,
    tunnel_id: Option<Uuid>,
) -> AppResult<()> {
    let exists = tunnels::Entity::find()
        .filter(tunnels::Column::Protocol.eq(protocol))
        .all(&state.db)
        .await?
        .into_iter()
        .any(|tunnel| {
            Some(tunnel.id) != tunnel_id
                && tunnel
                    .custom_domain
                    .as_deref()
                    .map(|domains| domains.split(',').any(|item| item.trim() == domain))
                    .unwrap_or(false)
        });
    if exists {
        return Err(AppError::BadRequest("域名已被占用".to_owned()));
    }
    Ok(())
}

async fn get_owned_certificate(
    state: &AppState,
    user_id: Uuid,
    certificate_id: Uuid,
) -> AppResult<certificates::Model> {
    let Some(cert) = certificates::Entity::find_by_id(certificate_id)
        .one(&state.db)
        .await?
    else {
        return Err(AppError::BadRequest("证书不存在".to_owned()));
    };
    if cert.user_id != user_id {
        return Err(AppError::Forbidden);
    }
    Ok(cert)
}

async fn get_owned_tunnel(
    state: &AppState,
    user_id: Uuid,
    tunnel_id: Uuid,
) -> AppResult<tunnels::Model> {
    let Some(tunnel) = tunnels::Entity::find_by_id(tunnel_id)
        .one(&state.db)
        .await?
    else {
        return Err(AppError::NotFound);
    };
    if tunnel.user_id != user_id {
        return Err(AppError::Forbidden);
    }
    Ok(tunnel)
}
