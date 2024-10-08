use crate::{
    models::{
        author_model::Author,
        channel_model::{Challenge, Channel, ChannelPayload, Followers},
        user_channel_model::UserChannel,
    },
    responses::ErrorResponse,
    AppState,
};
use axum::{extract::State, http::StatusCode, Extension, Json};
use bson::oid::ObjectId;
use chrono::Utc;
use std::sync::Arc;
use validator::Validate;

pub async fn new_channel(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Json(payload): Json<ChannelPayload>,
) -> Result<StatusCode, ErrorResponse> {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            eprintln!("Error while validating payload: {}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
        }
    }

    let now = Utc::now();

    let challenge = Challenge {
        challenge_type: ChannelPayload::challenge_type_enum(&payload),
        goal: payload.goal,
        points: 0,
        current_day: 1,
        streak: 1,
        missed_count: 0,
        missed_days: None,
    };

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
        author: author.to_owned(),
        name: payload.name.to_owned(),
        visibility: ChannelPayload::visibility_enum(&payload),
        description: payload.description,
        categories: payload.categories,
        challenge,
        contributors: payload.contributors,
        followers,
        channel_pfp_link: payload.channel_pfp_link,
        created_at: now,
    };

    let channel_result = state
        .db
        .channels_collection
        .insert_one(channel.to_owned(), None)
        .await;

    if let Err(err) = channel_result {
        eprintln!("Failed to insert channel: {}", err);

        return Err(ErrorResponse::ServerError(None));
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
        eprintln!("Failed to insert user-channel relationship: {}", err);
        return Err(ErrorResponse::ServerError(None));
    }

    Ok(StatusCode::CREATED)
}
