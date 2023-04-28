mod handlers;
mod middlewares;
mod models;
mod responses;
mod router;
mod routes;
mod utils;
mod db;

use router::create_router;

use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, Compressor},
    Client,
};
use std::{net::SocketAddr, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mongo_uri: String =
        std::env::var("MONGO_URI").expect("Failed to load `MONGO_URI` environment variable.");
    let mongo_connection_timeout: u64 = match std::env::var("MONGO_CONNECTION_TIMEOUT") {
        Ok(val) => val
            .parse()
            .expect("Failed to parse `MONGO_CONNECTION_TIMEOUT` environment variable."),
        Err(e) => panic!(
            "Failed to load `MONGO_CONNECTION_TIMEOUT` environment variable: {}",
            e
        ),
    };
    let mongo_min_pool_size: u32 = std::env::var("MONGO_MIN_POOL_SIZE")
        .expect("Failed to load `MONGO_MIN_POOL_SIZE` environment variable.")
        .parse()
        .expect("Failed to parse `MONGO_MIN_POOL_SIZE` environment variable.");
    let mongo_max_pool_size: u32 = std::env::var("MONGO_MAX_POOL_SIZE")
        .expect("Failed to load `MONGO_MAX_POOL_SIZE` environment variable.")
        .parse()
        .expect("Failed to parse `MONGO_MAX_POOL_SIZE` environment variable.");

    // Load the timeout value from the environment variable "REQUEST_TIMEOUT"
    let _jwt_secret: String =
        std::env::var("JWT_SECRET").expect("Failed to load `JWT_SECRET` environment variable.");

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    //specifying some connection settings
    let mut client_options = ClientOptions::parse(mongo_uri)
        .await
        .expect("Failed to parse MongoDB URI");
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

    // build application with a router
    let app = create_router(client);

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
