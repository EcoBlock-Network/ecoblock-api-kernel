use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct BlogCreate {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub author: String,
    pub is_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlogUpdate {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub body: Option<String>,
    pub author: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct BlogDto {
    pub id: uuid::Uuid,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub author: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
