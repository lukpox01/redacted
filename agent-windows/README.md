# Windows C2 Agent

A Windows-specific C2 agent with protocol-based commands for system orchestration. Can operate in two modes: **Restricted Mode** (limited operations) or **Full C2 Mode** (full capabilities).

## Features

### Dual-Mode Operation
- **Restricted Mode**: Limited operations for controlled environments
- **Full C2 Mode**: Complete C2 framework operations
- Server-side toggle between modes

### Command Execution
The agent automatically detects and executes both CMD and PowerShell commands:

#### CMD Commands
```bash
# Standard Windows commands
dir
ipconfig
whoami
netstat -ano
```

#### PowerShell Commands
The agent automatically detects PowerShell syntax and executes appropriately:
```powershell
# Direct PowerShell commands
Get-Process
Get-Service
Get-NetAdapter
Invoke-WebRequest -Uri "http://example.com"

# Explicit PowerShell invocation
powershell.exe Get-ComputerInfo
pwsh -Command "Get-ChildItem"
```

**Auto-Detection Triggers:**
- Commands starting with `powershell` or `pwsh`
- Commands containing `Get-`, `Set-`, `Invoke-`, `New-Object`
- PowerShell-specific cmdlets

### Protocol Commands (Restricted Mode)

#### QUIZ_MODE
Locks computers to a specific webpage with full focus mode.
```
PROTOCOL:QUIZ_MODE|https://example.com
```
**Actions:**
- Blocks DNS (except for the target domain)
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
PROTOCOL:BLOCK_DNS_WHITELIST|example.com|trusted.com
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

### Full C2 Mode (Default)

1. **Start the Server**
   ```bash
   cd server
   cargo run
   ```
   Server starts in Full C2 Mode by default.

2. **Deploy Agent on Windows Machines**
   - Copy `agent-windows.exe` to target computers
   - Run the agent (can be installed as a service)
   - Agent connects to C2 server

3. **Execute Commands**
   ```bash
   # CMD commands
   curl -X POST http://127.0.0.1:8080/add_task \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>",
       "command": "whoami"
     }'

   # PowerShell commands (auto-detected)
   curl -X POST http://127.0.0.1:8080/add_task \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>",
       "command": "Get-Process | Select-Object -First 10"
     }'
   ```

### Restricted Mode

1. **Switch Server Mode**
   ```bash
   curl -X POST http://127.0.0.1:8080/set_mode \
     -H "Content-Type: application/json" \
     -d '{"password": "admin", "school_mode": true}'
   ```

2. **Use Protocol Commands**
   ```bash
   curl -X POST http://127.0.0.1:8080/protocol \
     -H "Content-Type: application/json" \
     -d '{
       "password": "admin",
       "agent_id": "<uuid>",
       "command": "PROTOCOL:LOCK_SCREEN"
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
- `POST /add_task` - Standard command execution (CMD/PowerShell)
- `POST /protocol` - Protocol command (Restricted mode recommended)
- `POST /revert_all` - Revert all agent changes

## Command Examples

### CMD Commands
```bash
# System information
systeminfo
wmic os get caption,version,buildnumber

# Network information
ipconfig /all
netstat -ano
arp -a

# User information
whoami /all
net user
net localgroup administrators
```

### PowerShell Commands
```powershell
# System enumeration
Get-ComputerInfo
Get-HotFix
Get-WmiObject Win32_OperatingSystem

# Process management
Get-Process
Stop-Process -Name "notepad" -Force

# Network operations
Get-NetAdapter
Get-NetIPAddress
Test-NetConnection google.com

# File operations
Get-ChildItem -Path C:\ -Recurse -Filter *.txt
Get-Content C:\file.txt
```

## Security Notes

⚠️ **Educational/Research Purposes Only**

This tool demonstrates:
- Command and Control architecture
- Windows system manipulation
- Network configuration management
- Process management
- PowerShell integration

**Production Requirements:**
- HTTPS/TLS encryption
- Certificate pinning
- Authentication tokens
- Command obfuscation
- Anti-analysis features
- Encrypted payloads

## License

Educational purposes only. Use responsibly and only on systems you own or have explicit permission to test.
