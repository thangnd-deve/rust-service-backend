use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub id: Uuid,
    pub story_id: Uuid,
    pub title: String,
    pub content: String,
    pub chapter_order: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChapter {
    pub story_id: Uuid,
    pub title: String,
    pub content: String,
    pub chapter_order: u32,
}
