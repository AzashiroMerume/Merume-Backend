use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::options::UpdateOptions;
use validator::Validate;

use crate::{models::user_model::UserPreferencesPayload, responses::BoolResponse, AppState};

pub async fn post_preferences(
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Json(payload): Json<UserPreferencesPayload>,
) -> impl IntoResponse {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    let filter = doc! {"_id": user_id};
    let update = doc! {"$set": {"preferences": payload.preferences}};
    let options = UpdateOptions::builder().upsert(false).build();
    let result = state
        .db
        .users_collection
        .update_one(filter, update, options)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(BoolResponse {
                success: true,
                error_message: None,
            }),
        ),
        Err(err) => {
            eprintln!("Failed to update preferences: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on server side. Please try again later.".to_string(),
                    ),
                }),
            );
        }
    }
}
