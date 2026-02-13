use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use protocol::*;
use crate::state::State;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use super::response::{json_response, error_response};

pub fn handle_add_task(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let request: AddTaskRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    if request.password != "admin" {
        return error_response("Unauthorized", StatusCode::UNAUTHORIZED);
    }

    let mut state = state.lock().unwrap();
    if !state.agent_exists(request.agent_id) {
        return error_response("Unknown agent", StatusCode::NOT_FOUND);
    }

    let task_id = state.add_task(request.agent_id, request.command);
    json_response(
        AddTaskResponse {
            task_id,
            message: format!("Task queued for agent {}", request.agent_id),
        },
        StatusCode::OK,
    )
}

pub fn handle_protocol(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let request: AddTaskRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    if request.password != "admin" {
        return error_response("Unauthorized", StatusCode::UNAUTHORIZED);
    }

    let mut state = state.lock().unwrap();
    
    if !state.school_mode && !request.command.starts_with("PROTOCOL:") {
        return error_response("Protocol commands only available in Restricted Mode", StatusCode::FORBIDDEN);
    }

    if !state.agent_exists(request.agent_id) {
        return error_response("Unknown agent", StatusCode::NOT_FOUND);
    }

    let task_id = state.add_task(request.agent_id, request.command);
    json_response(
        AddTaskResponse {
            task_id,
            message: format!("Protocol task queued for agent {}", request.agent_id),
        },
        StatusCode::OK,
    )
}

pub fn handle_revert_all(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    #[derive(serde::Deserialize)]
    struct RevertRequest {
        password: String,
        agent_id: Uuid,
    }

    let request: RevertRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    if request.password != "admin" {
        return error_response("Unauthorized", StatusCode::UNAUTHORIZED);
    }

    let mut state = state.lock().unwrap();
    if !state.agent_exists(request.agent_id) {
        return error_response("Unknown agent", StatusCode::NOT_FOUND);
    }

    let task_id = state.add_task(request.agent_id, "PROTOCOL:REVERT_ALL".to_string());
    json_response(
        AddTaskResponse {
            task_id,
            message: format!("Revert command queued for agent {}", request.agent_id),
        },
        StatusCode::OK,
    )
}

pub fn handle_get_task(
    task_id: Uuid,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let state = state.lock().unwrap();
    match state.get_task(task_id) {
        Some(task_info) => json_response(task_info, StatusCode::OK),
        None => error_response("Task not found", StatusCode::NOT_FOUND),
    }
}

pub fn handle_list_agent_tasks(
    agent_id: Uuid,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let state = state.lock().unwrap();
    let tasks = state.list_tasks_for_agent(agent_id);
    json_response(tasks, StatusCode::OK)
}
