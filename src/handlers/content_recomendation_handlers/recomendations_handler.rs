use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::{options::UpdateOptions, Client};

use crate::models::{post_model::Post, user_model::User};

//simple recomendation system that shows popular/trending channels/posts according to preferences
pub async fn recomendations(
    State(client): State<Client>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let user_preferences = user.preferences.unwrap();

    let post_collection = client.database("Merume").collection::<Post>("posts");
}
