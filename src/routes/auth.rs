use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    auth::{find_user_by_username, CurrentUser, SESSION_USER_ID},
    entities::{invite_codes, users},
    error::{AppError, AppResult},
    routes::types::{OkResponse, PublicUser, SessionResponse},
    services::{audit, password, validation},
    state::AppState,
};

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    invite_code: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct PasswordChangeForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

pub async fn session(CurrentUser(user): CurrentUser) -> AppResult<impl IntoResponse> {
    Ok(Json(SessionResponse {
        user: PublicUser::from(user),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(form): Json<LoginForm>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = find_user_by_username(&state.db, form.username.trim()).await? else {
        return Err(AppError::BadRequest("用户名或密码错误".to_owned()));
    };
    if user.disabled {
        return Err(AppError::Forbidden);
    }
    if !password::verify_password(&form.password, &user.password_hash)? {
        return Err(AppError::BadRequest("用户名或密码错误".to_owned()));
    }
    session.insert(SESSION_USER_ID, user.id).await?;
    audit::record(
        &state.db,
        audit::AuditEvent {
            actor: Some(&user),
            action: "auth.login",
            resource_type: "user",
            resource_id: Some(user.id),
            resource_name: Some(user.username.clone()),
            outcome: "success",
            message: None,
            metadata: None,
        },
    )
    .await;
    Ok(Json(SessionResponse {
        user: PublicUser::from(user),
    }))
}

pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Json(form): Json<RegisterForm>,
) -> AppResult<impl IntoResponse> {
    let username = validation::username(&form.username)?;
    validation::password(&form.password)?;
    if find_user_by_username(&state.db, &username).await?.is_some() {
        return Err(AppError::BadRequest("用户名已存在".to_owned()));
    }

    let Some(invite) = invite_codes::Entity::find()
        .filter(invite_codes::Column::Code.eq(form.invite_code.trim()))
        .one(&state.db)
        .await?
    else {
        return Err(AppError::BadRequest("邀请码无效".to_owned()));
    };
    if invite.used_by.is_some() {
        return Err(AppError::BadRequest("邀请码已使用".to_owned()));
    }
    if let Some(expires_at) = invite.expires_at {
        if expires_at < Utc::now().fixed_offset() {
            return Err(AppError::BadRequest("邀请码已过期".to_owned()));
        }
    }

    let user_id = Uuid::new_v4();
    users::ActiveModel {
        id: Set(user_id),
        username: Set(username),
        password_hash: Set(password::hash_password(&form.password)?),
        role: Set("user".to_owned()),
        disabled: Set(false),
        created_at: Set(Utc::now().fixed_offset()),
        max_tunnels: Set(None),
    }
    .insert(&state.db)
    .await?;

    let user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut active_invite: invite_codes::ActiveModel = invite.into();
    active_invite.used_by = Set(Some(user_id));
    active_invite.used_at = Set(Some(Utc::now().fixed_offset()));
    active_invite.update(&state.db).await?;

    session.insert(SESSION_USER_ID, user_id).await?;
    audit::record(
        &state.db,
        audit::AuditEvent {
            actor: Some(&user),
            action: "auth.register",
            resource_type: "user",
            resource_id: Some(user.id),
            resource_name: Some(user.username.clone()),
            outcome: "success",
            message: None,
            metadata: None,
        },
    )
    .await;
    Ok(Json(SessionResponse {
        user: PublicUser::from(user),
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(form): Json<PasswordChangeForm>,
) -> AppResult<impl IntoResponse> {
    if !password::verify_password(&form.current_password, &user.password_hash)? {
        return Err(AppError::BadRequest("当前密码错误".to_owned()));
    }
    validation::password(&form.new_password)?;
    if form.new_password != form.confirm_password {
        return Err(AppError::BadRequest("两次输入的新密码不一致".to_owned()));
    }

    let actor = user.clone();
    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(password::hash_password(&form.new_password)?);
    active.update(&state.db).await?;
    audit::record(
        &state.db,
        audit::AuditEvent {
            actor: Some(&actor),
            action: "auth.password_change",
            resource_type: "user",
            resource_id: Some(actor.id),
            resource_name: Some(actor.username.clone()),
            outcome: "success",
            message: None,
            metadata: None,
        },
    )
    .await;

    Ok(Json(OkResponse { ok: true }))
}

pub async fn logout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    session: Session,
) -> AppResult<impl IntoResponse> {
    session.remove::<Uuid>(SESSION_USER_ID).await?;
    audit::record(
        &state.db,
        audit::AuditEvent {
            actor: Some(&user),
            action: "auth.logout",
            resource_type: "user",
            resource_id: Some(user.id),
            resource_name: Some(user.username.clone()),
            outcome: "success",
            message: None,
            metadata: None,
        },
    )
    .await;
    Ok(Json(OkResponse { ok: true }))
}
