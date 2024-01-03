use crate::models::{channel_model::Channel, post_model::Post, user_info_model::UserInfo};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OperationStatusResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelResponse {
    pub success: bool,
    pub data: Option<Channel>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPostResponse {
    pub success: bool,
    pub data: Option<Vec<Post>>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecommendedChannelResponse {
    pub success: bool,
    pub data: Option<Vec<Channel>>,
    pub page: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_info: Option<UserInfo>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PreferencesResponse {
    pub success: bool,
    pub data: Option<Vec<String>>,
    pub error_message: Option<String>,
}
