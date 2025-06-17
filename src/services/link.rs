use sqlx::{Error};
use chrono::{Utc, Duration};



use crate::errors::AppError;

pub async fn create_short_link(
    db : &sqlx::PgPool,
    target_url: String,
    custom_slug: Option<String>,
    expires_in: Option<String>, 
) -> Result<String, AppError>{
    
    
    let slug = match custom_slug {
        Some(slug) => slug,
        None => nanoid::nanoid!(6)
    };

    //TODO :Check for collision

    //parse expiry
    let expiry = match expires_in {
         Some(ref raw) => {
           let dur = humantime::parse_duration(raw)
                .map_err(|_| AppError::ValidationError("Invalid expiry format".to_string()))?;
            Some(Utc::now() + Duration::from_std(dur).unwrap())
        }
        None => None,
    };

    let res = sqlx::query!(
        "INSERT INTO links (slug, target_url, expires_at) VALUES ($1, $2, $3)",
        slug,
        target_url,
        expiry
    )
    .execute(db)
    .await;
    
    match res {
        Ok(_) => Ok(slug),
        Err(e) => {
            if let Error::Database(db_err) = &e {
                if db_err.constraint() == Some("links_slug_key") {
                    return Err(AppError::ValidationError("Slug already exists".to_string()));
                }
            }
            Err(AppError::DatabaseError(e.to_string()))
        }
}
}

pub async fn resolve_slug(
    db: &sqlx::PgPool,
    slug: String,
) -> Result<String, AppError> {

    //TODO : Add cache layer to speed up lookups
    // Add rate limiting to prevent abuse
    let link = sqlx::query!(
        "SELECT target_url FROM links WHERE slug = $1",
        slug
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::NotFound(e.to_string()))?;

    Ok(link.target_url)
}