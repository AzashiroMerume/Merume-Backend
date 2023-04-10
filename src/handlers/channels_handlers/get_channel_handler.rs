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
