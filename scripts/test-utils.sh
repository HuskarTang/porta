#!/bin/bash
# ===========================================================================
# Porta Test Utilities
# ===========================================================================
# Common functions for test scripts
# ===========================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Default API base URL
API_BASE="${API_BASE:-http://localhost:8090}"

# ===========================================================================
# Output Functions
# ===========================================================================

print_test_header() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}TEST: $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_section() {
    echo ""
    echo -e "${BLUE}▶ $1${NC}"
}

print_request() {
    echo -e "${YELLOW}→ $1${NC}"
}

print_response() {
    echo -e "${NC}← $1${NC}"
}

print_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

print_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

print_summary() {
    echo ""
    echo -e "${BLUE}==========================================================================${NC}"
    echo -e "${BLUE}TEST SUMMARY${NC}"
    echo -e "${BLUE}==========================================================================${NC}"
    echo -e "Total:  $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo ""
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

# ===========================================================================
# HTTP Request Functions
# ===========================================================================

# Make a GET request and return response
# Usage: api_get "/path"
api_get() {
    local path="$1"
    local url="${API_BASE}${path}"
    
    print_request "GET $url"
    
    local response
    local http_code
    
    response=$(curl -s -w "\n%{http_code}" "$url" -H "Content-Type: application/json")
    http_code=$(echo "$response" | tail -n1)
    response=$(echo "$response" | sed '$d')
    
    echo "$response"
    print_response "HTTP $http_code"
    
    LAST_HTTP_CODE="$http_code"
    LAST_RESPONSE="$response"
}

# Make a POST request and return response
# Usage: api_post "/path" '{"json": "data"}'
api_post() {
    local path="$1"
    local data="$2"
    local url="${API_BASE}${path}"
    
    print_request "POST $url"
    echo "   Body: $data"
    
    local response
    local http_code
    
    response=$(curl -s -w "\n%{http_code}" -X POST "$url" \
        -H "Content-Type: application/json" \
        -d "$data")
    http_code=$(echo "$response" | tail -n1)
    response=$(echo "$response" | sed '$d')
    
    echo "$response"
    print_response "HTTP $http_code"
    
    LAST_HTTP_CODE="$http_code"
    LAST_RESPONSE="$response"
}

# Make a DELETE request
# Usage: api_delete "/path" '{"json": "data"}'
api_delete() {
    local path="$1"
    local data="$2"
    local url="${API_BASE}${path}"
    
    print_request "DELETE $url"
    
    local response
    local http_code
    
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$url" \
        -H "Content-Type: application/json" \
        -d "$data")
    http_code=$(echo "$response" | tail -n1)
    response=$(echo "$response" | sed '$d')
    
    echo "$response"
    print_response "HTTP $http_code"
    
    LAST_HTTP_CODE="$http_code"
    LAST_RESPONSE="$response"
}

# ===========================================================================
# Assertion Functions
# ===========================================================================

# Assert HTTP status code
# Usage: assert_status 200
assert_status() {
    local expected="$1"
    local description="${2:-HTTP status is $expected}"
    
    if [ "$LAST_HTTP_CODE" = "$expected" ]; then
        print_pass "$description"
        return 0
    else
        print_fail "$description (expected $expected, got $LAST_HTTP_CODE)"
        return 1
    fi
}

# Assert response contains string
# Usage: assert_contains "expected_string" "description"
assert_contains() {
    local expected="$1"
    local description="${2:-Response contains '$expected'}"
    
    if echo "$LAST_RESPONSE" | grep -q "$expected"; then
        print_pass "$description"
        return 0
    else
        print_fail "$description"
        return 1
    fi
}

# Assert response does not contain string
# Usage: assert_not_contains "string" "description"
assert_not_contains() {
    local expected="$1"
    local description="${2:-Response does not contain '$expected'}"
    
    if echo "$LAST_RESPONSE" | grep -q "$expected"; then
        print_fail "$description"
        return 1
    else
        print_pass "$description"
        return 0
    fi
}

# Assert JSON field equals value
# Usage: assert_json_eq ".data.name" "expected_value" "description"
assert_json_eq() {
    local jq_path="$1"
    local expected="$2"
    local description="${3:-$jq_path equals $expected}"
    
    local actual
    actual=$(echo "$LAST_RESPONSE" | jq -r "$jq_path" 2>/dev/null)
    
    if [ "$actual" = "$expected" ]; then
        print_pass "$description"
        return 0
    else
        print_fail "$description (expected '$expected', got '$actual')"
        return 1
    fi
}

# Assert JSON field is not null
# Usage: assert_json_not_null ".data.uuid" "description"
assert_json_not_null() {
    local jq_path="$1"
    local description="${2:-$jq_path is not null}"
    
    local actual
    actual=$(echo "$LAST_RESPONSE" | jq -r "$jq_path" 2>/dev/null)
    
    if [ -n "$actual" ] && [ "$actual" != "null" ]; then
        print_pass "$description"
        return 0
    else
        print_fail "$description (got null or empty)"
        return 1
    fi
}

# Assert JSON array length
# Usage: assert_json_length ".data" 3 "description"
assert_json_length() {
    local jq_path="$1"
    local expected="$2"
    local description="${3:-$jq_path has $expected items}"
    
    local actual
    actual=$(echo "$LAST_RESPONSE" | jq "$jq_path | length" 2>/dev/null)
    
    if [ "$actual" = "$expected" ]; then
        print_pass "$description"
        return 0
    else
        print_fail "$description (expected $expected, got $actual)"
        return 1
    fi
}

# Assert API success (code = 0)
# Usage: assert_api_success "description"
assert_api_success() {
    local description="${1:-API returns success}"
    assert_json_eq ".code" "0" "$description"
}

# Assert API error (code != 0)
# Usage: assert_api_error "description"
assert_api_error() {
    local description="${1:-API returns error}"
    local code
    code=$(echo "$LAST_RESPONSE" | jq -r ".code" 2>/dev/null)
    
    if [ "$code" != "0" ] && [ -n "$code" ]; then
        print_pass "$description"
        return 0
    else
        print_fail "$description (expected error, got code=$code)"
        return 1
    fi
}

# ===========================================================================
# Utility Functions
# ===========================================================================

# Wait for API to be ready
wait_for_api() {
    local url="${1:-$API_BASE}"
    local max_attempts="${2:-30}"
    local attempt=0
    
    echo "Waiting for API at $url..."
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$url/porta/node/info" > /dev/null 2>&1; then
            echo "API is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 0.5
    done
    
    echo "API not ready after $max_attempts attempts"
    return 1
}

# Generate random string
random_string() {
    local length="${1:-8}"
    cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w "$length" | head -n 1
}

# Extract value from JSON response
# Usage: value=$(json_get ".data.id")
json_get() {
    echo "$LAST_RESPONSE" | jq -r "$1" 2>/dev/null
}
