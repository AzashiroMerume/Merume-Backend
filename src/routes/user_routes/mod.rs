pub mod preferences_routes;
pub mod user_channels_routes;

use super::content_routes;
use crate::{
    handlers::user_handlers::{
        get_all_last_updates::all_last_updates, get_email_handler::get_email_by_nickname,
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
        .route("/last_updates", get(all_last_updates))
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
