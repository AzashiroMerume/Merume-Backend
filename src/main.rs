pub mod handlers;
mod models;
mod responses;

use axum::{
    http::{header, HeaderValue},
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, Compressor},
    Client,
};
use tower_http::{
    limit::RequestBodyLimitLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// use structopt::StructOpt;
use std::{net::SocketAddr, time::Duration};

use handlers::auth_handler;
use handlers::common_handler;
use handlers::websockets::channels_handler;

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
    // let mongo_db: String =
    //     std::env::var("MONGO_DB").expect("Failed to load `MONGO_DB` environment variable.");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "example_websockets=debug,rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
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

    let auth_routes = Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login));
    let channel_routes = Router::new().route("/", get(channels_handler::channels_handler));

    // build our application with a route
    let app = Router::new()
        .nest("/channels", channel_routes)
        .nest("/auth", auth_routes)
        // timeout requests after 10 secs, returning 408 status code
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // don't allow request bodies larger than 1024 bytes, returning 413 status code
        .layer(RequestBodyLimitLayer::new(1024))
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::SERVER,
            server_header_value,
        ));
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
