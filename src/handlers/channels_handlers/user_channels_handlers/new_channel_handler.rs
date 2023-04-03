use crate::{
    models::{
        channel_model::{Channel, ChannelPayload},
        user_channel_model::UserChannel,
    },
    responses::main_response::MainResponse,
    utils::jwt::Claims,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn new_channel(
    State(client): State<Client>,
    Extension(token_info): Extension<Claims>,
    Json(payload): Json<ChannelPayload>,
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

    if payload.name.is_none() || payload.description.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some("Missing fields".to_string()),
            }),
        );
    }

    let channels_collection = client.database("Merume").collection::<Channel>("channels");
    let user_channels_collection = client
        .database("Merume")
        .collection::<UserChannel>("user_channels");

    let channel = Channel {
        id: Some(ObjectId::new()),
        owner_id: Some(user_id),
        name: payload.name,
        description: payload.description,
        base_image: payload.base_image,
    };

    let channel_result = channels_collection
        .insert_one(channel.to_owned(), None)
        .await;

    if let Err(err) = channel_result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some(format!("Failed to insert channel: {}", err.to_string())),
            }),
        );
    }

    let channel_id = channel_result.unwrap().inserted_id.as_object_id();

    let user_channel = UserChannel {
        id: Some(ObjectId::new()),
        user_id: Some(user_id),
        channel_id,
        is_owner: Some(true),
    };

    let user_channel_result = user_channels_collection
        .insert_one(user_channel.to_owned(), None)
        .await;

    if let Err(err) = user_channel_result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some(format!(
                    "Failed to insert user-channel relationship: {}",
                    err.to_string()
                )),
            }),
        );
    }

    let response_data = channels_collection
        .find_one(doc! { "_id": channel_id.unwrap() }, None)
        .await;

    match response_data {
        Ok(channel) => (
            StatusCode::CREATED,
            Json(MainResponse {
                success: true,
                data: Some(vec![channel]),
                error_message: None,
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some(format!(
                    "Failed to retrieve newly created channel: {}",
                    err.to_string()
                )),
            }),
        ),
    }
}
