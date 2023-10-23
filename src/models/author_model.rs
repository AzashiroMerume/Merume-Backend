use std::usize;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub id: ObjectId,
    pub nickname: String,
    pub username: String,
}
