use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Agent {
    pub id: Uuid,
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub mac: String,
    pub ip: String,
    pub username: String,
    pub registered_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub command: String,
    pub created_at: DateTime<Utc>,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub output: String,
    pub exit_code: i32,
    pub completed_at: DateTime<Utc>,
}
