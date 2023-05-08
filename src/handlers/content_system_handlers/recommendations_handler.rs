use crate::{
    models::{channel_model::Channel, user_model::User},
    responses::MainResponse,
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
                    page: Some(pagination.page),
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
                    page: None,
                    error_message: Some("Failed to find recommendations".to_string()),
                }),
            )
        }
    }
}
