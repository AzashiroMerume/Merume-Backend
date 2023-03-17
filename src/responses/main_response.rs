use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MainResponse<T> {
    pub success: bool,
    pub data: Option<Vec<T>>,
    pub error_message: Option<String>,
}
