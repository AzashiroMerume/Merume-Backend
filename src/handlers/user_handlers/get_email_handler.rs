use crate::{responses::ErrorResponse, AppState};
use axum::{extract::State, Json};
use bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Payload {
    nickname: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EmailResponse {
    email: Option<String>,
}

pub async fn get_email_by_nickname(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Payload>,
) -> Result<Json<EmailResponse>, ErrorResponse> {
    match payload.validate() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error while validating error: {}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
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
            return Err(ErrorResponse::NotFound(None));
        }
        Err(err) => {
            eprintln!("Error finding user: {:?}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    Ok(Json(EmailResponse {
        email: Some(user.email),
    }))
}
