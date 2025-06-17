use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};

#[derive(Serialize, Deserialize)]
pub struct ShortenRequest{
    pub target_url : String,
}

#[derive(Serialize, Deserialize)]
pub struct ShortenResponse {
    pub slug: String,
}

#[derive(FromRow)]
pub struct Link {
    pub id: i32,
    pub slug: String,
    pub target_url: String,
    pub created_at: chrono::NaiveDateTime,
}