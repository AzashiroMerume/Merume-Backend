use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn create_post(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, Json("hello"))
}
