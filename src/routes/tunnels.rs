use axum::{
    extract::{Path, State},
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
    entities::tunnels,
    error::{AppError, AppResult},
    routes::types::{FrpcResponse, OkResponse, TunnelResponse},
    services::{frpc, ports, validation},
    state::AppState,
};

#[derive(Deserialize)]
pub struct TunnelForm {
    name: String,
    protocol: String,
    local_host: String,
    local_port: i32,
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
    if count >= state.config.user_max_tunnels {
        return Err(AppError::BadRequest("隧道数量已达上限".to_owned()));
    }

    let name = validation::tunnel_name(&form.name)?;
    let protocol = validation::tunnel_protocol(&form.protocol)?;
    let local_host = validation::local_host(&form.local_host)?;
    let local_port = validation::local_port(form.local_port)?;

    for _ in 0..5 {
        let remote_port = ports::allocate_remote_port(
            &state.db,
            state.config.remote_port_min,
            state.config.remote_port_max,
        )
        .await?;

        let result = tunnels::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            name: Set(name.clone()),
            protocol: Set(protocol.clone()),
            local_host: Set(local_host.clone()),
            local_port: Set(local_port),
            remote_port: Set(remote_port),
            created_at: Set(Utc::now().fixed_offset()),
        }
        .insert(&state.db)
        .await;

        match result {
            Ok(tunnel) => return Ok(Json(TunnelResponse::from(tunnel))),
            Err(sea_orm::DbErr::Exec(error)) if error.to_string().contains("remote_port") => {
                continue
            }
            Err(error) => return Err(error.into()),
        }
    }

    Err(AppError::BadRequest("远程端口分配冲突，请重试".to_owned()))
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
    Ok(Json(OkResponse { ok: true }))
}

pub async fn preview_frpc(
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

    let frpc_toml = frpc::render_frpc_toml(&state.config, &user, &tunnel);
    Ok(Json(FrpcResponse {
        tunnel: TunnelResponse::from(tunnel),
        frpc_toml,
    }))
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
