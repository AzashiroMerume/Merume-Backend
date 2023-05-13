use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::oid::ObjectId;
use chrono::Utc;
use validator::Validate;

use crate::responses::OperationStatusResponse;
use crate::{
    models::post_model::{Post, PostPayload},
    AppState,
};

pub async fn create_post(
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Extension(current_challenge_day): Extension<usize>,
    Path(channel_id): Path<ObjectId>,
    Json(payload): Json<PostPayload>,
) -> impl IntoResponse {
    println!("CURRENT CHALLENGE DAY: {}", current_challenge_day);
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            eprintln!("Error validating payload: {:?}", err);
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    let now = Utc::now();

    let post = Post {
        id: ObjectId::new(),
        owner_id: user_id,
        channel_id,
        body: payload.body,
        images: payload.images,
        written_challenge_day: current_challenge_day,
        likes: 0,
        dislikes: 0,
        already_changed: false,
        created_at: now,
        updated_at: now,
    };

    let result = state.db.posts_collection.insert_one(post, None).await;

    match result {
        Ok(_) => {
            return (
                StatusCode::CREATED,
                Json(OperationStatusResponse {
                    success: true,
                    error_message: None,
                }),
            );
        }
        Err(err) => {
            eprintln!("Error inserting user: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Failed to insert user".to_string()),
                }),
            )
        }
    }
}
