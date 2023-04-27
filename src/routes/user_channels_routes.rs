use crate::{handlers, middlewares};
use handlers::channels_handlers::user_channels_handlers;
use middlewares::{auth_middleware, verify_channel_owner_middleware};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use mongodb::Client;
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_channels_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/subscriptions",
            get(user_channels_handlers::subscribed_channels_handler::subscribed_channels),
        )
        .route(
            "/created",
            get(user_channels_handlers::created_channels_handler::created_channels),
        )
        .route(
            "/new",
            post(user_channels_handlers::new_channel_handler::new_channel),
        )
        .route(
            "/:channel_id/delete",
            post(user_channels_handlers::delete_channel_handler::delete_channel_by_id),
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
