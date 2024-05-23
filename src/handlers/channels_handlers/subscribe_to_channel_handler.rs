use crate::{
    models::{author_model::Author, user_channel_model::UserChannel},
    responses::ErrorResponse,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use std::sync::Arc;

pub async fn subscribe_to_channel(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
) -> Result<StatusCode, ErrorResponse> {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await;

    if let Err(err) = channel {
        eprintln!("Failed to retrieve channel: {}", err);
        return Err(ErrorResponse::ServerError(None));
    }

    let channel = match channel.unwrap() {
        Some(channel) => channel,
        None => {
            return Err(ErrorResponse::ServerError(None));
        }
    };

    // Check if the channel belongs to the user trying to subscribe
    if channel.author.id == author.id {
        return Err(ErrorResponse::BadRequest(Some(
            "Cannot subscribe to your own channel",
        )));
    }

    // Check if the user is already subscribed to the channel
    if let Ok(Some(_)) = state
        .db
        .user_channels_collection
        .find_one(doc! {"user_id": author.id, "channel_id": channel.id}, None)
        .await
    {
        return Err(ErrorResponse::BadRequest(Some("Already subscribed")));
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
                Ok(_) => Ok(StatusCode::OK),
                Err(err) => {
                    eprintln!("Failed to update channel subscription field: {}", err);
                    Err(ErrorResponse::ServerError(None))
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to insert user channel: {}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
