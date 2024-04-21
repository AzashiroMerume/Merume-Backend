use crate::{
    models::author_model::Author, responses::ErrorResponse,
    utils::jwt::firebase_token_jwt::verify_access_jwt_token, AppState,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use bson::doc;
use jsonwebtoken::errors::ErrorKind;
use std::sync::Arc;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
    pass_full_user: Option<bool>,
) -> Result<Response, ErrorResponse> {
    let access_token_header = match req.headers().get("access_token") {
        Some(header) => header.to_str().ok(),
        None => None,
    };

    let token = match access_token_header {
        Some(token) => token,
        None => {
            return Err(ErrorResponse::Unauthorized(None));
        }
    };

    let firebase_user_id =
        verify_access_jwt_token(token, state.firebase_config.token_decoding_key.clone()).map_err(
            |err| {
                eprintln!("{:?}", err);
                if err == ErrorKind::ExpiredSignature {
                    ErrorResponse::Unauthorized(Some("Expired"))
                } else {
                    ErrorResponse::Unauthorized(None)
                }
            },
        )?;

    let user = match state
        .db
        .users_collection
        .find_one(doc! {"firebase_user_id": firebase_user_id.clone()}, None)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(ErrorResponse::Unauthorized(None));
        }
        Err(err) => {
            eprintln!("The database error: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let author_info = Author {
        id: user.id,
        nickname: user.nickname.clone(),
        username: user.username.clone(),
        pfp_link: user.pfp_link.clone(),
        is_online: None,
        last_time_online: None,
    };

    if let Some(true) = pass_full_user {
        req.extensions_mut().insert(user);
    } else if let Some(false) = pass_full_user {
        req.extensions_mut().insert(author_info);
    } else {
        req.extensions_mut().insert(user.id);
    }

    Ok(next.run(req).await)
}
