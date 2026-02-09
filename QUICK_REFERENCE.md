# Quick Reference Card - School Orchestration / C2 Framework

## 🚀 Quick Start (3 Commands)

```bash
# 1. Start server
cd server && nix develop .. --command cargo run

# 2. Start agent (new terminal)
cd agent-windows && nix develop .. --command cargo run

# 3. Use control script (new terminal)
./school-control.sh agents  # Get agent ID
./school-control.sh quiz <AGENT_ID> https://kahoot.it
```

## 📋 Control Script Commands

```bash
# Mode Management
./school-control.sh mode                    # Check current mode
./school-control.sh set-mode school         # Switch to school mode
./school-control.sh set-mode c2             # Switch to C2 mode

# Agent Management
./school-control.sh agents                  # List all agents

# Quiz & Focus Protocols
./school-control.sh quiz <ID> <URL>         # Lock to quiz page (kiosk mode)
./school-control.sh lock <ID>               # Lock screen immediately

# Network Control
./school-control.sh block-dns <ID>          # Block all DNS
./school-control.sh whitelist <ID> <domains...>  # Whitelist specific sites

# System Control
./school-control.sh disable-taskmgr <ID>    # Disable Task Manager
./school-control.sh enable-taskmgr <ID>     # Enable Task Manager

# File Operations
./school-control.sh get-file <ID> <path>    # Download file
./school-control.sh upload-file <ID> <local> <remote>  # Upload file

# Safety & Cleanup
./school-control.sh revert <ID>             # REVERT ALL CHANGES

# Standard Commands
./school-control.sh cmd <ID> <command>      # Execute shell command
```

## 🔧 Direct API Usage

### Check Server Mode
```bash
curl http://127.0.0.1:8080/mode
```

### Switch to School Mode
```bash
curl -X POST http://127.0.0.1:8080/set_mode \
  -H "Content-Type: application/json" \
  -d '{"password":"admin","school_mode":true}'
```

### List All Agents
```bash
curl http://127.0.0.1:8080/agents | jq
```

### Execute Quiz Mode Protocol
```bash
curl -X POST http://127.0.0.1:8080/protocol \
  -H "Content-Type: application/json" \
  -d '{
    "password":"admin",
    "agent_id":"<UUID>",
    "command":"PROTOCOL:QUIZ_MODE|https://test.example.com"
  }'
```

### Revert All Changes
```bash
curl -X POST http://127.0.0.1:8080/revert_all \
  -H "Content-Type: application/json" \
  -d '{"password":"admin","agent_id":"<UUID>"}'
```

## 🎯 Protocol Reference

| Protocol | Syntax | Description |
|----------|--------|-------------|
| Quiz Mode | `PROTOCOL:QUIZ_MODE\|<url>` | Lock browser to URL in kiosk mode |
| Block DNS | `PROTOCOL:BLOCK_DNS` | Block all DNS resolution |
| Whitelist | `PROTOCOL:BLOCK_DNS_WHITELIST\|domain1\|domain2` | Allow only specified domains |
| Get File | `PROTOCOL:GET_FILE\|<path>` | Download file (returns base64) |
| Upload File | `PROTOCOL:UPLOAD_FILE\|<path>\|<base64>` | Upload file to agent |
| Lock Screen | `PROTOCOL:LOCK_SCREEN` | Lock Windows workstation |
| Disable Taskmgr | `PROTOCOL:DISABLE_TASK_MANAGER` | Prevent Task Manager access |
| Enable Taskmgr | `PROTOCOL:ENABLE_TASK_MANAGER` | Re-enable Task Manager |
| Revert All | `PROTOCOL:REVERT_ALL` | Restore all changes |

## 🎬 Demo Scenarios

### Scenario 1: Quiz Time (School Mode)
```bash
# 1. Lock students to quiz website
./school-control.sh quiz <ID> https://quiz.school.edu

# 2. Disable Task Manager to prevent cheating
./school-control.sh disable-taskmgr <ID>

# 3. After quiz, revert everything
./school-control.sh revert <ID>
```

### Scenario 2: Focused Study Session
```bash
# 1. Whitelist educational sites only
./school-control.sh whitelist <ID> wikipedia.org khanacademy.org

# 2. Students can only access approved sites
# DNS blocks everything else

# 3. After session, restore normal access
./school-control.sh revert <ID>
```

