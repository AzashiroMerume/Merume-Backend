use crate::AppState;

use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use bson::{doc, oid::ObjectId};

pub async fn verify_channel_owner<B>(
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let channel = state
        .db
        .channels_collection
        .find_one(
            doc! {"_id": ObjectId::parse_str(&channel_id).unwrap()},
            None,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let channel = channel.ok_or(StatusCode::NOT_FOUND)?;

    match channel.owner_id == user_id {
        true => Ok(next.run(req).await),
        false => Err(StatusCode::UNAUTHORIZED),
    }
}
