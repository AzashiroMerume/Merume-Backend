use crate::models::user_info_model::UserInfo;
use crate::models::{auth_model::RegisterPayload, user_model::User};
use crate::responses::{AuthResponse, ErrorResponse};
use crate::utils::jwt::firebase_token_jwt::generate_access_jwt_token;
use crate::utils::jwt::refresh_token_jwt::generate_refresh_jwt_token;
use crate::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use std::sync::Arc;
use validator::Validate;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponse>, ErrorResponse> {
    match payload.validate() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Error validating payload: {:?}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
        }
    }

    // Checking email and nickname for existence
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
        Ok(Some(_)) => {
            return Err(ErrorResponse::ServerError(None));
        }
        Err(err) => {
            eprintln!("Error checking email and nickname: {:?}", err);
            return Err(ErrorResponse::ServerError(None));
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
            return Err(ErrorResponse::ServerError(None));
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
        time_zone: payload.time_zone,
        created_at: now,
        updated_at: now,
        is_online: false,
        last_time_online: now,
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
                    state.firebase_config.token_encoding_key.clone(),
                    state.firebase_config.service_account.clone(),
                ) {
                    Ok(token) => token,
                    Err(err) => {
                        eprintln!("Error generating firebase JWT token: {:?}", err);
                        return Err(ErrorResponse::ServerError(None));
                    }
                };

                let refresh_token = match generate_refresh_jwt_token(
                    &user_id.to_hex(),
                    &state.refresh_jwt_secret,
                ) {
                    Ok(token) => token,
                    Err(err) => {
                        eprintln!("Error generating JWT token: {:?}", err);
                        return Err(ErrorResponse::ServerError(None));
                    }
                };

                let user_info = UserInfo {
                    id: inserted.inserted_id.as_object_id().unwrap(),
                    nickname: user.nickname,
                    username: user.username,
                    email: Some(user.email),
                    pfp_link: user.pfp_link,
                    preferences: user.preferences,
                    is_online: user.is_online,
                    last_time_online: user.last_time_online,
                };
                Ok(Json(AuthResponse {
                    token: Some(access_token),
                    refresh_token: Some(refresh_token),
                    user_info: Some(user_info),
                }))
            } else {
                Err(ErrorResponse::ServerError(None))
            }
        }
        Err(err) => {
            eprintln!("Error inserting user: {:?}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
