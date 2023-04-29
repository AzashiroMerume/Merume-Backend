use crate::{utils::jwt::verify_token, AppState};

use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use bson::{doc, oid::ObjectId};

pub async fn auth<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
    pass_full_user: Option<bool>,
) -> Result<Response, StatusCode> {
    let auth_header = match req.headers().get(http::header::AUTHORIZATION) {
        Some(header) => header.to_str().ok(),
        None => None,
    };

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token_claims = verify_token(token, &jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = ObjectId::parse_str(&token_claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = match state
        .db
        .users_collection
        .find_one(doc! {"_id": user_id}, None)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let Some(true) = pass_full_user {
        req.extensions_mut().insert(user);
    } else {
        req.extensions_mut().insert(user_id);
    }

    Ok(next.run(req).await)
}
