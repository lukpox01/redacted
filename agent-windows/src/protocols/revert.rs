use crate::state::AgentState;

pub async fn revert_all_changes(state: &mut AgentState) -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
        let mut output = String::new();
        let mut exit_code = 0;
        
        // Restore DNS
        let (dns_output, dns_code) = crate::network::dns::restore_dns(state).await;
        output.push_str(&dns_output);
        if dns_code != 0 {
            exit_code = dns_code;
        }
        
        // Clean hosts file
        crate::network::dns::clean_hosts_file();
        output.push_str("Hosts file cleaned\n");
        
        // Kill kiosk browsers
        if state.kiosk_active {
            crate::system::process::kill_browser_processes();
            state.kiosk_active = false;
            output.push_str("Kiosk mode terminated\n");
        }
        
        // Re-enable Task Manager
        let (taskmgr_output, _) = crate::system::registry::enable_task_manager().await;
        output.push_str(&taskmgr_output);
        output.push_str("\n");
        
        output.push_str("All changes reverted successfully");
        (output, exit_code)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        ("Revert only available on Windows".to_string(), 1)
    }
}
