mod handlers;
mod middlewares;
mod models;
mod responses;
mod routes;
mod utils;

use handlers::common_handler;
use routes::*;

use axum::{
    http::{
        header::{self, AUTHORIZATION},
        HeaderValue,
    },
    Router,
};
use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, Compressor},
    Client,
};
use std::{iter::once, net::SocketAddr, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // initialize tracing
    dotenv().ok();

    let mongo_uri: String =
        std::env::var("MONGO_URI").expect("Failed to load `MONGO_URI` environment variable.");
    let mongo_connection_timeout: u64 = std::env::var("MONGO_CONNECTION_TIMEOUT")
        .expect("Failed to load `MONGO_CONNECTION_TIMEOUT` environment variable.")
        .parse()
        .expect("Failed to parse `MONGO_CONNECTION_TIMEOUT` environment variable.");
    let mongo_min_pool_size: u32 = std::env::var("MONGO_MIN_POOL_SIZE")
        .expect("Failed to load `MONGO_MIN_POOL_SIZE` environment variable.")
        .parse()
        .expect("Failed to parse `MONGO_MIN_POOL_SIZE` environment variable.");
    let mongo_max_pool_size: u32 = std::env::var("MONGO_MAX_POOL_SIZE")
        .expect("Failed to load `MONGO_MAX_POOL_SIZE` environment variable.")
        .parse()
        .expect("Failed to parse `MONGO_MAX_POOL_SIZE` environment variable.");
    let _jwt_secret: String =
        std::env::var("JWT_SECRET").expect("Failed to load `JWT_SECRET` environment variable.");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut client_options = ClientOptions::parse(mongo_uri).await.unwrap();
    client_options.connect_timeout = Some(Duration::from_secs(mongo_connection_timeout));
    client_options.max_pool_size = Some(mongo_max_pool_size);
    client_options.min_pool_size = Some(mongo_min_pool_size);

    // the server will select the algorithm it supports from the list provided by the driver
    client_options.compressors = Some(vec![
        Compressor::Snappy,
        Compressor::Zlib {
            level: Default::default(),
        },
        Compressor::Zstd {
            level: Default::default(),
        },
    ]);

    let client = Client::with_options(client_options).unwrap();
    let server_header_value = HeaderValue::from_static("Merume");

    //creating routers
    let auth_routes = auth_routes(client.clone());
    let user_channels_routes = user_channels_routes(client.clone());
    let channel_system = channel_system(client.clone());
    let preferred_content_routes = preferred_content_routes(client.clone());

    // build application with a routes
    let app = Router::new()
        // .route("/test", get(common_handler::_test_handler))
        // .route_layer(middleware::from_fn_with_state(
        //     client.clone(),
        //     |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        // ))
        .nest("/users/channels", user_channels_routes)
        .nest("/auth", auth_routes)
        .nest("/channels", channel_system)
        .nest("/preferences", preferred_content_routes)
        .layer(
            ServiceBuilder::new()
                .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
                .layer(TraceLayer::new_for_http())
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::SERVER,
                    server_header_value,
                ))
                // timeout requests after 10 secs, returning 408 status code
                .layer(TimeoutLayer::new(Duration::from_secs(10))),
        );

    let app = app.fallback(common_handler::handler_404).with_state(client);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal_shutdown())
        .await
        .unwrap();

    async fn signal_shutdown() {
        tokio::signal::ctrl_c()
            .await
            .expect("Expect ctrl - ctrl shutdown");
        println!("Signal shutting down");
    }
}
