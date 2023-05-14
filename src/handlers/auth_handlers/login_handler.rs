use crate::models::auth_model::LoginPayload;
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
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    // Find the user with the given email
    let user = match state
        .db
        .users_collection
        .find_one(doc! {"email": &payload.email}, None)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(AuthResponse {
                success: false,
                token: None,
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
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    //check hash_represantation of string
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                success: false,
                token: None,
                error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
            }));
        }
    };

    //verify password using argon2 verifier
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, Json(AuthResponse {
            success: false,
            token: None,
            error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
        }));
    }

    let jwt_secret = std::env::var("JWT_SECRET");

    let token = match generate_jwt_token(&user.id.to_string(), &jwt_secret.unwrap()) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error while matching token: {:?}", err);
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

    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: Some(token),
            error_message: None,
        }),
    )
}
