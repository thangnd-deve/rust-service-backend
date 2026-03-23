use common::{
    error::AppError,
    models::user::{CreateUser, LoginUser, User},
};
use uuid::Uuid;

use crate::utils::{date_time::DateTime, hash_string::HashString, jwt::JWT};

#[derive(Clone)]
pub struct AuthService {
    pub db: sqlx::PgPool,
}

impl AuthService {
    pub async fn register(&self, params: CreateUser) -> Result<User, AppError> {
        let password_hash = HashString::hash_password(&params.password);
        let now = DateTime::now();

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            params.username,
            params.email,
            password_hash,
            now,
            now,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn login(&self, payload: LoginUser, secret_key: &str) -> Result<String, AppError> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", payload.email,)
            .fetch_one(&self.db)
            .await?;

        if !HashString::verify_password(&payload.password, &user.password_hash) {
            return Err(AppError::UnAuthorized("Invalid password".to_string()));
        }

        let token = JWT::generate_token(&user.id.to_string(), secret_key)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(token)
    }

    pub async fn verify_token(&self, token: &str, secret_key: &str) -> Result<User, AppError> {
        let claim =
            JWT::verify_token(token, secret_key).map_err(|e| AppError::Internal(e.to_string()))?;

        let user_id = Uuid::parse_str(&claim.sub).map_err(|e| AppError::Internal(e.to_string()))?;

        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(&self.db)
            .await?;
        Ok(user)
    }
}
