use std::sync::Arc;

use axum::{Json, extract::State};
use common::{
    error::AppError,
    models::user::{CreateUser, LoginUser},
};
use serde_json::{Value, json};

use crate::AppState;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Value>, AppError> {
    let user = state.auth_service.register(payload).await?;
    Ok(Json(json!(user)))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Value>, AppError> {
    let token = state
        .auth_service
        .login(payload, &state.config.jwt_secret)
        .await?;
    Ok(Json(json!(token)))
}

pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let token = payload
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or(AppError::BadRequest("token is required".to_string()))?;
    let claims = state
        .auth_service
        .verify_token(token, &state.config.jwt_secret)
        .await?;
    Ok(Json(json!(claims)))
}
