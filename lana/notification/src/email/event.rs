use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailEvent {
    Requested {
        id: uuid::Uuid,
        recipient: String,
        subject: String,
        template_name: String,
        template_data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    Sent {
        id: uuid::Uuid,
        timestamp: DateTime<Utc>,
    },
    Failed {
        id: uuid::Uuid,
        error: String,
        attempt: u32,
        timestamp: DateTime<Utc>,
    },
}
