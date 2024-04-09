use crate::{
    models::post_model::Post, responses::ChannelPostResponse, utils::pagination::Pagination,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use std::sync::Arc;

pub async fn more_channel_posts(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let skip = pagination.page * pagination.limit;

    let filter = doc! {"channel_id": channel_id};

    let mut options = FindOptions::default();
    options.sort = Some(doc! {"created_at": -1});
    options.skip = Some(skip as u64);
    options.limit = Some(pagination.limit as i64);

    let cursor = match state.db.posts_collection.find(filter, options).await {
        Ok(cursor) => cursor,
        Err(err) => {
            eprintln!("Cursor error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChannelPostResponse {
                    success: false,
                    data: None,
                    error_message: Some("There was an error on the server".to_string()),
                }),
            );
        }
    };

    let posts = match cursor.try_collect::<Vec<Post>>().await {
        Ok(posts) => posts,
        Err(err) => {
            eprintln!("Failed to collect posts: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChannelPostResponse {
                    success: false,
                    data: None,
                    error_message: Some("There was an error on the server".to_string()),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(ChannelPostResponse {
            success: true,
            data: Some(posts),
            error_message: None,
        }),
    )
}
