use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::{options::UpdateOptions, Client};

use crate::models::user_model::User;

pub async fn recomendations(
    State(client): State<Client>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
}
