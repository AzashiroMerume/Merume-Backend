use crate::{
    models::{channel_model::Channel, user_channel_model::UserChannel},
    responses::bool_response::BoolResponse,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn subscribe_to_channel(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let channels_collection = client.database("Merume").collection::<Channel>("channels");

    let channel = match channels_collection
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
    if channel.owner_id.unwrap() == user_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(BoolResponse {
                success: false,
                error_message: Some("You cannot subscribe to your own channel".to_string()),
            }),
        );
    }

    let user_channels_collection = client
        .database("Merume")
        .collection::<UserChannel>("user_channels");

    // Check if the user is already subscribed to the channel
    if let Ok(Some(_)) = user_channels_collection
        .find_one(
            doc! {"user_id": user_id, "channel_id": channel.id.unwrap()},
            None,
        )
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
        id: Some(ObjectId::new()),
        user_id: Some(user_id),
        channel_id: channel.id.clone(),
        is_owner: Some(false),
    };

    match user_channels_collection
        .insert_one(user_channel.clone(), None)
        .await
    {
        Ok(_) => {}
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

    (
        StatusCode::OK,
        Json(BoolResponse {
            success: true,
            error_message: None,
        }),
    )
}
