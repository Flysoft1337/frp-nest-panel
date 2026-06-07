use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    entities::users,
    error::{AppError, AppResult},
    state::AppState,
};

pub const SESSION_USER_ID: &str = "user_id";

#[derive(Clone, Debug)]
pub struct CurrentUser(pub users::Model);

#[derive(Clone, Debug)]
pub struct AdminUser(pub users::Model);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let Some(user_id) = session.get::<Uuid>(SESSION_USER_ID).await? else {
            return Err(AppError::Unauthorized);
        };
        let Some(user) = users::Entity::find_by_id(user_id)
            .one(&app_state.db)
            .await?
        else {
            session.remove::<Uuid>(SESSION_USER_ID).await?;
            return Err(AppError::Unauthorized);
        };
        if user.disabled {
            session.remove::<Uuid>(SESSION_USER_ID).await?;
            return Err(AppError::Forbidden);
        }
        Ok(CurrentUser(user))
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        if user.role != "admin" {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(user))
    }
}

pub async fn find_user_by_username(
    db: &sea_orm::DatabaseConnection,
    username: &str,
) -> AppResult<Option<users::Model>> {
    Ok(users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await?)
}
