mod link;
mod analytics;

use axum::{routing::{post, get}, Router};
use sqlx::PgPool;

use crate::routes::{analytics::analytics_handler, link::{resolve_handler, shorten_handler}};


pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/shorten", post(shorten_handler))
        .route("/{capture}", get(resolve_handler))
        .route("/analytics/{capture}", get(analytics_handler))
        .with_state(db)
}
