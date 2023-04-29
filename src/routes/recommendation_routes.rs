use crate::{handlers, middlewares, AppState};
use handlers::content_recommendation_handlers;
use middlewares::auth_middleware;

use axum::{extract::State, middleware, routing::get, Router};

pub fn recomendations_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(content_recommendation_handlers::recommendations_handler::recommendations),
        )
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(true))
        }))
}
