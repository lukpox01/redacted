use crate::state::AgentState;

pub async fn quiz_mode_activate(state: &mut AgentState, url: &str) -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
        let mut output = String::new();
        
        if !state.dns_blocked {
            let (dns_output, _) = crate::network::dns::block_dns_whitelist(state, vec![url]).await;
            output.push_str(&format!("DNS Configuration: {}\n", dns_output));
        }
        
        match crate::system::process::launch_browser_kiosk(url) {
            Ok(_) => {
                state.kiosk_active = true;
                output.push_str(&format!("Quiz mode activated: Browser opened in kiosk mode to {}", url));
                (output, 0)
            }
            Err(e) => {
                output.push_str(&e);
                (output, 1)
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        ("Quiz mode only available on Windows".to_string(), 1)
    }
}
