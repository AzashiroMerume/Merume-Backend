use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelReadTracker {
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub channel_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_post_id: Option<ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelReadTrackerPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_post_id: Option<ObjectId>,
}
