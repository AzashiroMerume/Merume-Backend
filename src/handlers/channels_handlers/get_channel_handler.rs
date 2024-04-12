use crate::{models::channel_model::Channel, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelResponse {
    pub success: bool,
    pub data: Option<Channel>,
    pub error_message: Option<String>,
}

pub async fn get_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    let channel = match state
        .db
        .channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await
    {
        Ok(channel) => channel,
        Err(err) => {
            eprintln!("Failed to retrieve channel: {}", err.to_string());
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChannelResponse {
                    success: false,
                    data: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
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
