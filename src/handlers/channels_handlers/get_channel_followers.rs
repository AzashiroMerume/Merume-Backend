use crate::{
    models::{user_channel_model::UserChannel, user_info_model::UserInfo, user_model::User},
    responses::ErrorResponse,
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use bson::{doc, oid::ObjectId, Document};
use futures::StreamExt;
use mongodb::Collection;
use serde::Serialize;
use std::{sync::Arc, usize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelFollowersResponse {
    pub data: Option<Vec<UserInfo>>,
}

pub async fn get_channel_followers(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
) -> Result<Json<ChannelFollowersResponse>, ErrorResponse> {
    let user_channel_collection: Collection<UserChannel> =
        state.db.user_channels_collection.clone();
    let user_collection: Collection<User> = state.db.users_collection.clone();

    let pipeline: Vec<Document> = vec![
        doc! {
            "$match": {
                "channel_id": channel_id
            }
        },
        doc! {
            "$project": {
                "user_id": 1,
            }
        },
    ];

    if let Ok(mut cursor) = user_channel_collection.aggregate(pipeline, None).await {
        let mut subscribers_info = Vec::new();

        while let Some(doc) = cursor.next().await {
            if let Ok(doc) = doc {
                if let Some(user_id) = doc.get_object_id("user_id").ok() {
                    if let Ok(user) = user_collection.find_one(doc! {"_id": user_id}, None).await {
                        if let Some(user) = user {
                            // Convert User model to UserInfo here
                            let user_info = UserInfo {
                                id: user.id,
                                nickname: user.nickname,
                                username: user.username,
                                email: None,
                                pfp_link: user.pfp_link,
                                preferences: user.preferences,
                                is_online: user.is_online,
                                last_time_online: user.last_time_online,
                            };
                            subscribers_info.push(user_info);
                        }
                    }
                }
            }
        }

        return Ok(Json(ChannelFollowersResponse {
            data: Some(subscribers_info),
        }));
    }

    Err(ErrorResponse::ServerError(None))
}
