use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::ops::Index;
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
    id: Uuid,
}

type Agents = Arc<Mutex<Vec<Agent>>>;

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    agents: Agents,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/checkin") => {
            // get the request into bytes for serde
            let body_bytes = match req.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Internal Server Error")));
                    *bad_request.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(bad_request);
                }
            };

            let check_in: Result<CheckIn, _> = serde_json::from_slice(&body_bytes);

            match check_in {
                Ok(parsed_content) => {
                    let agent_id = {
                        let mut guard = agents.lock().unwrap();
                        let agent = Agent {
                            os: parsed_content.os,
                            hostname: parsed_content.hostname,
                            mac: parsed_content.mac,
                            ip: parsed_content.ip,
                            id: Uuid::new_v4(),
                        };
                        guard.push(agent);
                        guard.last().unwrap().id
                    };
                    println!("{:#?}", Arc::clone(&agents));
                    Ok(Response::new(Full::new(Bytes::from(agent_id.to_string()))))
                }
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Bad Request: Invalid Json")));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    Ok(bad_request)
                }
            }
        }

        (&Method::POST, "/beacon") => {
            // 1. Collect the body using `BodyExt::collect()`. This replaces `to_bytes`.
            let body_bytes = match req.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => {
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Internal Server Error")));
                    *bad_request.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(bad_request);
                }
            };

            let beacon: Result<Beacon, _> = serde_json::from_slice(&body_bytes);

            match beacon {
                Ok(parsed_beacon) => {
                    println!("Received beacon: {:?}", parsed_beacon);
                    let agent = {
                        let guard = agents.lock().unwrap();
                        let idx = guard
                            .iter()
                            .position(|pos| pos.id.to_string() == parsed_beacon.id)
                            .unwrap();
                        guard[idx].mac.clone()
                    };
                    Ok(Response::new(Full::new(Bytes::from(agent))))
                }
                Err(e) => {
                    println!("Failed to parse beacon: {}", e);
                    let mut bad_request =
                        Response::new(Full::new(Bytes::from("Bad Request: Invalid JSON")));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    Ok(bad_request)
                }
            }
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
    let agents: Agents = Arc::new(Mutex::new(Vec::new()));
    // The new server loop. It spawns a task for each incoming connection.
    loop {
        let agents = Arc::clone(&agents);
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let svc = service_fn(move |req| {
            let agents = Arc::clone(&agents);
            handle_request(req, agents)
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
