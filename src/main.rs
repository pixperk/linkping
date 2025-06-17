use tracing_subscriber::FmtSubscriber;

use crate::{config::Config, routes::create_router};


mod config;
mod routes;
mod db;
mod models;
mod services;
mod errors;
mod validation;

#[tokio::main]
async fn main() {
   //Logger
   let subscriber = FmtSubscriber::new();
   tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    // Load configuration
    let config = Config::new();
    let addr = format!("0.0.0.0:{}", config.port);
    let db_url = config.db_url;

    // Initialize database connection
    let db_pool = db::connect_db(&db_url).await.expect("Failed to connect to the database");

    tracing::info!("Starting LinkPing on {}", addr);

    //Router
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = create_router(db_pool);
    axum::serve(listener, app).await.unwrap();
}
