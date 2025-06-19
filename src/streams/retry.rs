use std::time::Duration;

use rand::{rng, Rng};
use sqlx::PgPool;
use tokio::time::sleep;

use crate::{models::click::ClickEvent, streams::consumer::insert_click};

pub const RETRY_DELAY_MS: u64 = 500;
const MAX_RETRIES: u32 = 3;

pub async fn insert_click_with_retry(db: &PgPool, click: &ClickEvent) -> Result<(), sqlx::Error> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < MAX_RETRIES {
        match insert_click(db, click).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);

                if attempts < MAX_RETRIES {
                    // Exponential backoff base (e.g., 2^attempts * base)
                    let backoff_base = 2u64.pow(attempts);
                    let jitter: u64 = rng().random_range(0..100);
                    let delay_ms = RETRY_DELAY_MS * backoff_base + jitter;

                    tracing::warn!(
                        "Insert failed (attempt {}/{}) - retrying in {} ms...",
                        attempts,
                        MAX_RETRIES,
                        delay_ms
                    );
                    sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}
