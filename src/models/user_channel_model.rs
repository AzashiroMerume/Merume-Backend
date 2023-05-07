use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserChannel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub channel_id: ObjectId,
    pub is_owner: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}
