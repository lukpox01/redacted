# C2 Framework - School Project

A simple Command & Control (C2) framework written in Rust for educational purposes.

## Project Structure

```
.
├── protocol/       # Shared protocol definitions between server and agent
├── server/         # C2 server that manages agents and tasks
├── agent/          # Agent that runs on target machines
└── client/         # CLI client to interact with the server
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

3. **Client** - CLI tool for operators to:
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

In a separate terminal:

```bash
cd agent
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

## API Endpoints

### Server Endpoints

- `POST /register` - Register a new agent
- `POST /beacon` - Agent check-in (heartbeat)
- `POST /task_result` - Submit task execution results
- `POST /add_task` - Queue a task for an agent (requires password: "admin")
- `GET /agents` - List all registered agents

## Protocol

All communication uses JSON over HTTP. The shared `protocol` crate defines the message structures:

- **RegisterRequest/Response** - Agent registration
- **BeaconRequest/Response** - Periodic check-in with optional task assignment
- **TaskResultRequest/Response** - Task execution result submission
- **AddTaskRequest/Response** - Queue new task for agent

## Example Workflow

1. Start the server
2. Run one or more agents (they auto-register)
3. Open web client at `http://localhost:3000`
4. View all connected agents on the dashboard
5. Click on an agent to see details
6. Enter a command in the task input field (e.g., "whoami", "ls -la", "pwd")
7. Click "Execute Task" button
8. Wait a few seconds and refresh to see the result
9. Task output will appear below with color-coding (green for success, red for errors)

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
