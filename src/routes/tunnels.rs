use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use minijinja::context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    entities::tunnels,
    error::{AppError, AppResult},
    services::{frpc, ports, validation},
    state::AppState,
    web,
};

#[derive(Deserialize)]
pub struct TunnelForm {
    name: String,
    local_host: String,
    local_port: i32,
}

pub async fn new_page(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    web::render(
        &state.templates,
        "tunnel_new.html",
        context! { user => user },
    )
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Form(form): Form<TunnelForm>,
) -> AppResult<impl IntoResponse> {
    let count = tunnels::Entity::find()
        .filter(tunnels::Column::UserId.eq(user.id))
        .count(&state.db)
        .await?;
    if count >= state.config.user_max_tunnels {
        return Err(AppError::BadRequest("隧道数量已达上限".to_owned()));
    }

    let name = validation::tunnel_name(&form.name)?;
    let local_host = validation::local_host(&form.local_host)?;
    let local_port = validation::local_port(form.local_port)?;

    let remote_port = ports::allocate_remote_port(
        &state.db,
        state.config.remote_port_min,
        state.config.remote_port_max,
    )
    .await?;

    tunnels::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        name: Set(name),
        protocol: Set("tcp".to_owned()),
        local_host: Set(local_host),
        local_port: Set(local_port),
        remote_port: Set(remote_port),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(&state.db)
    .await?;

    Ok(Redirect::to("/dashboard"))
}

pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let Some(tunnel) = tunnels::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    if tunnel.user_id != user.id {
        return Err(AppError::Forbidden);
    }
    tunnels::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Redirect::to("/dashboard"))
}

pub async fn download_frpc(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let Some(tunnel) = tunnels::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    if tunnel.user_id != user.id {
        return Err(AppError::Forbidden);
    }

    let body = frpc::render_frpc_toml(&state.config, &user, &tunnel);
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
