# [ REDACTED ] - Complete Feature List

## 🎯 Core Features

### Command & Control Architecture
- **Agent-Server-Client Model**: Separated components for security and flexibility
- **RESTful API**: Server exposes REST endpoints for agent and client communication
- **Beacon System**: Agents check in at configurable intervals (default: 10 seconds)
- **Task Queue**: Tasks are queued and executed sequentially
- **Real-time Status**: Live agent status tracking (Online/Offline)

### Configurable Deployment
- **Environment Variables**: Set `C2_SERVER` for custom server URL
- **Command-Line Arguments**: Agent accepts server URL as first argument
- **Default Localhost**: Falls back to `http://127.0.0.1:8080` for local testing
- **Cross-Platform**: Supports Linux, Windows, and macOS

---

## 🔌 Reverse Shell Options (5 Methods)

1. **Bash Reverse Shell**
   - Native Linux shell
   - No dependencies
   - `/dev/tcp` redirection

2. **Python Reverse Shell**
   - Cross-platform
   - Python 3 socket connection
   - Full interactive shell

3. **Netcat Reverse Shell**
   - Classic method
   - Requires `nc` installed
   - Simple and reliable

4. **PHP Reverse Shell**
   - Web server exploitation
   - One-liner execution
   - PHP runtime required

5. **Perl Reverse Shell**
   - Perl socket programming
   - Often available on systems
   - Legacy system support

---

## 🔍 Reconnaissance (10 Commands)

### System Information
- **Full System Info**: OS, kernel, hostname, uptime, user context
- **Environment Variables**: All env vars and exported variables
- **Disk Usage**: Filesystem usage and block devices
- **Process List**: All running processes with details

### Network Intelligence
- **Network Configuration**: IPs, routes, DNS, ARP cache
- **Listening Services**: All open ports and services
- **Firewall Rules**: iptables, ufw, firewall-cmd configurations

### Security Assessment
- **List Users**: All system users and currently logged in users
- **Find SSH Keys**: Locate all SSH private and public keys
- **Find Writable Directories**: User-writable paths for exploitation

---

## ⬆️ Privilege Escalation (6 Commands)

1. **Find SUID Binaries**
   - Locate all SUID-root binaries
   - Common privilege escalation vector

2. **Find SGID Binaries**
   - Locate all SGID binaries
   - Group-based escalation

3. **Check Sudo Rights**
   - Non-interactive sudo check
   - No password prompts
   - Lists available commands

4. **Check for Docker**
   - Docker group membership
   - Container escape potential

5. **Kernel Exploits Check**
   - Kernel version detection
   - Look for known vulnerabilities

6. **Capabilities Check**
   - Find files with Linux capabilities
   - Alternative to SUID

---

## 📁 File Operations (7 Commands)

### Download Methods
- **wget**: Download files via HTTP/HTTPS
- **curl**: Alternative download method

### Upload/Exfiltration
- **Upload File**: POST file to attacker server
- **Base64 Exfiltration**: Encode and display for manual copy

### Search & Discovery
- **Search for Passwords**: Grep for password strings in files
- **Find Config Files**: Locate .conf, .config, .ini files
- **Find Database Files**: Locate .db, .sql, .sqlite files

---

## ♾️ Persistence Mechanisms (6 Methods)

1. **Check Cron Jobs**
   - System and user crontabs
   - Scheduled task discovery

2. **Add Cron Persistence**
   - Install backdoor cron job
   - Runs every 5 minutes

3. **Check Systemd Services**
   - List running services
   - Service manipulation potential

4. **Check RC Scripts**
   - Startup script locations
   - Boot persistence

5. **Add SSH Key**
   - Install authorized_keys
   - Persistent SSH access

6. **Check Bash History**
   - View command history
   - Credential discovery

---

## 🔄 Lateral Movement (4 Commands)

1. **Ping Sweep**
   - Scan subnet for live hosts
   - Network discovery
   - /24 subnet scan

2. **Port Scan Common**
   - Scan common ports
   - Pure bash implementation
   - No tools required

3. **Check Network Shares**
   - Mounted NFS/CIFS shares
   - Network filesystem access

4. **Find SSH Config**
   - SSH configuration files
   - Known hosts for targeting

---

## 🧹 Cleanup Operations (4 Commands)

1. **Clear Bash History**
   - Remove command history
   - Cover tracks

2. **Clear Logs**
   - Wipe auth.log and syslog
   - Requires root access

3. **Remove Files**
   - Delete temporary files
   - Remove evidence

4. **Disable History**
   - Turn off history logging
   - Current session only

---

## 🎮 Agent Control (4 Commands)

1. **Sleep 30 seconds**
   - Pause agent execution
   - Evasion technique

2. **Sleep 5 minutes**
   - Extended pause
   - Avoid detection

3. **Change Directory**
   - Navigate filesystem
   - Execution context

