use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserChannel {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub channel_id: Option<ObjectId>,
    pub is_owner: Option<bool>,
    pub subscribed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}
