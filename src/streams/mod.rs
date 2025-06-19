pub mod producer;
pub mod consumer;

pub async fn get_redis_conn() -> redis::RedisResult<redis::aio::MultiplexedConnection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|err| {
            tracing::error!("Redis connection error: {:?}", err);
            redis::RedisError::from((redis::ErrorKind::IoError, "Multiplexed connection failed"))
        })
}
