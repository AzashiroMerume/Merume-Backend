use crate::{
    models::{author_model::Author, user_model::UserPreferencesPayload},
    responses::ErrorResponse,
    AppState,
};
use axum::{extract::State, http::StatusCode, Extension, Json};
use bson::doc;
use mongodb::options::UpdateOptions;
use std::sync::Arc;
use validator::Validate;

pub async fn post_preferences(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Json(payload): Json<UserPreferencesPayload>,
) -> Result<StatusCode, ErrorResponse> {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            eprintln!("Error validating payload: {}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
        }
    }

    let filter = doc! {"_id": author.id};
    let update = doc! {"$set": {"preferences": payload.preferences}};
    let options = UpdateOptions::builder().upsert(false).build();
    let result = state
        .db
        .users_collection
        .update_one(filter, update, options)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            eprintln!("Failed to update preferences: {}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
