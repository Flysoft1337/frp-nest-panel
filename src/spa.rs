use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn index() -> impl IntoResponse {
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(index) => Html(index).into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "frontend build is missing").into_response(),
    }
}
