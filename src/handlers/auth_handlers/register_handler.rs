use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::AuthResponse;
use crate::utils::jwt::generate_jwt_token;
use crate::AppState;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use validator::Validate;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    //checking email and nickname for existence
    match state
        .db
        .users_collection
        .find_one(
            doc! {
                "$or": [{"email": payload.email.clone()}, {"nickname": payload.nickname.clone().to_lowercase()}]
            },
            None,
        )
        .await
    {
        Ok(Some(existing_user)) => {
            let error_message = if existing_user.nickname == payload.nickname.to_lowercase() {
                "Nickname already in use. Please try sign in."
            } else {
                "Email already in use. Please try sign in."
            };

            let main_response = AuthResponse {
                success: false,
                token: None,
                error_message: Some(error_message.to_string()),
            };
            return (StatusCode::CONFLICT, Json(main_response));
        }
        Err(err) => {
            eprintln!("Error checking email and nickname: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
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
        Err(err) => {
            eprintln!("Error hashing password: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
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
        nickname: payload.nickname.to_lowercase(),
        email: payload.email,
        password: hashed_password,
        preferences: None,
        liked: None,
        bookmarks: None,
        created_at: now,
        updated_at: now,
    };

    let result = state
        .db
        .users_collection
        .insert_one(user.to_owned(), None)
        .await;

    match result {
        Ok(_) => {
            let jwt_secret = std::env::var("JWT_SECRET");

            let token = generate_jwt_token(&user.id.to_string(), &jwt_secret.unwrap()).unwrap();
            return (
                StatusCode::CREATED,
                Json(AuthResponse {
                    success: true,
                    token: Some(token),
                    error_message: None,
                }),
            );
        }
        Err(err) => {
            eprintln!("Error inserting user: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        }
    }
}
