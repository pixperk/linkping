use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};
use validator::Validate;
use crate::validation::url::{validate_scheme, validate_expiry};

#[derive(Serialize, Deserialize, Validate)]
pub struct ShortenRequest{
     #[validate(url)]
     #[validate(custom(
            function = "validate_scheme",
            message = "Invalid URL scheme. Only http and https are allowed."
     ))]
    pub target_url : String,
    #[validate(length(min = 3, max = 20))]
    pub custom_slug: Option<String>,

    #[validate(custom(
        function = "validate_expiry",
        message = "Expiry must be a valid duration like '1d', '6h', '30m'"
    ))]
    pub expires_in: Option<String>
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