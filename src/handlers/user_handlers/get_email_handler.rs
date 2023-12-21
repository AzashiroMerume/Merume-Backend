use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Payload {
    nickname: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct EmailResponse {
    success: bool,
    email: Option<String>,
    error_message: Option<String>,
}

pub async fn get_email_by_nickname(
    State(state): State<AppState>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse {
    match payload.validate() {
        Ok(_) => {}
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(EmailResponse {
                    success: false,
                    email: None,
                    error_message: Some(err.to_string()),
                }),
            );
        }
    }

    let user = match state
        .db
        .users_collection
        .find_one(doc! {"nickname": &payload.nickname.to_lowercase()}, None)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(EmailResponse {
                    success: false,
                    email: None,
                    error_message: Some(
                        "Nickname not found. Please provide a valid nickname.".to_string(),
                    ),
                }),
            );
        }
        Err(err) => {
            eprintln!("Error finding user: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EmailResponse {
                    success: false,
                    email: None,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(EmailResponse {
            success: true,
            email: Some(user.email),
            error_message: None,
        }),
    )
}
