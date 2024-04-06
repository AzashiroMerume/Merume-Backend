use crate::{models::author_model::Author, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::doc;
use futures::TryStreamExt;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct ReadTrackersResponse {
    success: bool,
    data: Option<HashMap<String, i32>>,
    error_message: Option<String>,
}

pub async fn get_read_tracker(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
) -> impl IntoResponse {
    let pipeline = vec![
        doc! {
            "$match": { "user_id": author.id }
        },
        doc! {
            "$lookup": {
                "from": "channel_read_trackers",
                "let": { "channel_id": "$channel_id", "user_id": author.id },
                "pipeline": [
                    { "$match": {
                        "$expr": { "$and": [
                            { "$eq": [ "$user_id", "$$user_id" ] },
                            { "$eq": [ "$channel_id", "$$channel_id" ] }
                        ]}
                    }},
                    { "$project": { "_id": 0, "last_read_post_id": 1 }}
                ],
                "as": "read_trackers"
            }
        },
        doc! {
            "$unwind": {
                "path": "$read_trackers",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$lookup": {
                "from": "posts",
                "localField": "read_trackers.last_read_post_id",
                "foreignField": "_id",
                "as": "last_read_post"
            }
        },
        doc! {
            "$unwind": {
                "path": "$last_read_post",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$lookup": {
                "from": "posts",
                "let": { "last_read_post_date": "$last_read_post.created_at", "channel_id": "$channel_id" },
                "pipeline": [
                    { "$match": {
                        "$expr": {
                            "$and": [
                                { "$eq": [ "$channel_id", "$$channel_id" ] },
                                { "$gt": [ "$created_at", "$$last_read_post_date" ] }
                            ]
                        }
                    }},
                    { "$count": "unread_count" }
                ],
                "as": "unread_count"
            }
        },
        doc! {
            "$unwind": {
                "path": "$unread_count",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$match": {
                "unread_count": { "$exists": true, "$ne": null }
            }
        },
        doc! {
            "$project": {
                "_id": "$channel_id",
                "unread_count": "$unread_count.unread_count"
            }
        },
    ];

    let mut unread_counts_map: HashMap<String, i32> = HashMap::new();

    match state
        .db
        .user_channels_collection
        .aggregate(pipeline, None)
        .await
    {
        Ok(mut cursor) => {
            while let Ok(result) = cursor.try_next().await {
                match result {
                    Some(doc) => {
                        let unread_count = match doc.get("unread_count") {
                            Some(count) => match count.as_i32() {
                                Some(val) => val,
                                None => continue,
                            },
                            None => continue,
                        };

                        let channel_id = match doc.get("_id") {
                            Some(channel_id) => match channel_id.as_object_id() {
                                Some(id) => id.to_hex(),
                                None => continue,
                            },
                            None => continue,
                        };

                        unread_counts_map.insert(channel_id, unread_count);
                    }
                    None => {
                        break;
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error executing aggregation pipeline: {:?}", err);
            return (
                StatusCode::OK,
                Json(ReadTrackersResponse {
                    success: false,
                    data: None,
                    error_message: Some("Error on the server side".to_string()),
                }),
            );
        }
    };

    let response = if unread_counts_map.is_empty() {
        None
    } else {
        Some(unread_counts_map)
    };

    (
        StatusCode::OK,
        Json(ReadTrackersResponse {
            success: true,
            data: response,
            error_message: None,
        }),
    )
}
