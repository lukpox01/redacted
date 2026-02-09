# Windows Agent with School Orchestration Protocols - Implementation Summary

## What Was Created

### 1. Windows-Specific Agent (`agent-windows/`)
A new Windows agent with advanced protocol support for classroom management scenarios.

**Key Features:**
- Protocol-based command system
- Windows API integration
- Stateful operation tracking
- Automatic cleanup/revert capability
- Browser kiosk mode support
- DNS manipulation
- Registry modification for system control

**Protocols Implemented:**
- `QUIZ_MODE` - Lock browser to test webpage in kiosk mode
- `BLOCK_DNS` - Block all DNS resolution
- `BLOCK_DNS_WHITELIST` - Allow only specific domains
- `GET_FILE` - Download files from agent
- `UPLOAD_FILE` - Upload files to agent
- `LOCK_SCREEN` - Lock Windows workstation
- `DISABLE_TASK_MANAGER` - Prevent Task Manager access
- `ENABLE_TASK_MANAGER` - Re-enable Task Manager
- `REVERT_ALL` - Restore all changes to original state

### 2. Enhanced Server (`server/src/main.rs`)
Updated the C2 server with dual-mode operation capability.

**New Features:**
- Mode switching (School Orchestration / Red Team C2)
- New endpoints:
  - `GET /mode` - Check current mode
  - `POST /set_mode` - Switch modes
  - `POST /protocol` - Execute protocol commands
  - `POST /revert_all` - Revert all agent changes
- Enhanced startup messages showing available protocols
- Mode-aware logging and terminology

### 3. Control Script (`school-control.sh`)
A comprehensive CLI helper script for easy protocol execution.

**Commands:**
```bash
./school-control.sh mode                          # Check mode
./school-control.sh set-mode school|c2           # Switch mode
./school-control.sh agents                       # List agents
./school-control.sh quiz <id> <url>              # Quiz mode
./school-control.sh block-dns <id>               # Block DNS
./school-control.sh whitelist <id> <domains...>  # Whitelist
./school-control.sh lock <id>                    # Lock screen
./school-control.sh disable-taskmgr <id>         # Disable Task Manager
./school-control.sh enable-taskmgr <id>          # Enable Task Manager
./school-control.sh revert <id>                  # Revert all
./school-control.sh get-file <id> <path>         # Get file
./school-control.sh upload-file <id> <l> <r>     # Upload file
./school-control.sh cmd <id> <command>           # Execute command
```

### 4. Documentation

**Competition Guide (`COMPETITION_GUIDE.md`):**
- Presentation strategies for different audiences
- Demo scripts (family-friendly and technical)
- Q&A preparation
- Ethical considerations
- Scoring criteria alignment

**Windows Agent README (`agent-windows/README.md`):**
- Complete protocol documentation
- Building instructions (cross-compile and native)
- Deployment scenarios
- API endpoint reference
- Use cases and examples

**Updated Main README:**
- Dual-mode architecture explanation
- Quick start guides for both modes
- Protocol command reference
- Competition presentation tips
- Enhanced workflow examples

## Architecture Decisions

### Why Two Agents?
1. **`agent/`** - Cross-platform basic agent
   - Simple command execution
   - Works on Linux, macOS, Windows
   - Minimal dependencies
   - Good for general C2 demonstrations

2. **`agent-windows/`** - Windows-specific protocol agent
   - Windows API integration
   - Advanced system manipulation
   - Protocol-based commands
   - Perfect for competition demos

### Why Protocol System?
Instead of raw commands like:
```bash
powershell -c "Get-NetAdapter | Set-DnsClientServerAddress -ServerAddresses 127.0.0.1"
```

Use clean protocols:
```bash
PROTOCOL:BLOCK_DNS
```

**Benefits:**
- Cleaner interface for operators
- Better error handling
- Stateful operation tracking
- Platform abstraction
- Easier to present to non-technical audiences

### Why Dual-Mode Server?
**Same functionality, different presentation:**

**School Mode:**
- "Management Server"
- "School Orchestration"
- "Student computers"
- "Quiz Mode"
- Focus on legitimate educational use

**C2 Mode:**
- "Command & Control Server"
- "Red Team Operations"
- "Agents/Implants"
- "Task Execution"
- Focus on security research

This lets you present the project appropriately based on your audience without changing any actual code.

## Technical Implementation Highlights

### 1. Stateful Agent Design
The Windows agent tracks all changes it makes:
```rust
struct AgentState {
    dns_blocked: bool,
    original_dns: Vec<String>,
    kiosk_active: bool,
    // ... agent maintains state
}
```

This enables the `REVERT_ALL` protocol to properly clean up.

### 2. Windows API Integration
Uses both WinAPI and PowerShell for system manipulation:
- Registry editing for Task Manager control
- DNS client configuration via PowerShell
- Browser process spawning with kiosk flags
- Hosts file manipulation for DNS filtering

### 3. Async Task Execution
Both server and agents use Tokio async runtime:
- Non-blocking beacon intervals
- Concurrent task processing
- Efficient HTTP communication

### 4. Protocol Parsing
Simple pipe-delimited protocol format:
```
PROTOCOL:COMMAND|param1|param2|param3
```

Example:
```
PROTOCOL:QUIZ_MODE|https://test.example.com
PROTOCOL:BLOCK_DNS_WHITELIST|google.com|wikipedia.org
PROTOCOL:UPLOAD_FILE|C:\path\file.txt|<base64_content>
```

