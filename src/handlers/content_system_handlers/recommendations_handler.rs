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
    let user_preferences = user.preferences.unwrap();

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
        // Project two_week_subscribers field and percentage increase
        doc! {
            "$project": {
                "two_week_subscribers": 1,
                "percentage_increase": {
                    "$cond": {
                        "if": {
                            "$eq": [
                                { "$arrayElemAt": ["$two_week_subscribers", -2] },
                                0
                            ]
                        },
                        "then": 0,
                        "else": {
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
                        }
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
