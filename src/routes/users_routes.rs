use crate::middlewares::auth_middleware::PassFromAuth;
//Consider: USERS routes are different form USER routes
use crate::{
    handlers::users_handlers::get_user_channels_handler::get_user_channels,
    middlewares::auth_middleware, AppState,
};
use axum::{extract::State, middleware, routing::get, Router};
use std::sync::Arc;
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_routes(State(state): State<Arc<AppState>>) -> Router<Arc<AppState>> {
    let user_routes = Router::new()
        .route("/:user_id", get(get_user_channels))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, PassFromAuth::UserId),
        ))
        .layer(RequestBodyLimitLayer::new(1024));

    user_routes
}
