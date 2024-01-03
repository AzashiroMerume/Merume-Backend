use crate::{
    models::{user_info_model::UserInfo, user_model::User},
    responses::AuthResponse,
};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

pub async fn verify_auth(Extension(user): Extension<User>) -> impl IntoResponse {
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
            token: None,
            refresh_token: None,
            user_info: Some(user_info),
            error_message: None,
        }),
    )
}
