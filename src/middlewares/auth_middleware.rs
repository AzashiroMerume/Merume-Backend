use crate::utils::jwt::verify_token;
use axum::{
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};

// Make validation for id here not in handler
pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = if let Some(token) = auth_header {
        token
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let jwt_secret = std::env::var("JWT_SECRET");

    if let Ok(token_info) = verify_token(token, &jwt_secret.unwrap()) {
        req.extensions_mut().insert(token_info);
        return Ok(next.run(req).await);
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}

// pub async fn verify_user_by_id(user_id: &str) -> bool {}
