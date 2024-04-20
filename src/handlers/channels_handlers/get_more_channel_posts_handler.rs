use crate::{
    models::post_model::Post, responses::ErrorResponse, utils::pagination::Pagination, AppState,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPostResponse {
    pub data: Option<Vec<Post>>,
}

pub async fn more_channel_posts(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<ChannelPostResponse>, ErrorResponse> {
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
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let posts = match cursor.try_collect::<Vec<Post>>().await {
        Ok(posts) => posts,
        Err(err) => {
            eprintln!("Failed to collect posts: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    Ok(Json(ChannelPostResponse { data: Some(posts) }))
}
