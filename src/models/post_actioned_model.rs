use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReadPost {
    #[serde(rename = "_id")]
    #[serde(skip_deserializing)]
    pub id: ObjectId,
    pub post_id: ObjectId,
    #[serde(skip_deserializing)]
    pub user_id_who_read: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarked: Option<bool>,
}
