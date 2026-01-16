#!/bin/bash
# ===========================================================================
# Porta Test Runner
# ===========================================================================
# Execute all test scripts with comprehensive reporting
# ===========================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Results tracking
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_SCRIPTS=0
FAILED_SCRIPTS=()
SCRIPT_RESULTS=()

# Timing
START_TIME=$(date +%s)

# ===========================================================================
# Functions
# ===========================================================================

print_banner() {
    echo ""
    echo -e "${MAGENTA}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║                                                                       ║${NC}"
    echo -e "${MAGENTA}║              ██████╗  ██████╗ ██████╗ ████████╗ █████╗                ║${NC}"
    echo -e "${MAGENTA}║              ██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝██╔══██╗               ║${NC}"
    echo -e "${MAGENTA}║              ██████╔╝██║   ██║██████╔╝   ██║   ███████║               ║${NC}"
    echo -e "${MAGENTA}║              ██╔═══╝ ██║   ██║██╔══██╗   ██║   ██╔══██║               ║${NC}"
    echo -e "${MAGENTA}║              ██║     ╚██████╔╝██║  ██║   ██║   ██║  ██║               ║${NC}"
    echo -e "${MAGENTA}║              ╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝               ║${NC}"
    echo -e "${MAGENTA}║                                                                       ║${NC}"
    echo -e "${MAGENTA}║                     SYSTEM TEST RUNNER                                ║${NC}"
    echo -e "${MAGENTA}║                                                                       ║${NC}"
    echo -e "${MAGENTA}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_separator() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════${NC}"
}

run_test_script() {
    local script="$1"
    local name=$(basename "$script" .sh)
    local start_time=$(date +%s)
    
    echo ""
    print_separator
    echo -e "${CYAN}Running: $name${NC}"
    print_separator
    
    TOTAL_SCRIPTS=$((TOTAL_SCRIPTS + 1))
    
    # Run the test and capture output
    local output
    local exit_code
    
    if output=$("$script" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo "$output"
    
    # Parse results from output
    local passed
    local failed
    passed=$(echo "$output" | grep -c "✓ PASS" 2>/dev/null) || passed=0
    failed=$(echo "$output" | grep -c "✗ FAIL" 2>/dev/null) || failed=0
    
    # Ensure numeric values
    passed=$((passed + 0))
    failed=$((failed + 0))
    
    TOTAL_PASSED=$((TOTAL_PASSED + passed))
    TOTAL_FAILED=$((TOTAL_FAILED + failed))
    
    if [ "$exit_code" -eq 0 ] && [ "$failed" -eq 0 ]; then
        SCRIPT_RESULTS+=("${GREEN}✓${NC} $name: $passed passed, $failed failed (${duration}s)")
    else
        SCRIPT_RESULTS+=("${RED}✗${NC} $name: $passed passed, $failed failed (${duration}s)")
        FAILED_SCRIPTS+=("$name")
    fi
}

run_api_tests() {
    echo ""
    echo -e "${YELLOW}▶ Running API Tests...${NC}"
    echo ""
    
    for script in "$SCRIPT_DIR/tests/"*.sh; do
        if [ -f "$script" ]; then
            run_test_script "$script"
        fi
    done
}

run_e2e_tests() {
    echo ""
    echo -e "${YELLOW}▶ Running End-to-End Tests...${NC}"
    echo ""
    
    if [ -f "$SCRIPT_DIR/e2e-cascade-test.sh" ]; then
        run_test_script "$SCRIPT_DIR/e2e-cascade-test.sh"
    fi
}

check_prerequisites() {
    echo -e "${YELLOW}Checking prerequisites...${NC}"
    echo ""
    
    # Check jq is installed
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}ERROR: jq is not installed${NC}"
        echo "Please install jq: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi
    print_pass "jq is installed"
    
    # Check curl is installed
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}ERROR: curl is not installed${NC}"
        exit 1
    fi
    print_pass "curl is installed"
    
    # Check if API is reachable
    API_URL="${API_BASE:-http://localhost:8090}"
    if curl -s --connect-timeout 2 "$API_URL/porta/node/info" > /dev/null 2>&1; then
        print_pass "API is reachable at $API_URL"
    else
        echo -e "${RED}ERROR: API not reachable at $API_URL${NC}"
        echo ""
        echo "Please start the backend first:"
        echo "  cd $PROJECT_ROOT/backend && cargo run"
        echo ""
        echo "Or start the test environment:"
        echo "  $SCRIPT_DIR/test-setup.sh start-single"
        echo ""
        exit 1
    fi
    
    echo ""
}

print_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

