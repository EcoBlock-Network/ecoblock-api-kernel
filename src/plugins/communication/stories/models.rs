use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct StoryCreate {
    pub title: Option<String>,
    pub media_url: String,
    pub caption: Option<String>,
    pub is_active: Option<bool>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoryUpdate {
    pub title: Option<String>,
    pub media_url: Option<String>,
    pub caption: Option<String>,
    pub is_active: Option<bool>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct StoryDto {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub media_url: String,
    pub caption: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: String,
}

