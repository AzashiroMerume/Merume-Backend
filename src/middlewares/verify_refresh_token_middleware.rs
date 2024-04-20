use crate::{
    responses::ErrorResponse,
    utils::jwt::{
        firebase_token_jwt::generate_access_jwt_token, refresh_token_jwt::verify_refresh_jwt_token,
    },
    AppState,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use bson::{doc, oid::ObjectId};
use jsonwebtoken::errors::ErrorKind;
use std::sync::Arc;

pub async fn verify_refresh_token(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let refresh_token_header = match req.headers().get("refresh_token") {
        Some(header) => header.to_str().ok(),
        None => None,
    };

    let refresh_token = match refresh_token_header {
        Some(token) => token,
        None => {
            return Err(ErrorResponse::Unauthorized(None));
        }
    };

    let user_id = match verify_refresh_jwt_token(refresh_token, &state.refresh_jwt_secret) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("{:?}", err);
            if err == ErrorKind::ExpiredSignature {
                return Err(ErrorResponse::Unauthorized(Some("Expired")));
            } else {
                return Err(ErrorResponse::Unauthorized(None));
            }
        }
    };

    let user_id_object = match ObjectId::parse_str(&user_id) {
        Ok(object_id) => object_id,
        Err(_) => {
            return Err(ErrorResponse::BadRequest(None));
        }
    };

    match state
        .db
        .users_collection
        .find_one(doc! {"_id": user_id_object}, None)
        .await
    {
        Ok(Some(user)) => {
            // After generating the access token, extract the String value from the Result
            let access_token = match generate_access_jwt_token(
                &user.firebase_user_id,
                state.firebase_config.token_encoding_key.clone(),
                state.firebase_config.service_account.clone(),
            ) {
                Ok(token) => token,
                Err(err) => {
                    eprintln!("Error while generating access token: {}", err);
                    return Err(ErrorResponse::ServerError(None));
                }
            };

            // Insert the access token string into the request extensions
            req.extensions_mut().insert(access_token);
        }
        Ok(None) => {
            return Err(ErrorResponse::Unauthorized(None));
        }
        Err(err) => {
            eprintln!("The database error: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    Ok(next.run(req).await)
}
