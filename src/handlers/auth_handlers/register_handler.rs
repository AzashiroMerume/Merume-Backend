use crate::models::user_info_model::UserInfo;
use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::AuthResponse;
use crate::utils::jwt::firebase_token_jwt::generate_access_jwt_token;
use crate::utils::jwt::refresh_token_jwt::generate_refresh_jwt_token;
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
                    refresh_token: None,
                    user_info: None,
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
                refresh_token: None,
                user_info: None,
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
                    refresh_token: None,
                    user_info: None,
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
                    refresh_token: None,
                    user_info: None,
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
        firebase_user_id: payload.clone().firebase_user_id,
        username: payload.username,
        nickname: payload.nickname.to_lowercase(),
        email: payload.email,
        password: hashed_password,
        pfp_link: None,
        preferences: None,
        liked: None,
        bookmarks: None,
        created_at: now,
        updated_at: now,
        is_online: false,
    };

    let result = state
        .db
        .users_collection
        .insert_one(user.to_owned(), None)
        .await;

    match result {
        Ok(inserted) => {
            if let Some(user_id) = inserted.inserted_id.as_object_id() {
                let access_token = match generate_access_jwt_token(
                    &payload.firebase_user_id,
                    state.firebase_token_encoding_key,
                    state.firebase_service_account,
                ) {
                    Ok(token) => token,
                    Err(err) => {
                        eprintln!("Error generating firebase JWT token: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(AuthResponse {
                                success: false,
                                token: None,
                                refresh_token: None,
                                user_info: None,
                                error_message: Some(
                                    "There was an error on the server side, try again later."
                                        .to_string(),
                                ),
                            }),
                        );
                    }
                };

                let refresh_token = match generate_refresh_jwt_token(
                    &user_id.to_hex(),
                    &state.refresh_jwt_secret,
                ) {
                    Ok(token) => token,
                    Err(err) => {
                        eprintln!("Error generating JWT token: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(AuthResponse {
                                success: false,
                                token: None,
                                refresh_token: None,
                                user_info: None,
                                error_message: Some(
                                    "There was an error on the server side, try again later."
                                        .to_string(),
                                ),
                            }),
                        );
                    }
                };

                let user_info = UserInfo {
                    id: inserted.inserted_id.as_object_id().unwrap(),
                    nickname: user.nickname,
                    username: user.username,
                    email: user.email,
                    pfp_link: user.pfp_link,
                    preferences: user.preferences,
                    is_online: user.is_online,
                };
                return (
                    StatusCode::CREATED,
                    Json(AuthResponse {
                        success: true,
                        token: Some(access_token),
                        refresh_token: Some(refresh_token),
                        user_info: Some(user_info),
                        error_message: None,
                    }),
                );
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse {
                        success: false,
                        token: None,
                        refresh_token: None,
                        user_info: None,
                        error_message: Some(
                            "There was an error on the server side, try again later.".to_string(),
                        ),
                    }),
                );
            }
        }
        Err(err) => {
            eprintln!("Error inserting user: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    refresh_token: None,
                    user_info: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        }
    }
}
