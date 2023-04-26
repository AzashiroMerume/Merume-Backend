use crate::{models::channel_model::Channel, responses::main_response::MainResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn get_channel_by_id(
    State(client): State<Client>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let channels_collection = client.database("Merume").collection::<Channel>("channels");

    let channel_id = match ObjectId::parse_str(&channel_id) {
        Ok(channel_id) => channel_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some("Invalid channel ID".to_string()),
                }),
            )
        }
    };

    let channel = match channels_collection
        .find_one(doc! { "_id": channel_id }, None)
        .await
    {
        Ok(channel) => channel,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
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
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some("Channel not found".to_string()),
                }),
            )
        }
    };

    (
        StatusCode::OK,
        Json(MainResponse {
            success: true,
            data: Some(vec![channel]),
            error_message: None,
        }),
    )
}
