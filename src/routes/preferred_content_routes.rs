use crate::{handlers, middlewares};
use handlers::preferred_content_handlers;
use middlewares::auth_middleware;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use mongodb::Client;
use tower_http::limit::RequestBodyLimitLayer;

pub fn preferred_content_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/",
            get(preferred_content_handlers::get_preferences_handler::get_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            client.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, Some(true)),
        ))
        .route(
            "/",
            post(preferred_content_handlers::post_preferences_handler::post_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .layer(RequestBodyLimitLayer::new(1024))
}
