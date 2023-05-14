use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};

use crate::responses::OperationStatusResponse;
use crate::AppState;

pub async fn delete_post_by_id(
    State(state): State<AppState>,
    Path((_channel_id, post_id)): Path<(ObjectId, ObjectId)>,
) -> impl IntoResponse {
    let deletion_result = state
        .db
        .posts_collection
        .delete_one(doc! { "_id": post_id }, None)
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
                        error_message: Some("Post not found".to_string()),
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
