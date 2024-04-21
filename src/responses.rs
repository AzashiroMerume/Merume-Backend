use crate::models::{channel_model::Channel, user_info_model::UserInfo};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum ErrorResponse {
    BadRequest(Option<&'static str>),
    Unauthorized(Option<&'static str>),
    NotFound(Option<&'static str>),
    Forbidden(Option<&'static str>),
    Conflict(Option<&'static str>),
    UnprocessableEntity(Option<&'static str>),
    ServerError(Option<&'static str>),
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let (code, message) = match self {
            ErrorResponse::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg.unwrap_or("Bad request"))
            }
            ErrorResponse::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, msg.unwrap_or("Unauthorized"))
            }
            ErrorResponse::NotFound(msg) => (StatusCode::NOT_FOUND, msg.unwrap_or("Not found")),
            ErrorResponse::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.unwrap_or("Forbidden")),
            ErrorResponse::Conflict(msg) => (StatusCode::CONFLICT, msg.unwrap_or("Conflict")),
            ErrorResponse::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                msg.unwrap_or("Unprocessable entity"),
            ),
            ErrorResponse::ServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg.unwrap_or("Server error"),
            ),
        };

        (code, message).into_response()
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OperationStatusResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecommendedChannelResponse {
    pub data: Option<Vec<Channel>>,
    pub page: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthResponse {
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_info: Option<UserInfo>,
}
