use crate::{
    models::author_model::Author, responses::OperationStatusResponse, utils::jwt::verify_token,
    AppState,
};

use axum::{
    extract::{Request, State},
    http::{self, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use bson::doc;

pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
    pass_full_user: Option<bool>,
) -> Result<Response, (StatusCode, Json<OperationStatusResponse>)> {
    let auth_header = match req.headers().get(http::header::AUTHORIZATION) {
        Some(header) => header.to_str().ok(),
        None => None,
    };

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

    let token_claims = verify_token(token, state.firebase_token_decoding_key).map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some(format!("{}", err)),
            }),
        )
    })?;

    let firebase_user_id = token_claims.uid;

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
    };

    if let Some(true) = pass_full_user {
        req.extensions_mut().insert(user);
    } else {
        req.extensions_mut().insert(author_info);
    }

    Ok(next.run(req).await)
}
