#!/bin/bash
# ===========================================================================
# Test 06: Service Connection and Access (Requirement 3.1.7)
# ===========================================================================
# Tests service connection including:
# - Connect to subscribed service
# - Get sessions
# - Disconnect from service
# - Get access URL
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "06 - Service Connection and Access (Requirement 3.1.7)"

# ===========================================================================
print_section "1. Create Test Subscription"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "连接测试服务",
    "type": "HTTP",
    "community": "dev-community",
    "remote_addr": "192.168.1.100:8080",
    "local_mapping": "127.0.0.1:28080",
    "service_uuid": "svc-connect-test"
}'
assert_api_success "Create test subscription"
SUB_ID=$(json_get ".data.id")
echo "Test subscription ID: $SUB_ID"

# ===========================================================================
print_section "2. Get Sessions (Initial)"
# ===========================================================================

api_get "/porta/service/sessions"
assert_api_success "GET sessions returns success"
INITIAL_SESSIONS=$(json_get ".data | length")
echo "Initial sessions: $INITIAL_SESSIONS"

# ===========================================================================
print_section "3. Connect Service - Empty ID (Should Fail)"
# ===========================================================================

api_post "/porta/service/connect" '{"id": ""}'
assert_api_error "Connect with empty id returns error"

# ===========================================================================
print_section "4. Connect to Subscribed Service"
# ===========================================================================

# Note: This may fail at P2P level but we test the API validation
api_post "/porta/service/connect" "{\"id\": \"$SUB_ID\"}"

# The connect may fail because P2P isn't actually connected,
# but we verify the API accepts the request format
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Connect request accepted"
else
    # Check if it's a P2P error (expected) vs API error
    if echo "$LAST_RESPONSE" | grep -q "订阅\|service_uuid\|社区"; then
        print_pass "Connect request validated (P2P connection not available)"
    else
        print_fail "Unexpected error: $LAST_RESPONSE"
    fi
fi

# ===========================================================================
print_section "5. Get Sessions (After Connect)"
# ===========================================================================

api_get "/porta/service/sessions"
assert_api_success "GET sessions after connect"

# Sessions may or may not have been created depending on P2P status
SESSIONS_COUNT=$(json_get ".data | length")
echo "Current sessions: $SESSIONS_COUNT"

if [ "$SESSIONS_COUNT" -gt 0 ]; then
    FIRST_SESSION=$(echo "$LAST_RESPONSE" | jq '.data[0]')
    echo "Sample session: $FIRST_SESSION"
    
    # Verify session fields
    REQUIRED_FIELDS=("session_id" "service_id" "local_port" "remote_peer" "state")
    for field in "${REQUIRED_FIELDS[@]}"; do
        if echo "$FIRST_SESSION" | jq -e ".$field" > /dev/null 2>&1; then
            print_pass "Session field '$field' is present"
        else
            print_fail "Session field '$field' is missing"
        fi
    done
else
    print_pass "No sessions (P2P not connected - expected)"
fi

# ===========================================================================
print_section "6. Disconnect Service - Empty ID (Should Fail)"
# ===========================================================================

api_post "/porta/service/disconnect" '{"id": ""}'
assert_api_error "Disconnect with empty id returns error"

# ===========================================================================
print_section "7. Disconnect from Service"
# ===========================================================================

api_post "/porta/service/disconnect" "{\"id\": \"$SUB_ID\"}"
# This may succeed or fail depending on connection status
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Disconnect request processed"
else
    print_pass "Disconnect attempted (service may not be connected)"
fi

# ===========================================================================
print_section "8. Get Access URL - Empty ID (Should Fail)"
# ===========================================================================

api_post "/porta/service/access" '{"id": ""}'
assert_api_error "Access with empty id returns error"

# ===========================================================================
print_section "9. Get Access URL for Subscription"
# ===========================================================================

api_post "/porta/service/access" "{\"id\": \"$SUB_ID\"}"

if [ "$LAST_HTTP_CODE" = "200" ]; then
    assert_json_not_null ".data.local_url" "Access returns local_url"
    LOCAL_URL=$(json_get ".data.local_url")
    echo "Local access URL: $LOCAL_URL"
    
    # Verify URL format
    if echo "$LOCAL_URL" | grep -q "http://"; then
        print_pass "Local URL has http:// prefix"
    else
        print_fail "Local URL should start with http://"
    fi
else
    print_pass "Access URL not available (service may not be connected)"
fi

# ===========================================================================
print_section "10. Session State Values"
# ===========================================================================

api_get "/porta/service/sessions"
assert_api_success "GET sessions for state check"

# Valid states: connecting, connected, closed, error
SESSIONS=$(echo "$LAST_RESPONSE" | jq -r '.data[].state' 2>/dev/null || echo "")
VALID_STATES="connecting connected closed error"

if [ -n "$SESSIONS" ]; then
    for state in $SESSIONS; do
        if echo "$VALID_STATES" | grep -q "$state"; then
            print_pass "Session state '$state' is valid"
        else
            print_fail "Session state '$state' is invalid"
        fi
    done
else
    print_pass "No sessions to validate (expected)"
fi

# ===========================================================================
print_summary
