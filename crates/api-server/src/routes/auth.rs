use crate::AppState;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(crate::handlers::auth::register))
        .route("/login", post(crate::handlers::auth::login))
        .route("/verify-token", post(crate::handlers::auth::verify_token))
}
