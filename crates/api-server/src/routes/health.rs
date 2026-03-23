use crate::AppState;
use crate::handlers::health;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", post(health::health_check))
}
