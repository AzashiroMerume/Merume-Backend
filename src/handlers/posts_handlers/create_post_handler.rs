use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::Client;

use crate::models::post_model::{Post, PostPayload};
use crate::responses::bool_response::BoolResponse;

pub async fn create_post(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
    Json(payload): Json<PostPayload>,
) -> impl IntoResponse {
    let post_collection = client.database("Merume").collection::<Post>("posts");

    if payload.body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(BoolResponse {
                success: false,
                error_message: Some("Missing fields".to_string()),
            }),
        );
    }

    let post = Post {
        id: ObjectId::new(),
        owner_id: user_id,
        channel_id: channel_id,
        body: payload.body,
        images: payload.images,
        created_at: Utc::now(),
        updated_at: Utc::now(),
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
