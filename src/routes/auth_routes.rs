use std::sync::Arc;

use crate::{
    handlers::{self},
    middlewares::{auth_middleware, verify_refresh_token_middleware},
    AppState,
};
use axum::{
    extract::State,
    handler::Handler,
    middleware,
    routing::{get, post},
    Router,
};
use handlers::auth_handlers;
use tower_http::limit::RequestBodyLimitLayer;

pub fn auth_routes(State(state): State<Arc<AppState>>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(auth_handlers::verify_auth_handler::verify_auth))
        .route("/logout", post(auth_handlers::logout_handler::logout))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .route("/register", post(auth_handlers::register_handler::register))
        .route("/login", post(auth_handlers::login_handler::login))
        .route(
            "/refresh",
            get(auth_handlers::access_token_handler::access_token.layer(
                middleware::from_fn_with_state(state, |state, req, next| {
                    verify_refresh_token_middleware::verify_refresh_token(state, req, next)
                }),
            )),
        )
        .layer(RequestBodyLimitLayer::new(1024))
}
