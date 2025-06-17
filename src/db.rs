use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_db(db_url: &str) -> Result<PgPool, sqlx::Error> {
    // Create a connection pool to the PostgreSQL database
    let pool = PgPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections in the pool
        .connect(db_url)
        .await?;

    Ok(pool)

}