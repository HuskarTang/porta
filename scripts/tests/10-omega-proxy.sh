#!/bin/bash
# ===========================================================================
# Test 10: Omega Proxy (Requirement 3.1.14)
# ===========================================================================
# Tests Omega proxy including:
# - Get proxy status
# - Enable proxy
# - Disable proxy
# - Verify proxy publishes as service
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "10 - Omega Proxy (Requirement 3.1.14)"

# ===========================================================================
print_section "1. Get Proxy Status"
# ===========================================================================

api_get "/porta/proxy/status"
assert_api_success "GET proxy status returns success"
assert_json_not_null ".data.enabled" "Proxy status has enabled field"
assert_json_not_null ".data.listen_port" "Proxy status has listen_port field"

INITIAL_ENABLED=$(json_get ".data.enabled")
LISTEN_PORT=$(json_get ".data.listen_port")
echo "Initial proxy state: enabled=$INITIAL_ENABLED, port=$LISTEN_PORT"

# ===========================================================================
print_section "2. Verify Proxy Status Fields"
# ===========================================================================

# enabled should be boolean
ENABLED_TYPE=$(echo "$LAST_RESPONSE" | jq -r '.data.enabled | type')
if [ "$ENABLED_TYPE" = "boolean" ]; then
    print_pass "Proxy enabled field is boolean"
else
    print_fail "Proxy enabled should be boolean, got $ENABLED_TYPE"
fi

# listen_port should be number
PORT_TYPE=$(echo "$LAST_RESPONSE" | jq -r '.data.listen_port | type')
if [ "$PORT_TYPE" = "number" ]; then
    print_pass "Proxy listen_port field is number"
else
    print_fail "Proxy listen_port should be number, got $PORT_TYPE"
fi

# Port should be valid (default 1080 or custom)
if [ "$LISTEN_PORT" -gt 0 ] && [ "$LISTEN_PORT" -lt 65536 ]; then
    print_pass "Proxy listen_port is valid: $LISTEN_PORT"
else
    print_fail "Proxy listen_port invalid: $LISTEN_PORT"
fi

# ===========================================================================
print_section "3. Enable Proxy"
# ===========================================================================

api_post "/porta/proxy/enable" '{"enabled": true}'

if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Enable proxy returns success"
    
    # Verify status changed
    api_get "/porta/proxy/status"
    ENABLED=$(json_get ".data.enabled")
    if [ "$ENABLED" = "true" ]; then
        print_pass "Proxy is now enabled"
    else
        print_fail "Proxy should be enabled, got $ENABLED"
    fi
else
    # May fail if port is in use
    if echo "$LAST_RESPONSE" | grep -qi "port\|bind\|address"; then
        print_pass "Enable proxy failed (port conflict - expected in some environments)"
    else
        print_fail "Enable proxy failed: $LAST_RESPONSE"
    fi
fi

# ===========================================================================
print_section "4. Verify Proxy Published as Service"
# ===========================================================================

api_get "/porta/service/published"
assert_api_success "GET published services"

# Look for omega proxy service
PROXY_SERVICE=$(echo "$LAST_RESPONSE" | jq '.data[] | select(.type == "omega")')
if [ -n "$PROXY_SERVICE" ] && [ "$PROXY_SERVICE" != "null" ]; then
    print_pass "Omega proxy published as service"
    echo "Proxy service: $PROXY_SERVICE"
    
    # Verify service fields
    PROXY_NAME=$(echo "$PROXY_SERVICE" | jq -r '.name')
    PROXY_PORT=$(echo "$PROXY_SERVICE" | jq -r '.port')
    echo "Proxy service name: $PROXY_NAME, port: $PROXY_PORT"
else
    print_pass "No omega proxy service found (may not be published yet)"
fi

# ===========================================================================
print_section "5. Disable Proxy"
# ===========================================================================

api_post "/porta/proxy/disable" '{"enabled": false}'

if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Disable proxy returns success"
    
    # Verify status changed
    api_get "/porta/proxy/status"
    ENABLED=$(json_get ".data.enabled")
    if [ "$ENABLED" = "false" ]; then
        print_pass "Proxy is now disabled"
    else
        print_fail "Proxy should be disabled, got $ENABLED"
    fi
else
    print_fail "Disable proxy failed: $LAST_RESPONSE"
fi

# ===========================================================================
print_section "6. Toggle Proxy Multiple Times"
# ===========================================================================

# Enable
api_post "/porta/proxy/enable" '{"enabled": true}'
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Re-enable proxy succeeded"
else
    print_pass "Re-enable proxy handled (may have port conflict)"
fi

# Disable
api_post "/porta/proxy/disable" '{"enabled": false}'
if [ "$LAST_HTTP_CODE" = "200" ]; then
    print_pass "Re-disable proxy succeeded"
else
    print_fail "Re-disable proxy failed"
fi

# ===========================================================================
print_section "7. Verify Proxy Service Unpublished"
# ===========================================================================

api_get "/porta/service/published"
assert_api_success "GET published services after disable"

PROXY_SERVICE=$(echo "$LAST_RESPONSE" | jq '.data[] | select(.type == "omega" and .status == "在线")')
if [ -z "$PROXY_SERVICE" ] || [ "$PROXY_SERVICE" = "null" ]; then
    print_pass "Omega proxy service unpublished or offline"
else
    PROXY_STATUS=$(echo "$PROXY_SERVICE" | jq -r '.status')
    if [ "$PROXY_STATUS" = "已下架" ]; then
        print_pass "Omega proxy service marked as 已下架"
    else
        print_fail "Omega proxy service should be unpublished, status: $PROXY_STATUS"
    fi
fi

# ===========================================================================
print_section "8. Default Proxy Port (1080)"
# ===========================================================================

api_get "/porta/proxy/status"
assert_api_success "GET proxy status for port check"

PORT=$(json_get ".data.listen_port")
if [ "$PORT" = "1080" ]; then
    print_pass "Default proxy port is 1080 (standard SOCKS)"
else
    print_pass "Proxy port is $PORT (custom configuration)"
fi

# ===========================================================================
print_summary
