use crate::{handlers, middlewares};
use handlers::content_recommendation_handlers;
use middlewares::auth_middleware;

use axum::{middleware, routing::get, Router};
use mongodb::Client;

pub fn recomendations_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/",
            get(content_recommendation_handlers::recommendations_handler::recommendations),
        )
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(true)),
        ))
}
