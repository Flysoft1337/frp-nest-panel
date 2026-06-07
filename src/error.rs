use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::Error),
    #[error("password error")]
    Password,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Unauthorized => StatusCode::SEE_OTHER,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if matches!(self, AppError::Unauthorized) {
            return ([(axum::http::header::LOCATION, "/login")], "").into_response();
        }

        (status, self.to_string()).into_response()
    }
}
