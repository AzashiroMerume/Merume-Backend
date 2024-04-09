use crate::{responses::OperationStatusResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn delete_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    let deletion_result = state
        .db
        .channels_collection
        .delete_one(doc! { "_id": channel_id }, None)
        .await;

    match deletion_result {
        Ok(result) => {
            if result.deleted_count == 1 {
                (
                    StatusCode::OK,
                    Json(OperationStatusResponse {
                        success: true,
                        error_message: None,
                    }),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(OperationStatusResponse {
                        success: false,
                        error_message: Some("Channel not found".to_string()),
                    }),
                )
            }
        }
        Err(err) => {
            eprintln!("Error deleting post: {:?}", err);
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
