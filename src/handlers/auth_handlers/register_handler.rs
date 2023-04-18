use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::main_response::MainResponse;
use crate::utils::jwt::generate_jwt_token;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use mongodb::Client;
use validator::Validate;

pub async fn register(
    State(client): State<Client>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let collection = client.database("Merume").collection::<User>("users");

    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(e.to_string()),
                }),
            );
        }
    }

    //checking email and nickname for existence
    match collection
        .find_one(doc! {"email": payload.email.clone()}, None)
        .await
    {
        Ok(Some(_)) => {
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some("Email already in use. Please try to sign in.".to_string()),
            };
            return (StatusCode::BAD_REQUEST, Json(main_response));
        }
        Err(e) => {
            eprintln!("Error checking email: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
        _ => {} // continue checking for nickname
    }

    match collection
        .find_one(doc! {"nickname": payload.nickname.clone()}, None)
        .await
    {
        Ok(Some(_)) => {
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some("Nickname already in use. Please choose another.".to_string()),
            };
            return (StatusCode::BAD_REQUEST, Json(main_response));
        }
        Err(e) => {
            eprintln!("Error checking nickname: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
        _ => {} // continue with registration
    }

    //hashing password using default argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            eprintln!("Error hashing password: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    let now = Utc::now();

    let user = User {
        id: ObjectId::new(),
        username: payload.username,
        nickname: payload.nickname,
        email: payload.email,
        password: hashed_password,
        preferences: None,
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
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        }
    }
}
