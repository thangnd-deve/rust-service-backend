use ::serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a user to be created.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Represents a user to be logged in.
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}
