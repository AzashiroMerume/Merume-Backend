use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub owner_id: Option<ObjectId>,
    pub channel_id: Option<String>,
    pub body: Option<String>,
    pub images: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostPayload {
    pub body: Option<String>,
    pub images: Option<String>,
}
