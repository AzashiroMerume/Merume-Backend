use crate::{
    models::channel_model::Channel,
    responses::{ErrorResponse, RecommendedChannelResponse},
    utils::pagination::Pagination,
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use bson::doc;
use futures::StreamExt;
use std::sync::Arc;

pub async fn trendings(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<RecommendedChannelResponse>, ErrorResponse> {
    let skip = pagination.page * pagination.limit;

    let pipeline = vec![
        // Project channel fields and percentage increase
        doc! {
            "$project": {
                "_id": 0,
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
    ];

    let mut cursor = match state.db.channels_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(err) => {
            eprintln!("Cursor error: {}", err);
            return Err(ErrorResponse::ServerError(None));
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

    Ok(Json(RecommendedChannelResponse {
        data: Some(result),
        page: Some(pagination.page),
    }))
}
