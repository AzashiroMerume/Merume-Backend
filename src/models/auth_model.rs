use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RegisterPayload {
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoginPayload {
    pub email: Option<String>,
    pub password: Option<String>,
}
