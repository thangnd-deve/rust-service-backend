mod handlers;
mod routes;
mod services;
mod utils;

use axum::Router;
use common::{config::AppConfig, db, redis::create_redis_pool};
use std::sync::Arc;

use crate::services::auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub auth_service: AuthService,
}

// init service
async fn init() {
    let config = AppConfig::from_env().expect("Failed to load config");
    let pool = db::create_pool(&config.database_url).await.unwrap();
    let redis = create_redis_pool(&config.redis_url);

    let state = Arc::new(AppState {
        config: config.clone(),
        redis: redis,
        db: pool.clone(),
        auth_service: AuthService { db: pool },
    });

    let address_string = format!("{}:{}", &config.app_url.to_string(), config.app_port);
    let listener = tokio::net::TcpListener::bind(&address_string)
        .await
        .unwrap();

    let app = Router::new()
        .nest("/auth", routes::auth::router())
        .nest("/health", routes::health::router())
        .with_state(state);

    println!("Server is running: {}", address_string);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // init service
    init().await
}
