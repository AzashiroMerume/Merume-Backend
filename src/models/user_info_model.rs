use std::usize;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
}
