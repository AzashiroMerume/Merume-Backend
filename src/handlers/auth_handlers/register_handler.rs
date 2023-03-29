use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::main_response::MainResponse;
use crate::utils::jwt::generate_jwt_token;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn register(
    State(client): State<Client>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let collection_name = "users";
    let collection = client
        .database("Merume")
        .collection::<User>(collection_name);

    // Validate the payload
    if payload.nickname.is_none() || payload.email.is_none() || payload.password.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MainResponse::<User> {
                success: false,
                data: None,
                error_message: Some("Missing fields".to_string()),
            }),
        )
            .into_response();
    }

    if let Some(_) = collection
        .find_one(doc! {"email": payload.email.clone().unwrap()}, None)
        .await
        .unwrap()
    {
        let main_response = MainResponse::<User> {
            success: false,
            data: None,
            error_message: Some("Email already in use".to_string()),
        };
        return (StatusCode::BAD_REQUEST, Json(main_response)).into_response();
    }

    let user = User {
        id: Some(ObjectId::new()),
        nickname: payload.nickname,
        email: payload.email,
        password: payload.password,
    };

    let result = collection.insert_one(user.to_owned(), None).await;

    match result {
        Ok(_) => {
            let jwt_secret = std::env::var("JWT_SECRET");

            let token =
                generate_jwt_token(&user.id.unwrap().to_string(), &jwt_secret.unwrap()).unwrap();
            return (
                StatusCode::CREATED,
                Json(MainResponse {
                    success: true,
                    data: Some(vec![token]),
                    error_message: None,
                }),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse::<User> {
                    success: false,
                    data: None,
                    error_message: Some("Failed to insert user".to_string()),
                }),
            )
                .into_response()
        }
    }
}
