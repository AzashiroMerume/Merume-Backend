use crate::middlewares::auth_middleware::PassFromAuth;
use crate::{handlers, middlewares, AppState};
use handlers::user_handlers::content_system_handlers;
use middlewares::auth_middleware;
use std::sync::Arc;

use axum::{extract::State, middleware, routing::get, Router};

pub fn content_routes(State(state): State<Arc<AppState>>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(content_system_handlers::recommendations_handler::recommendations),
        )
        .route(
            "/trending",
            get(content_system_handlers::trendings_handler::trendings),
        )
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, PassFromAuth::FullUser)
        }))
}
