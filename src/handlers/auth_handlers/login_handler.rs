use crate::models::{auth_model::LoginPayload, user_info_model::UserInfo};
use crate::responses::{AuthResponse, ErrorResponse};
use crate::utils::jwt::firebase_token_jwt::generate_access_jwt_token;
use crate::utils::jwt::refresh_token_jwt::generate_refresh_jwt_token;
use crate::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{extract::State, Json};
use bson::doc;
use std::sync::Arc;
use validator::Validate;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, ErrorResponse> {
    match payload.validate() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Error validating payload: {:?}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
        }
    }

    let user = if payload.by_email {
        // Find the user with the given email
        match state
            .db
            .users_collection
            .find_one(doc! {"email": &payload.identifier}, None)
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(ErrorResponse::NotFound(None));
            }
            Err(err) => {
                eprintln!("Error finding user: {:?}", err);
                return Err(ErrorResponse::ServerError(None));
            }
        }
    } else {
        // Find the user with the given nickname
        match state
            .db
            .users_collection
            .find_one(doc! {"nickname": &payload.identifier.to_lowercase()}, None)
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(ErrorResponse::NotFound(None));
            }
            Err(err) => {
                eprintln!("Error finding user: {:?}", err);
                return Err(ErrorResponse::ServerError(None));
            }
        }
    };

    // Check hash_representation of the password
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(ErrorResponse::Unauthorized(None));
        }
    };

    // Verify the password using the argon2 verifier
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(ErrorResponse::Unauthorized(None));
    }

    let token = match generate_access_jwt_token(
        &payload.firebase_user_id,
        state.firebase_config.token_encoding_key.clone(),
        state.firebase_config.service_account.clone(),
    ) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error while generating token: {:?}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let refresh_token =
        match generate_refresh_jwt_token(&user.id.to_hex(), &state.refresh_jwt_secret) {
            Ok(token) => token,
            Err(err) => {
                eprintln!("Error generating JWT token: {:?}", err);
                return Err(ErrorResponse::ServerError(None));
            }
        };

    let user_info = UserInfo {
        id: user.id,
        nickname: user.nickname,
        username: user.username,
        email: Some(user.email),
        pfp_link: user.pfp_link,
        preferences: user.preferences,
        is_online: user.is_online,
        last_time_online: user.last_time_online,
    };

    Ok(Json(AuthResponse {
        token: Some(token),
        refresh_token: Some(refresh_token),
        user_info: Some(user_info),
    }))
}
