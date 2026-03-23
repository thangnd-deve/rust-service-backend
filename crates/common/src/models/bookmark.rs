use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookMark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub chapter_id: Uuid,
    pub created_at: DateTime<Utc>,
}

pub struct CreateBookMark {
    pub user_id: Uuid,
    pub chapter_id: Uuid,
}
