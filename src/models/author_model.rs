use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub id: ObjectId,
    pub nickname: String,
    pub username: String,
}
