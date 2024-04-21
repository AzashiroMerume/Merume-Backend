use crate::{models::channel_model::Channel, responses::ErrorResponse, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use bson::{doc, oid::ObjectId};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelResponse {
    pub data: Option<Channel>,
}

pub async fn get_channel_by_id(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
) -> Result<Json<ChannelResponse>, ErrorResponse> {
    let channel = match state
        .db
        .channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await
    {
        Ok(channel) => channel,
        Err(err) => {
            eprintln!("Failed to retrieve channel: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let channel = match channel {
        Some(channel) => channel,
        None => return Err(ErrorResponse::NotFound(None)),
    };

    let response = ChannelResponse {
        data: Some(channel),
    };

    Ok(Json(response))
}
