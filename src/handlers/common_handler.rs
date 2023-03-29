use axum::{http::StatusCode, response::IntoResponse};

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn test_handler() -> impl IntoResponse {
    (StatusCode::OK, "well done, it works")
}
