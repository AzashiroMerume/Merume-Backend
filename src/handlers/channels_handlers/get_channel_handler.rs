use crate::{responses::ChannelResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};

pub async fn get_channel_by_id(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let channel_id = match ObjectId::parse_str(&channel_id) {
        Ok(channel_id) => channel_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ChannelResponse {
                    success: false,
                    data: None,
                    error_message: Some("Invalid channel ID".to_string()),
                }),
            )
        }
    };

    let channel = match state
        .db
        .channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await
    {
        Ok(channel) => channel,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChannelResponse {
                    success: false,
                    data: None,
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
                Json(ChannelResponse {
                    success: false,
                    data: None,
                    error_message: Some("Channel not found".to_string()),
                }),
            )
        }
    };

    (
        StatusCode::OK,
        Json(ChannelResponse {
            success: true,
            data: Some(channel),
            error_message: None,
        }),
    )
}
