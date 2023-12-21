use crate::{
    handlers::{common_handler, user_handlers::get_email_handler},
    routes::{
        auth_routes, channel_system_routes, content_routes, mark_as_read_posts_routes,
        preferences_routes, user_channels_routes,
    },
    AppState,
};

use axum::{
    extract::State,
    http::{
        header::{self, AUTHORIZATION},
        HeaderValue,
    },
    routing::post,
    Router,
};
use std::{iter::once, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    limit::RequestBodyLimitLayer, sensitive_headers::SetSensitiveRequestHeadersLayer,
    set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
// use std::sync::Arc;

pub fn create_router(State(state): State<AppState>) -> Router {
    //setting server configs
    let server_header_value = HeaderValue::from_static("Merume");
    let request_timeout: u64 = std::env::var("REQUEST_TIMEOUT")
        .expect("Failed to load `REQUEST_TIMEOUT` environment variable.")
        .parse()
        .expect("Failed to parse `REQUEST_TIMEOUT` environment variable.");

    //creating base routes
    let auth_routes = auth_routes::auth_routes(State(state.clone()));
    let user_channels_routes = user_channels_routes::user_channels_routes(State(state.clone()));
    let channel_system = channel_system_routes::channel_system(State(state.clone()));
    let read_post_routes = mark_as_read_posts_routes::read_posts_routes(State(state.clone()));
    let content_routes = content_routes::content_routes(State(state.clone()));
    let preferences_routes = preferences_routes::preferences_routes(State(state.clone()));

    let user_routes = Router::new()
        .route("/get_email", post(get_email_handler::get_email_by_nickname))
        .layer(RequestBodyLimitLayer::new(1024))
        .nest("/channels", user_channels_routes)
        .nest("/recommendations", content_routes)
        .nest("/preferences", preferences_routes);

    let app = Router::new()
        // .route("/test", get(common_handler::_test_handler))
        // .route_layer(middleware::from_fn_with_state(
        //     client.clone(),
        //     |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        // ))
        .nest("/user", user_routes)
        .nest("/auth", auth_routes)
        .nest("/channels", channel_system)
        .nest("/mark", read_post_routes)
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
