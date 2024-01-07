use crate::{
    models::{user_channel_model::UserChannel, user_info_model::UserInfo, user_model::User},
    responses::ChannelFollowersResponse,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId, Document};
use futures::stream::StreamExt;
use mongodb::Collection; // Import StreamExt to work with streams

pub async fn get_channel_followers(
    State(state): State<AppState>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
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
                                email: user.email,
                                pfp_link: user.pfp_link,
                                preferences: user.preferences,
                            };
                            subscribers_info.push(user_info);
                        }
                    }
                }
            }
        }

        return (
            StatusCode::OK,
            Json(ChannelFollowersResponse {
                success: true,
                data: Some(subscribers_info),
                error_message: None,
            }),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ChannelFollowersResponse {
            success: false,
            data: None,
            error_message: Some(
                "There was an error on the server side, try again later.".to_string(),
            ),
        }),
    )
}
