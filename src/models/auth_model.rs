use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct RegisterPayload {
    #[validate(length(min = 1, max = 20))]
    pub username: String,
    #[validate(length(min = 6, max = 20))]
    pub nickname: String,
    pub email: String,
    #[validate(length(min = 8, max = 50))]
    pub password: String,
    pub firebase_user_id: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct LoginPayload {
    pub identifier: String,
    #[validate(length(min = 8, max = 50))]
    pub password: String,
    pub by_email: bool,
    pub firebase_user_id: String,
}
