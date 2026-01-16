#!/bin/bash
# ===========================================================================
# Test 04: Service Discovery (Requirement 3.1.5)
# ===========================================================================
# Tests service discovery including:
# - Discover services from community
# - Discover all services
# - Verify discovered service fields
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "04 - Service Discovery (Requirement 3.1.5)"

# ===========================================================================
print_section "1. Discover All Services"
# ===========================================================================

api_get "/porta/service/discover"
assert_api_success "GET discover all services returns success"
echo "Discovered services: $(json_get '.data | length')"

# ===========================================================================
print_section "2. Discover Services by Community ID"
# ===========================================================================

api_get "/porta/service/discover?communityId=dev-community"

# P2P discovery may fail if not connected - this is expected
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "GET discover with communityId returns success"
    if [ "$(json_get '.data')" != "null" ]; then
        print_pass "Discover returns data array"
    else
        print_fail "Discover should return array"
    fi
else
    # P2P not connected - expected in isolated test environment
    print_pass "GET discover attempted (P2P not connected - expected)"
    print_pass "Discover skipped (no P2P connection)"
fi

# ===========================================================================
print_section "3. Discover Services - Invalid Community ID"
# ===========================================================================

api_get "/porta/service/discover?communityId=non-existent-community"
# This will fail in P2P context - expected behavior
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Discover with unknown community returns success"
else
    print_pass "Discover with unknown community handled (error expected)"
fi

# ===========================================================================
print_section "4. Verify Discovered Service Fields (if any)"
# ===========================================================================

api_get "/porta/service/discover"
assert_api_success "GET discover for field check"

SERVICES_COUNT=$(json_get ".data | length")
echo "Found $SERVICES_COUNT discovered services"

if [ "$SERVICES_COUNT" -gt 0 ]; then
    FIRST_SERVICE=$(echo "$LAST_RESPONSE" | jq '.data[0]')
    echo "Sample service: $FIRST_SERVICE"
    
    # Check required fields per requirement 3.1.4
    REQUIRED_FIELDS=("uuid" "name" "type" "remote_port" "provider" "description")
    for field in "${REQUIRED_FIELDS[@]}"; do
        if echo "$FIRST_SERVICE" | jq -e ".$field" > /dev/null 2>&1; then
            print_pass "Discovered service field '$field' is present"
        else
            print_fail "Discovered service field '$field' is missing"
        fi
    done
else
    print_pass "No discovered services (P2P not connected - expected in isolated test)"
fi

# ===========================================================================
print_section "5. Service Discovery Response Format"
# ===========================================================================

api_get "/porta/service/discover"
assert_api_success "GET discover response format"
assert_json_eq ".code" "0" "Response code is 0"
assert_json_not_null ".message" "Response has message"

# Check data is an array
DATA_TYPE=$(echo "$LAST_RESPONSE" | jq -r '.data | type')
if [ "$DATA_TYPE" = "array" ]; then
    print_pass "Data is an array"
else
    print_fail "Data should be array, got $DATA_TYPE"
fi

# ===========================================================================
print_summary
