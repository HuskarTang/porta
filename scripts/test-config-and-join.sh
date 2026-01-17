#!/bin/bash
# Comprehensive test script for config system and community join

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

echo "=== Testing Porta Configuration System and Community Join ==="
echo ""

# Clean up
echo "1. Cleaning up..."
pkill -f "porta-server" 2>/dev/null || true
sleep 2
rm -f /tmp/porta-test-*.db
echo "   ✅ Cleaned up"
echo ""

# Test 1: Config creation
echo "2. Testing config file creation..."
CONFIG_OUTPUT=$("$SERVER_DIR/target/release/porta-server" --print-config 2>&1)
echo "$CONFIG_OUTPUT" | head -10
if echo "$CONFIG_OUTPUT" | grep -q "Default config created"; then
  echo "   ✅ Config file auto-created"
else
  echo "   ℹ️  Using existing config"
fi
echo ""

# Test 2: Start CommunityNode with unique key
echo "3. Starting CommunityNode..."
cd "$PROJECT_ROOT"
COMMUNITY_CONFIG="/tmp/porta-test-community-config.toml"
cat > "$COMMUNITY_CONFIG" <<EOF
[server]
listen_addr = "0.0.0.0"
port = 8091

[node]
name = "Test Community Node"
role = "community"
key_path = "/tmp/porta-test-community.key"

[database]
path = "/tmp/porta-test-community.db"

[p2p]
tcp_port = 9010
quic_port = 9011
mdns_enable = true
dht_enable = true

[logging]
level = "info"
format = "compact"
EOF

PORTA_ROLE=community \
"$SERVER_DIR/target/release/porta-server" --config "$COMMUNITY_CONFIG" > /tmp/porta-test-community.log 2>&1 &
COMMUNITY_PID=$!
echo "   PID: $COMMUNITY_PID"

# Wait for startup
for i in {1..20}; do
  if curl -s http://localhost:8091/porta/node/info > /dev/null 2>&1; then
    echo "   ✅ CommunityNode ready"
    break
  fi
  if [ $i -eq 20 ]; then
    echo "   ❌ CommunityNode failed to start"
    tail -20 /tmp/porta-test-community.log
    exit 1
  fi
  sleep 1
done

COMMUNITY_INFO=$(curl -s http://localhost:8091/porta/node/info)
COMMUNITY_NODE_ID=$(echo "$COMMUNITY_INFO" | jq -r '.data.node_id')
echo "   Node ID: $COMMUNITY_NODE_ID"
echo ""

# Test 3: Start EdgeNode with unique key
echo "4. Starting EdgeNode..."
EDGE_CONFIG="/tmp/porta-test-edge-config.toml"
cat > "$EDGE_CONFIG" <<EOF
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "Test Edge Node"
role = "edge"
key_path = "/tmp/porta-test-edge.key"

[database]
path = "/tmp/porta-test-edge.db"

[p2p]
tcp_port = 9000
quic_port = 9001
mdns_enable = true
dht_enable = true

[logging]
level = "info"
format = "compact"
EOF

PORTA_ROLE=edge \
"$SERVER_DIR/target/release/porta-server" --config "$EDGE_CONFIG" > /tmp/porta-test-edge.log 2>&1 &
EDGE_PID=$!
echo "   PID: $EDGE_PID"

# Wait for startup
for i in {1..20}; do
  if curl -s http://localhost:8090/porta/node/info > /dev/null 2>&1; then
    echo "   ✅ EdgeNode ready"
    break
  fi
  if [ $i -eq 20 ]; then
    echo "   ❌ EdgeNode failed to start"
    tail -20 /tmp/porta-test-edge.log
    kill $COMMUNITY_PID 2>/dev/null || true
    exit 1
  fi
  sleep 1
done

EDGE_INFO=$(curl -s http://localhost:8090/porta/node/info)
EDGE_NODE_ID=$(echo "$EDGE_INFO" | jq -r '.data.node_id')
echo "   Node ID: $EDGE_NODE_ID"
echo ""

# Verify Peer IDs are different
if [ "$COMMUNITY_NODE_ID" = "$EDGE_NODE_ID" ]; then
  echo "   ❌ ERROR: Both nodes have the same Peer ID!"
  kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
  exit 1
else
  echo "   ✅ Peer IDs are different"
fi
echo ""

# Test 4: Add community
echo "5. Adding community..."
# Wait a bit for P2P to fully initialize
sleep 3
MULTIADDR="/ip4/127.0.0.1/tcp/9010/p2p/$COMMUNITY_NODE_ID"
echo "   Multiaddr: $MULTIADDR"

ADD_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/add \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"测试社区\", \"description\": \"配置系统测试社区\", \"multiaddr\": \"$MULTIADDR\"}")

COMMUNITY_ID=$(echo "$ADD_RESPONSE" | jq -r '.data.id // empty')
if [ -n "$COMMUNITY_ID" ] && [ "$COMMUNITY_ID" != "null" ]; then
  echo "   ✅ Community added: $COMMUNITY_ID"
else
  echo "   ❌ Failed to add community"
  echo "$ADD_RESPONSE" | jq '.'
  exit 1
fi
echo ""

# Test 5: Join community
echo "6. Joining community..."
sleep 3

JOIN_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/connect \
  -H "Content-Type: application/json" \
  -d "{\"id\": \"$COMMUNITY_ID\"}")

if echo "$JOIN_RESPONSE" | jq -e '.code == 0' > /dev/null 2>&1; then
  echo "   ✅ Community join successful!"
  echo ""
  echo "   Community status:"
  curl -s http://localhost:8090/porta/community/list | jq ".data[] | select(.id == \"$COMMUNITY_ID\")" | head -10
  echo ""
  echo "=== All tests passed! ==="
  echo ""
  echo "To stop nodes: pkill -f porta-server"
  # Cleanup
  kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
  rm -f "$COMMUNITY_CONFIG" "$EDGE_CONFIG"
  exit 0
else
  echo "   ❌ Community join failed"
  echo "$JOIN_RESPONSE" | jq '.'
  echo ""
  echo "=== Logs (last 50 lines) ==="
  echo "--- EdgeNode ---"
  tail -50 /tmp/porta-test-edge.log | grep -E "\[社区连接\]|\[P2P\]|ERROR|WARN" || tail -30 /tmp/porta-test-edge.log
  echo ""
  echo "--- CommunityNode ---"
  tail -50 /tmp/porta-test-community.log | grep -E "\[社区连接\]|\[P2P\]|ERROR|WARN" || tail -30 /tmp/porta-test-community.log
  kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
  rm -f "$COMMUNITY_CONFIG" "$EDGE_CONFIG"
  exit 1
fi
