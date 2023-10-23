use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::doc;
use mongodb::options::UpdateOptions;
use validator::Validate;

use crate::{
    models::{author_model::Author, user_model::UserPreferencesPayload},
    responses::OperationStatusResponse,
    AppState,
};

pub async fn post_preferences(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Json(payload): Json<UserPreferencesPayload>,
) -> impl IntoResponse {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(err.to_string()),
                }),
            );
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
        Ok(_) => (
            StatusCode::OK,
            Json(OperationStatusResponse {
                success: true,
                error_message: None,
            }),
        ),
        Err(err) => {
            eprintln!("Failed to update preferences: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on server side. Please try again later.".to_string(),
                    ),
                }),
            );
        }
    }
}
