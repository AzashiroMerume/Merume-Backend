use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserInfo {
    pub id: ObjectId,
    pub nickname: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Vec<String>>,
    pub is_online: bool,
    pub last_time_online: DateTime<Utc>,
}
