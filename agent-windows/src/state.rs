use uuid::Uuid;

#[derive(Debug)]
pub struct AgentState {
    pub agent_id: Option<Uuid>,
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub username: String,
    pub dns_blocked: bool,
    pub original_dns: Vec<String>,
    pub kiosk_active: bool,
}

impl AgentState {
    pub fn new() -> Self {
        let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());
        let os = sys_info::os_type().unwrap_or_else(|_| "unknown".to_string());
        let os_version = crate::system::info::get_detailed_os_version();
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

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}
