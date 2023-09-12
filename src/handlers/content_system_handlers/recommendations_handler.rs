use crate::{
    models::{channel_model::Channel, user_model::User},
    responses::RecommendedChannelResponse,
    utils::pagination::Pagination,
    AppState,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::doc;
use futures::StreamExt;

pub async fn recommendations(
    State(state): State<AppState>,
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
        doc! {
            "$lookup": {
                "from": "read_posts",
                "let": { "channelId": "$channel._id" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$post_owner_id", "$$channelId"] },
                                    { "$eq": ["$user_id_who_read", user.id] }
                                ]
                            }
                        }
                    }
                ],
                "as": "read_posts"
            }
        },
        doc! {
            "$match": {
                "read_posts": {
                    "$size": 0
                }
            }
        },
        // Filter channels based on user preferences
        doc! {
            "$match": {
                "categories": {
                    "$in": user_preferences
                }
            }
        },
        // Project channel fields and percentage increase
        doc! {
            "$project": {
                // "_id": 0,  // Exclude the _id field from the root document
                "channel": "$$ROOT",
                "two_week_subscribers": 1,
                "percentage_increase": {
                    "$cond": {
                        "if": {
                            "$and": [
                                { "$isArray": "$two_week_subscribers" },
                                { "$gte": [ { "$size": "$two_week_subscribers" }, 2 ] }
                            ]
                        },
                        "then": {
                            "$multiply": [
                                {
                                    "$divide": [
                                        {
                                            "$subtract": [
                                                { "$arrayElemAt": ["$two_week_subscribers", -1] },
                                                { "$arrayElemAt": ["$two_week_subscribers", -2] }
                                            ]
                                        },
                                        { "$arrayElemAt": ["$two_week_subscribers", -2] }
                                    ]
                                },
                                100
                            ]
                        },
                        "else": 0
                    }
                }
            }
        },
        // Sort channels by percentage increase in two-week subscribers in descending order
        doc! {
            "$sort": {
                "percentage_increase": -1
            }
        },
        // Skip the first N channels based on the page number and limit to the next N channels
        doc! {
            "$skip": skip
        },
        doc! {
            "$limit": pagination.limit
        },
        // Create a new field called "channel" and assign the existing root document to it
        doc! {
            "$addFields": {
                "channel": "$channel"
            }
        },
        // Exclude the _id field from the projection
        doc! {
            "$project": {
                "_id": 0,
                "channel": 1,
            }
        },
        // Sort the channels again by percentage increase after the lookup
        doc! {
            "$sort": {
                "percentage_increase": -1
            }
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

    let mut result = Vec::<Channel>::default(); // Change the type to Vec<Channel>

    while let Some(channel_doc) = cursor.next().await {
        let recommended_channel: Channel = match channel_doc {
            Ok(channel_doc) => match bson::from_bson(bson::Bson::Document(channel_doc)) {
                Ok(recommended_channel) => recommended_channel,
                Err(err) => {
                    eprintln!("Failed to deserialize channel with latest post: {}", err);
                    continue;
                }
            },
            Err(err) => {
                eprintln!("Error retrieving channel document: {}", err);
                continue;
            }
        };

        result.push(recommended_channel); // Only add the channel data
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
