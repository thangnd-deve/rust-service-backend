use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Story {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub cover_image_url: String,
    pub author_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStory {
    pub title: String,
    pub description: String,
    pub cover_image_url: String,
}
