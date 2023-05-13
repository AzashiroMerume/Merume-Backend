use crate::{models::post_model::UpdatePost, responses::OperationStatusResponse, AppState};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use bson::{doc, oid::ObjectId};
use chrono::Utc;

pub async fn update_post_by_id(
    State(state): State<AppState>,
    Path(post_id): Path<ObjectId>,
    Json(payload): Json<UpdatePost>,
) -> impl IntoResponse {
    //check if post already changed once
    match state
        .db
        .posts_collection
        .find_one(doc! {"_id": post_id}, None)
        .await
    {
        Ok(Some(post)) => {
            if post.already_changed {
                return (
                    StatusCode::CONFLICT,
                    Json(OperationStatusResponse {
                        success: false,
                        error_message: Some("Post has already been changed once!".to_string()),
                    }),
                );
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("There is no such post".to_string()),
                }),
            );
        }
        Err(err) => {
            eprintln!("Failed to find a post: {}", err.to_string());
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Failed to update post".to_string()),
                }),
            );
        }
    }

    let mut payload = payload;
    let now = Utc::now();
    payload.updated_at = Some(now);
    payload.already_changed = Some(true);

    let serialized_data = match bson::to_bson(&payload) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize payload: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Failed to serialize payload".to_string()),
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
                        "Failed to convert serialized data to document".to_string(),
                    ),
                }),
            );
        }
    };

    match state
        .db
        .posts_collection
        .update_one(doc! {"_id": post_id}, doc! {"$set": document}, None)
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
            eprintln!("Failed to update post: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Failed to update post".to_string()),
                }),
            )
        }
    }
}
