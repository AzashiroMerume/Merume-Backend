use axum::{extract::State, response::IntoResponse, Extension, Json};
use bson::oid::ObjectId;
use mongodb::Client;

use crate::models::user_model::{User, UserPreferencesPayload};

pub async fn post_preferences(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Json(payload): Json<UserPreferencesPayload>,
) -> impl IntoResponse {
}
