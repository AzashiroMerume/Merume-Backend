use crate::{models::channel_model::Channel, responses::MainResponse, AppState};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use futures::StreamExt;

pub async fn trendings(State(state): State<AppState>) -> impl IntoResponse {
    let pipeline = vec![
        // Filter channels to only those with at least two entries in the two_week_subscribers array
        doc! {
            "$match": {
                "subscriptions.two_week_subscribers.1": { "$exists": true }
            }
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

    match cursor {
        Ok(cursor) => {
            let channels = cursor
                .map(|doc| {
                    let channel = bson::from_bson(bson::Bson::Document(doc.unwrap())).unwrap();
                    channel
                })
                .collect::<Vec<Channel>>()
                .await;

            (
                StatusCode::OK,
                Json(MainResponse {
                    success: true,
                    data: Some(vec![channels]),
                    error_message: None,
                }),
            )
        }
        Err(err) => {
            eprintln!("Cursor error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some("Failed to find recomendations".to_string()),
                }),
            )
        }
    }
}
