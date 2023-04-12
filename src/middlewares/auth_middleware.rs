use crate::{models::user_model::User, utils::jwt::verify_token};
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

pub async fn auth<B>(
    State(client): State<Client>,
    mut req: Request<B>,
    next: Next<B>,
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

    let collection: Collection<User> = client.database("Merume").collection("users");
    match collection.find_one(doc! {"_id": user_id}, None).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}

pub async fn auth_with_user<B>(
    State(client): State<Client>,
    mut req: Request<B>,
    next: Next<B>,
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

    let collection: Collection<User> = client.database("Merume").collection("users");
    let user = match collection.find_one(doc! {"_id": user_id}, None).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
