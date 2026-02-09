# C2 Framework - School Project / Red Team Tool

A dual-mode Command & Control framework written in Rust for educational purposes. Can operate as either:
- **School Orchestration System**: Family-friendly classroom management tool
- **Red Team C2 Framework**: Technical penetration testing framework

## Dual-Mode Operation

The server can switch between two presentation modes:

### School Orchestration Mode (Default)
- Family-friendly terminology
- Protocol-based commands for classroom management
- Quiz mode, DNS filtering, screen locking
- Perfect for educational demonstrations and competitions

### Red Team C2 Mode
- Technical C2 framework terminology  
- Full command execution capabilities
- File transfer and system manipulation
- For security research and penetration testing

Switch modes via API or control script - same underlying functionality, different presentation.

## Project Structure

```
.
├── protocol/        # Shared protocol definitions between server and agent
├── server/          # C2 server that manages agents and tasks
├── agent/           # Cross-platform agent (Linux/macOS/Windows)
├── agent-windows/   # Windows-specific agent with protocol support
├── client/          # Web-based client UI
└── school-control.sh # Helper script for protocol commands
```

## Architecture

The C2 framework consists of three main components:

1. **Server** - HTTP-based C2 server that:
   - Registers new agents
   - Receives beacon check-ins from agents
   - Queues tasks for agents to execute
   - Collects task execution results
   - Provides API for operator interaction

2. **Agent** - Lightweight implant that:
   - Registers with the server on first run
   - Sends periodic beacon check-ins
   - Executes commands received from server
   - Reports task results back to server

   **Two versions available:**
   - `agent/` - Cross-platform (Linux/macOS/Windows) basic agent
   - `agent-windows/` - Windows-specific with protocol support

3. **Client** - Web-based UI for operators to:
   - List connected agents
   - Queue tasks for agents to execute
   - View agent information

## Building

All projects use the Nix development environment. Enter the environment:

```bash
nix develop
```

Then build each component:

```bash
# Build protocol library
cd protocol && cargo build

# Build server
cd ../server && cargo build

# Build agent
cd ../agent && cargo build

# Build Windows protocol agent
cd ../agent-windows && cargo build

# Build client
cd ../client && cargo build
```

Or build everything at once:

```bash
cargo build --all
```

## Running

### 1. Start the Server

```bash
cd server
nix develop .. --command cargo run
```

The server will start on `http://127.0.0.1:8080`

### 2. Run an Agent

**Basic Agent (any platform):**
```bash
cd agent
nix develop .. --command cargo run
```

**Windows Protocol Agent (recommended for Windows):**
```bash
cd agent-windows
nix develop .. --command cargo run
```

The agent will:
- Register with the server and receive a UUID
- Send beacon check-ins every 10 seconds
- Execute any queued tasks

### 3. Use the Web Client

In another terminal:

```bash
cd client
nix develop .. --command cargo run
```

Then open your browser and navigate to `http://localhost:3000`

The web interface provides:
- **Dashboard** - View all registered agents with online/offline status
- **Agent Details** - Click any agent to see detailed information
- **Task Management** - Create and execute tasks directly from the browser
- **Real-time Results** - View command outputs and exit codes
- **Task History** - See all tasks with their status (Pending/Done/Failed)

## Quick Start - School Orchestration Mode

### Using the Control Script

```bash
# Check current mode
./school-control.sh mode

# List all connected agents
./school-control.sh agents

# Activate quiz mode on an agent
./school-control.sh quiz <agent-id> https://quiz.school.edu

# Block all DNS except whitelisted sites
./school-control.sh whitelist <agent-id> school.edu google.com

# Lock student screen
./school-control.sh lock <agent-id>

# Disable Task Manager during test
./school-control.sh disable-taskmgr <agent-id>

# Revert all changes after class
./school-control.sh revert <agent-id>

# Switch to C2 mode for technical demonstration
./school-control.sh set-mode c2

# Execute shell command
./school-control.sh cmd <agent-id> "whoami"
```

## Protocol Commands (Windows Agent)

The Windows agent supports special protocol commands for classroom management:

