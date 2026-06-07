use axum::{extract::State, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

use crate::{
    auth::CurrentUser,
    entities::tunnels,
    error::AppResult,
    routes::types::{DashboardSummaryResponse, TunnelResponse},
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
        .await?
        .into_iter()
        .map(TunnelResponse::from)
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
    let user_max_tunnels = state.config.user_max_tunnels;

    Ok(Json(DashboardSummaryResponse {
        tunnel_count,
        user_max_tunnels,
        remaining_tunnels: user_max_tunnels.saturating_sub(tunnel_count),
        frps_server_addr: state.config.frps_server_addr.clone(),
        frps_bind_port: state.config.frps_bind_port,
        remote_port_min: state.config.remote_port_min,
        remote_port_max: state.config.remote_port_max,
    }))
}
