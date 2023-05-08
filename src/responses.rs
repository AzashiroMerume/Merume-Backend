use crate::models::{channel_model::Channel, post_model::Post};

use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MainResponse<T> {
    pub success: bool,
    pub data: Option<Vec<T>>,
    pub page: Option<i32>,
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
pub struct RecomendedContentResponse {
    pub success: bool,
    pub data: Option<HashMap<Channel, Post>>,
    pub page: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OperationStatusResponse {
    pub success: bool,
    pub error_message: Option<String>,
}
