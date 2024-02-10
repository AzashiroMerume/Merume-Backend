use crate::{handlers, middlewares, AppState};
use handlers::{channels_handlers, posts_handlers};
use middlewares::{auth_middleware, verify_channel_access_middleware};

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn channels_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/:channel_id",
            get(channels_handlers::get_channel_handler::get_channel_by_id),
        )
        .route(
            "/:channel_id/followers",
            get(channels_handlers::get_channel_followers::get_channel_followers),
        )
        .route(
            "/:channel_id/subscribe",
            get(channels_handlers::subscribe_to_channel_handler::subscribe_to_channel),
        )
        .route(
            "/:channel_id/content",
            get(channels_handlers::get_channel_posts_handler::channel_posts),
        )
        .route(
            "/:channel_id/more_content",
            get(channels_handlers::get_more_channel_posts_handler::more_channel_posts),
        )
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(false))
        }))
}

pub fn post_routes(State(state): State<AppState>) -> Router<AppState> {
    let without_post_id = Router::new()
        .route(
            "/:channel_id/post",
            post(posts_handlers::create_post_handler::create_post),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            verify_channel_access_middleware::verify_channel_access,
        ));

    let with_post_id = Router::new()
        .route(
            "/:channel_id/:post_id/delete",
            post(posts_handlers::delete_post_handler::delete_post_by_id),
        )
        .route(
            "/:channel_id/:post_id/update",
            post(posts_handlers::update_post_handler::update_post_by_id),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            verify_channel_access_middleware::verify_channel_access,
        ));

    Router::new()
        .merge(without_post_id)
        .merge(with_post_id)
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(false))
        }))
        .layer(RequestBodyLimitLayer::new(1024))
}

pub fn channel_system(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .merge(channels_routes(State(state.clone())))
        .merge(post_routes(State(state)))
}
