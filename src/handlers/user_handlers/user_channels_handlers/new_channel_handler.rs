use crate::{
    models::{
        author_model::Author,
        channel_model::{Channel, ChannelPayload, Followers},
        user_channel_model::UserChannel,
    },
    responses::OperationStatusResponse,
    AppState,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::oid::ObjectId;
use chrono::Utc;
use validator::Validate;

pub async fn new_channel(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Json(payload): Json<ChannelPayload>,
) -> impl IntoResponse {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    let now = Utc::now();

    //init followers for channel
    let followers = Followers {
        current_following: 0,
        monthly_followers: vec![0],
        yearly_followers: vec![0],
        two_week_followers: vec![0],
        last_updated: now,
    };

    let author = Author {
        id: author.id,
        nickname: author.nickname,
        username: author.username,
        pfp_link: author.pfp_link,
        is_online: None,
        last_time_online: None,
    };

    let channel = Channel {
        id: ObjectId::new(),
        channel_type: payload.channel_type,
        author: author.to_owned(),
        name: payload.name,
        goal: payload.goal,
        channel_visibility: payload.channel_visibility,
        description: payload.description,
        categories: payload.categories,
        participants: payload.participants,
        followers,
        current_challenge_day: 1,
        channel_profile_picture_url: payload.channel_profile_picture_url,
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
            Json(OperationStatusResponse {
                success: false,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            }),
        );
    }

    let channel_id = channel_result.unwrap().inserted_id.as_object_id();

    let user_channel = UserChannel {
        id: ObjectId::new(),
        user_id: author.id,
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
            Json(OperationStatusResponse {
                success: false,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            }),
        );
    }

    (
        StatusCode::CREATED,
        Json(OperationStatusResponse {
            success: true,
            error_message: None,
        }),
    )
}
