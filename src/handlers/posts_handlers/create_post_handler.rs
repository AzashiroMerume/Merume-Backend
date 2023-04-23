use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::Client;
use validator::Validate;

use crate::models::post_model::{Post, PostPayload};
use crate::responses::bool_response::BoolResponse;

pub async fn create_post(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
    Json(payload): Json<PostPayload>,
) -> impl IntoResponse {
    let post_collection = client.database("Merume").collection::<Post>("posts");

    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(e.to_string()),
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
        created_at: now,
        updated_at: now,
    };

    let result = post_collection.insert_one(post, None).await;

    match result {
        Ok(_) => {
            return (
                StatusCode::CREATED,
                Json(BoolResponse {
                    success: true,
                    error_message: None,
                }),
            );
        }
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some("Failed to insert user".to_string()),
                }),
            )
        }
    }
}
