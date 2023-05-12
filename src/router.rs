use crate::{
    handlers::common_handler,
    routes::{
        auth_routes, channel_system_routes, content_routes, preferences_routes,
        user_channels_routes,
    },
    AppState,
};

use axum::{
    extract::State,
    http::{
        header::{self, AUTHORIZATION},
        HeaderValue,
    },
    Router,
};
use std::{iter::once, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
// use std::sync::Arc;

pub fn create_router(State(state): State<AppState>) -> Router {
    //setting server configs
    let server_header_value = HeaderValue::from_static("Merume");
    let request_timeout: u64 = std::env::var("REQUEST_TIMEOUT")
        .expect("Failed to load `REQUEST_TIMEOUT` environment variable.")
        .parse()
        .expect("Failed to parse `REQUEST_TIMEOUT` environment variable.");

    //creating routers
    let auth_routes = auth_routes::auth_routes(State(state.clone()));
    let user_channels_routes = user_channels_routes::user_channels_routes(State(state.clone()));
    let channel_system = channel_system_routes::channel_system(State(state.clone()));
    let content_routes = content_routes::content_routes(State(state.clone()));
    let preferences_routes = preferences_routes::preferences_routes(State(state.clone()));

    let app = Router::new()
        // .route("/test", get(common_handler::_test_handler))
        // .route_layer(middleware::from_fn_with_state(
        //     client.clone(),
        //     |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        // ))
        .nest("/users/channels", user_channels_routes)
        .nest("/users/recommendations", content_routes)
        .nest("/auth", auth_routes)
        .nest("/channels", channel_system)
        .nest("/preferences", preferences_routes)
        .layer(
            ServiceBuilder::new()
                //sensetive header authorization from request
                .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
                .layer(TraceLayer::new_for_http())
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::SERVER,
                    server_header_value,
                ))
                // timeout requests after 10 secs, returning 408 status code
                .layer(TimeoutLayer::new(Duration::from_secs(request_timeout))),
        );
    app.fallback(common_handler::handler_404).with_state(state)
}
