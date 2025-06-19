use redis::AsyncCommands;
use serde_json::json;

use crate::{models::click::ClickEvent, streams::get_redis_conn};


pub const STREAM_KEY : &str = "click_events";

pub async fn publish_click_event(event : ClickEvent) -> redis::RedisResult<()>{
    let mut conn = get_redis_conn().await?;
    let payload = json!({
        "slug": event.slug,
        "ip": event.ip,
        "user_agent": event.user_agent,
        "referer": event.referer,
        "timestamp": event.timestamp.to_string()
    });
    
    let event_json = serde_json::to_string(&payload)
        .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization error", e.to_string())))?;
    let _ : String = conn
        .xadd(
            STREAM_KEY,
            "*",
            &[("event", event_json.as_str())]
        ).await?;

    tracing::info!("Published click event: {:?}", payload);
    Ok(())
}