### Scenario 3: Lab Lockdown
```bash
# 1. Lock all screens
./school-control.sh lock <ID1>
./school-control.sh lock <ID2>
./school-control.sh lock <ID3>

# 2. Students must wait for teacher
# (Teacher unlocks with password)
```

### Scenario 4: Technical Demo (C2 Mode)
```bash
# 1. Switch to C2 mode
./school-control.sh set-mode c2

# 2. Execute reconnaissance
./school-control.sh cmd <ID> "whoami"
./school-control.sh cmd <ID> "ipconfig /all"

# 3. File exfiltration demo
./school-control.sh get-file <ID> "C:\\Windows\\System32\\drivers\\etc\\hosts"

# 4. Show protocol abstraction
./school-control.sh quiz <ID> https://example.com

# 5. Switch back to school mode
./school-control.sh set-mode school
```

## 🏗️ Building

### Basic Build (All Platforms)
```bash
cd server && cargo build
cd ../agent && cargo build
cd ../agent-windows && cargo build
cd ../client && cargo build
```

### Cross-Compile Windows Agent (from Linux)
```bash
rustup target add x86_64-pc-windows-gnu
cd agent-windows
cargo build --release --target x86_64-pc-windows-gnu
# Binary: target/x86_64-pc-windows-gnu/release/agent-windows.exe
```

### With Nix (Recommended)
```bash
nix develop
cd server && cargo build
cd ../agent-windows && cargo build
```

## 🌐 Web UI Access

```bash
# Start web client
cd client
nix develop .. --command cargo run

# Open browser
firefox http://localhost:3000
```

**Web UI Features:**
- View all connected agents
- Agent status (online/offline)
- Execute tasks via GUI
- View task history and results
- Real-time output display

## 📝 Important Notes

### Default Credentials
- **Password:** `admin` (hardcoded for demo)
- **Server:** `http://127.0.0.1:8080`
- ⚠️ Change for production use!

### Server Modes
- **School Mode** (default): Family-friendly terminology
- **C2 Mode**: Technical/security terminology
- Switch anytime without restarting server

### Agent Types
- **agent/**: Cross-platform, basic commands
- **agent-windows/**: Windows-specific, protocol support
- Use Windows agent for protocol demonstrations

### Safety Features
- `REVERT_ALL` protocol restores all changes
- Original DNS settings stored before modification
- Browser processes terminated cleanly
- Registry changes reversed

## 🎤 Competition Presentation Tips

### For Non-Technical Judges
1. ✅ Stay in School Orchestration Mode
2. ✅ Use terms: "classroom management", "quiz mode"
3. ✅ Demo Quiz Mode protocol first
4. ✅ Show revert capability
5. ✅ Mention real products: LanSchool, NetSupport

### For Technical Judges
1. ✅ Show mode switching
2. ✅ Explain protocol architecture
3. ✅ Demo both agent types
4. ✅ Discuss Windows API integration
5. ✅ Cover security implications

### Always Mention
- Educational purposes only
- Same tech in legitimate tools
- Understanding helps defenders
- Dual-purpose architecture innovation

## 🚨 Troubleshooting

| Issue | Solution |
|-------|----------|
| Agent won't connect | Check server URL, ensure server running |
| Command doesn't execute | Verify agent ID is correct |
| Protocol fails | Must use agent-windows, not basic agent |
| Browser won't open | Check Chrome/Edge installation path |
| DNS not blocking | Run agent with admin/elevated privileges |
| Revert fails | May need manual DNS reset or reboot |

## 📚 Documentation Files

- `README.md` - Main project overview
- `COMPETITION_GUIDE.md` - Presentation strategies
- `IMPLEMENTATION_SUMMARY.md` - Technical details
- `agent-windows/README.md` - Windows agent docs
- `QUICK_REFERENCE.md` - This file

## 🎓 Key Concepts for Judges

**Innovation:**
- Dual-mode architecture (same code, different presentation)
- Protocol abstraction layer
- Stateful change tracking with revert

**Technical:**
- Modern Rust with async/await
- Windows API integration
- Clean separation of concerns
- Cross-platform + platform-specific agents

**Practical:**
- Real classroom use case
- Security research applications
- Production-ready architecture patterns

---

**Remember:** Technology is neutral. This framework demonstrates legitimate classroom management and security research capabilities. Always use responsibly and with proper authorization.

**Good luck with your presentation! 🚀**
