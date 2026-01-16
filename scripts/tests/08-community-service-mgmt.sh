#!/bin/bash
# ===========================================================================
# Test 08: Community Service Management (Requirement 3.1.9)
# ===========================================================================
# Tests community service management including:
# - List community services
# - Announce service
# - Disable service
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "08 - Community Service Management (Requirement 3.1.9)"

# ===========================================================================
print_section "1. List Community Services"
# ===========================================================================

api_get "/porta/community/service/list"
assert_api_success "GET community services returns success"

SERVICES_COUNT=$(json_get ".data | length")
echo "Community services: $SERVICES_COUNT"

if [ "$SERVICES_COUNT" -gt 0 ]; then
    FIRST_SERVICE=$(echo "$LAST_RESPONSE" | jq '.data[0]')
    echo "Sample service: $FIRST_SERVICE"
    
    # Check required fields
    REQUIRED_FIELDS=("id" "name" "uuid" "protocol" "port" "online" "announced")
    for field in "${REQUIRED_FIELDS[@]}"; do
        if echo "$FIRST_SERVICE" | jq -e ".$field" > /dev/null 2>&1; then
            print_pass "Service field '$field' is present"
        else
            print_fail "Service field '$field' is missing"
        fi
    done
    
    # Get a service ID for testing
    TEST_SERVICE_ID=$(echo "$FIRST_SERVICE" | jq -r '.uuid')
else
    print_pass "No community services (expected in isolated test)"
    TEST_SERVICE_ID=""
fi

# ===========================================================================
print_section "2. Announce Service - Non-existent (Should Fail)"
# ===========================================================================

api_post "/porta/community/service/announce" '{"id": "non-existent-service-id"}'
assert_api_error "Announce non-existent service returns error"

# ===========================================================================
print_section "3. Disable Service - Non-existent (Should Fail)"
# ===========================================================================

api_post "/porta/community/service/disable" '{"id": "non-existent-service-id"}'
assert_api_error "Disable non-existent service returns error"

# ===========================================================================
print_section "4. Disable Service (if available)"
# ===========================================================================

if [ -n "$TEST_SERVICE_ID" ] && [ "$TEST_SERVICE_ID" != "null" ]; then
    api_post "/porta/community/service/disable" "{\"id\": \"$TEST_SERVICE_ID\"}"
    assert_api_success "Disable service returns success"
    
    # Verify announced status
    api_get "/porta/community/service/list"
    ANNOUNCED=$(echo "$LAST_RESPONSE" | jq -r ".data[] | select(.uuid == \"$TEST_SERVICE_ID\") | .announced")
    
    if [ "$ANNOUNCED" = "false" ]; then
        print_pass "Service announced status is false"
    else
        print_fail "Service should be disabled, got announced=$ANNOUNCED"
    fi
else
    print_pass "No services available to test disable (expected)"
fi

# ===========================================================================
print_section "5. Announce Service (if available)"
# ===========================================================================

if [ -n "$TEST_SERVICE_ID" ] && [ "$TEST_SERVICE_ID" != "null" ]; then
    api_post "/porta/community/service/announce" "{\"id\": \"$TEST_SERVICE_ID\"}"
    assert_api_success "Announce service returns success"
    
    # Verify announced status
    api_get "/porta/community/service/list"
    ANNOUNCED=$(echo "$LAST_RESPONSE" | jq -r ".data[] | select(.uuid == \"$TEST_SERVICE_ID\") | .announced")
    
    if [ "$ANNOUNCED" = "true" ]; then
        print_pass "Service announced status is true"
    else
        print_fail "Service should be announced, got announced=$ANNOUNCED"
    fi
else
    print_pass "No services available to test announce (expected)"
fi

# ===========================================================================
print_section "6. Verify Service Protocol Values"
# ===========================================================================

api_get "/porta/community/service/list"
assert_api_success "GET services for protocol check"

# Valid protocols per requirement 3.1.4: http, https, tcp, udp, omega, ws
PROTOCOLS=$(echo "$LAST_RESPONSE" | jq -r '.data[].protocol' 2>/dev/null || echo "")
VALID_PROTOCOLS="http https tcp udp omega ws HTTP HTTPS TCP UDP WS"

if [ -n "$PROTOCOLS" ]; then
    for protocol in $PROTOCOLS; do
        if echo "$VALID_PROTOCOLS" | grep -qi "$protocol"; then
            print_pass "Service protocol '$protocol' is valid"
        else
            print_fail "Service protocol '$protocol' is invalid"
        fi
    done
else
    print_pass "No service protocols to validate (expected)"
fi

# ===========================================================================
print_section "7. Verify Online Status Type"
# ===========================================================================

if [ "$SERVICES_COUNT" -gt 0 ]; then
    ONLINE_TYPE=$(echo "$LAST_RESPONSE" | jq -r '.data[0].online | type')
    if [ "$ONLINE_TYPE" = "boolean" ]; then
        print_pass "Service online field is boolean"
    else
        print_fail "Service online should be boolean, got $ONLINE_TYPE"
    fi
    
    ANNOUNCED_TYPE=$(echo "$LAST_RESPONSE" | jq -r '.data[0].announced | type')
    if [ "$ANNOUNCED_TYPE" = "boolean" ]; then
        print_pass "Service announced field is boolean"
    else
        print_fail "Service announced should be boolean, got $ANNOUNCED_TYPE"
    fi
else
    print_pass "No services to validate types (expected)"
fi

# ===========================================================================
print_summary
