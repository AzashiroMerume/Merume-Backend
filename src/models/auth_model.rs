use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct RegisterPayload {
    #[validate(length(min = 1, max = 255))]
    pub username: String,
    #[validate(length(min = 6, max = 20))]
    pub nickname: String,
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct LoginPayload {
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
