use crate::responses::main_response::MainResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn verify_auth() -> impl IntoResponse {
    let main_response: MainResponse<bool> = MainResponse {
        success: true,
        data: None,
        error_message: None,
    };

    (StatusCode::OK, Json(main_response))
}
