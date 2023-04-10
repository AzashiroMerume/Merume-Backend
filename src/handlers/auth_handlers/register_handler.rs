use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::main_response::MainResponse;
use crate::utils::jwt::generate_jwt_token;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use mongodb::Client;

pub async fn register(
    State(client): State<Client>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let collection = client.database("Merume").collection::<User>("users");

    // Validate the payload
    if payload.nickname.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some("Missing fields".to_string()),
            }),
        );
    }

    if let Some(_) = collection
        .find_one(doc! {"email": payload.email.clone()}, None)
        .await
        .unwrap()
    {
        let main_response = MainResponse {
            success: false,
            data: None,
            error_message: Some("Email already in use".to_string()),
        };
        return (StatusCode::BAD_REQUEST, Json(main_response));
    }

    let now = Utc::now();

    let user = User {
        id: ObjectId::new(),
        nickname: payload.nickname,
        email: payload.email,
        password: payload.password,
        created_at: now,
        updated_at: now,
    };

    let result = collection.insert_one(user.to_owned(), None).await;

    match result {
        Ok(_) => {
            let jwt_secret = std::env::var("JWT_SECRET");

            let token = generate_jwt_token(&user.id.to_string(), &jwt_secret.unwrap()).unwrap();
            return (
                StatusCode::CREATED,
                Json(MainResponse {
                    success: true,
                    data: Some(vec![token]),
                    error_message: None,
                }),
            );
        }
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some("Failed to insert user".to_string()),
                }),
            )
        }
    }
}
