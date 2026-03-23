use chrono::{DateTime, Days, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error};
use serde::{Deserialize, Serialize};

pub struct JWT {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl JWT {
    pub fn generate_token(user_id: &str, secret: &str) -> Result<String, Error> {
        let now: DateTime<Utc> = Utc::now();
        let exp = now + Days::new(7);
        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }
    pub fn verify_token(token: &str, secret: &str) -> Result<Claims, Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}