- `PROTOCOL:QUIZ_MODE|<url>` - Lock browser to quiz page in kiosk mode
- `PROTOCOL:BLOCK_DNS` - Block all DNS resolution
- `PROTOCOL:BLOCK_DNS_WHITELIST|<domains>` - Whitelist specific domains
- `PROTOCOL:GET_FILE|<path>` - Download file from agent
- `PROTOCOL:UPLOAD_FILE|<path>|<base64>` - Upload file to agent
- `PROTOCOL:LOCK_SCREEN` - Lock the workstation
- `PROTOCOL:DISABLE_TASK_MANAGER` - Prevent Task Manager access
- `PROTOCOL:ENABLE_TASK_MANAGER` - Re-enable Task Manager
- `PROTOCOL:REVERT_ALL` - Restore all changes

See `agent-windows/README.md` for detailed protocol documentation.

## API Endpoints

### Server Endpoints

**Mode Management:**
- `GET /mode` - Check current server mode
- `POST /set_mode` - Switch between School/C2 mode

**Agent Management:**
- `POST /register` - Register a new agent
- `POST /beacon` - Agent check-in (heartbeat)
- `POST /task_result` - Submit task execution results
- `GET /agents` - List all registered agents

**Task Management:**
- `POST /add_task` - Queue a standard command task (password: "admin")
- `POST /protocol` - Execute protocol command (password: "admin")
- `POST /revert_all` - Revert all changes on agent (password: "admin")
- `GET /tasks/<task_id>` - Get task details and results
- `GET /agent/<agent_id>/tasks` - List all tasks for specific agent

## Protocol

All communication uses JSON over HTTP. The shared `protocol` crate defines the message structures:

- **RegisterRequest/Response** - Agent registration
- **BeaconRequest/Response** - Periodic check-in with optional task assignment
- **TaskResultRequest/Response** - Task execution result submission
- **AddTaskRequest/Response** - Queue new task for agent

## Example Workflow

### School Orchestration Demo

1. Start the server (defaults to School Mode)
2. Deploy Windows agent on classroom computers
3. Open web client at `http://localhost:3000`
4. View all connected student computers
5. Select a computer and execute protocol:
   - "Quiz Mode" with test URL
   - DNS whitelist for approved sites
   - Lock screens during instructions
6. After class, click "Revert All" to restore computers

### Red Team C2 Demo

1. Switch server to C2 mode: `./school-control.sh set-mode c2`
2. Run agents on target systems
3. Use web client for command execution
4. Demonstrate file transfer capabilities
5. Show task history and results
6. Explain security implications

## Competition Presentation Tips

**For Non-Technical Judges:**
- Keep server in School Orchestration Mode
- Focus on legitimate classroom use cases
- Demonstrate Quiz Mode protocol
- Show the safety features (revert functionality)
- Emphasize educational applications

**For Technical Judges:**
- Show dual-mode architecture
- Explain protocol system design
- Demonstrate both modes
- Discuss security considerations
- Highlight Windows-specific implementations

## Use Cases

### Educational Environment
- ✅ Classroom computer management
- ✅ Quiz/exam mode enforcement
- ✅ Content filtering for students
- ✅ Lab configuration management
- ✅ Digital signage/kiosk mode

### Security Research
- 🔴 Red team operations
- 🔴 Penetration testing
- 🔴 Security awareness training
- 🔴 Defensive capability testing

## Security Notes

⚠️ **This is a school project for educational purposes only!**

Current limitations (DO NOT use in production):
- No encryption (plain HTTP)
- Hardcoded password ("admin")
- No authentication beyond simple password
- No input validation for commands
- Runs on localhost only

For a real C2 framework, you would need:
- HTTPS/TLS encryption
- Proper authentication (tokens, certificates)
- Command whitelisting/sandboxing
- Encrypted payloads
- Proper error handling
- Database for persistence
- Multi-threaded task management

## Development

The project uses:
- Rust 2021 edition
- Tokio for async runtime
- Hyper for HTTP server
- Reqwest for HTTP client
- Serde for JSON serialization
- UUID for agent identification
- Chrono for timestamps
- Clap for CLI parsing

## License

Educational purposes only. Use responsibly and only on systems you own or have permission to test.
