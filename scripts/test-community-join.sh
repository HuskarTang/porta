#!/bin/bash
# Test script for community join functionality

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

# Clean up
echo "=== Cleaning up ==="
pkill -f "porta-server" 2>/dev/null || true
sleep 2
rm -f /tmp/porta-test-community.db /tmp/porta-test-edge1.db
echo "✅ Cleaned up databases"

# Start CommunityNode
echo ""
echo "=== Starting CommunityNode ==="
cd "$PROJECT_ROOT"
PORTA_ROLE=community PORTA_DB=/tmp/porta-test-community.db \
  "$SERVER_DIR/target/release/porta-server" --port 8091 --log-level info > /tmp/porta-test-community.log 2>&1 &
COMMUNITY_PID=$!
echo "CommunityNode PID: $COMMUNITY_PID"

# Wait for CommunityNode
echo "Waiting for CommunityNode to start..."
for i in {1..30}; do
  if curl -s http://localhost:8091/porta/node/info > /dev/null 2>&1; then
    echo "✅ CommunityNode ready"
    break
  fi
  sleep 1
done

COMMUNITY_NODE_ID=$(curl -s http://localhost:8091/porta/node/info | jq -r '.data.node_id')
echo "CommunityNode ID: $COMMUNITY_NODE_ID"

# Start EdgeNode
echo ""
echo "=== Starting EdgeNode ==="
PORTA_ROLE=edge PORTA_DB=/tmp/porta-test-edge1.db \
  "$SERVER_DIR/target/release/porta-server" --port 8090 --log-level info > /tmp/porta-test-edge1.log 2>&1 &
EDGE_PID=$!
echo "EdgeNode PID: $EDGE_PID"

# Wait for EdgeNode
echo "Waiting for EdgeNode to start..."
for i in {1..30}; do
  if curl -s http://localhost:8090/porta/node/info > /dev/null 2>&1; then
    echo "✅ EdgeNode ready"
    break
  fi
  sleep 1
done

# Test community join
echo ""
echo "=== Testing Community Join ==="
MULTIADDR="/ip4/127.0.0.1/tcp/9000/p2p/$COMMUNITY_NODE_ID"
echo "Multiaddr: $MULTIADDR"
echo ""

echo "Adding community..."
ADD_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/add \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"测试社区\",
    \"description\": \"这是一个测试社区\",
    \"multiaddr\": \"$MULTIADDR\"
  }")

echo "$ADD_RESPONSE" | jq '.'

COMMUNITY_ID=$(echo "$ADD_RESPONSE" | jq -r '.data.id // empty')

if [ -z "$COMMUNITY_ID" ] || [ "$COMMUNITY_ID" == "null" ]; then
  echo "❌ Failed to add community"
  exit 1
fi

echo ""
echo "✅ Community added: $COMMUNITY_ID"
echo "Waiting 3 seconds before joining..."
sleep 3

echo ""
echo "Joining community..."
JOIN_RESPONSE=$(curl -s -X POST http://localhost:8090/porta/community/connect \
  -H "Content-Type: application/json" \
  -d "{\"id\": \"$COMMUNITY_ID\"}")

echo "$JOIN_RESPONSE" | jq '.'

if echo "$JOIN_RESPONSE" | jq -e '.code == 0' > /dev/null 2>&1; then
  echo ""
  echo "✅✅✅ SUCCESS: Community join successful! ✅✅✅"
  echo ""
  echo "Community status:"
  curl -s http://localhost:8090/porta/community/list | jq ".data[] | select(.id == \"$COMMUNITY_ID\")"
  echo ""
  echo "=== Test Summary ==="
  echo "✅ CommunityNode started on port 8091"
  echo "✅ EdgeNode started on port 8090"
  echo "✅ Community added successfully"
  echo "✅ Community join successful"
  echo ""
  echo "To stop nodes: pkill -f porta-server"
else
  echo ""
  echo "❌ Community join failed"
  echo ""
  echo "=== EdgeNode Logs (errors) ==="
  tail -50 /tmp/porta-test-edge1.log | grep -E "\[社区连接\]|\[P2P\]|ERROR|WARN" | tail -20 || tail -30 /tmp/porta-test-edge1.log
  echo ""
  echo "=== CommunityNode Logs (errors) ==="
  tail -50 /tmp/porta-test-community.log | grep -E "\[社区连接\]|\[P2P\]|ERROR|WARN" | tail -20 || tail -30 /tmp/porta-test-community.log
  exit 1
fi
