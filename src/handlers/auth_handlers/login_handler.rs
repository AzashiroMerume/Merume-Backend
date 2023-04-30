use crate::models::auth_model::LoginPayload;
use crate::responses::MainResponse;
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
                Json(MainResponse {
                    success: false,
                    data: None,
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
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "Email or password are incorrect, please try a different email or sign up for a new account."
                        .to_string(),
                ),
            };
            return (StatusCode::NOT_FOUND, Json(main_response));
        }
        Err(err) => {
            eprintln!("Error finding user: {:?}", err);
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(main_response));
        }
    };

    //check hash_represantation of string
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, Json(main_response));
        }
    };

    //verify password using argon2 verifier
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        let main_response = MainResponse {
            success: false,
            data: None,
            error_message: Some("Email or password are incorrect, please try a different email or sign up for a new account.".to_string()),
        };
        return (StatusCode::UNAUTHORIZED, Json(main_response));
    }

    let jwt_secret = std::env::var("JWT_SECRET");

    let token = match generate_jwt_token(&user.id.to_string(), &jwt_secret.unwrap()) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error while matching token: {:?}", err);
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(main_response));
        }
    };

    let main_response = MainResponse {
        success: true,
        data: Some(vec![token]),
        error_message: None,
    };
    (StatusCode::OK, Json(main_response))
}
