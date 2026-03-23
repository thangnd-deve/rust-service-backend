use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadingHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub story_id: Uuid,
    pub chapter_id: Uuid,
    pub progress: u64,
    pub last_read_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
