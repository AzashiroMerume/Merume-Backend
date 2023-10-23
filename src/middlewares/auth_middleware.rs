use crate::{
    models::author_model::Author, responses::OperationStatusResponse, utils::jwt::verify_token,
    AppState,
};

use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use bson::{doc, oid::ObjectId};

pub async fn auth<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
    pass_full_user: Option<bool>,
) -> Result<Response, (StatusCode, Json<OperationStatusResponse>)> {
    let auth_header = match req.headers().get(http::header::AUTHORIZATION) {
        Some(header) => header.to_str().ok(),
        None => None,
    };

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|err| {
        eprintln!("There is an error with `JWT_SECRET`: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            }),
        )
    })?;

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Authorization header missing".to_string()),
                }),
            ))
        }
    };

    let token_claims = verify_token(token, &jwt_secret).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("Invalid token".to_string()),
            }),
        )
    })?;

    let user_id = ObjectId::parse_str(&token_claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("Invalid user ID in token".to_string()),
            }),
        )
    })?;

    let user = match state
        .db
        .users_collection
        .find_one(doc! {"_id": user_id}, None)
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
        id: user_id,
        nickname: user.nickname.clone(),
        username: user.username.clone(),
    };

    if let Some(true) = pass_full_user {
        req.extensions_mut().insert(user);
    } else {
        req.extensions_mut().insert(author_info);
    }

    Ok(next.run(req).await)
}
