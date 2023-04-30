use crate::{models::user_channel_model::UserChannel, responses::BoolResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;

pub async fn subscribe_to_channel(
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let channel = match state
        .db
        .channels_collection
        .find_one(
            doc! { "_id": ObjectId::parse_str(&channel_id).unwrap() },
            None,
        )
        .await
    {
        Ok(channel) => channel,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(format!("Failed to retrieve channel: {}", err.to_string())),
                }),
            )
        }
    };

    let channel = match channel {
        Some(channel) => channel,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(BoolResponse {
                    success: false,
                    error_message: Some("Channel not found".to_string()),
                }),
            )
        }
    };

    // Check if the channel belongs to the user trying to subscribe
    if channel.owner_id == user_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(BoolResponse {
                success: false,
                error_message: Some("You cannot subscribe to your own channel".to_string()),
            }),
        );
    }

    // Check if the user is already subscribed to the channel
    if let Ok(Some(_)) = state
        .db
        .user_channels_collection
        .find_one(doc! {"user_id": user_id, "channel_id": channel.id}, None)
        .await
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(BoolResponse {
                success: false,
                error_message: Some("User is already subscribed to this channel".to_string()),
            }),
        );
    }

    let user_channel = UserChannel {
        id: ObjectId::new(),
        user_id,
        channel_id: channel.id.clone(),
        is_owner: false,
        subscribed_at: Some(Utc::now()),
        created_at: None,
    };

    match state
        .db
        .user_channels_collection
        .insert_one(user_channel.clone(), None)
        .await
    {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(BoolResponse {
                    success: true,
                    error_message: None,
                }),
            )
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(format!(
                        "Failed to insert user channel: {}",
                        err.to_string()
                    )),
                }),
            )
        }
    }
}
