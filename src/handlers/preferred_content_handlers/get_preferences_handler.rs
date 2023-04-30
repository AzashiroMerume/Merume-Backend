use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{models::user_model::User, responses::MainResponse};

pub async fn get_preferences(Extension(user): Extension<User>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(MainResponse {
            success: true,
            data: user.preferences,
            error_message: None,
        }),
    )
}
