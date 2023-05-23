use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{models::user_model::User, responses::AuthResponse};

pub async fn verify_auth(Extension(user): Extension<User>) -> impl IntoResponse {
    if user.preferences.is_none() {
        (
            StatusCode::OK,
            Json(AuthResponse {
                success: false,
                token: None,
                error_message: Some("User does not have preferences".to_string()),
            }),
        )
    } else {
        (
            StatusCode::OK,
            Json(AuthResponse {
                success: true,
                token: None,
                error_message: None,
            }),
        )
    }
}
