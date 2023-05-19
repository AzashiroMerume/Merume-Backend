use crate::{handlers, middlewares, AppState};
use handlers::posts_handlers;
use middlewares::auth_middleware;

use axum::{extract::State, middleware, routing::post, Router};

pub fn read_posts_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(posts_handlers::mark_as_read_post_handler::mark_as_read),
        )
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(false))
        }))
}
