#!/bin/bash

# Office Hours Testing Script for aipriceaction-proxy
# Tests various office hours scenarios and validates behavior

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_PORT=8899
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ SUCCESS${NC}: $1"
    ((PASSED_TESTS++))
}

print_failure() {
    echo -e "${RED}❌ FAILURE${NC}: $1"
    ((FAILED_TESTS++))
}

print_info() {
    echo -e "${YELLOW}ℹ️  INFO${NC}: $1"
}

print_test() {
    echo -e "\n${BLUE}🧪 TEST $((TOTAL_TESTS + 1)):${NC} $1"
    ((TOTAL_TESTS++))
}

# Start server and run a test
run_test() {
    local test_name="$1"
    local debug_time="$2"
    local enable_office_hours="$3"
    local environment="$4"
    local expected_office_hours="$5"
    local expected_interval="$6"
    local expected_debug_override="$7"
    
    print_test "$test_name"
    
    # Set environment variables
    export PORT="$TEST_PORT"
    export RUST_LOG="warn"  # Show warnings but reduce info noise
    
    if [ -n "$debug_time" ]; then
        export DEBUG_SYSTEM_TIME="$debug_time"
    else
        unset DEBUG_SYSTEM_TIME
    fi
    
    if [ -n "$enable_office_hours" ]; then
        export ENABLE_OFFICE_HOURS="$enable_office_hours"
    else
        unset ENABLE_OFFICE_HOURS
    fi
    
    if [ -n "$environment" ]; then
        export ENVIRONMENT="$environment"
    else
        export ENVIRONMENT="development"
    fi
    
    # Start server in background
    cargo run > /tmp/test_server.log 2>&1 &
    local server_pid=$!
    
    # Wait for server to be ready
    local ready=false
    for attempt in {1..15}; do
        if curl -s "http://localhost:$TEST_PORT/health" > /dev/null 2>&1; then
            ready=true
            break
        fi
        sleep 0.5
    done
    
    if [ "$ready" = false ]; then
        print_failure "Server failed to start"
        kill $server_pid 2>/dev/null || true
        wait $server_pid 2>/dev/null || true
        return 1
    fi
    
    # Get health info
    local health_response=$(curl -s "http://localhost:$TEST_PORT/health")
    local is_office_hours=$(echo "$health_response" | jq -r '.is_office_hours')
    local current_interval=$(echo "$health_response" | jq -r '.current_interval_secs')
    local debug_override=$(echo "$health_response" | jq -r '.debug_time_override // "null"')
    
    print_info "Office hours: $is_office_hours, Interval: ${current_interval}s, Debug: $debug_override"
    
    # Check expectations
    local test_passed=true
    
    if [ "$is_office_hours" != "$expected_office_hours" ]; then
        print_failure "Expected office_hours=$expected_office_hours, got $is_office_hours"
        test_passed=false
    fi
    
    if [ "$current_interval" != "$expected_interval" ]; then
        print_failure "Expected interval=${expected_interval}s, got ${current_interval}s"
        test_passed=false
    fi
    
    if [ "$debug_override" != "$expected_debug_override" ]; then
        print_failure "Expected debug_override=$expected_debug_override, got $debug_override"
        test_passed=false
    fi
    
    if [ "$test_passed" = true ]; then
        print_success "All checks passed"
    fi
    
    # Check for specific log messages if needed
    if [ -n "$debug_time" ] && [ "$environment" != "production" ] && [ "$debug_override" != "null" ]; then
        if grep -q "DEBUG TIME OVERRIDE ACTIVE" /tmp/test_server.log; then
            print_success "Debug warning found in logs"
        else
            print_failure "Debug warning not found in logs"
        fi
    fi
    
    if [ -n "$debug_time" ] && [ "$environment" = "production" ]; then
        if grep -q "DEBUG_SYSTEM_TIME ignored in production" /tmp/test_server.log; then
            print_success "Production safety warning found in logs"
        else
            print_failure "Production safety warning not found in logs"
        fi
    fi
    
    # Stop server
    kill $server_pid 2>/dev/null || true
    wait $server_pid 2>/dev/null || true
    
    return 0
}

# Check and kill processes on test port
check_and_kill_port() {
    print_info "Checking for existing processes on port $TEST_PORT..."
    local pids=$(lsof -ti :$TEST_PORT 2>/dev/null || true)
    if [ -n "$pids" ]; then
        print_info "Killing existing processes on port $TEST_PORT: $pids"
        echo $pids | xargs kill -9 2>/dev/null || true
        sleep 1
    else
        print_info "Port $TEST_PORT is free"
    fi
}

# Main execution
main() {
    print_header "Office Hours Testing Script"
    print_info "Testing various office hours scenarios..."
    print_info "Using test port: $TEST_PORT"
    
    # Check and kill any existing processes on test port
    check_and_kill_port
    
    # Remove existing log file
    rm -f /tmp/test_server.log
    print_info "Cleaned up existing log files"
    
    # Build the project
    print_info "Building project..."
    cargo build --quiet
    print_info "Build completed"
    
    # Run tests
    # Test 1: Normal operation (current time)
    run_test "Normal operation (current system time)" "" "" "" "false" "300" "null"
    
    # Test 2: Office hours active (11am Vietnam = 2am UTC)
    run_test "Office hours active (11am Vietnam time)" "2025-08-15T02:00:00Z" "" "" "true" "30" "2025-08-15T02:00:00Z"
    
    # Test 3: Non-office hours (7pm Vietnam = 12pm UTC)
    run_test "Non-office hours (7pm Vietnam time)" "2025-08-15T12:00:00Z" "" "" "false" "300" "2025-08-15T12:00:00Z"
    
    # Test 4: Weekend (Saturday 11am Vietnam)
    run_test "Weekend during business hours (Saturday 11am Vietnam)" "2025-08-16T02:00:00Z" "" "" "false" "300" "2025-08-16T02:00:00Z"
    
    # Test 5: Office hours disabled
    run_test "Office hours feature disabled" "2025-08-15T12:00:00Z" "false" "" "false" "30" "2025-08-15T12:00:00Z"
    
    # Test 6: Production environment (debug time ignored)
    run_test "Production environment safety" "2025-08-15T02:00:00Z" "" "production" "false" "300" "null"
    
    # Test 7: Invalid debug time format
    run_test "Invalid debug time format handling" "invalid-time-format" "" "" "false" "300" "null"
    
    # Test 8: Office start time (9am Vietnam = 2am UTC)
    run_test "Edge case: Office start time (9am Vietnam)" "2025-08-15T02:00:00Z" "" "" "true" "30" "2025-08-15T02:00:00Z"
    
    # Test 9: Office end time (4pm Vietnam = 9am UTC)
    run_test "Edge case: Office end time (4pm Vietnam)" "2025-08-15T09:00:00Z" "" "" "false" "300" "2025-08-15T09:00:00Z"
    
    # Print results
    print_header "Test Results"
    echo -e "Total Tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        print_success "All tests passed! 🎉"
        exit 0
    else
        print_failure "Some tests failed. Check the output above for details."
        exit 1
    fi
}

# Cleanup function
cleanup() {
    # Kill any remaining server processes on our test port
    pkill -f "PORT=$TEST_PORT" 2>/dev/null || true
    rm -f /tmp/test_server.log
}

trap cleanup EXIT INT TERM

# Run the tests
main "$@"