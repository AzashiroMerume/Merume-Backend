use crate::models::users_model::User;
use crate::responses::main_response::MainResponse;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::Client;

pub async fn login(State(client): State<Client>, Json(payload): Json<User>) -> impl IntoResponse {
    let collection = client.database("Merume").collection::<User>("users");

    let user = User {
        id: None,
        nickname: payload.nickname,
        email: payload.email,
        password: payload.password,
    };

    let inserted = collection.insert_one(user, None).await.unwrap();

    (
        StatusCode::CREATED,
        Json(MainResponse {
            success: true,
            data: Some(vec![inserted]),
            error_message: None,
        }),
    )
}