## Usage Examples

### Quick Demo Setup

**Terminal 1 - Server:**
```bash
cd server
nix develop .. --command cargo run
# Server starts in School Orchestration Mode
```

**Terminal 2 - Windows Agent (on Windows or via Wine):**
```bash
cd agent-windows
cargo build --release --target x86_64-pc-windows-gnu
# Copy to Windows machine and run
./agent-windows.exe
```

**Terminal 3 - Control:**
```bash
# Get agent ID
./school-control.sh agents

# Activate quiz mode
./school-control.sh quiz <agent-id> https://kahoot.it

# Wait for demo...

# Revert everything
./school-control.sh revert <agent-id>

# Switch to C2 mode for technical demo
./school-control.sh set-mode c2
```

### API Usage Examples

**Switch to School Mode:**
```bash
curl -X POST http://127.0.0.1:8080/set_mode \
  -H "Content-Type: application/json" \
  -d '{"password":"admin", "school_mode":true}'
```

**Execute Quiz Mode Protocol:**
```bash
curl -X POST http://127.0.0.1:8080/protocol \
  -H "Content-Type: application/json" \
  -d '{
    "password":"admin",
    "agent_id":"<uuid>",
    "command":"PROTOCOL:QUIZ_MODE|https://test.school.edu"
  }'
```

**Revert All Changes:**
```bash
curl -X POST http://127.0.0.1:8080/revert_all \
  -H "Content-Type: application/json" \
  -d '{"password":"admin", "agent_id":"<uuid>"}'
```

## Competition Strategy

### For Non-Technical Judges
1. Keep server in School Mode
2. Use terms: "classroom management", "focus mode", "test mode"
3. Demo the Quiz Mode protocol
4. Emphasize safety features (revert capability)
5. Mention real-world equivalents (LanSchool, NetSupport)

### For Technical Judges
1. Show the mode switching capability
2. Explain protocol architecture
3. Demonstrate both agents (basic and Windows)
4. Discuss Windows API integration
5. Cover security implications and defensive considerations

### Universal Talking Points
- ✅ Clean Rust architecture
- ✅ Stateful operation with cleanup
- ✅ Protocol abstraction layer
- ✅ Dual presentation capability
- ✅ Real-world applicability (both educational and security)

## Security & Ethics

**Always Emphasize:**
- Educational and research purposes only
- Only use on systems you own or have explicit permission
- Same technology in legitimate tools (classroom management, IT admin)
- Understanding C2 helps defenders

**Competition Disclaimer:**
"This project demonstrates that security tools can serve dual purposes. The same capabilities used in classroom management software can be found in offensive security tools. Understanding these systems makes us better defenders and more responsible technologists."

## Files Created/Modified

**New Files:**
- `agent-windows/Cargo.toml` - Windows agent dependencies
- `agent-windows/src/main.rs` - Windows agent implementation (589 lines)
- `agent-windows/README.md` - Windows agent documentation
- `agent-windows/.gitignore` - Git ignore rules
- `school-control.sh` - Control script (243 lines)
- `COMPETITION_GUIDE.md` - Competition presentation guide (303 lines)

**Modified Files:**
- `server/src/main.rs` - Added mode switching and protocol endpoints
- `README.md` - Updated with dual-mode documentation and examples

**Total New Code:**
- ~1,000 lines of Rust (Windows agent + server enhancements)
- ~250 lines of Bash (control script)
- ~600 lines of Markdown (documentation)

## Building & Testing

### Build All Components
```bash
cd /home/lukpox01/code/redacted

# Build protocol library
cd protocol && cargo build

# Build server
cd ../server && cargo build

# Build basic agent
cd ../agent && cargo build

# Build Windows agent
cd ../agent-windows && cargo build

# Build client
cd ../client && cargo build
```

### Cross-Compile Windows Agent (from Linux)
```bash
rustup target add x86_64-pc-windows-gnu
cd agent-windows
cargo build --release --target x86_64-pc-windows-gnu
```

Binary will be at: `target/x86_64-pc-windows-gnu/release/agent-windows.exe`

## Next Steps / Future Enhancements

### Potential Additions:
1. **Screenshot Protocol** - Capture screens during quiz mode
2. **Process Monitor Protocol** - Track running applications
3. **Time-based Protocols** - Auto-revert after duration
4. **Batch Operations** - Execute protocol on multiple agents
5. **Web UI Enhancements** - Add protocol buttons to web client
6. **Logging** - Enhanced audit trail for all protocol executions
7. **Encrypted Communications** - TLS/mTLS for production use

### For Production Use:
1. Replace hardcoded password with proper authentication
2. Add HTTPS/TLS encryption
3. Implement certificate pinning
4. Add command whitelisting
5. Database persistence for agents and tasks
6. Rate limiting and access controls
7. Proper error recovery and retry logic

## Conclusion

You now have a complete dual-mode C2 framework with:
- ✅ Windows-specific protocol agent
- ✅ Server mode switching (School/C2)
- ✅ Comprehensive protocol system
- ✅ Easy-to-use control script
- ✅ Complete documentation for competitions
- ✅ Family-friendly and technical presentations

The system is ready for demonstration at your competition. Choose your presentation mode based on your audience, and use the competition guide to prepare your talking points.

Good luck with your presentation! 🎉
