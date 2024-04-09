use crate::{models::channel_model::UpdateChannel, responses::OperationStatusResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use bson::{doc, oid::ObjectId};

pub async fn update_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
    Json(payload): Json<UpdateChannel>,
) -> impl IntoResponse {
    let serialized_data = match bson::to_bson(&payload) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize payload: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    let document = match serialized_data.as_document() {
        Some(document) => document,
        None => {
            eprintln!("Failed to convert serialized data to document");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    match state
        .db
        .channels_collection
        .find_one_and_update(doc! {"_id": channel_id}, doc! {"$set": document}, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(OperationStatusResponse {
                success: true,
                error_message: None,
            }),
        ),
        Err(err) => {
            eprintln!("Failed to update channel: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        }
    }
}
