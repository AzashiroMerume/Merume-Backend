use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use mongodb::Client;

use crate::{models::user_model::User, responses::main_response::MainResponse};

pub async fn get_preferences(
    State(_client): State<Client>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let main_response = MainResponse {
        success: true,
        data: Some(user.preferences.unwrap()),
        error_message: None,
    };

    (StatusCode::OK, Json(main_response))
}
