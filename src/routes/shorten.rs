use axum::{
    routing::post,
    Router,
    extract::{State, Json},
};
use sqlx::PgPool;
use crate::models::link::{ShortenRequest, ShortenResponse};
use crate::services::link::create_short_link;
use crate::errors::AppError;

pub fn shorten_routes(db: PgPool) -> Router {
    Router::new()
        .route("/shorten", post(shorten_handler))
        .with_state(db)
}

async fn shorten_handler(
    State(db) : State<sqlx::PgPool>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> {    
    let slug = create_short_link(&db, payload.target_url)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(ShortenResponse { slug }))
}