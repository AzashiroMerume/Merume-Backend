use crate::{
    models::{channel_model::Channel, post_model::Post},
    responses::RecommendedContentResponse,
    utils::pagination::Pagination,
    AppState,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::doc;
use futures::StreamExt;
use mongodb::options::FindOneOptions;

pub async fn trendings(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let skip = pagination.page * pagination.limit;

    let pipeline = vec![
        // Filter channels to only those with at least two entries in the two_week_subscribers array
        doc! {
            "$match": {
                "subscriptions.two_week_subscribers.1": { "$exists": true }
            }
        },
        // Skip the first n documents
        doc! {
            "$skip": skip
        },
        // Project the last two entries in the two_week_subscribers array and calculate percentage increase
        doc! {
            "$project": {
                "channel": "$$ROOT",
                "two_week_subscribers": {
                    "$slice": ["$subscriptions.two_week_subscribers", -2]
                },
                "percentage_increase": {
                    "$multiply": [
                        {
                            "$divide": [
                                {
                                    "$subtract": [
                                        { "$arrayElemAt": ["$subscriptions.two_week_subscribers", -1] },
                                        { "$arrayElemAt": ["$subscriptions.two_week_subscribers", -2] }
                                    ]
                                },
                                { "$arrayElemAt": ["$subscriptions.two_week_subscribers", -2] }
                            ]
                        },
                        100
                    ]
                }
            }
        },
        // Sort channels by percentage increase in two-week subscribers in descending order
        doc! {
            "$sort": {
                "percentage_increase": -1
            }
        },
        // Limit the result to 20 channels
        doc! {
            "$limit": 20
        },
        // Replace the channel field with the full channel document
        doc! {
            "$replaceRoot": {
                "newRoot": "$channel"
            }
        },
    ];

    let cursor = state.db.channels_collection.aggregate(pipeline, None).await;

    let mut channel_post_vec = Vec::<(Channel, Post)>::default();

    match cursor {
        Ok(mut cursor) => {
            while let Some(channel_doc) = cursor.next().await {
                let channel: Channel =
                    bson::from_bson(bson::Bson::Document(channel_doc.unwrap())).unwrap();

                let latest_post = state
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
                    .await;

                if let Ok(Some(post)) = latest_post {
                    channel_post_vec.push((channel, post));
                } else if let Err(err) = latest_post {
                    eprintln!("Failed to find latest post: {}", err);
                }
            }

            return (
                StatusCode::OK,
                Json(RecommendedContentResponse {
                    success: true,
                    data: Some(channel_post_vec),
                    page: Some(pagination.page),
                    error_message: None,
                }),
            );
        }
        Err(err) => {
            eprintln!("Cursor error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecommendedContentResponse {
                    success: false,
                    data: None,
                    page: None,
                    error_message: Some("Failed to find recommendations".to_string()),
                }),
            );
        }
    }
}
