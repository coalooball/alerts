use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertMessage {
    pub id: String,
    pub level: String,
    pub message: String,
    pub timestamp: i64,
}

impl AlertMessage {
    pub fn new(id: String, level: String, message: String) -> Self {
        Self {
            id,
            level,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn critical(id: String, message: String) -> Self {
        Self::new(id, "critical".to_string(), message)
    }

    pub fn warning(id: String, message: String) -> Self {
        Self::new(id, "warning".to_string(), message)
    }

    pub fn info(id: String, message: String) -> Self {
        Self::new(id, "info".to_string(), message)
    }
} 