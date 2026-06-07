use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use minijinja::context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder};
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    entities::{invite_codes, tunnels, users},
    error::{AppError, AppResult},
    services::invite,
    state::AppState,
    web,
};

pub async fn index(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
) -> AppResult<impl IntoResponse> {
    web::render(
        &state.templates,
        "admin.html",
        context! { user => user, config => state.config },
    )
}

pub async fn invites(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let invites = invite_codes::Entity::find()
        .order_by_desc(invite_codes::Column::CreatedAt)
        .all(&state.db)
        .await?;
    web::render(
        &state.templates,
        "admin_invites.html",
        context! { user => user, invites => invites },
    )
}

pub async fn create_invite(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
) -> AppResult<impl IntoResponse> {
    invite_codes::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(invite::generate_invite_code()),
        created_by: Set(user.id),
        used_by: Set(None),
        used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(&state.db)
    .await?;
    Ok(Redirect::to("/admin/invites"))
}

pub async fn users(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let users = users::Entity::find()
        .order_by_asc(users::Column::CreatedAt)
        .all(&state.db)
        .await?;
    web::render(
        &state.templates,
        "admin_users.html",
        context! { user => admin, users => users },
    )
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
    Ok(Redirect::to("/admin/users"))
}

pub async fn enable_user(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    set_user_disabled(&state, id, false).await?;
    Ok(Redirect::to("/admin/users"))
}

pub async fn all_tunnels(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
) -> AppResult<impl IntoResponse> {
    let tunnels = tunnels::Entity::find()
        .order_by_asc(tunnels::Column::RemotePort)
        .all(&state.db)
        .await?;
    web::render(
        &state.templates,
        "admin_tunnels.html",
        context! { user => user, tunnels => tunnels },
    )
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
