use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BoolResponse {
    pub success: bool,
    pub error_message: Option<String>,
}
