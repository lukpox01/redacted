use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn get_detailed_os_version() -> String {
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

pub fn get_mac_address() -> String {
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

pub fn get_local_ip() -> String {
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
