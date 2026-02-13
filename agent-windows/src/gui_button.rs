#[cfg(windows)]
use std::ptr::null_mut;
#[cfg(windows)]
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(windows)]
use winapi::um::shellapi::ShellExecuteW;
#[cfg(windows)]
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, SW_SHOW,
    WM_COMMAND, WM_DESTROY, WNDCLASSEXW, BS_PUSHBUTTON, WS_VISIBLE, WS_OVERLAPPEDWINDOW, ShowWindow,
};

#[cfg(windows)]
const BUTTON_ID: u16 = 1001;

#[cfg(windows)]
const DEFAULT_URL: &str = "https://www.google.com";

#[cfg(windows)]
fn to_wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let control_id = (wparam & 0xFFFF) as u16;
            if control_id == BUTTON_ID {
                // Open the web page when button is clicked
                let url = to_wide_string(DEFAULT_URL);
                let operation = to_wide_string("open");
                ShellExecuteW(
                    null_mut(),
                    operation.as_ptr(),
                    url.as_ptr(),
                    null_mut(),
                    null_mut(),
                    SW_SHOW,
                );
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
pub fn create_button_window() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let h_instance = GetModuleHandleW(null_mut());
        let class_name = to_wide_string("AgentButtonWindow");
        let window_title = to_wide_string("School Management Agent");
        
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: (5 + 1) as _,
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };

        if RegisterClassExW(&wnd_class) == 0 {
            return Err("Failed to register window class".into());
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            300,
            150,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        );

        if hwnd.is_null() {
            return Err("Failed to create window".into());
        }

        // Create button
        let button_text = to_wide_string("Open Web Page");
        let button_class = to_wide_string("BUTTON");
        
        let button_hwnd = CreateWindowExW(
            0,
            button_class.as_ptr(),
            button_text.as_ptr(),
            WS_VISIBLE | BS_PUSHBUTTON,
            50,
            40,
            200,
            40,
            hwnd,
            BUTTON_ID as _,
            h_instance,
            null_mut(),
        );

        if button_hwnd.is_null() {
            return Err("Failed to create button".into());
        }

        ShowWindow(hwnd, SW_SHOW);

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn create_button_window() -> Result<(), Box<dyn std::error::Error>> {
    Err("GUI button only available on Windows".into())
}

pub fn start_gui_thread() {
    #[cfg(windows)]
    {
        std::thread::spawn(|| {
            if let Err(e) = create_button_window() {
                eprintln!("[-] GUI button error: {}", e);
            }
        });
    }
}
