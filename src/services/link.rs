use crate::errors::AppError;

pub async fn create_short_link(
    db : &sqlx::PgPool,
    target_url: String,
) -> Result<String, AppError>{
    let slug = nanoid::nanoid!(6);

    //TODO :Check for collision

    sqlx::query!(
        "INSERT INTO links (slug, target_url) VALUES ($1, $2)",
        slug,
        target_url
    )
    .execute(db)
    .await
   .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(slug)
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