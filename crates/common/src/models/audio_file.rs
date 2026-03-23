use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioFile {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub file_path: String,
    pub file_size: u64,
    pub duration_second: u64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioFileCreate {
    pub chapter_id: Uuid,
    pub file_path: String,
    pub file_size: u64,
    pub duration_second: u64,
}
