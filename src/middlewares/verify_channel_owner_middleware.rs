use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

pub async fn verify_channel_owner<B>(
    State(client): State<Client>,
    Extension(user_id): Extension<Option<ObjectId>>,
    Path(channel_id): Path<String>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    Ok(next.run(req).await)
}
