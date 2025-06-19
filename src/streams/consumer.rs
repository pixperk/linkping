
use std::time::Duration;

use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use tokio::time::sleep;
use crate::{models::click::ClickEvent, streams::producer::STREAM_KEY};

const CONSUMER_GROUP: &str = "click_consumers";
const CONSUMER_NAME: &str = "linkping_consumer";
const BLOCK_TIME_MS: u64 = 5000;
const BATCH_SIZE: u64 = 10;
const RETRY_DELAY_MS: u64 = 500;
const MAX_RETRIES: u32 = 3;

pub async fn consume_click_events(db: &PgPool, mut conn: MultiplexedConnection) -> redis::RedisResult<()> {
    // Initialize consumer group
    initialize_consumer_group(&mut conn).await?;

    loop {
        // Read messages from stream
        match read_stream_messages(&mut conn).await {
            Ok(reply) => {
                // Process messages if any were received
                if !reply.keys.is_empty() {
                    process_stream_reply(db, &mut conn, reply).await;
                }
            },
            Err(e) => {
                tracing::error!("Redis XREADGROUP error: {:?}", e);
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        // Small delay between polling iterations
        sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
    }
}

async fn initialize_consumer_group(conn: &mut MultiplexedConnection) -> redis::RedisResult<()> {
    // Try to create consumer group (only once)
    let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(STREAM_KEY)
        .arg(CONSUMER_GROUP)
        .arg("0") // read from beginning
        .arg("MKSTREAM") // create stream if it doesn't exist
        .query_async(conn)
        .await;
    
    // Ignore BUSYGROUP error (group already exists)
    match result {
        Ok(_) => {
            tracing::info!("Created consumer group: {}", CONSUMER_GROUP);
            Ok(())
        },
        Err(e) => {
            if e.to_string().contains("BUSYGROUP") {
                tracing::info!("Consumer group already exists: {}", CONSUMER_GROUP);
                Ok(())
            } else {
                tracing::error!("Failed to create consumer group: {:?}", e);
                Err(e)
            }
        }
    }
}

async fn read_stream_messages(conn: &mut MultiplexedConnection) -> redis::RedisResult<redis::streams::StreamReadReply> {
    redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(CONSUMER_GROUP)
        .arg(CONSUMER_NAME)
        .arg("BLOCK")
        .arg(BLOCK_TIME_MS)
        .arg("COUNT")
        .arg(BATCH_SIZE)
        .arg("STREAMS")
        .arg(STREAM_KEY)
        .arg(">") // Only new messages
        .query_async(conn)
        .await
}

async fn process_stream_reply(db: &PgPool, conn: &mut MultiplexedConnection, reply: redis::streams::StreamReadReply) {
    for stream_key in reply.keys {
        for stream_id in stream_key.ids {
            process_stream_message(db, conn, &stream_key.key, &stream_id).await;
        }
    }
}

async fn process_stream_message(
    db: &PgPool,
    conn: &mut MultiplexedConnection,
    stream_key: &str,
    stream_id: &redis::streams::StreamId,
) {
    for (_field, value) in stream_id.map.iter() {
        if let Some(event) = parse_event_from_value(value) {
            tracing::info!("Processing click event for slug: {}", event.slug);
            
            if let Err(e) = insert_click_with_retry(db, &event).await {
                tracing::error!("Failed to insert click after retries: {:?}", e);
            }
        }

        // Always acknowledge the message to prevent reprocessing
        acknowledge_message(conn, stream_key, &stream_id.id).await;
    }
}

fn parse_event_from_value(value: &redis::Value) -> Option<ClickEvent> {
    match value {
        redis::Value::BulkString(bytes) => {
            match std::str::from_utf8(bytes) {
                Ok(json_str) => {
                    match serde_json::from_str::<ClickEvent>(json_str) {
                        Ok(event) => Some(event),
                        Err(e) => {
                            tracing::warn!("Failed to parse event: {:?}, error: {:?}", json_str, e);
                            None
                        }
                    }
                },
                Err(_) => {
                    tracing::warn!("Value is not valid UTF-8: {:?}", bytes);
                    None
                }
            }
        },
        _ => {
            tracing::warn!("Value is not a BulkString variant: {:?}", value);
            None
        }
    }
}

async fn acknowledge_message(conn: &mut MultiplexedConnection, stream_key: &str, message_id: &str) {
    let result: Result<(), redis::RedisError> = redis::cmd("XACK")
        .arg(stream_key)
        .arg(CONSUMER_GROUP)
        .arg(message_id)
        .query_async(conn)
        .await;
        
    if let Err(e) = result {
        tracing::error!("Failed to ACK message {}: {:?}", message_id, e);
    }
}

async fn insert_click_with_retry(db: &PgPool, click: &ClickEvent) -> Result<(), sqlx::Error> {
    //TODO : Implement exponential backoff with jitter
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < MAX_RETRIES {
        match insert_click(db, click).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                
                if attempts < MAX_RETRIES {
                    tracing::warn!(
                        "Database insert failed (attempt {}/{}), retrying...",
                        attempts,
                        MAX_RETRIES
                    );
                    sleep(Duration::from_millis(RETRY_DELAY_MS * attempts as u64)).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

async fn insert_click(db: &PgPool, click: &ClickEvent) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO clicks (slug, ip, user_agent, referer, timestamp) VALUES ($1, $2, $3, $4, $5)",
        click.slug,
        click.ip,
        click.user_agent,
        click.referer,
        click.timestamp
    )
    .execute(db)
    .await?;

    Ok(())
}