print_results() {
    local end_time=$(date +%s)
    local total_duration=$((end_time - START_TIME))
    
    echo ""
    echo ""
    echo -e "${MAGENTA}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║                         TEST RESULTS                                  ║${NC}"
    echo -e "${MAGENTA}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    echo -e "${BLUE}Script Results:${NC}"
    echo ""
    for result in "${SCRIPT_RESULTS[@]}"; do
        echo -e "  $result"
    done
    
    echo ""
    print_separator
    echo ""
    
    echo -e "${BLUE}Summary:${NC}"
    echo ""
    echo -e "  Total Scripts: $TOTAL_SCRIPTS"
    echo -e "  Total Tests:   $((TOTAL_PASSED + TOTAL_FAILED))"
    echo -e "  ${GREEN}Passed:${NC}        $TOTAL_PASSED"
    echo -e "  ${RED}Failed:${NC}        $TOTAL_FAILED"
    echo -e "  Duration:      ${total_duration}s"
    
    if [ ${#FAILED_SCRIPTS[@]} -gt 0 ]; then
        echo ""
        echo -e "${RED}Failed Scripts:${NC}"
        for script in "${FAILED_SCRIPTS[@]}"; do
            echo -e "  - $script"
        done
    fi
    
    echo ""
    print_separator
    echo ""
    
    if [ $TOTAL_FAILED -eq 0 ]; then
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                    ALL TESTS PASSED! 🎉                               ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
        return 0
    else
        echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║                    SOME TESTS FAILED ❌                                ║${NC}"
        echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
        return 1
    fi
}

generate_report() {
    local report_file="$PROJECT_ROOT/test-report-$(date +%Y%m%d-%H%M%S).txt"
    
    echo "Generating test report: $report_file"
    
    {
        echo "Porta System Test Report"
        echo "========================"
        echo "Date: $(date)"
        echo ""
        echo "Summary"
        echo "-------"
        echo "Total Scripts: $TOTAL_SCRIPTS"
        echo "Total Tests: $((TOTAL_PASSED + TOTAL_FAILED))"
        echo "Passed: $TOTAL_PASSED"
        echo "Failed: $TOTAL_FAILED"
        echo ""
        echo "Script Results"
        echo "--------------"
        for result in "${SCRIPT_RESULTS[@]}"; do
            echo "$result" | sed 's/\x1b\[[0-9;]*m//g'
        done
        echo ""
        if [ ${#FAILED_SCRIPTS[@]} -gt 0 ]; then
            echo "Failed Scripts"
            echo "--------------"
            for script in "${FAILED_SCRIPTS[@]}"; do
                echo "- $script"
            done
        fi
    } > "$report_file"
    
    echo "Report saved to: $report_file"
}

show_usage() {
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all       Run all tests (default)"
    echo "  api       Run API tests only"
    echo "  e2e       Run end-to-end tests only"
    echo "  single    Run a single test script"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -r, --report   Generate a test report file"
    echo "  -v, --verbose  Show verbose output"
    echo ""
    echo "Environment Variables:"
    echo "  API_BASE       Base URL for API tests (default: http://localhost:8090)"
    echo "  COMMUNITY_URL  URL for community node (default: http://localhost:8091)"
    echo "  EDGE1_URL      URL for edge node 1 (default: http://localhost:8090)"
    echo "  EDGE2_URL      URL for edge node 2 (default: http://localhost:8092)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 api                # Run API tests only"
    echo "  $0 e2e                # Run E2E tests only"
    echo "  $0 -r all             # Run all tests and generate report"
    echo "  $0 single 01-node     # Run specific test"
}

# ===========================================================================
# Main
# ===========================================================================

GENERATE_REPORT=false
COMMAND="all"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -r|--report)
            GENERATE_REPORT=true
            shift
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        all|api|e2e|single)
            COMMAND=$1
            shift
            ;;
        *)
            SINGLE_TEST="$1"
            shift
            ;;
    esac
done

print_banner
check_prerequisites

case $COMMAND in
    all)
        run_api_tests
        # Skip E2E tests in all mode unless multi-node is running
        if curl -s "http://localhost:8091/porta/node/info" > /dev/null 2>&1; then
            run_e2e_tests
        else
            echo ""
            echo -e "${YELLOW}Skipping E2E tests (multi-node environment not detected)${NC}"
            echo "To run E2E tests, start all nodes: $SCRIPT_DIR/test-setup.sh start"
        fi
        ;;
    api)
        run_api_tests
        ;;
    e2e)
        run_e2e_tests
        ;;
    single)
        if [ -n "$SINGLE_TEST" ]; then
            # Find the test script
            TEST_SCRIPT=$(find "$SCRIPT_DIR/tests" -name "*$SINGLE_TEST*.sh" | head -1)
            if [ -n "$TEST_SCRIPT" ]; then
                run_test_script "$TEST_SCRIPT"
            else
                echo -e "${RED}Test script not found: $SINGLE_TEST${NC}"
                exit 1
            fi
        else
            echo -e "${RED}Please specify a test name${NC}"
            exit 1
        fi
        ;;
esac

print_results
FINAL_RESULT=$?

if [ "$GENERATE_REPORT" = true ]; then
    generate_report
fi

exit $FINAL_RESULT
