#!/bin/bash

echo "🧪 P2P mTLS Test"
echo "================="

# Cleanup
echo "🧹 Cleanup old test files..."
rm -rf test-agent1 test-agent2
mkdir -p test-agent1/certs test-agent2/certs

# Build
echo "🔨 Building agent..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

# Create configs
echo "📝 Creating configurations..."

CURRENT_DIR=$(pwd)

cat > test-agent1/config.toml << EOF
agent_id = "11111111-1111-1111-1111-111111111111"
name = "test-agent-1"
server_url = "http://localhost:8000"
api_key = "test-key"
p2p_only_mode = true
collection_interval = 30
heartbeat_interval = 60
tags = []

[p2p]
enabled = true
listen_port = 8443
peers = ["127.0.0.1:8444"]
cert_path = "$CURRENT_DIR/test-agent1/certs/agent.crt"
key_path = "$CURRENT_DIR/test-agent1/certs/agent.key"
ca_cert_path = "$CURRENT_DIR/test-agent1/certs/ca.crt"
auto_generate_certs = true
EOF

cat > test-agent2/config.toml << EOF
agent_id = "22222222-2222-2222-2222-222222222222"
name = "test-agent-2"
server_url = "http://localhost:8000"
api_key = "test-key"
p2p_only_mode = true
collection_interval = 30
heartbeat_interval = 60
tags = []

[p2p]
enabled = true
listen_port = 8444
peers = ["127.0.0.1:8443"]
cert_path = "$CURRENT_DIR/test-agent2/certs/agent.crt"
key_path = "$CURRENT_DIR/test-agent2/certs/agent.key"
ca_cert_path = "$CURRENT_DIR/test-agent2/certs/ca.crt"
auto_generate_certs = true
EOF

# Start Agent 1
echo "🚀 Starting Agent 1 (port 8443)..."
cd test-agent1
RUST_LOG=info ../target/release/csf-agent > agent.log 2>&1 &
AGENT1_PID=$!
cd ..
echo "   PID: $AGENT1_PID"

# Wait for Agent 1 to start and generate certs
echo "⏳ Waiting for Agent 1 to generate certificates..."
sleep 4

# Copy CA cert from Agent 1 to Agent 2
if [ -f "test-agent1/certs/ca.crt" ]; then
    echo "📋 Sharing CA certificate with Agent 2..."
    cp test-agent1/certs/ca.crt test-agent2/certs/
    cp test-agent1/certs/ca.key test-agent2/certs/
    echo "✅ CA certificate shared"
else
    echo "❌ CA cert not found at test-agent1/certs/ca.crt"
    echo "   Check Agent 1 log for errors"
    tail -10 test-agent1/agent.log
    exit 1
fi

# Start Agent 2
echo "🚀 Starting Agent 2 (port 8444)..."
cd test-agent2
RUST_LOG=info ../target/release/csf-agent > agent.log 2>&1 &
AGENT2_PID=$!
cd ..
echo "   PID: $AGENT2_PID"

# Wait for connection
echo "⏳ Waiting for connection (10 seconds)..."
sleep 10

# Check logs
echo ""
echo "📊 Agent 1 Log:"
echo "==============="
tail -25 test-agent1/agent.log

echo ""
echo "📊 Agent 2 Log:"
echo "==============="
tail -25 test-agent2/agent.log

echo ""
echo "🔍 Checking for successful P2P connection..."
if grep -q "Connected to peer" test-agent1/agent.log && grep -q "Connected to peer" test-agent2/agent.log; then
    echo "✅ SUCCESS! Both agents connected via mTLS"
elif grep -q "P2P server listening" test-agent1/agent.log && grep -q "P2P server listening" test-agent2/agent.log; then
    echo "⚠️  Servers running but connection pending. Check logs for details."
else
    echo "❌ Connection not established"
fi

echo ""
echo "📝 Process Info:"
echo "   Agent 1 PID: $AGENT1_PID"
echo "   Agent 2 PID: $AGENT2_PID"
echo ""
echo "🛑 To stop agents:"
echo "   kill $AGENT1_PID $AGENT2_PID"
echo ""
echo "📖 To follow logs:"
echo "   tail -f test-agent1/agent.log"
echo "   tail -f test-agent2/agent.log"
echo ""
echo "🧹 To cleanup:"
echo "   kill $AGENT1_PID $AGENT2_PID 2>/dev/null"
echo "   rm -rf test-agent1 test-agent2"
