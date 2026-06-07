use axum::{extract::State, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    auth::CurrentUser, entities::tunnels, error::AppResult, routes::types::TunnelResponse,
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
