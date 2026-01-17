#!/bin/bash
# Script to start Community Node and Edge Node using the provided config files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Porta Node Startup Script ===${NC}"
echo ""

# Check if config files exist
if [ ! -f "$SERVER_DIR/community.toml" ]; then
    echo -e "${YELLOW}Warning: community.toml not found in $SERVER_DIR${NC}"
    exit 1
fi

if [ ! -f "$SERVER_DIR/edge.toml" ]; then
    echo -e "${YELLOW}Warning: edge.toml not found in $SERVER_DIR${NC}"
    exit 1
fi

# Check if server binary exists
if [ ! -f "$SERVER_DIR/target/release/porta-server" ]; then
    echo -e "${YELLOW}Server binary not found. Building...${NC}"
    cd "$PROJECT_ROOT"
    npm run build:server
fi

# Clean up any existing processes
echo "Cleaning up existing processes..."
pkill -f "porta-server" 2>/dev/null || true
sleep 2

# Start Community Node
echo ""
echo -e "${GREEN}Starting Community Node...${NC}"
cd "$SERVER_DIR"
"$SERVER_DIR/target/release/porta-server" --config community.toml > /tmp/porta-community.log 2>&1 &
COMMUNITY_PID=$!
echo "Community Node PID: $COMMUNITY_PID"
echo "Log file: /tmp/porta-community.log"

# Wait for Community Node to start
echo "Waiting for Community Node to start..."
for i in {1..15}; do
    if curl -s http://localhost:8091/porta/node/info > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Community Node is ready${NC}"
        COMMUNITY_NODE_ID=$(curl -s http://localhost:8091/porta/node/info | jq -r '.data.node_id')
        echo "   Node ID: $COMMUNITY_NODE_ID"
        echo "   HTTP API: http://localhost:8091"
        echo "   P2P Multiaddr: /ip4/127.0.0.1/tcp/9010/p2p/$COMMUNITY_NODE_ID"
        break
    fi
    sleep 1
done

# Start Edge Node
echo ""
echo -e "${GREEN}Starting Edge Node...${NC}"
"$SERVER_DIR/target/release/porta-server" --config edge.toml > /tmp/porta-edge.log 2>&1 &
EDGE_PID=$!
echo "Edge Node PID: $EDGE_PID"
echo "Log file: /tmp/porta-edge.log"

# Wait for Edge Node to start
echo "Waiting for Edge Node to start..."
for i in {1..15}; do
    if curl -s http://localhost:8090/porta/node/info > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Edge Node is ready${NC}"
        echo "   HTTP API: http://localhost:8090"
        break
    fi
    sleep 1
done

echo ""
echo -e "${BLUE}=== Nodes Started Successfully ===${NC}"
echo ""
echo "Community Node:"
echo "  - PID: $COMMUNITY_PID"
echo "  - HTTP: http://localhost:8091"
echo "  - P2P TCP: 9010"
echo ""
echo "Edge Node:"
echo "  - PID: $EDGE_PID"
echo "  - HTTP: http://localhost:8090"
echo "  - P2P TCP: 9000"
echo ""
echo "To stop nodes: pkill -f porta-server"
echo ""
echo "To add community to Edge Node:"
if [ -n "$COMMUNITY_NODE_ID" ]; then
    echo "  curl -X POST http://localhost:8090/porta/community/add \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    -d '{\"name\": \"测试社区\", \"description\": \"本地测试社区\", \"multiaddr\": \"/ip4/127.0.0.1/tcp/9010/p2p/$COMMUNITY_NODE_ID\"}'"
fi
