use crate::{models::channel_model::UpdateChannel, responses::ErrorResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn update_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
    Json(payload): Json<UpdateChannel>,
) -> Result<StatusCode, ErrorResponse> {
    let serialized_data = match bson::to_bson(&payload) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize payload: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let document = match serialized_data.as_document() {
        Some(document) => document,
        None => {
            eprintln!("Failed to convert serialized data to document");
            return Err(ErrorResponse::ServerError(None));
        }
    };

    match state
        .db
        .channels_collection
        .find_one_and_update(doc! {"_id": channel_id}, doc! {"$set": document}, None)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            eprintln!("Failed to update channel: {}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
