use crate::responses::AuthResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn verify_auth() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: None,
            refresh_token: None,
            user_info: None,
            error_message: None,
        }),
    )
}
