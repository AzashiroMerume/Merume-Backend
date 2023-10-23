use crate::{
    models::{author_model::Author, user_channel_model::UserChannel},
    responses::OperationStatusResponse,
    AppState,
};
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
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await;

    if let Err(err) = channel {
        eprintln!("Failed to retrieve channel: {}", err.to_string());
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

    let channel = match channel.unwrap() {
        Some(channel) => channel,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Channel not found".to_string()),
                }),
            );
        }
    };

    // Check if the channel belongs to the user trying to subscribe
    if channel.author.id == author.id {
        return (
            StatusCode::BAD_REQUEST,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("You cannot subscribe to your own channel".to_string()),
            }),
        );
    }

    // Check if the user is already subscribed to the channel
    if let Ok(Some(_)) = state
        .db
        .user_channels_collection
        .find_one(doc! {"user_id": author.id, "channel_id": channel.id}, None)
        .await
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("User is already subscribed to this channel".to_string()),
            }),
        );
    }

    let user_channel = UserChannel {
        id: ObjectId::new(),
        user_id: author.id,
        channel_id: channel.id,
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
            match state
                .db
                .channels_collection
                .update_one(
                    doc! {"_id": channel.id},
                    doc! {"$inc": {"subscriptions.current_subscriptions": 1}},
                    None,
                )
                .await
            {
                Ok(_) => {
                    return (
                        StatusCode::OK,
                        Json(OperationStatusResponse {
                            success: true,
                            error_message: None,
                        }),
                    )
                }
                Err(err) => {
                    eprintln!(
                        "Failed to update channel subscription field: {}",
                        err.to_string()
                    );
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(OperationStatusResponse {
                            success: false,
                            error_message: Some(
                                "There was an error on the server side, try again later."
                                    .to_string(),
                            ),
                        }),
                    );
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to insert user channel: {}", err.to_string());
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
    }
}
