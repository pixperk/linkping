use axum::{
    extract::{ Json, State}, routing::{get, post}, Router
};
use crate::{models::{click::ClickEvent, link::{ShortenRequest, ShortenResponse}}, routes::analytics, streams::producer::publish_click_event};
use crate::services::link::create_short_link;
use crate::errors::AppError;
use validator::Validate;



pub async fn shorten_handler(
    State(db) : State<sqlx::PgPool>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> { 

    if let Err(e) = payload.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let slug = create_short_link(&db, payload.target_url, payload.custom_slug, payload.expires_in)
        .await?;

    Ok(Json(ShortenResponse { slug }))
}

pub async fn resolve_handler(
    State(db): State<sqlx::PgPool>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    metadata : ClickEvent
) -> Result<axum::response::Redirect, AppError> {
    let target_url = match crate::services::link::resolve_slug(&db, slug).await {
        Ok(url) => url,
        Err(AppError::NotFound(_)) => {
            return Err(AppError::NotFound("Shortlink not found".to_string()));
        }
        Err(e) => {
            return Err(e);
        }
    };

   
       publish_click_event(metadata).await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
          

    Ok(axum::response::Redirect::to(&target_url))
}