use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::{options::UpdateOptions, Client};

use crate::{
    models::user_model::{User, UserPreferencesPayload},
    responses::bool_response::BoolResponse,
};

pub async fn post_preferences(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Json(payload): Json<UserPreferencesPayload>,
) -> impl IntoResponse {
    let collection = client.database("Merume").collection::<User>("users");

    if payload.preferences.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(BoolResponse {
                success: false,
                error_message: Some("Please fill in all required fields".to_string()),
            }),
        );
    }

    let filter = doc! {"_id": user_id};
    let update = doc! {"$set": {"preferences": payload.preferences}};
    let options = UpdateOptions::builder().upsert(false).build();
    let result = collection.update_one(filter, update, options).await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(BoolResponse {
                success: true,
                error_message: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BoolResponse {
                success: false,
                error_message: Some(format!("Failed to update preferences")),
            }),
        ),
    }
}
