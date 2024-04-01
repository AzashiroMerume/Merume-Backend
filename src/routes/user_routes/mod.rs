pub mod preferences_routes;
pub mod user_channels_routes;

use super::content_routes;
use crate::{
    handlers::user_handlers::{
        all_user_channel_updates_handler::all_channels_updates,
        get_email_handler::get_email_by_nickname, get_user_channels_handler::get_user_channels,
        heartbeat_handler::heartbeat,
    },
    middlewares::auth_middleware,
    AppState,
};
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_routes(State(state): State<AppState>) -> Router<AppState> {
    let preferences_routes = preferences_routes::preferences_routes(State(state.clone()));
    let user_channels_routes = user_channels_routes::user_channels_routes(State(state.clone()));
    let content_routes = content_routes::content_routes(State(state.clone()));

    let user_routes = Router::new()
        .route("/heartbeat", get(heartbeat))
        .route("/all_updates", get(all_channels_updates))
        .route("/:user_id", get(get_user_channels))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, None),
        ))
        .route("/get_email", post(get_email_by_nickname))
        .layer(RequestBodyLimitLayer::new(1024))
        .nest("/channels", user_channels_routes)
        .nest("/recommendations", content_routes)
        .nest("/preferences", preferences_routes);

    user_routes
}
