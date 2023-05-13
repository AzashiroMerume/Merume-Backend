use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner_id: ObjectId,
    pub channel_id: ObjectId,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    pub written_challenge_day: usize,
    pub likes: usize,
    pub dislikes: usize,
    pub already_changed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdatePost {
    pub body: Option<String>,
    pub images: Option<Vec<String>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub already_changed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct PostPayload {
    #[validate(length(min = 1))]
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}
