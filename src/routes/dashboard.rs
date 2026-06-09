use std::collections::HashMap;

use axum::{extract::State, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

use crate::{
    auth::CurrentUser,
    entities::tunnels,
    error::AppResult,
    routes::types::{DashboardSummaryResponse, TunnelResponse, TunnelWithTrafficResponse},
    services::frps,
    state::AppState,
};

pub async fn tunnels(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let tunnels = tunnels::Entity::find()
        .filter(tunnels::Column::UserId.eq(user.id))
        .order_by_asc(tunnels::Column::CreatedAt)
        .all(&state.db)
        .await?;
    let persistent = crate::services::traffic::latest_by_tunnel(
        &state.db,
        &tunnels.iter().map(|tunnel| tunnel.id).collect::<Vec<_>>(),
    )
    .await?;
    let frps_config = state.frps.read().await.clone();
    let snapshot = frps::traffic_snapshot(&frps_config).await;
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
    let tunnels = tunnels
        .into_iter()
        .map(|tunnel| {
            let user_key = (
                tunnel.protocol.clone(),
                format!("{}.{}", user.username, tunnel.name),
            );
            let key = (tunnel.protocol.clone(), tunnel.name.clone());
            let (traffic_in, traffic_out) = traffic
                .get(&user_key)
                .or_else(|| traffic.get(&key))
                .copied()
                .unwrap_or((0, 0));
            let persistent_traffic = persistent.get(&tunnel.id);
            TunnelWithTrafficResponse {
                tunnel: TunnelResponse::from(tunnel),
                traffic_available: snapshot.available,
                traffic_in,
                traffic_out,
                persistent_traffic_available: persistent_traffic.is_some(),
                persistent_traffic_in: persistent_traffic.map(|item| item.traffic_in).unwrap_or(0),
                persistent_traffic_out: persistent_traffic
                    .map(|item| item.traffic_out)
                    .unwrap_or(0),
                last_sampled_at: persistent_traffic.map(|item| item.sampled_at),
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(tunnels))
}

pub async fn summary(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let tunnel_count = tunnels::Entity::find()
        .filter(tunnels::Column::UserId.eq(user.id))
        .count(&state.db)
        .await?;
    let user_max_tunnels = user
        .max_tunnels
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or(state.config.user_max_tunnels);
    let frps = state.frps.read().await;

    Ok(Json(DashboardSummaryResponse {
        tunnel_count,
        user_max_tunnels,
        remaining_tunnels: user_max_tunnels.saturating_sub(tunnel_count),
        username: user.username,
        role: user.role,
        disabled: user.disabled,
        created_at: user.created_at,
        effective_max_tunnels: user_max_tunnels,
        frps_server_addr: frps.server_addr.clone(),
        frps_bind_port: frps.bind_port,
        remote_port_min: frps.remote_port_min,
        remote_port_max: frps.remote_port_max,
        vhost_http_port: frps.vhost_http_port,
        vhost_https_port: frps.vhost_https_port,
    }))
}
