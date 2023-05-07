use crate::{
    models::{channel_model::Channel, user_model::User},
    responses::MainResponse,
    AppState,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::doc;
use futures::StreamExt;

pub async fn recommendations(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let user_preferences = user.preferences.unwrap();

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
        // Limit the result to 20 channels
        doc! {
            "$limit": 20
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
