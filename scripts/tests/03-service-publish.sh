#!/bin/bash
# ===========================================================================
# Test 03: Service Publishing (Requirement 3.1.8)
# ===========================================================================
# Tests service publishing including:
# - Publish new service
# - List published services
# - Unpublish service
# - Remove service
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "03 - Service Publishing (Requirement 3.1.8)"

# ===========================================================================
print_section "1. List Published Services (Initial)"
# ===========================================================================

api_get "/porta/service/published"
assert_api_success "GET published services returns success"
INITIAL_COUNT=$(json_get ".data | length")
echo "Initial published services: $INITIAL_COUNT"

# ===========================================================================
print_section "2. Publish Service - Missing Name (Should Fail)"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "",
    "type": "HTTP",
    "port": 8080,
    "summary": "Test service"
}'
assert_api_error "Publish without name returns error"

# ===========================================================================
print_section "3. Publish Service - Missing Type (Should Fail)"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "Test Service",
    "type": "",
    "port": 8080,
    "summary": "Test service"
}'
assert_api_error "Publish without type returns error"

# ===========================================================================
print_section "4. Publish HTTP Service"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "Web API 服务",
    "type": "HTTP",
    "port": 3000,
    "summary": "REST API 后端服务"
}'
assert_api_success "Publish HTTP service returns success"
assert_json_not_null ".data.id" "Published service has id"
assert_json_eq ".data.name" "Web API 服务" "Service name matches"
assert_json_eq ".data.type" "HTTP" "Service type is HTTP"
assert_json_eq ".data.port" "3000" "Service port is 3000"
assert_json_eq ".data.status" "在线" "Service status is 在线"
assert_json_not_null ".data.publish_date" "Service has publish date"

HTTP_SERVICE_ID=$(json_get ".data.id")
echo "Published HTTP service: $HTTP_SERVICE_ID"

# ===========================================================================
print_section "5. Publish HTTPS Service"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "安全网站",
    "type": "HTTPS",
    "port": 443,
    "summary": "HTTPS 加密网站"
}'
assert_api_success "Publish HTTPS service returns success"
assert_json_eq ".data.type" "HTTPS" "Service type is HTTPS"

HTTPS_SERVICE_ID=$(json_get ".data.id")
echo "Published HTTPS service: $HTTPS_SERVICE_ID"

# ===========================================================================
print_section "6. Publish TCP Service"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "数据库服务",
    "type": "TCP",
    "port": 5432,
    "summary": "PostgreSQL 数据库"
}'
assert_api_success "Publish TCP service returns success"
assert_json_eq ".data.type" "TCP" "Service type is TCP"

TCP_SERVICE_ID=$(json_get ".data.id")
echo "Published TCP service: $TCP_SERVICE_ID"

# ===========================================================================
print_section "7. Publish WebSocket Service"
# ===========================================================================

api_post "/porta/service/publish" '{
    "name": "实时通信",
    "type": "ws",
    "port": 8080,
    "summary": "WebSocket 实时服务"
}'
assert_api_success "Publish WebSocket service returns success"
assert_json_eq ".data.type" "ws" "Service type is ws"

WS_SERVICE_ID=$(json_get ".data.id")

# ===========================================================================
print_section "8. List Published Services (After Publishing)"
# ===========================================================================

api_get "/porta/service/published"
assert_api_success "GET published services after publishing"
NEW_COUNT=$(json_get ".data | length")
EXPECTED_COUNT=$((INITIAL_COUNT + 4))

if [ "$NEW_COUNT" = "$EXPECTED_COUNT" ]; then
    print_pass "Published services count increased to $NEW_COUNT"
else
    print_fail "Expected $EXPECTED_COUNT services, got $NEW_COUNT"
fi

# ===========================================================================
print_section "9. Verify Published Service Fields"
# ===========================================================================

# Check required fields per requirement 3.1.4 (Service Definition)
REQUIRED_FIELDS=("id" "name" "type" "port" "summary" "subscriptions" "status" "publish_date")
FIRST_SERVICE=$(echo "$LAST_RESPONSE" | jq '.data[0]')

for field in "${REQUIRED_FIELDS[@]}"; do
    if echo "$FIRST_SERVICE" | jq -e ".$field" > /dev/null 2>&1; then
        print_pass "Service field '$field' is present"
    else
        print_fail "Service field '$field' is missing"
    fi
done

# ===========================================================================
print_section "10. Unpublish Service"
# ===========================================================================

api_post "/porta/service/unpublish" "{\"id\": \"$HTTP_SERVICE_ID\"}"
assert_api_success "Unpublish service returns success"

# Verify status changed
api_get "/porta/service/published"
assert_api_success "GET published services after unpublish"

# Find the service and check status
SERVICE_STATUS=$(echo "$LAST_RESPONSE" | jq -r ".data[] | select(.id == \"$HTTP_SERVICE_ID\") | .status")
if [ "$SERVICE_STATUS" = "已下架" ]; then
    print_pass "Service status changed to 已下架"
else
    print_fail "Service status should be 已下架, got $SERVICE_STATUS"
fi

# ===========================================================================
print_section "11. Unpublish Non-existent Service (Should Fail)"
# ===========================================================================

api_post "/porta/service/unpublish" '{"id": ""}'
assert_api_error "Unpublish with empty id returns error"

# ===========================================================================
print_section "12. Remove Service"
# ===========================================================================

api_post "/porta/service/remove" "{\"id\": \"$HTTP_SERVICE_ID\"}"
assert_api_success "Remove service returns success"

# Verify removed
api_get "/porta/service/published"
AFTER_REMOVE_COUNT=$(json_get ".data | length")
EXPECTED_AFTER_REMOVE=$((EXPECTED_COUNT - 1))

if [ "$AFTER_REMOVE_COUNT" = "$EXPECTED_AFTER_REMOVE" ]; then
    print_pass "Service removed from list"
else
    print_fail "Expected $EXPECTED_AFTER_REMOVE services, got $AFTER_REMOVE_COUNT"
fi

# ===========================================================================
print_section "13. Remove Non-existent Service (Should Fail)"
# ===========================================================================

api_post "/porta/service/remove" '{"id": "non-existent-service"}'
assert_api_error "Remove non-existent service returns error"

# ===========================================================================
print_section "14. Remove Remaining Test Services (Cleanup)"
# ===========================================================================

api_post "/porta/service/remove" "{\"id\": \"$HTTPS_SERVICE_ID\"}"
api_post "/porta/service/remove" "{\"id\": \"$TCP_SERVICE_ID\"}"
api_post "/porta/service/remove" "{\"id\": \"$WS_SERVICE_ID\"}"
print_pass "Cleanup completed"

# ===========================================================================
print_summary
