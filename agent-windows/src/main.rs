use protocol::*;
use std::process::Command;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use std::env;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const BEACON_INTERVAL: u64 = 10;
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    dns_blocked: bool,
    original_dns: Vec<String>,
    kiosk_active: bool,
}

impl AgentState {
    fn new() -> Self {
        let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());
        let os = sys_info::os_type().unwrap_or_else(|_| "unknown".to_string());
        let os_version = get_detailed_os_version();
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string());

        AgentState {
            agent_id: None,
            hostname,
            os,
            os_version,
            username,
            dns_blocked: false,
            original_dns: Vec::new(),
            kiosk_active: false,
        }
    }
}

fn get_detailed_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("cmd")
            .args(["/C", "ver"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }
    
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

async fn execute_task(task: &Task, state: &mut AgentState) -> (String, i32) {
    println!("[*] Executing task {}: {}", task.task_id, task.command);

    if task.command.starts_with("PROTOCOL:") {
        return execute_protocol(&task.command, state).await;
    }

    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", &task.command])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

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

async fn execute_protocol(command: &str, state: &mut AgentState) -> (String, i32) {
    let protocol_cmd = command.strip_prefix("PROTOCOL:").unwrap_or("");
    let parts: Vec<&str> = protocol_cmd.split('|').collect();
    
    if parts.is_empty() {
        return ("Invalid protocol command".to_string(), 1);
    }

    let protocol_name = parts[0];
    
    match protocol_name {
        "QUIZ_MODE" => {
            if parts.len() < 2 {
                return ("Quiz mode requires URL parameter".to_string(), 1);
            }
            let url = parts[1];
            quiz_mode_activate(state, url).await
        }
        "BLOCK_DNS" => block_dns(state).await,
        "BLOCK_DNS_WHITELIST" => {
            if parts.len() < 2 {
                return ("Whitelist mode requires allowed domains".to_string(), 1);
            }
            let whitelist = parts[1..].to_vec();
            block_dns_whitelist(state, whitelist).await
        }
        "GET_FILE" => {
            if parts.len() < 2 {
                return ("Get file requires path parameter".to_string(), 1);
            }
            let path = parts[1];
            get_file(path).await
        }
        "UPLOAD_FILE" => {
            if parts.len() < 3 {
                return ("Upload file requires path and content parameters".to_string(), 1);
            }
            let path = parts[1];
            let content = parts[2..].join("|");
            upload_file(path, &content).await
        }
        "REVERT_ALL" => revert_all_changes(state).await,
        "LOCK_SCREEN" => lock_screen().await,
        "DISABLE_TASK_MANAGER" => disable_task_manager().await,
        "ENABLE_TASK_MANAGER" => enable_task_manager().await,
        _ => (format!("Unknown protocol: {}", protocol_name), 1),
    }
}

#[cfg(windows)]
async fn quiz_mode_activate(state: &mut AgentState, url: &str) -> (String, i32) {
    let mut output = String::new();
    
    if !state.dns_blocked {
        let (dns_output, _) = block_dns_whitelist(state, vec![url]).await;
        output.push_str(&format!("DNS Configuration: {}\n", dns_output));
    }
    
    let browser_path = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
    let browser_path_alt = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";
    
    let browser = if std::path::Path::new(browser_path).exists() {
        browser_path
    } else if std::path::Path::new(browser_path_alt).exists() {
        browser_path_alt
    } else {
        return ("No suitable browser found".to_string(), 1);
    };
    
    match Command::new(browser)
        .args(["--kiosk", "--disable-pinch", "--overscroll-history-navigation=0", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        Ok(_) => {
            state.kiosk_active = true;
            output.push_str(&format!("Quiz mode activated: Browser opened in kiosk mode to {}", url));
            (output, 0)
        }
        Err(e) => {
            output.push_str(&format!("Failed to start browser: {}", e));
            (output, 1)
        }
    }
}

#[cfg(not(windows))]
async fn quiz_mode_activate(_state: &mut AgentState, _url: &str) -> (String, i32) {
    ("Quiz mode only available on Windows".to_string(), 1)
}

#[cfg(windows)]
async fn block_dns(state: &mut AgentState) -> (String, i32) {
    if state.dns_blocked {
        return ("DNS already blocked".to_string(), 0);
    }
    
    let get_dns = Command::new("powershell")
        .args(["-Command", "Get-DnsClientServerAddress -AddressFamily IPv4 | Where-Object {$_.ServerAddresses} | Select-Object -ExpandProperty ServerAddresses"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    
    if let Ok(output) = get_dns {
        let dns_servers = String::from_utf8_lossy(&output.stdout);
        state.original_dns = dns_servers.lines().map(|s| s.trim().to_string()).collect();
    }
    
    let result = Command::new("powershell")
        .args(["-Command", "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ServerAddresses @('127.0.0.1')"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    
    match result {
        Ok(output) if output.status.success() => {
            state.dns_blocked = true;
            ("DNS blocked for all active adapters".to_string(), 0)
        }
        Ok(_) => ("Failed to block DNS".to_string(), 1),
        Err(e) => (format!("Error blocking DNS: {}", e), 1),
    }
}

#[cfg(not(windows))]
async fn block_dns(_state: &mut AgentState) -> (String, i32) {
    ("DNS blocking only available on Windows".to_string(), 1)
}

#[cfg(windows)]
async fn block_dns_whitelist(state: &mut AgentState, whitelist: Vec<&str>) -> (String, i32) {
    if state.dns_blocked {
        return ("DNS already configured".to_string(), 0);
    }
    
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let mut hosts_content = match std::fs::read_to_string(hosts_path) {
        Ok(content) => content,
        Err(e) => return (format!("Failed to read hosts file: {}", e), 1),
    };
    
    hosts_content.push_str("\n# SCHOOL ORCHESTRATION - Quiz Mode\n");
    hosts_content.push_str("# Block all except whitelisted domains\n");
    
    for domain in whitelist {
        hosts_content.push_str(&format!("# ALLOW: {}\n", domain));
    }
    
    if let Err(e) = std::fs::write(hosts_path, hosts_content) {
        return (format!("Failed to write hosts file: {}", e), 1);
    }
    
    state.dns_blocked = true;
    ("DNS configured for quiz mode".to_string(), 0)
}

#[cfg(not(windows))]
async fn block_dns_whitelist(_state: &mut AgentState, _whitelist: Vec<&str>) -> (String, i32) {
    ("DNS whitelist only available on Windows".to_string(), 1)
}

async fn get_file(path: &str) -> (String, i32) {
    match std::fs::read(path) {
        Ok(content) => {
            use base64::{Engine as _, engine::general_purpose};
            let encoded = general_purpose::STANDARD.encode(&content);
            (format!("FILE_CONTENT:{}", encoded), 0)
        }
        Err(e) => (format!("Failed to read file: {}", e), 1),
    }
}

async fn upload_file(path: &str, content: &str) -> (String, i32) {
    use base64::{Engine as _, engine::general_purpose};
    let decoded = match general_purpose::STANDARD.decode(content) {
        Ok(data) => data,
        Err(e) => return (format!("Failed to decode content: {}", e), 1),
    };
    
    match std::fs::write(path, decoded) {
        Ok(_) => (format!("File uploaded to: {}", path), 0),
        Err(e) => (format!("Failed to write file: {}", e), 1),
    }
}

#[cfg(windows)]
async fn revert_all_changes(state: &mut AgentState) -> (String, i32) {
    let mut output = String::new();
    let mut exit_code = 0;
    
    if state.dns_blocked {
        if !state.original_dns.is_empty() {
            let dns_list = state.original_dns.join("','");
            let cmd = format!("Get-NetAdapter | Where-Object {{$_.Status -eq 'Up'}} | Set-DnsClientServerAddress -ServerAddresses @('{}')", dns_list);
            
            if let Ok(result) = Command::new("powershell")
                .args(["-Command", &cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                if result.status.success() {
                    output.push_str("DNS restored\n");
                    state.dns_blocked = false;
                } else {
                    output.push_str("Failed to restore DNS\n");
                    exit_code = 1;
                }
            }
        } else {
            let _ = Command::new("powershell")
                .args(["-Command", "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ResetServerAddresses"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            output.push_str("DNS reset to automatic\n");
            state.dns_blocked = false;
        }
    }
    
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    if let Ok(content) = std::fs::read_to_string(hosts_path) {
        let cleaned: Vec<&str> = content.lines()
            .filter(|line| !line.contains("SCHOOL ORCHESTRATION"))
            .collect();
        let _ = std::fs::write(hosts_path, cleaned.join("\n"));
        output.push_str("Hosts file cleaned\n");
    }
    
    if state.kiosk_active {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "chrome.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "msedge.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        state.kiosk_active = false;
        output.push_str("Kiosk mode terminated\n");
    }
    
    let _ = enable_task_manager().await;
    output.push_str("Task Manager enabled\n");
    
    output.push_str("All changes reverted successfully");
    (output, exit_code)
}

#[cfg(not(windows))]
async fn revert_all_changes(_state: &mut AgentState) -> (String, i32) {
    ("Revert only available on Windows".to_string(), 1)
}

#[cfg(windows)]
async fn lock_screen() -> (String, i32) {
    let result = Command::new("rundll32.exe")
        .args(["user32.dll,LockWorkStation"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    
    match result {
        Ok(_) => ("Screen locked".to_string(), 0),
        Err(e) => (format!("Failed to lock screen: {}", e), 1),
    }
}

#[cfg(not(windows))]
async fn lock_screen() -> (String, i32) {
    ("Screen lock only available on Windows".to_string(), 1)
}

#[cfg(windows)]
async fn disable_task_manager() -> (String, i32) {
    let result = Command::new("reg")
        .args(["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System", 
               "/v", "DisableTaskMgr", "/t", "REG_DWORD", "/d", "1", "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    
    match result {
        Ok(output) if output.status.success() => ("Task Manager disabled".to_string(), 0),
        Ok(_) => ("Failed to disable Task Manager".to_string(), 1),
        Err(e) => (format!("Error: {}", e), 1),
    }
}

#[cfg(not(windows))]
async fn disable_task_manager() -> (String, i32) {
    ("Task Manager control only available on Windows".to_string(), 1)
}

#[cfg(windows)]
async fn enable_task_manager() -> (String, i32) {
    let result = Command::new("reg")
        .args(["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System", 
               "/v", "DisableTaskMgr", "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    
    match result {
        Ok(_) => ("Task Manager enabled".to_string(), 0),
        Err(e) => (format!("Error: {}", e), 1),
    }
}

#[cfg(not(windows))]
async fn enable_task_manager() -> (String, i32) {
    ("Task Manager control only available on Windows".to_string(), 1)
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
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("getmac")
            .arg("/FO").arg("CSV").arg("/NH")
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
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
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("ipconfig")
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        if let Ok(output) = result {
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
    println!("[*] School Management Agent starting...");
    println!("[*] Management Server: {}", server_url);
    
    let client = reqwest::Client::new();
    let mut state = AgentState::new();
    let mut interval = time::interval(Duration::from_secs(BEACON_INTERVAL));

    loop {
        interval.tick().await;

        if state.agent_id.is_none() {
            println!("[*] Attempting to register with management server...");
            if let Err(e) = register_agent(&client, &mut state, &server_url).await {
                println!("[-] Registration failed: {}. Retrying...", e);
                continue;
            }
        }

        match send_beacon(&client, &state, &server_url).await {
            Ok(Some(task)) => {
                let (output, exit_code) = execute_task(&task, &mut state).await;
                
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
