use axum::{http::StatusCode, response::IntoResponse};

pub async fn verify_auth() -> impl IntoResponse {
    StatusCode::OK
}
