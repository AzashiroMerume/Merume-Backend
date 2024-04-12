use crate::models::user_model::User;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PreferencesResponse {
    pub success: bool,
    pub data: Option<Vec<String>>,
    pub error_message: Option<String>,
}

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
