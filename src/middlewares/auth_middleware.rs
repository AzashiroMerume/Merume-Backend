use crate::{
    models::author_model::Author, responses::OperationStatusResponse,
    utils::jwt::firebase_token_jwt::verify_access_jwt_token, AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use bson::doc;
use jsonwebtoken::errors::ErrorKind;

pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
    pass_full_user: Option<bool>,
) -> Result<Response, (StatusCode, Json<OperationStatusResponse>)> {
    let access_token_header = match req.headers().get("access_token") {
        Some(header) => header.to_str().ok(),
        None => None,
    };

    let token = match access_token_header {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Token header missing".to_string()),
                }),
            ))
        }
    };

    let firebase_user_id = verify_access_jwt_token(token, state.firebase_token_decoding_key)
        .map_err(|err| {
            eprintln!("{:?}", err);
            if err == ErrorKind::ExpiredSignature {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(OperationStatusResponse {
                        success: false,
                        error_message: Some(("Expired").to_string()),
                    }),
                )
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(OperationStatusResponse {
                        success: false,
                        error_message: Some(("Token authentication failed").to_string()),
                    }),
                )
            }
        })?;

    let user = match state
        .db
        .users_collection
        .find_one(doc! {"firebase_user_id": firebase_user_id.clone()}, None)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("User not found".to_string()),
                }),
            ))
        }
        Err(err) => {
            eprintln!("The database error: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            ));
        }
    };

    let author_info = Author {
        id: user.id,
        nickname: user.nickname.clone(),
        username: user.username.clone(),
        pfp_link: user.pfp_link.clone(),
        is_online: Some(user.is_online),
    };

    if let Some(true) = pass_full_user {
        req.extensions_mut().insert(user);
    } else {
        req.extensions_mut().insert(author_info);
    }

    Ok(next.run(req).await)
}
