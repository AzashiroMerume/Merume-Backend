use crate::{models::author_model::Author, responses::OperationStatusResponse, AppState};
use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};

pub async fn verify_channel_owner<B>(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
    mut req: Request<B>,
    next: Next<B>,
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

    match channel.author.id == author.id {
        true => {
            req.extensions_mut().insert(channel.current_challenge_day);
            return Ok(next.run(req).await);
        }
        false => Err((
            StatusCode::CONFLICT,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("CONFLICT".to_string()),
            }),
        )),
    }
}

pub async fn verify_channel_owner_with_post_id<B>(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path((channel_id, _post_id)): Path<(ObjectId, ObjectId)>,
    mut req: Request<B>,
    next: Next<B>,
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

    match channel.author.id == author.id {
        true => {
            req.extensions_mut().insert(channel.current_challenge_day);
            return Ok(next.run(req).await);
        }
        false => Err((
            StatusCode::CONFLICT,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("CONFLICT".to_string()),
            }),
        )),
    }
}
