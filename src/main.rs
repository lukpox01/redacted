use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::net::SocketAddr;
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

struct Task {
    command: String,
}

type AgentId = Uuid;
type TaskId = Uuid;

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
    let agent = {
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
        match guard.agents.get(&uuid_v4) {
            Some(agent) => Agent {
                os: agent.os.clone(),
                hostname: agent.hostname.clone(),
                mac: agent.mac.clone(),
                ip: agent.ip.clone(),
            },
            None => {
                let mut bad_request =
                    Response::new(Full::new(Bytes::from("Bad Request: Invalid AgentId")));
                *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_request);
            }
        }
    };
    Ok(Response::new(Full::new(Bytes::from(format!(
        "agent found thanks to you \n{:#?}",
        agent
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
