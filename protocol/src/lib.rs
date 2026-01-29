use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub mac: String,
    pub ip: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconRequest {
    pub agent_id: Uuid,
    pub hostname: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconResponse {
    pub task: Option<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: Uuid,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultRequest {
    pub agent_id: Uuid,
    pub task_id: Uuid,
    pub output: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskRequest {
    pub password: String,
    pub agent_id: Uuid,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskResponse {
    pub task_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub command: String,
    pub created_at: String,
    pub result: Option<TaskResultInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultInfo {
    pub output: String,
    pub exit_code: i32,
    pub completed_at: String,
}
