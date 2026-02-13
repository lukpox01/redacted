use crate::state::AgentState;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn block_dns(state: &mut AgentState) -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
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
    
    #[cfg(not(target_os = "windows"))]
    {
        ("DNS blocking only available on Windows".to_string(), 1)
    }
}

pub async fn block_dns_whitelist(state: &mut AgentState, whitelist: Vec<&str>) -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
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
    
    #[cfg(not(target_os = "windows"))]
    {
        ("DNS whitelist only available on Windows".to_string(), 1)
    }
}

pub async fn restore_dns(state: &mut AgentState) -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
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
        
        (output, exit_code)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        ("DNS restore only available on Windows".to_string(), 0)
    }
}

pub fn clean_hosts_file() {
    #[cfg(target_os = "windows")]
    {
        let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
        if let Ok(content) = std::fs::read_to_string(hosts_path) {
            let cleaned: Vec<&str> = content.lines()
                .filter(|line| !line.contains("SCHOOL ORCHESTRATION"))
                .collect();
            let _ = std::fs::write(hosts_path, cleaned.join("\n"));
        }
    }
}
