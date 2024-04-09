use crate::{
    models::channel_model::Channel, models::user_model::User,
    responses::RecommendedChannelResponse, utils::pagination::Pagination, AppState,
};
use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::doc;
use futures::StreamExt;

pub async fn recommendations(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let user_preferences = match user.preferences {
        Some(preferences) => preferences,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(RecommendedChannelResponse {
                    success: false,
                    data: None,
                    page: None,
                    error_message: Some(
                        "User does not have any preferences, try to add preferences".to_string(),
                    ),
                }),
            )
        }
    };

    let skip = pagination.page * pagination.limit;

    let pipeline = vec![
        // Match channels based on user preferences
        doc! {
            "$match": {
                "categories": {
                    "$in": user_preferences
                }
            }
        },
        // Lookup user channels to find channels that the user follows
        doc! {
            "$lookup": {
                "from": "user_channels", // Name of the user_channels collection
                "localField": "_id", // Field in the channels collection
                "foreignField": "channel_id", // Field in the user_channels collection
                "as": "user_channels"
            }
        },
        // Match channels where the user is not the owner and is not following
        doc! {
            "$match": {
                "user_channels.user_id": {
                    "$ne": user.id
                }
            }
        },
        // Sort channels by some criteria, e.g., current_following in descending order
        doc! {
            "$sort": {
                "followers.current_following": -1
            }
        },
        // Skip the first N channels based on the page number and limit to the next N channels
        doc! {
            "$skip": skip
        },
        doc! {
            "$limit": pagination.limit
        },
    ];

    let mut cursor = match state.db.channels_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(err) => {
            eprintln!("Cursor error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecommendedChannelResponse {
                    success: false,
                    data: None,
                    page: None,
                    error_message: Some("There was an error on the server".to_string()),
                }),
            );
        }
    };

    let mut result = Vec::<Channel>::default();

    while let Some(channel_doc) = cursor.next().await {
        let recommended_channel: Channel = match channel_doc {
            Ok(channel_doc) => match bson::from_bson(bson::Bson::Document(channel_doc)) {
                Ok(recommended_channel) => recommended_channel,
                Err(err) => {
                    eprintln!("Failed to deserialize channel: {}", err);
                    continue;
                }
            },
            Err(err) => {
                eprintln!("Error retrieving channel document: {}", err);
                continue;
            }
        };

        result.push(recommended_channel);
    }

    (
        StatusCode::OK,
        Json(RecommendedChannelResponse {
            success: true,
            data: Some(result),
            page: Some(pagination.page),
            error_message: None,
        }),
    )
}
