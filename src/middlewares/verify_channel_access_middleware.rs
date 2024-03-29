use crate::{models::author_model::Author, responses::OperationStatusResponse, AppState};
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};

pub async fn verify_channel_access(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
    _post_id: Option<Path<ObjectId>>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<OperationStatusResponse>)> {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
        .map_err(|err| {
            eprintln!("The database error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        })?;

    let channel = channel.ok_or((
        StatusCode::NOT_FOUND,
        Json(OperationStatusResponse {
            success: false,
            error_message: Some("Channel not found".to_string()),
        }),
    ))?;

    match (channel.author.id == author.id)
        || (channel
            .contributors
            .unwrap_or_default()
            .contains(&author.id))
    {
        true => {
            req.extensions_mut().insert(channel.challenge.current_day);
            return Ok(next.run(req).await);
        }
        false => Err((
            StatusCode::FORBIDDEN,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("Access forbidden".to_string()),
            }),
        )),
    }
}

pub async fn verify_channel_access_with_post_id(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path((channel_id, _post_id)): Path<(ObjectId, ObjectId)>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<OperationStatusResponse>)> {
    let channel = state
        .db
        .channels_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
        .map_err(|err| {
            eprintln!("The database error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        })?;

    let channel = channel.ok_or((
        StatusCode::NOT_FOUND,
        Json(OperationStatusResponse {
            success: false,
            error_message: Some("Channel not found".to_string()),
        }),
    ))?;

    match (channel.author.id == author.id)
        || (channel
            .contributors
            .unwrap_or_default()
            .contains(&author.id))
    {
        true => {
            req.extensions_mut().insert(channel.challenge.current_day);
            return Ok(next.run(req).await);
        }
        false => Err((
            StatusCode::FORBIDDEN,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("Access forbidden".to_string()),
            }),
        )),
    }
}
