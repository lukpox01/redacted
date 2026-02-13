# Testing the GUI Button Feature

## Prerequisites
- Windows machine (Windows 7 or later)
- Rust toolchain installed
- Build the agent: `cargo build --release`

## Manual Testing Steps

### 1. Build and Run the Agent
```bash
cd agent-windows
cargo run
```

### 2. Verify GUI Window Appears
- A window titled "School Management Agent" should appear
- The window should contain a button labeled "Open Web Page"
- The window should be visible on the desktop

### 3. Test Button Functionality
- Click the "Open Web Page" button
- Verify that the default browser opens
- Verify that it navigates to https://www.google.com

### 4. Test Customization
- Edit `src/gui_button.rs` and change the `DEFAULT_URL` constant:
  ```rust
  const DEFAULT_URL: &str = "https://example.com";
  ```
- Rebuild: `cargo build --release`
- Run again and verify the button now opens the new URL

### 5. Test with Agent Operations
- Start the server: `cd ../server && cargo run`
- Run the agent with GUI: `cd ../agent-windows && cargo run`
- Verify the GUI button remains visible and responsive
- Send commands to the agent via the server
- Verify the button continues to work during agent operations

## Expected Behavior

### Successful Test
✅ GUI window appears immediately after starting the agent
✅ Button is visible and clickable
✅ Clicking opens the configured URL in the default browser
✅ Window remains visible during agent operations
✅ Agent continues to beacon and execute commands normally

### Common Issues

1. **Window doesn't appear**
   - Check console for error messages
   - Verify Windows version supports Win32 GUI
   - Check if process has permission to create windows

2. **Button click does nothing**
   - Check console for "Failed to open URL" error
   - Verify default browser is configured
   - Check if URL is valid

3. **Agent stops responding**
   - Shouldn't happen - GUI runs in separate thread
   - If it does, report as a bug

## Integration Testing

### Test with Multiple Agents
1. Deploy agent to multiple Windows PCs
2. Verify each PC shows the GUI button
3. Test that all buttons work independently
4. Verify button visibility across different Windows versions

### Test URL Customization for School Use
1. Set URL to school portal: `const DEFAULT_URL: &str = "https://school.portal.edu";`
2. Deploy to student computers
3. Verify students can click button to access school resources
4. Test with different types of URLs (HTTP, HTTPS, localhost)

## Security Considerations

- The button opens URLs using Windows ShellExecuteW
- Only the configured URL can be opened (no user input)
- URL should be hardcoded in source (not configurable at runtime)
- Consider restricting to HTTPS URLs only for security
