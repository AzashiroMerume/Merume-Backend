use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};

use crate::responses::OperationStatusResponse;
use crate::AppState;

pub async fn mark_as_read(
    State(state): State<AppState>,
    Path(post_id): Path<ObjectId>,
) -> impl IntoResponse {
}
