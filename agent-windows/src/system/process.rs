use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn lock_screen() -> (String, i32) {
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("rundll32.exe")
            .args(["user32.dll,LockWorkStation"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
        
        match result {
            Ok(_) => ("Screen locked".to_string(), 0),
            Err(e) => (format!("Failed to lock screen: {}", e), 1),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        ("Screen lock only available on Windows".to_string(), 1)
    }
}

pub fn kill_browser_processes() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "chrome.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "msedge.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
}

pub fn launch_browser_kiosk(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let browser_path = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
        let browser_path_alt = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";
        
        let browser = if std::path::Path::new(browser_path).exists() {
            browser_path
        } else if std::path::Path::new(browser_path_alt).exists() {
            browser_path_alt
        } else {
            return Err("No suitable browser found".to_string());
        };
        
        match Command::new(browser)
            .args(["--kiosk", "--disable-pinch", "--overscroll-history-navigation=0", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to start browser: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Browser kiosk mode only available on Windows".to_string())
    }
}
