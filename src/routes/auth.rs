use axum::{
    extract::{Form, Query, State},
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use minijinja::context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    auth::{find_user_by_username, CurrentUser, SESSION_USER_ID},
    entities::{invite_codes, users},
    error::{AppError, AppResult},
    services::{password, validation},
    state::AppState,
    web,
};

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterQuery {
    code: Option<String>,
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

pub async fn login_page(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    web::render(&state.templates, "login.html", web::empty_context())
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
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
    Ok(Redirect::to("/dashboard"))
}

pub async fn register_page(
    State(state): State<AppState>,
    Query(query): Query<RegisterQuery>,
) -> AppResult<impl IntoResponse> {
    web::render(
        &state.templates,
        "register.html",
        context! { invite_code => query.code.unwrap_or_default() },
    )
}

pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<RegisterForm>,
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
    }
    .insert(&state.db)
    .await?;

    let mut active_invite: invite_codes::ActiveModel = invite.into();
    active_invite.used_by = Set(Some(user_id));
    active_invite.used_at = Set(Some(Utc::now().fixed_offset()));
    active_invite.update(&state.db).await?;

    session.insert(SESSION_USER_ID, user_id).await?;
    Ok(Redirect::to("/dashboard"))
}

pub async fn password_page(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    web::render(&state.templates, "password.html", context! { user => user })
}

pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Form(form): Form<PasswordChangeForm>,
) -> AppResult<impl IntoResponse> {
    if !password::verify_password(&form.current_password, &user.password_hash)? {
        return Err(AppError::BadRequest("当前密码错误".to_owned()));
    }
    validation::password(&form.new_password)?;
    if form.new_password != form.confirm_password {
        return Err(AppError::BadRequest("两次输入的新密码不一致".to_owned()));
    }

    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(password::hash_password(&form.new_password)?);
    active.update(&state.db).await?;

    Ok(Redirect::to("/dashboard"))
}

pub async fn logout(session: Session) -> AppResult<impl IntoResponse> {
    session.remove::<Uuid>(SESSION_USER_ID).await?;
    Ok(Redirect::to("/login"))
}
