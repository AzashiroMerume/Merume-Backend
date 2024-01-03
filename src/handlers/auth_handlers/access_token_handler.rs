use crate::responses::AuthResponse;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

pub async fn access_token(Extension(access_token): Extension<String>) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(AuthResponse {
            success: true,
            token: Some(access_token),
            refresh_token: None,
            user_info: None,
            error_message: None,
        }),
    )
}
