use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthResponse {
    pub token: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BoolResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MainResponse<T> {
    pub success: bool,
    pub data: Option<Vec<T>>,
    pub error_message: Option<String>,
}
