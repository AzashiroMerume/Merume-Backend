use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub owner_id: Option<ObjectId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_image: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_image: Option<String>,
}
