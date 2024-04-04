//Consider: USERS routes are different form USER routes
use crate::{
    handlers::users_handlers::get_user_channels_handler::get_user_channels,
    middlewares::auth_middleware, AppState,
};
use axum::{extract::State, middleware, routing::get, Router};
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_routes(State(state): State<AppState>) -> Router<AppState> {
    let user_routes = Router::new()
        .route("/:user_id", get(get_user_channels))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, None),
        ))
        .layer(RequestBodyLimitLayer::new(1024));

    user_routes
}
