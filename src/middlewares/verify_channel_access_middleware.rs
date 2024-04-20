use crate::{models::author_model::Author, responses::ErrorResponse, AppState};
use axum::{
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn verify_channel_access(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
    _post_id: Option<Path<ObjectId>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
        .map_err(|err| {
            eprintln!("The database error: {}", err);

            ErrorResponse::ServerError(None)
        })?;

    let channel = channel.ok_or(ErrorResponse::NotFound(None))?;

    match (channel.author.id == author.id)
        || (channel
            .contributors
            .unwrap_or_default()
            .contains(&author.id))
    {
        true => {
            req.extensions_mut().insert(channel.challenge.current_day);
            Ok(next.run(req).await)
        }
        false => Err(ErrorResponse::Forbidden(None)),
    }
}

pub async fn verify_channel_access_with_post_id(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Path((channel_id, _post_id)): Path<(ObjectId, ObjectId)>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
        .map_err(|err| {
            eprintln!("The database error: {}", err);
            ErrorResponse::ServerError(None)
        })?;

    let channel = channel.ok_or(ErrorResponse::NotFound(None))?;

    match (channel.author.id == author.id)
        || (channel
            .contributors
            .unwrap_or_default()
            .contains(&author.id))
    {
        true => {
            req.extensions_mut().insert(channel.challenge.current_day);
            Ok(next.run(req).await)
        }
        false => Err(ErrorResponse::Forbidden(None)),
    }
}
