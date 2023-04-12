use crate::models::{auth_model::LoginPayload, user_model::User};
use crate::responses::main_response::MainResponse;
use crate::utils::jwt::generate_jwt_token;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use mongodb::Client;

pub async fn login(
    State(client): State<Client>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let collection_name = "users";
    let collection = client
        .database("Merume")
        .collection::<User>(collection_name);

    if payload.email.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(MainResponse {
                success: false,
                data: None,
                error_message: Some("Please fill in all required fields".to_string()),
            }),
        );
    }

    // Find the user with the given email
    let user = match collection
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
        Err(e) => {
            eprintln!("Error finding user: {:?}", e);
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later."
                        .to_string(),
                ),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(main_response));
        }
    };

    // Check if the password is correct
    if user.password != payload.password {
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
        Err(e) => {
            eprintln!("Error while matching token: {:?}", e);
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
