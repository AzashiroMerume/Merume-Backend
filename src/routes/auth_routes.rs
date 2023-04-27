use crate::{handlers, middlewares::auth_middleware};
use handlers::auth_handlers;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use mongodb::Client;
use tower_http::limit::RequestBodyLimitLayer;

pub fn auth_routes(client: Client) -> Router<Client> {
    Router::new()
        .route("/", get(auth_handlers::verify_auth_handler::verify_auth))
        .route_layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .route("/register", post(auth_handlers::register_handler::register))
        .route("/login", post(auth_handlers::login_handler::login))
        .layer(RequestBodyLimitLayer::new(1024))
}
