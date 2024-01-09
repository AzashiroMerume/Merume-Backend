use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub id: ObjectId,
    pub nickname: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_online: Option<bool>,
}
