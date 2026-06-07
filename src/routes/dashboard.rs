use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use minijinja::context;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{auth::CurrentUser, entities::tunnels, error::AppResult, state::AppState, web};

pub async fn home() -> impl IntoResponse {
    Redirect::to("/login")
}

pub async fn dashboard(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let tunnels = tunnels::Entity::find()
        .filter(tunnels::Column::UserId.eq(user.id))
        .order_by_asc(tunnels::Column::CreatedAt)
        .all(&state.db)
        .await?;

    web::render(
        &state.templates,
        "dashboard.html",
        context! { user => user, tunnels => tunnels },
    )
}
