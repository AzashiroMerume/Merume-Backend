use crate::{models::user_model::User, responses::ErrorResponse};
use axum::{Extension, Json};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PreferencesResponse {
    pub data: Option<Vec<String>>,
}

pub async fn get_preferences(
    Extension(user): Extension<User>,
) -> Result<Json<PreferencesResponse>, ErrorResponse> {
    Ok(Json(PreferencesResponse {
        data: user.preferences,
    }))
}
