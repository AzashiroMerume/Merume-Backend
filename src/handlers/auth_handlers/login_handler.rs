use crate::models::{auth_model::LoginPayload, user_info_model::UserInfo};
use crate::responses::AuthResponse;
use crate::utils::jwt::generate_jwt_token;
use crate::AppState;

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use validator::Validate;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    error_message: Some(err.to_string()),
                }),
            );
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
                return (StatusCode::NOT_FOUND, Json(AuthResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    error_message: Some(
                        "Email or password are incorrect, please try a different email or sign up for a new account."
                            .to_string(),
                    ),
                }));
            }
            Err(err) => {
                eprintln!("Error finding user: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse {
                        success: false,
                        token: None,
                        user_info: None,
                        error_message: Some(
                            "There was an error on the server side, try again later.".to_string(),
                        ),
                    }),
                );
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
                return (StatusCode::NOT_FOUND, Json(AuthResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    error_message: Some(
                        "Nickname or password are incorrect, please try a different nickname or sign up for a new account."
                            .to_string(),
                    ),
                }));
            }
            Err(err) => {
                eprintln!("Error finding user: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse {
                        success: false,
                        token: None,
                        user_info: None,
                        error_message: Some(
                            "There was an error on the server side, try again later.".to_string(),
                        ),
                    }),
                );
            }
        }
    };

    // Check hash_representation of the password
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                success: false,
                token: None,
                user_info: None,
                error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
            }));
        }
    };

    // Verify the password using the argon2 verifier
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, Json(AuthResponse {
            success: false,
            token: None,
            user_info: None,
            error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
        }));
    }

    let token = match generate_jwt_token(
        &payload.firebase_user_id,
        state.firebase_token_encoding_key,
        state.firebase_service_account,
    ) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error while generating token: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    let user_info = UserInfo {
        id: user.id,
        nickname: user.nickname,
        username: user.username,
        email: user.email,
        preferences: user.preferences,
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: Some(token),
            user_info: Some(user_info),
            error_message: None,
        }),
    )
}
