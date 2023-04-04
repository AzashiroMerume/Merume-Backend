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
        let user_id = match ObjectId::parse_str(&token_info.sub) {
            Ok(id) => id,
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
        };

        //access collection to check user_id for existence in user table
        let collection: Collection<User> = client.database("Merume").collection("users");

        if let Ok(Some(_)) = collection.find_one(doc! {"_id": user_id}, None).await {
            // Document with the specified ObjectId exists in the collection
            req.extensions_mut().insert(user_id);
            return Ok(next.run(req).await);
        } else {
            // Document with the specified ObjectId does not exist in the collection
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}
