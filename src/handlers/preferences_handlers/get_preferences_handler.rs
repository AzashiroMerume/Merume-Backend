use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{models::user_model::User, responses::PreferencesResponse};

pub async fn get_preferences(Extension(user): Extension<User>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(PreferencesResponse {
            success: true,
            data: user.preferences,
            error_message: None,
        }),
    )
}
