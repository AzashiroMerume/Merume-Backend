use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(status_code: u16, message: &str) -> Self {
        ErrorResponse {
            status_code,
            message: message.to_string(),
        }
    }
}
