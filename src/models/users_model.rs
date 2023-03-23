use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub nickname: String,
    pub email: String,
    pub password: String,
}
