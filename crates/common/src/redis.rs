use deadpool_redis::{Config, Pool};

pub fn create_redis_pool(redis_url: &str) -> Pool {
    let config = Config::from_url(redis_url);
    config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}
