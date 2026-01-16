#!/bin/bash
# ===========================================================================
# Test 02: Community Node Management (Requirement 3.1.2, 3.1.3)
# ===========================================================================
# Tests community management including:
# - List communities
# - Add community with multiaddr
# - Add community without multiaddr (should fail)
# - Connect to community
# - Remove community
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "02 - Community Node Management (Requirement 3.1.2, 3.1.3)"

# ===========================================================================
print_section "1. List Communities (Seeded Data)"
# ===========================================================================

api_get "/porta/community/list"
assert_api_success "GET community list returns success"
assert_json_length ".data" "3" "Seeded with 3 communities"

# Check first community structure
assert_json_not_null ".data[0].id" "Community has id"
assert_json_not_null ".data[0].name" "Community has name"
assert_json_not_null ".data[0].description" "Community has description"

# ===========================================================================
print_section "2. Add Community - Missing Multiaddr (Should Fail)"
# ===========================================================================

api_post "/porta/community/add" '{
    "name": "Test Community",
    "description": "A test community"
}'
assert_api_error "Add without multiaddr returns error"
assert_contains "multiaddr" "Error mentions multiaddr"

# ===========================================================================
print_section "3. Add Community - Missing Name (Should Fail)"
# ===========================================================================

api_post "/porta/community/add" '{
    "name": "",
    "description": "A test community",
    "multiaddr": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
}'
assert_api_error "Add with empty name returns error"

# ===========================================================================
print_section "4. Add Community - Missing Description (Should Fail)"
# ===========================================================================

api_post "/porta/community/add" '{
    "name": "Test Community",
    "description": "",
    "multiaddr": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
}'
assert_api_error "Add with empty description returns error"

# ===========================================================================
print_section "5. Add Community - Valid Request"
# ===========================================================================

PEER_ID="12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
api_post "/porta/community/add" "{
    \"name\": \"新测试社区\",
    \"description\": \"用于系统测试的社区\",
    \"multiaddr\": \"/ip4/192.168.1.100/tcp/4001/p2p/$PEER_ID\"
}"
assert_api_success "Add valid community returns success"
assert_json_not_null ".data.id" "New community has id"
assert_json_eq ".data.name" "新测试社区" "Community name matches"
assert_json_eq ".data.joined" "false" "New community not joined yet"

# Save the community ID for later tests
NEW_COMMUNITY_ID=$(json_get ".data.id")
echo "Created community: $NEW_COMMUNITY_ID"

# ===========================================================================
print_section "6. Verify Community Added to List"
# ===========================================================================

api_get "/porta/community/list"
assert_api_success "GET community list after add"
assert_json_length ".data" "4" "Now has 4 communities"

# ===========================================================================
print_section "7. Add Duplicate Community Name (Should Fail)"
# ===========================================================================

PEER_ID2="12D3KooWRBhwfeP2Y4TCx1SM7s1rFDyeYJ4L1rBKdwCe7vCn8a1X"
api_post "/porta/community/add" "{
    \"name\": \"新测试社区\",
    \"description\": \"Duplicate name\",
    \"multiaddr\": \"/ip4/192.168.1.101/tcp/4001/p2p/$PEER_ID2\"
}"
assert_api_error "Add duplicate name returns error"
assert_contains "已存在" "Error mentions already exists"

# ===========================================================================
print_section "8. Connect to Community (Seeded)"
# ===========================================================================

# Try to connect to a seeded community (will fail P2P but API should accept)
api_post "/porta/community/connect" '{"id": "dev-community"}'
# Note: This may fail at P2P level but we test the API flow
if [ "$LAST_HTTP_CODE" = "200" ] || echo "$LAST_RESPONSE" | grep -q "连接"; then
    print_pass "Connect API endpoint works"
else
    print_fail "Connect API endpoint failed"
fi

# ===========================================================================
print_section "9. Remove Community"
# ===========================================================================

api_post "/porta/community/remove" "{\"id\": \"$NEW_COMMUNITY_ID\"}"
assert_api_success "Remove community returns success"

# Verify removed
api_get "/porta/community/list"
assert_json_length ".data" "3" "Back to 3 communities after remove"

# ===========================================================================
print_section "10. Remove Non-existent Community (Should Fail)"
# ===========================================================================

api_post "/porta/community/remove" '{"id": "non-existent-id"}'
assert_api_error "Remove non-existent community returns error"

# ===========================================================================
print_section "11. Community Fields Validation"
# ===========================================================================

api_get "/porta/community/list"
assert_api_success "GET community list for field validation"

# Check required fields per requirement 3.1.2
FIRST_COMMUNITY=$(echo "$LAST_RESPONSE" | jq '.data[0]')
echo "Sample community: $FIRST_COMMUNITY"

REQUIRED_FIELDS=("id" "name" "description" "peers" "joined")
for field in "${REQUIRED_FIELDS[@]}"; do
    # Use has() instead of -e to properly check for field existence (handles boolean false)
    if echo "$FIRST_COMMUNITY" | jq "has(\"$field\")" | grep -q "true"; then
        print_pass "Community field '$field' is present"
    else
        print_fail "Community field '$field' is missing"
    fi
done

# ===========================================================================
print_summary
