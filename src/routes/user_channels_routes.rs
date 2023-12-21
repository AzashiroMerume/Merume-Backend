use crate::{handlers, middlewares, AppState};
use handlers::user_handlers::user_channels_handlers;
use middlewares::{auth_middleware, verify_channel_owner_middleware};

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_channels_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/:channel_id/delete",
            post(user_channels_handlers::delete_channel_handler::delete_channel_by_id),
        )
        .route(
            "/:channel_id/update",
            post(user_channels_handlers::update_channels_handler::update_channel_by_id),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            verify_channel_owner_middleware::verify_channel_owner,
        ))
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
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            auth_middleware::auth(state, req, next, Some(false))
        }))
        .layer(RequestBodyLimitLayer::new(1024))
}
