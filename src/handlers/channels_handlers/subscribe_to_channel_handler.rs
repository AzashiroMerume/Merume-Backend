use crate::{
    models::{channel_model::Channel, user_channel_model::UserChannel},
    responses::main_response::MainResponse,
    utils::jwt::Claims,
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
    Extension(token_info): Extension<Claims>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match ObjectId::parse_str(&token_info.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some("Unauthorized".to_string()),
                }),
            )
        }
    };

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

    let user_channels_collection = client
        .database("Merume")
        .collection::<UserChannel>("user_channels");

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
                Json(MainResponse {
                    success: false,
                    data: None,
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
        Json(MainResponse {
            success: true,
            data: Some(vec![channel]),
            error_message: None,
        }),
    )
}
