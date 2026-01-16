#!/bin/bash
# ===========================================================================
# Porta System Test Infrastructure Setup
# ===========================================================================
# This script sets up the test environment with multiple node instances:
# - CommunityNode on port 8091 (PORTA_ROLE=community)
# - EdgeNode1 on port 8090 (PORTA_ROLE=edge)
# - EdgeNode2 on port 8092 (PORTA_ROLE=edge)
# ===========================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Node configurations
COMMUNITY_PORT=8091
EDGE1_PORT=8090
EDGE2_PORT=8092

# Database paths (in-memory for tests, or temp files)
COMMUNITY_DB="/tmp/porta-test-community.db"
EDGE1_DB="/tmp/porta-test-edge1.db"
EDGE2_DB="/tmp/porta-test-edge2.db"

# PID files
PID_DIR="/tmp/porta-test-pids"
COMMUNITY_PID="$PID_DIR/community.pid"
EDGE1_PID="$PID_DIR/edge1.pid"
EDGE2_PID="$PID_DIR/edge2.pid"

# Log files
LOG_DIR="/tmp/porta-test-logs"
COMMUNITY_LOG="$LOG_DIR/community.log"
EDGE1_LOG="$LOG_DIR/edge1.log"
EDGE2_LOG="$LOG_DIR/edge2.log"

print_header() {
    echo -e "${BLUE}==========================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}==========================================================================${NC}"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    print_header "Cleaning up test environment"
    
    # Kill existing processes
    if [ -f "$COMMUNITY_PID" ]; then
        kill $(cat "$COMMUNITY_PID") 2>/dev/null || true
        rm -f "$COMMUNITY_PID"
    fi
    if [ -f "$EDGE1_PID" ]; then
        kill $(cat "$EDGE1_PID") 2>/dev/null || true
        rm -f "$EDGE1_PID"
    fi
    if [ -f "$EDGE2_PID" ]; then
        kill $(cat "$EDGE2_PID") 2>/dev/null || true
        rm -f "$EDGE2_PID"
    fi
    
    # Clean up databases
    rm -f "$COMMUNITY_DB" "$EDGE1_DB" "$EDGE2_DB"
    
    # Kill any remaining porta processes on test ports
    lsof -ti:$COMMUNITY_PORT | xargs kill -9 2>/dev/null || true
    lsof -ti:$EDGE1_PORT | xargs kill -9 2>/dev/null || true
    lsof -ti:$EDGE2_PORT | xargs kill -9 2>/dev/null || true
    
    print_success "Cleanup complete"
}

build_backend() {
    print_header "Building server (release mode)"
    cd "$SERVER_DIR"
    cargo build --release
    print_success "Server built successfully"
}

start_community_node() {
    print_info "Starting CommunityNode on port $COMMUNITY_PORT..."
    
    "$SERVER_DIR/target/release/porta-server" \
        --port "$COMMUNITY_PORT" \
        --log-level info &
    
    echo $! > "$COMMUNITY_PID"
    sleep 2
    
    if curl -s "http://localhost:$COMMUNITY_PORT/porta/node/info" > /dev/null 2>&1; then
        print_success "CommunityNode started (PID: $(cat $COMMUNITY_PID))"
    else
        print_error "Failed to start CommunityNode"
        return 1
    fi
}

start_edge_node1() {
    print_info "Starting EdgeNode1 on port $EDGE1_PORT..."
    
    "$SERVER_DIR/target/release/porta-server" \
        --port "$EDGE1_PORT" \
        --log-level info &
    
    echo $! > "$EDGE1_PID"
    sleep 2
    
    if curl -s "http://localhost:$EDGE1_PORT/porta/node/info" > /dev/null 2>&1; then
        print_success "EdgeNode1 started (PID: $(cat $EDGE1_PID))"
    else
        print_error "Failed to start EdgeNode1"
        return 1
    fi
}

