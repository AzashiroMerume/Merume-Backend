use crate::{handlers, middlewares};
use handlers::{channels_handlers, posts_handlers};
use middlewares::{auth_middleware, verify_channel_owner_middleware};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use mongodb::Client;
use tower_http::limit::RequestBodyLimitLayer;

pub fn channels_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/:channel_id",
            get(channels_handlers::get_channel_handler::get_channel_by_id),
        )
        .route(
            "/:channel_id/subscribe",
            get(channels_handlers::subscribe_to_channel_handler::subscribe_to_channel),
        )
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
}

pub fn post_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/:channel_id/post",
            post(posts_handlers::create_post_handler::create_post),
        )
        .route(
            "/:channel_id/:post_id/delete",
            post(posts_handlers::delete_post_handler::delete_post_by_id),
        )
        .layer(middleware::from_fn_with_state(
            client.clone(),
            verify_channel_owner_middleware::verify_channel_owner,
        ))
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .layer(RequestBodyLimitLayer::new(1024))
}

pub fn channel_system(client: Client) -> Router<Client> {
    Router::new()
        .merge(channels_routes(client.clone()))
        .merge(post_routes(client))
}
