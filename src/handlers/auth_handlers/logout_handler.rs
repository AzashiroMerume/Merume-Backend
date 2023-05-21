use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::responses::AuthResponse;

pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: None,
            error_message: None,
        }),
    )
}
