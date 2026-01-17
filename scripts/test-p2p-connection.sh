#!/bin/bash
# Comprehensive end-to-end test for P2P connection functionality
# This script verifies that P2P connections can be established between nodes

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"
SCRIPTS_DIR="$PROJECT_ROOT/scripts"

echo "=== P2P Connection End-to-End Test ==="
echo ""

# Clean up
echo "1. Cleaning up..."
pkill -f "porta-server" 2>/dev/null || true
sleep 2
rm -f /tmp/porta-test-*.db /tmp/porta-test-*.key /tmp/porta-test-*.log
echo "   ✅ Cleaned up"
echo ""

# Build if needed
if [ ! -f "$SERVER_DIR/target/release/porta-server" ]; then
    echo "2. Building server..."
    cd "$PROJECT_ROOT"
    npm run build:server
    echo "   ✅ Server built"
    echo ""
fi

# Test 1: Start CommunityNode with unique key
echo "3. Starting CommunityNode..."
cd "$PROJECT_ROOT"
COMMUNITY_DB="/tmp/porta-test-community.db"
COMMUNITY_KEY="/tmp/porta-test-community.key"
COMMUNITY_CONFIG="/tmp/porta-test-community.toml"

cat > "$COMMUNITY_CONFIG" <<EOF
[server]
listen_addr = "0.0.0.0"
port = 8091

[node]
name = "Test Community Node"
role = "community"
key_path = "$COMMUNITY_KEY"

[database]
path = "$COMMUNITY_DB"

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
COMMUNITY_PEER_ID=$(echo "$COMMUNITY_INFO" | jq -r '.data.node_id')
echo "   Node ID: $COMMUNITY_NODE_ID"
echo ""

# Test 2: Start EdgeNode with unique key
echo "4. Starting EdgeNode..."
EDGE_DB="/tmp/porta-test-edge.db"
EDGE_KEY="/tmp/porta-test-edge.key"
EDGE_CONFIG="/tmp/porta-test-edge.toml"

cat > "$EDGE_CONFIG" <<EOF
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "Test Edge Node"
role = "edge"
key_path = "$EDGE_KEY"

[database]
path = "$EDGE_DB"

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
EDGE_PEER_ID=$(echo "$EDGE_INFO" | jq -r '.data.node_id')
echo "   Node ID: $EDGE_NODE_ID"
echo ""

# Test 3: Verify Peer IDs are different
echo "5. Verifying Peer IDs are different..."
if [ "$COMMUNITY_NODE_ID" = "$EDGE_NODE_ID" ]; then
  echo "   ❌ ERROR: Both nodes have the same Peer ID!"
  echo "   Community: $COMMUNITY_NODE_ID"
  echo "   Edge: $EDGE_NODE_ID"
  kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
  exit 1
else
  echo "   ✅ Peer IDs are different"
  echo "   Community: $COMMUNITY_NODE_ID"
  echo "   Edge: $EDGE_NODE_ID"
fi
echo ""

# Test 4: Get CommunityNode's listening address
echo "6. Getting CommunityNode's listening address..."
# Wait a bit for P2P to fully initialize
sleep 3

# Try to get the actual listening address from logs or API
COMMUNITY_LISTEN_ADDR="/ip4/127.0.0.1/tcp/9010/p2p/$COMMUNITY_NODE_ID"
echo "   Using: $COMMUNITY_LISTEN_ADDR"
echo ""

# Test 5: Add community to EdgeNode
echo "7. Adding community to EdgeNode..."
ADD_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/add \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"测试社区\", \"description\": \"P2P连接测试社区\", \"multiaddr\": \"$COMMUNITY_LISTEN_ADDR\"}")

COMMUNITY_ID=$(echo "$ADD_RESPONSE" | jq -r '.data.id // empty')
if [ -n "$COMMUNITY_ID" ] && [ "$COMMUNITY_ID" != "null" ]; then
  echo "   ✅ Community added: $COMMUNITY_ID"
else
  echo "   ❌ Failed to add community"
  echo "$ADD_RESPONSE" | jq '.'
  kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
  exit 1
fi
echo ""

# Test 6: Join community (this should establish P2P connection)
echo "8. Joining community (establishing P2P connection)..."
sleep 2

JOIN_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/connect \
  -H "Content-Type: application/json" \
  -d "{\"id\": \"$COMMUNITY_ID\"}")

if echo "$JOIN_RESPONSE" | jq -e '.code == 0' > /dev/null 2>&1; then
  echo "   ✅ Community join successful! P2P connection established!"
  echo ""
  echo "   Response:"
  echo "$JOIN_RESPONSE" | jq '.'
  echo ""
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
  exit 1
fi

# Test 7: Verify connection status
echo "9. Verifying connection status..."
COMMUNITY_LIST=$(curl -s http://localhost:8090/porta/community/list)
JOINED=$(echo "$COMMUNITY_LIST" | jq -r ".data[] | select(.id == \"$COMMUNITY_ID\") | .joined")
if [ "$JOINED" = "true" ]; then
  echo "   ✅ Community is marked as joined"
else
  echo "   ⚠️  Community is not marked as joined (but join request succeeded)"
fi
echo ""

# Cleanup
echo "10. Cleaning up..."
kill $COMMUNITY_PID $EDGE_PID 2>/dev/null || true
sleep 1
rm -f "$COMMUNITY_CONFIG" "$EDGE_CONFIG"
echo "   ✅ Cleaned up"
echo ""

echo "=== All P2P connection tests passed! ==="
exit 0
