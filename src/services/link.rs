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