use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use minijinja::context;
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
    services::{invite, password, validation},
    state::AppState,
    web,
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
    Form(form): Form<CreateInvitesForm>,
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

    for _ in 0..form.count {
        invite_codes::ActiveModel {
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
    }
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
    let mut user_rows = Vec::with_capacity(users.len());
    for item in users {
        let tunnel_count = tunnels::Entity::find()
            .filter(tunnels::Column::UserId.eq(item.id))
            .count(&state.db)
            .await?;
        user_rows.push(context! { item => item, tunnel_count => tunnel_count });
    }
    web::render(
        &state.templates,
        "admin_users.html",
        context! { user => admin, user_rows => user_rows },
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

pub async fn reset_user_password(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
    Form(form): Form<ResetPasswordForm>,
) -> AppResult<impl IntoResponse> {
    validation::password(&form.new_password)?;
    let Some(user) = users::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(password::hash_password(&form.new_password)?);
    active.update(&state.db).await?;
    Ok(Redirect::to("/admin/users"))
}

pub async fn delete_tunnel(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    tunnels::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Redirect::to("/admin/tunnels"))
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
