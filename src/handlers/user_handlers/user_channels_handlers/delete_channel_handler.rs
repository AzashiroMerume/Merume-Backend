use crate::{responses::ErrorResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn delete_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
) -> Result<StatusCode, ErrorResponse> {
    let deletion_result = state
        .db
        .channels_collection
        .delete_one(doc! { "_id": channel_id }, None)
        .await;

    match deletion_result {
        Ok(result) => {
            if result.deleted_count == 1 {
                Ok(StatusCode::OK)
            } else {
                Err(ErrorResponse::NotFound(None))
            }
        }
        Err(err) => {
            eprintln!("Error deleting post: {:?}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
