use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liked: Option<Vec<ObjectId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarks: Option<Vec<ObjectId>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UserPreferencesPayload {
    #[validate(length(min = 1))]
    pub preferences: Vec<String>,
}
