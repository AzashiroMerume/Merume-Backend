use crate::{
    models::{channel_model::Channel, post_model::Post, user_model::User},
    responses::RecommendedContentResponse,
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
use mongodb::options::FindOneOptions;

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
                Json(RecommendedContentResponse {
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
        // Filter channels based on user preferences
        doc! {
            "$match": {
                "categories": {
                    "$in": user_preferences
                }
            }
        },
        // Project channel fields, two_week_subscribers field, and percentage increase
        doc! {
            "$project": {
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
        // Replace the channel document with its fields
        doc! {
            "$replaceRoot": {
                "newRoot": "$channel"
            }
        },
        // Lookup the latest post for each channel
        doc! {
            "$lookup": {
                "from": "posts",
                "localField": "_id",
                "foreignField": "channel_id",
                "as": "latest_post"
            }
        },
        // Unwind the "latest_post" array
        doc! {
            "$unwind": {
                "path": "$latest_post",
                "preserveNullAndEmptyArrays": true
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
                Json(RecommendedContentResponse {
                    success: false,
                    data: None,
                    page: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    let mut result = Vec::<(Channel, Post)>::default();

    while let Some(channel_doc) = cursor.next().await {
        let channel: Channel = match bson::from_bson(bson::Bson::Document(channel_doc.unwrap())) {
            Ok(channel) => channel,
            Err(err) => {
                eprintln!("Failed to deserialize channel: {}", err);
                continue;
            }
        };

        let latest_post = match state
            .db
            .posts_collection
            .find_one(
                doc! {
                    "channel_id": channel.id
                },
                FindOneOptions::builder()
                    .sort(doc! {"created_at": -1})
                    .build(),
            )
            .await
        {
            Ok(Some(post)) => post,
            Err(err) => {
                eprintln!("Failed to find latest post: {}", err);
                continue;
            }
            _ => continue,
        };

        result.push((channel, latest_post));
    }

    (
        StatusCode::OK,
        Json(RecommendedContentResponse {
            success: true,
            data: Some(result),
            page: Some(pagination.page),
            error_message: None,
        }),
    )
}