4. **Test Connection**
   - Verify agent alive
   - Connectivity check

---

## 🖥️ Web Interface Features

### [ REDACTED ] Theme
- **Terminal Aesthetic**: Green-on-black color scheme
- **Classified Design**: "CLASSIFIED" headers and badges
- **CRT Scanlines**: Retro terminal effect overlay
- **Glowing Effects**: Green glow on interactive elements
- **Monospace Fonts**: Authentic terminal typography

### Dashboard
- **Agent Overview**: Card-based agent display
- **Status Indicators**: Pulsing online/offline badges
- **Statistics**: Total agents, active connections, security level
- **Real-time Updates**: Refresh button for latest status

### Agent Control Panel
- **System Information**: Detailed agent metadata
- **Task History**: All executed commands with output
- **Quick Commands**: 50+ predefined operations
- **Custom Commands**: Free-form command execution
- **Task Status**: Pending, Done, Failed indicators

### UI Components
- **Categorized Commands**: Organized by function
- **One-Click Execution**: Click command to load it
- **Task Output Display**: Color-coded success/error
- **UUID Tracking**: Unique identifiers for all tasks
- **Time Stamps**: Created and completed times

---

## 🔧 System Information Collection

### Automatic Agent Enumeration
- **Hostname**: System identification
- **OS Type**: Linux/Windows/macOS detection
- **OS Version**: Detailed version strings
  - Linux: Reads `/etc/os-release` (PRETTY_NAME)
  - Windows: `ver` command output
  - macOS: `sw_vers` output
- **Username**: Current user context
- **IP Address**: First non-loopback IP
  - Linux: `hostname -I` or `ip addr`
  - Windows: `ipconfig` parsing
- **MAC Address**: Physical address
  - Tries multiple interfaces: eth0, enp0s3, ens33, wlan0, wlp2s0
  - Fallback to `ip link` command

---

## 🛡️ Security Features

### Agent Protection
- **Error Handling**: Graceful failure recovery
- **Retry Logic**: Automatic reconnection attempts
- **Non-interactive**: No user prompts required
- **Stealth Mode**: Minimal console output

### Command Safety
- **Non-blocking**: Sudo commands use `-n` flag
- **Error Suppression**: `2>/dev/null` where appropriate
- **Timeout Protection**: Prevents hanging commands

---

## 📊 Statistics

### Command Library
- **Total Commands**: 54 predefined operations
- **Categories**: 8 operational categories
- **Shell Methods**: 5 different reverse shell techniques
- **Recon Tools**: 10 intelligence gathering commands
- **Privilege Escalation**: 6 escalation vectors
- **File Operations**: 7 file manipulation commands
- **Persistence**: 6 persistence mechanisms
- **Lateral Movement**: 4 network traversal commands
- **Cleanup**: 4 anti-forensics operations
- **Agent Control**: 4 control commands

---

## 🚀 Deployment Scenarios

### Lab Testing
```bash
export C2_SERVER="http://127.0.0.1:8080"
# Local development and testing
```

### Internal Network
```bash
export C2_SERVER="http://192.168.1.100:8080"
# LAN-based C2 infrastructure
```

### Remote VPS
```bash
export C2_SERVER="http://vps.attacker.com:8080"
# Internet-facing C2 server
```

### Covert Channel
```bash
export C2_SERVER="http://legitimate-domain.com:8080"
# Domain fronting / legitimate infrastructure
```

---

## 🔐 Operational Security

### Best Practices
- ✅ Use HTTPS in production
- ✅ Change default passwords
- ✅ Implement authentication
- ✅ Rate limit API endpoints
- ✅ Log all operations
- ✅ Use domain fronting
- ✅ Implement kill switches
- ✅ Regular beacon jitter

### Red Team Usage
- ⚠️ Authorized engagements only
- ⚠️ Document all actions
- ⚠️ Maintain command audit logs
- ⚠️ Clean up post-engagement
- ⚠️ Never use on unauthorized systems

---

## 📝 Technical Specifications

### Technology Stack
- **Language**: Rust (agent, client, server)
- **Web Framework**: Axum (client)
- **Template Engine**: Askama (HTML templating)
- **HTTP Client**: Reqwest (agent, client)
- **Serialization**: Serde (JSON)
- **Build System**: Cargo + Nix

### Performance
- **Agent Binary**: ~2-5 MB (release build)
- **Beacon Interval**: 10 seconds (configurable)
- **Task Execution**: Sequential, non-blocking
- **Memory Usage**: < 10 MB per agent

### Compatibility
- **Linux**: Full support (primary target)
- **Windows**: Partial support (basic commands)
- **macOS**: Partial support (basic commands)

---

**[ CLASSIFIED - FOR AUTHORIZED USE ONLY ]**

**WARNING**: This tool is for educational and authorized security testing purposes only. Unauthorized use is illegal.
