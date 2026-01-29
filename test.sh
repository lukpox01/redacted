#!/usr/bin/env bash

# Simple test script to demonstrate C2 framework
# This script should be run from the project root

echo "=== C2 Framework Test ==="
echo ""

# Check if we're in nix develop shell
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please run: nix develop"
    exit 1
fi

echo "Starting server in background..."
cd server
cargo run &> ../server.log &
SERVER_PID=$!
cd ..
sleep 3

echo "Starting agent in background..."
cd agent
cargo run &> ../agent.log &
AGENT_PID=$!
cd ..
sleep 5

echo "Waiting for agent to register..."
sleep 3

echo ""
echo "=== Listing agents ==="
cd client
cargo run -- agents
cd ..

echo ""
echo "=== Logs ==="
echo "Server log (last 20 lines):"
tail -20 server.log
echo ""
echo "Agent log (last 20 lines):"
tail -20 agent.log

echo ""
echo "=== Cleanup ==="
echo "Killing server (PID: $SERVER_PID) and agent (PID: $AGENT_PID)..."
kill $SERVER_PID $AGENT_PID 2>/dev/null
sleep 1

echo ""
echo "Test complete! Check server.log and agent.log for details."
echo ""
echo "To manually test:"
echo "  1. Terminal 1: cd server && cargo run"
echo "  2. Terminal 2: cd agent && cargo run"
echo "  3. Terminal 3: cd client && cargo run -- agents"
echo "  4. Terminal 3: cd client && cargo run -- task <UUID> 'whoami'"
