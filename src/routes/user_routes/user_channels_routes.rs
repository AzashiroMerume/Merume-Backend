use std::sync::Arc;

use crate::{
    handlers::user_handlers::user_channels_handlers as channel_handlers, middlewares, AppState,
};

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_channels_routes(State(state): State<Arc<AppState>>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/:channel_id/delete",
            post(channel_handlers::delete_channel_handler::delete_channel_by_id),
        )
        .route(
            "/:channel_id/update",
            post(channel_handlers::update_channels_handler::update_channel_by_id),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::verify_channel_access_middleware::verify_channel_access,
        ))
        .route("/read_trackers", get(channel_handlers::channel_read_tracker_handlers::get_read_trackers_handler::get_read_trackers))
        .route(
            "/read_trackers/:channel_id",
            post(channel_handlers::channel_read_tracker_handlers::update_read_tracker_handler::update_read_tracker_handler),
        )
        .route(
            "/subscriptions",
            get(channel_handlers::subscribed_channels_handler::subscribed_channels),
        )
        .route(
            "/created",
            get(channel_handlers::created_channels_handler::created_channels),
        )
        .route(
            "/new",
            post(channel_handlers::new_channel_handler::new_channel),
        )
        .layer(middleware::from_fn_with_state(state, |state, req, next| {
            middlewares::auth_middleware::auth(state, req, next, Some(false))
        }))
        .layer(RequestBodyLimitLayer::new(1024))
}
