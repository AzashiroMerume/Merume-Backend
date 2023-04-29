mod db;
mod handlers;
mod middlewares;
mod models;
mod responses;
mod router;
mod routes;
mod utils;

use crate::db::DB;
use axum::extract::State;
use router::create_router;

use dotenv::dotenv;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: DB,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = DB::init().await.unwrap();

    // router creation
    let app = create_router(State(AppState { db: db.clone() }));

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
