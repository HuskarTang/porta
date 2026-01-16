#!/bin/bash
# ===========================================================================
# Test 07: Community Node Management (Requirement 3.1.10)
# ===========================================================================
# Tests community node management including:
# - List community nodes
# - Ban edge node
# - Unban edge node
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "07 - Community Node Management (Requirement 3.1.10)"

# ===========================================================================
print_section "1. List Community Nodes"
# ===========================================================================

api_get "/porta/community/node/list"
assert_api_success "GET community nodes returns success"

NODES_COUNT=$(json_get ".data | length")
echo "Community nodes: $NODES_COUNT"

if [ "$NODES_COUNT" -gt 0 ]; then
    FIRST_NODE=$(echo "$LAST_RESPONSE" | jq '.data[0]')
    echo "Sample node: $FIRST_NODE"
    
    # Check required fields
    REQUIRED_FIELDS=("id" "uuid" "status" "banned")
    for field in "${REQUIRED_FIELDS[@]}"; do
        if echo "$FIRST_NODE" | jq -e ".$field" > /dev/null 2>&1; then
            print_pass "Node field '$field' is present"
        else
            print_fail "Node field '$field' is missing"
        fi
    done
    
    # Get a node ID for testing ban/unban
    TEST_NODE_ID=$(echo "$FIRST_NODE" | jq -r '.id')
else
    print_pass "No community nodes (expected in isolated test)"
    TEST_NODE_ID=""
fi

# ===========================================================================
print_section "2. Ban Node - Non-existent (Should Fail)"
# ===========================================================================

api_post "/porta/community/node/ban" '{"id": "non-existent-peer-id"}'
assert_api_error "Ban non-existent node returns error"

# ===========================================================================
print_section "3. Unban Node - Non-existent (Should Fail)"
# ===========================================================================

api_post "/porta/community/node/unban" '{"id": "non-existent-peer-id"}'
assert_api_error "Unban non-existent node returns error"

# ===========================================================================
print_section "4. Ban Node (if available)"
# ===========================================================================

if [ -n "$TEST_NODE_ID" ] && [ "$TEST_NODE_ID" != "null" ]; then
    api_post "/porta/community/node/ban" "{\"id\": \"$TEST_NODE_ID\"}"
    assert_api_success "Ban node returns success"
    
    # Verify banned status
    api_get "/porta/community/node/list"
    BANNED_STATUS=$(echo "$LAST_RESPONSE" | jq -r ".data[] | select(.id == \"$TEST_NODE_ID\") | .banned")
    
    if [ "$BANNED_STATUS" = "true" ]; then
        print_pass "Node banned status is true"
    else
        print_fail "Node should be banned, got $BANNED_STATUS"
    fi
else
    print_pass "No nodes available to test ban (expected)"
fi

# ===========================================================================
print_section "5. Unban Node (if available)"
# ===========================================================================

if [ -n "$TEST_NODE_ID" ] && [ "$TEST_NODE_ID" != "null" ]; then
    api_post "/porta/community/node/unban" "{\"id\": \"$TEST_NODE_ID\"}"
    assert_api_success "Unban node returns success"
    
    # Verify unbanned status
    api_get "/porta/community/node/list"
    BANNED_STATUS=$(echo "$LAST_RESPONSE" | jq -r ".data[] | select(.id == \"$TEST_NODE_ID\") | .banned")
    
    if [ "$BANNED_STATUS" = "false" ]; then
        print_pass "Node banned status is false"
    else
        print_fail "Node should be unbanned, got $BANNED_STATUS"
    fi
else
    print_pass "No nodes available to test unban (expected)"
fi

# ===========================================================================
print_section "6. Verify Node Status Values"
# ===========================================================================

api_get "/porta/community/node/list"
assert_api_success "GET nodes for status check"

# Valid statuses: 在线, 离线
STATUSES=$(echo "$LAST_RESPONSE" | jq -r '.data[].status' 2>/dev/null || echo "")
VALID_STATUSES="在线 离线 online offline"

if [ -n "$STATUSES" ]; then
    for status in $STATUSES; do
        if echo "$VALID_STATUSES" | grep -qi "$status"; then
            print_pass "Node status '$status' is valid"
        else
            print_fail "Node status '$status' is invalid"
        fi
    done
else
    print_pass "No node statuses to validate (expected)"
fi

# ===========================================================================
print_summary
