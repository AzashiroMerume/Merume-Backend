use crate::{handlers, middlewares::auth_middleware, AppState};
use handlers::auth_handlers;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn auth_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route("/", get(auth_handlers::verify_auth_handler::verify_auth))
        .route("/logout", post(auth_handlers::logout_handler::logout))
        .route_layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(true))
        }))
        .route("/register", post(auth_handlers::register_handler::register))
        .route("/login", post(auth_handlers::login_handler::login))
        .layer(RequestBodyLimitLayer::new(1024))
}
