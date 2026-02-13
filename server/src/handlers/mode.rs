use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use crate::state::State;
use std::sync::{Arc, Mutex};
use super::response::{json_response, error_response};

pub fn handle_get_mode(state: Arc<Mutex<State>>) -> Response<Full<Bytes>> {
    let state = state.lock().unwrap();
    let mode_name = if state.school_mode { "Restricted Mode" } else { "Full C2 Mode" };
    json_response(
        serde_json::json!({
            "school_mode": state.school_mode,
            "mode": mode_name
        }),
        StatusCode::OK,
    )
}

pub fn handle_set_mode(
    body_bytes: &Bytes,
    state: Arc<Mutex<State>>,
) -> Response<Full<Bytes>> {
    #[derive(serde::Deserialize)]
    struct SetModeRequest {
        password: String,
        school_mode: bool,
    }

    let request: SetModeRequest = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(_) => return error_response("Invalid JSON", StatusCode::BAD_REQUEST),
    };

    if request.password != "admin" {
        return error_response("Unauthorized", StatusCode::UNAUTHORIZED);
    }

    let mut state = state.lock().unwrap();
    state.school_mode = request.school_mode;
    
    let mode_name = if request.school_mode { "Restricted Mode" } else { "Full C2 Mode" };
    json_response(
        serde_json::json!({
            "success": true,
            "mode": mode_name,
            "school_mode": request.school_mode
        }),
        StatusCode::OK,
    )
}
