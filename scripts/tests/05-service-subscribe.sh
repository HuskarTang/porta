#!/bin/bash
# ===========================================================================
# Test 05: Service Subscription (Requirement 3.1.6)
# ===========================================================================
# Tests service subscription including:
# - Subscribe to a service
# - List subscribed services
# - Validate subscription fields
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "05 - Service Subscription (Requirement 3.1.6)"

# ===========================================================================
print_section "1. List Subscribed Services (Initial)"
# ===========================================================================

api_get "/porta/service/subscriptions"
assert_api_success "GET subscriptions returns success"
INITIAL_COUNT=$(json_get ".data | length")
echo "Initial subscriptions: $INITIAL_COUNT"

# ===========================================================================
print_section "2. Subscribe - Missing Name (Should Fail)"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "",
    "type": "HTTP",
    "community": "dev-community",
    "remote_addr": "192.168.1.100:8080",
    "local_mapping": "127.0.0.1:18080"
}'
assert_api_error "Subscribe without name returns error"

# ===========================================================================
print_section "3. Subscribe - Missing Type (Should Fail)"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "Test Service",
    "type": "",
    "community": "dev-community",
    "remote_addr": "192.168.1.100:8080",
    "local_mapping": "127.0.0.1:18080"
}'
assert_api_error "Subscribe without type returns error"

# ===========================================================================
print_section "4. Subscribe - Missing Community (Should Fail)"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "Test Service",
    "type": "HTTP",
    "community": "",
    "remote_addr": "192.168.1.100:8080",
    "local_mapping": "127.0.0.1:18080"
}'
assert_api_error "Subscribe without community returns error"

# ===========================================================================
print_section "5. Subscribe to HTTP Service"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "远程 Web 服务",
    "type": "HTTP",
    "community": "dev-community",
    "remote_addr": "192.168.1.100:3000",
    "local_mapping": "127.0.0.1:13000",
    "service_uuid": "svc-http-001"
}'
assert_api_success "Subscribe to HTTP service returns success"
assert_json_not_null ".data.id" "Subscription has id"
assert_json_eq ".data.name" "远程 Web 服务" "Subscription name matches"
assert_json_eq ".data.type" "HTTP" "Subscription type matches"
assert_json_eq ".data.status" "畅通" "Initial status is 畅通"

HTTP_SUB_ID=$(json_get ".data.id")
echo "Created subscription: $HTTP_SUB_ID"

# ===========================================================================
print_section "6. Subscribe to TCP Service"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "数据库连接",
    "type": "TCP",
    "community": "data-team",
    "remote_addr": "10.0.0.50:5432",
    "local_mapping": "127.0.0.1:15432",
    "service_uuid": "svc-tcp-001"
}'
assert_api_success "Subscribe to TCP service returns success"
assert_json_eq ".data.type" "TCP" "Subscription type is TCP"

TCP_SUB_ID=$(json_get ".data.id")
echo "Created subscription: $TCP_SUB_ID"

# ===========================================================================
print_section "7. Subscribe to WebSocket Service"
# ===========================================================================

api_post "/porta/service/subscribe" '{
    "name": "实时推送",
    "type": "ws",
    "community": "dev-community",
    "remote_addr": "192.168.1.100:8080",
    "local_mapping": "127.0.0.1:18080",
    "service_uuid": "svc-ws-001"
}'
assert_api_success "Subscribe to WebSocket service returns success"

WS_SUB_ID=$(json_get ".data.id")

# ===========================================================================
print_section "8. List Subscribed Services (After Subscribing)"
# ===========================================================================

api_get "/porta/service/subscriptions"
assert_api_success "GET subscriptions after subscribing"
NEW_COUNT=$(json_get ".data | length")
EXPECTED_COUNT=$((INITIAL_COUNT + 3))

if [ "$NEW_COUNT" = "$EXPECTED_COUNT" ]; then
    print_pass "Subscription count increased to $NEW_COUNT"
else
    print_fail "Expected $EXPECTED_COUNT subscriptions, got $NEW_COUNT"
fi

# ===========================================================================
print_section "9. Verify Subscription Fields"
# ===========================================================================

FIRST_SUB=$(echo "$LAST_RESPONSE" | jq '.data[0]')
echo "Sample subscription: $FIRST_SUB"

# Check required fields
REQUIRED_FIELDS=("id" "name" "type" "community" "remote_addr" "local_mapping" "status")
for field in "${REQUIRED_FIELDS[@]}"; do
    if echo "$FIRST_SUB" | jq -e ".$field" > /dev/null 2>&1; then
        print_pass "Subscription field '$field' is present"
    else
        print_fail "Subscription field '$field' is missing"
    fi
done

# ===========================================================================
print_section "10. Verify Status Values"
# ===========================================================================

# Status should be one of: 畅通, 连接中, 断开
STATUS=$(echo "$FIRST_SUB" | jq -r '.status')
if [ "$STATUS" = "畅通" ] || [ "$STATUS" = "连接中" ] || [ "$STATUS" = "断开" ]; then
    print_pass "Status value '$STATUS' is valid"
else
    print_fail "Status value '$STATUS' is not a valid status"
fi

# ===========================================================================
print_summary
