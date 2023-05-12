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

    let cursor = state.db.channels_collection.aggregate(pipeline, None).await;

    let mut channel_post_vec = Vec::<(Channel, Post)>::default();

    match cursor {
        Ok(mut cursor) => {
            while let Some(channel_doc) = cursor.next().await {
                println!("Channel doc: {:?}", channel_doc);
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
