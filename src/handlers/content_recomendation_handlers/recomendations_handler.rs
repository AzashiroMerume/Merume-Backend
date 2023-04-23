use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::{options::UpdateOptions, Client};

use crate::models::{user_model::User, post_model::Post};

pub async fn recomendations(
    State(client): State<Client>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let user_preferences = user.preferences.unwrap();

    let post_collection = client.database("Merume").collection::<Post>("posts");
}
