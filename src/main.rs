mod db;
mod firebase_config;
mod handlers;
mod middlewares;
mod models;
mod responses;
mod router;
mod routes;
mod utils;

use axum::extract::State;
use db::DB;
use dotenv::dotenv;
use firebase_config::FirebaseConfig;
use jsonwebtoken::{DecodingKey, EncodingKey};
use router::create_router;
use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: DB,
    firebase_config: FirebaseConfig,
    refresh_jwt_secret: String,
    //Use Redis
    // _redis_client: redis::Client,
}

impl AppState {
    pub fn new(db: DB, firebase_config: FirebaseConfig, refresh_jwt_secret: String) -> Self {
        AppState {
            db,
            firebase_config,
            refresh_jwt_secret,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "backend=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_port = std::env::var("SERVER_PORT")
        .expect("Failed to load `SERVER_PORT` environment variable.")
        .parse()
        .expect("Failed to load `SERVER_PORT` environment variable.");

    let db = DB::init()
        .await
        .expect("The Database initialization failed..");

    let firebase_private_key = std::env::var("FIREBASE_SERVICE_PRIVATE_KEY")
        .expect("Failed to load `FIREBASE_SERVICE_PRIVATE_KEY` environment variable.");
    let firebase_public_key = std::env::var("FIREBASE_SERVICE_PUBLIC_KEY")
        .expect("Failed to load `FIREBASE_SERVICE_PUBLIC_KEY` environment variable.");
    let firebase_service_account = std::env::var("FIREBASE_SERVICE_ACCOUNT_EMAIL")
        .expect("Failed to load `FIREBASE_SERVICE_ACCOUNT_EMAIL` environment variable.");
    let firebase_token_encoding_key =
        EncodingKey::from_rsa_pem(firebase_private_key.as_bytes()).unwrap();
    let firebase_token_decoding_key =
        DecodingKey::from_rsa_pem(firebase_public_key.as_bytes()).unwrap();

    let refresh_jwt_secret = std::env::var("REFRESH_JWT_SECRET")
        .expect("Failed to load `REFRESH_JWT_SECRET` environment variable.");

    //redis initialization
    // let redis_uri =
    //     std::env::var("REDIS_URI").expect("Failed to load `REDIS_URI` environment variable.");
    // let redis_client = redis::Client::open(redis_uri).expect("Failed to create redis_client");

    let firebase_config = FirebaseConfig {
        token_encoding_key: firebase_token_encoding_key,
        token_decoding_key: firebase_token_decoding_key,
        service_account: firebase_service_account,
    };
    // router creation
    let app = create_router(State(Arc::new(AppState::new(
        db,
        firebase_config,
        refresh_jwt_secret,
    ))));

    let addr = SocketAddr::from(([127, 0, 0, 1], server_port));
    tracing::debug!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app)
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