start_edge_node2() {
    print_info "Starting EdgeNode2 on port $EDGE2_PORT..."
    
    "$SERVER_DIR/target/release/porta-server" \
        --port "$EDGE2_PORT" \
        --log-level info &
    
    echo $! > "$EDGE2_PID"
    sleep 2
    
    if curl -s "http://localhost:$EDGE2_PORT/porta/node/info" > /dev/null 2>&1; then
        print_success "EdgeNode2 started (PID: $(cat $EDGE2_PID))"
    else
        print_error "Failed to start EdgeNode2"
        return 1
    fi
}

setup_dirs() {
    mkdir -p "$PID_DIR" "$LOG_DIR"
}

wait_for_port() {
    local port=$1
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "http://localhost:$port/porta/node/info" > /dev/null 2>&1; then
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 0.5
    done
    return 1
}

start_single_node() {
    # Start a single node for basic API testing
    print_header "Starting single node for API testing"
    setup_dirs
    cleanup
    
    print_info "Starting EdgeNode on port $EDGE1_PORT..."
    
    "$SERVER_DIR/target/release/porta-server" \
        --port "$EDGE1_PORT" \
        --log-level info > "$EDGE1_LOG" 2>&1 &
    
    echo $! > "$EDGE1_PID"
    
    if wait_for_port $EDGE1_PORT; then
        print_success "Node started on port $EDGE1_PORT"
        echo ""
        echo "API Base URL: http://localhost:$EDGE1_PORT"
        echo "PID: $(cat $EDGE1_PID)"
    else
        print_error "Failed to start node"
        cat "$EDGE1_LOG"
        return 1
    fi
}

start_all_nodes() {
    print_header "Starting all test nodes"
    setup_dirs
    cleanup
    
    start_community_node
    start_edge_node1
    start_edge_node2
    
    print_header "Test Environment Ready"
    echo ""
    echo "CommunityNode: http://localhost:$COMMUNITY_PORT"
    echo "EdgeNode1:     http://localhost:$EDGE1_PORT"
    echo "EdgeNode2:     http://localhost:$EDGE2_PORT"
    echo ""
    echo "To stop all nodes: $0 stop"
}

show_status() {
    print_header "Node Status"
    
    echo -n "CommunityNode ($COMMUNITY_PORT): "
    if curl -s "http://localhost:$COMMUNITY_PORT/porta/node/info" > /dev/null 2>&1; then
        echo -e "${GREEN}RUNNING${NC}"
    else
        echo -e "${RED}STOPPED${NC}"
    fi
    
    echo -n "EdgeNode1 ($EDGE1_PORT): "
    if curl -s "http://localhost:$EDGE1_PORT/porta/node/info" > /dev/null 2>&1; then
        echo -e "${GREEN}RUNNING${NC}"
    else
        echo -e "${RED}STOPPED${NC}"
    fi
    
    echo -n "EdgeNode2 ($EDGE2_PORT): "
    if curl -s "http://localhost:$EDGE2_PORT/porta/node/info" > /dev/null 2>&1; then
        echo -e "${GREEN}RUNNING${NC}"
    else
        echo -e "${RED}STOPPED${NC}"
    fi
}

# Export variables for test scripts
export COMMUNITY_PORT EDGE1_PORT EDGE2_PORT
export COMMUNITY_URL="http://localhost:$COMMUNITY_PORT"
export EDGE1_URL="http://localhost:$EDGE1_PORT"
export EDGE2_URL="http://localhost:$EDGE2_PORT"

case "${1:-}" in
    start)
        start_all_nodes
        ;;
    start-single)
        start_single_node
        ;;
    stop)
        cleanup
        ;;
    status)
        show_status
        ;;
    build)
        build_backend
        ;;
    restart)
        cleanup
        sleep 1
        start_all_nodes
        ;;
    *)
        echo "Usage: $0 {start|start-single|stop|status|build|restart}"
        echo ""
        echo "Commands:"
        echo "  start        - Start all test nodes (Community + 2 Edge)"
        echo "  start-single - Start single edge node for API testing"
        echo "  stop         - Stop all test nodes"
        echo "  status       - Show node status"
        echo "  build        - Build backend in release mode"
        echo "  restart      - Stop and start all nodes"
        exit 1
        ;;
esac
