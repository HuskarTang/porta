#!/bin/bash
# ===========================================================================
# Test 01: Node Configuration (Requirement 3.1.1)
# ===========================================================================
# Tests node configuration including:
# - Get node info
# - Update node name
# - Update TCP/QUIC settings
# - Generate/Import key
# ===========================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test-utils.sh"

API_BASE="${API_BASE:-http://localhost:8090}"

print_test_header "01 - Node Configuration (Requirement 3.1.1)"

# ===========================================================================
print_section "1. Get Node Info"
# ===========================================================================

api_get "/porta/node/info"
assert_api_success "GET node info returns success"
assert_json_not_null ".data.name" "Node has name"
assert_json_not_null ".data.node_id" "Node has node_id"
assert_json_not_null ".data.uuid" "Node has uuid"
assert_json_not_null ".data.key_path" "Node has key_path"

# ===========================================================================
print_section "2. Update Node Name"
# ===========================================================================

api_post "/porta/node/config" '{"name": "测试节点"}'
assert_api_success "Update node name returns success"
assert_json_eq ".data.name" "测试节点" "Node name updated correctly"

# Verify the change persisted
api_get "/porta/node/info"
assert_json_eq ".data.name" "测试节点" "Node name persisted"

# ===========================================================================
print_section "3. Update TCP Settings"
# ===========================================================================

api_post "/porta/node/config" '{"tcp_listen_enable": true, "tcp_listen_port": 9001}'
assert_api_success "Update TCP settings returns success"
assert_json_eq ".data.tcp_listen_enable" "true" "TCP enabled"
assert_json_eq ".data.tcp_listen_port" "9001" "TCP port updated"

# ===========================================================================
print_section "4. Update QUIC Settings"
# ===========================================================================

api_post "/porta/node/config" '{"quci_listen_enable": true, "quci_listen_port": 9002}'
assert_api_success "Update QUIC settings returns success"
assert_json_eq ".data.quci_listen_enable" "true" "QUIC enabled"
assert_json_eq ".data.quci_listen_port" "9002" "QUIC port updated"

# ===========================================================================
print_section "5. Update External Address"
# ===========================================================================

api_post "/porta/node/config" '{"external_addr": ["192.168.1.100:9001", "10.0.0.1:9001"]}'
assert_api_success "Update external address returns success"
assert_json_length ".data.external_addr" "2" "External addresses count is 2"

# ===========================================================================
print_section "6. Update mDNS and DHT Settings"
# ===========================================================================

api_post "/porta/node/config" '{"mdns_enable": false, "dht_enable": false}'
assert_api_success "Update discovery settings returns success"
assert_json_eq ".data.mdns_enable" "false" "mDNS disabled"
assert_json_eq ".data.dht_enable" "false" "DHT disabled"

# Re-enable
api_post "/porta/node/config" '{"mdns_enable": true, "dht_enable": true}'
assert_api_success "Re-enable discovery settings"

# ===========================================================================
print_section "7. Generate New Key"
# ===========================================================================

api_post "/porta/node/key/generate" '{}'
assert_api_success "Generate key returns success"
assert_contains "porta-" "Key path contains porta- prefix"
assert_contains ".key" "Key path has .key extension"

# ===========================================================================
print_section "8. Import Key - Empty Path (Should Fail)"
# ===========================================================================

api_post "/porta/node/key/import" '{"key_path": ""}'
assert_api_error "Import with empty path returns error"

# ===========================================================================
print_section "9. Import Key - Valid Path"
# ===========================================================================

api_post "/porta/node/key/import" '{"key_path": "/tmp/test-key.key"}'
assert_api_success "Import with valid path returns success"
assert_json_eq ".data.key_path" "/tmp/test-key.key" "Key path updated"

# ===========================================================================
print_section "10. Verify All Fields Present"
# ===========================================================================

api_get "/porta/node/info"
assert_api_success "Final node info check"

# Check all required fields from requirement 3.1.1
REQUIRED_FIELDS=("name" "node_id" "uuid" "key_path" "tcp_listen_enable" "tcp_listen_port" "quci_listen_enable" "quci_listen_port" "external_addr" "mdns_enable" "dht_enable")

for field in "${REQUIRED_FIELDS[@]}"; do
    if echo "$LAST_RESPONSE" | jq -e ".data.$field" > /dev/null 2>&1; then
        print_pass "Field '$field' is present"
    else
        print_fail "Field '$field' is missing"
    fi
done

# ===========================================================================
print_summary
