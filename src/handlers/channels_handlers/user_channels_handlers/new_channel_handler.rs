use crate::{
    models::{
        channel_model::{Channel, ChannelPayload, Subscriptions},
        user_channel_model::UserChannel,
    },
    responses::MainResponse,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use validator::Validate;

pub async fn new_channel(
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Json(payload): Json<ChannelPayload>,
) -> impl IntoResponse {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    let now = Utc::now();

    //init subscriptions for channel
    let subscriptions = Subscriptions {
        current_subscriptions: 0,
        monthly_subscribers: vec![0],
        yearly_subscribers: vec![0],
        two_week_subscribers: vec![0],
        last_updated: now,
    };

    let channel = Channel {
        id: ObjectId::new(),
        owner_id: user_id,
        name: payload.name,
        channel_type: payload.channel_type,
        description: payload.description,
        categories: payload.categories,
        subscriptions,
        current_challenge_day: 1,
        base_image: payload.base_image,
        created_at: now,
    };

    let channel_result = state
        .db
        .channels_collection
        .insert_one(channel.to_owned(), None)
        .await;

    if let Err(err) = channel_result {
        eprintln!("Failed to insert channel: {}", err.to_string());

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            }),
        );
    }

    let channel_id = channel_result.unwrap().inserted_id.as_object_id();

    let user_channel = UserChannel {
        id: ObjectId::new(),
        user_id,
        channel_id: channel_id.unwrap(),
        is_owner: true,
        subscribed_at: None,
        created_at: Some(now),
    };

    let user_channel_result = state
        .db
        .user_channels_collection
        .insert_one(user_channel.to_owned(), None)
        .await;

    if let Err(err) = user_channel_result {
        eprintln!(
            "Failed to insert user-channel relationship: {}",
            err.to_string()
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            }),
        );
    }

    let response_data = state
        .db
        .channels_collection
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
        Err(err) => {
            eprintln!(
                "Failed to retrieve newly created channel: {}",
                err.to_string()
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    }
}
