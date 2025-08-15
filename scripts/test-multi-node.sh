#!/bin/bash

# Multi-Node Testing Script for aipriceaction-proxy
# This script starts 3 nodes with different configurations and tests gossip communication
# Usage: ./test-multi-node.sh [--runtime native|docker]

set -e

# Parse command line arguments
RUNTIME="native"  # Default to native
while [[ $# -gt 0 ]]; do
    case $1 in
        --runtime)
            RUNTIME="$2"
            shift 2
            ;;
        *)
            echo "Unknown option $1"
            echo "Usage: $0 [--runtime native|docker]"
            exit 1
            ;;
    esac
done

# Validate runtime parameter
if [[ "$RUNTIME" != "native" && "$RUNTIME" != "docker" ]]; then
    echo "Error: --runtime must be either 'native' or 'docker'"
    echo "Usage: $0 [--runtime native|docker]"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Fix for symlink issues - use realpath if available
if command -v realpath >/dev/null 2>&1; then
    PROJECT_ROOT="$(realpath "$PROJECT_ROOT")"
fi
TEST_DURATION=60  # Run for 60s - fast test with office hours enabled
NODE_CONFIGS=("node1.yml" "node2.yml" "node3.yml")
PORTS=(8888 8889 8890)
VCI_SYMBOLS=("VCB" "TCB" "FPT" "ACB")

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions for test reporting
print_success() {
    echo -e "${GREEN}✅ SUCCESS${NC}: $1"
    ((PASSED_TESTS++))
}

print_failure() {
    echo -e "${RED}❌ FAILURE${NC}: $1"
    ((FAILED_TESTS++))
}

print_test() {
    echo -e "\n${BLUE}🧪 TEST $((TOTAL_TESTS + 1)):${NC} $1"
    ((TOTAL_TESTS++))
}

echo -e "${BLUE}=== Multi-Node Testing Script ===${NC}"
echo "Runtime: $RUNTIME"
echo "Project root: $PROJECT_ROOT"
echo "Test duration: ${TEST_DURATION}s (office hours enabled for fast testing)"
echo

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up background processes...${NC}"
    
    if [ "$RUNTIME" = "docker" ]; then
        # Stop Docker Compose services
        cd "$PROJECT_ROOT"
        docker-compose -f docker-compose.test.yml down --remove-orphans 2>/dev/null || true
        echo -e "${YELLOW}Docker Compose services stopped and removed${NC}"
    else
        # Kill native processes
        jobs -p | xargs -r kill 2>/dev/null || true
        wait 2>/dev/null || true
    fi
    
    echo -e "${GREEN}Cleanup complete${NC}"
}

# Set trap for cleanup on script exit
trap cleanup EXIT

# Check if ports are available
check_ports() {
    echo -e "${BLUE}Checking port availability...${NC}"
    for port in "${PORTS[@]}"; do
        if lsof -i :$port >/dev/null 2>&1; then
            echo -e "${RED}Error: Port $port is already in use${NC}"
            exit 1
        fi
        echo -e "${GREEN}✓ Port $port is available${NC}"
    done
}

# Start nodes using Docker Compose or native
start_nodes() {
    if [ "$RUNTIME" = "docker" ]; then
        echo -e "${BLUE}Starting all nodes with Docker Compose...${NC}"
        cd "$PROJECT_ROOT"
        
        # Start all services with Docker Compose
        if ! docker-compose -f docker-compose.test.yml up -d; then
            echo -e "${RED}Failed to start Docker Compose services${NC}"
            return 1
        fi
        
        echo -e "${GREEN}✓ Docker Compose services started${NC}"
        
        # Wait for all services to be healthy
        echo -e "${BLUE}Waiting for services to be healthy...${NC}"
        local max_wait=60
        local wait_time=0
        
        while [ $wait_time -lt $max_wait ]; do
            local healthy_count=0
            for port in "${PORTS[@]}"; do
                if curl -s "http://localhost:$port/health" > /dev/null 2>&1; then
                    ((healthy_count++))
                fi
            done
            
            if [ $healthy_count -eq ${#PORTS[@]} ]; then
                echo -e "${GREEN}✓ All nodes are healthy and responding${NC}"
                return 0
            fi
            
            sleep 2
            wait_time=$((wait_time + 2))
            echo "  → Waiting for nodes to be ready... ($wait_time/${max_wait}s)"
        done
        
        echo -e "${RED}✗ Timeout waiting for nodes to be healthy${NC}"
        echo "Container status:"
        docker-compose -f docker-compose.test.yml ps
        return 1
    else
        # Native execution - start each node individually
        echo -e "${BLUE}Starting nodes natively...${NC}"
        for i in "${!NODE_CONFIGS[@]}"; do
            local node_name="Node$((i+1))"
            local config_file="${NODE_CONFIGS[$i]}"
            local port="${PORTS[$i]}"
            
            echo -e "${BLUE}Starting $node_name (port $port)...${NC}"
            
            # Set log level based on node name - DEBUG for Node1, INFO for others
            local log_level="info"
            if [ "$node_name" = "Node1" ]; then
                log_level="debug"
                echo -e "${YELLOW}  → Using DEBUG logging for $node_name${NC}"
            fi
            
            cd "$PROJECT_ROOT"
            
            # Force office hours to ensure 30s intervals for faster testing
            DEBUG_SYSTEM_TIME="2025-08-15T02:00:00Z" RUST_LOG=$log_level CONFIG_FILE="examples/configs/$config_file" \
                cargo run 2>&1 | sed "s/^/[$node_name] /" &
            
            local pid=$!
            echo "  → Node $node_name started with PID $pid"
            
            # Wait a moment before starting next node
            sleep 2
        done
        
        # Wait for all native nodes to be responding
        echo -e "${BLUE}Waiting for native nodes to be ready...${NC}"
        sleep 5
        
        for i in "${!PORTS[@]}"; do
            local port="${PORTS[$i]}"
            local node_name="Node$((i+1))"
            local attempts=0
            
            while [ $attempts -lt 15 ]; do
                if curl -s "http://localhost:$port/tickers" > /dev/null 2>&1; then
                    echo -e "${GREEN}✓ $node_name is responding on port $port${NC}"
                    break
                fi
                sleep 1
                attempts=$((attempts + 1))
            done
            
            if [ $attempts -eq 15 ]; then
                echo -e "${RED}✗ $node_name failed to start properly${NC}"
                return 1
            fi
        done
        
        echo -e "${GREEN}✓ All native nodes started successfully${NC}"
        return 0
    fi
}

# Test 1: Check VCI data fetching
test_vci_data_fetching() {
    print_test "VCI data fetching across all nodes"
    
    echo "Waiting 45 seconds for VCI data to be fetched..."
    sleep 45
    
    local all_nodes_have_data=true
    local vci_symbols_found=0
    
    for i in "${!PORTS[@]}"; do
        local port=${PORTS[$i]}
        local node_name="Node$((i+1))"
        
        local response=$(curl -s "http://localhost:$port/tickers" 2>/dev/null || echo "{}")
        local symbol_count=$(echo "$response" | jq -r 'keys | length' 2>/dev/null || echo "0")
        
        echo "  $node_name: $symbol_count symbols"
        
        if [ "$symbol_count" -lt 10 ]; then
            all_nodes_have_data=false
        fi
        
        # Count VCI symbols
        for symbol in "${VCI_SYMBOLS[@]}"; do
            if echo "$response" | jq -e ".$symbol" > /dev/null 2>&1; then
                ((vci_symbols_found++))
                break
            fi
        done
    done
    
    if [ "$all_nodes_have_data" = true ]; then
        print_success "All nodes have fetched ticker data"
    else
        print_failure "Some nodes failed to fetch sufficient ticker data"
    fi
    
    if [ "$vci_symbols_found" -ge 2 ]; then
        print_success "VCI symbols found across nodes ($vci_symbols_found nodes with VCI data)"
    else
        print_failure "Insufficient VCI symbols found ($vci_symbols_found nodes with VCI data)"
    fi
}

# Test 2: Gossip communication
test_gossip_communication() {
    print_test "Gossip communication between nodes"
    
    echo "Sending VND data to Node 2 via gossip..."
    local gossip_response=$(curl -X POST "http://localhost:8889/gossip" \
        -H "Authorization: Bearer secret-token-A-12345" \
        -H "Content-Type: application/json" \
        -d '{
            "time": "2025-08-14T00:00:00Z",
            "open": 28500.0,
            "high": 29200.0,
            "low": 28300.0,
            "close": 29000.0,
            "volume": 15000000,
            "symbol": "VND"
        }' -s -w "%{http_code}")
    
    if [[ "$gossip_response" =~ 200$ ]]; then
        print_success "Gossip message sent successfully to Node 2"
    else
        print_failure "Failed to send gossip message (HTTP: ${gossip_response})"
        return
    fi
    
    sleep 2
    
    # Check if Node 2 received the gossip data
    local vnd_data=$(curl -s "http://localhost:8889/tickers" | jq -r '.VND // empty' 2>/dev/null)
    if [ -n "$vnd_data" ] && [ "$vnd_data" != "null" ] && [ "$vnd_data" != "empty" ]; then
        local vnd_price=$(echo "$vnd_data" | jq -r '.[0].close' 2>/dev/null || echo "unknown")
        print_success "Node 2 received VND gossip data (close: $vnd_price)"
    else
        print_failure "Node 2 did not receive VND gossip data"
    fi
}

# Test 3: Health endpoints and office hours
test_health_endpoints() {
    print_test "Health endpoints and office hours detection"
    
    local all_healthy=true
    
    for i in "${!PORTS[@]}"; do
        local port=${PORTS[$i]}
        local node_name="Node$((i+1))"
        
        local health_response=$(curl -s "http://localhost:$port/health" 2>/dev/null || echo "{}")
        local is_office_hours=$(echo "$health_response" | jq -r '.is_office_hours // false' 2>/dev/null)
        local current_interval=$(echo "$health_response" | jq -r '.current_interval_secs // 0' 2>/dev/null)
        local debug_override=$(echo "$health_response" | jq -r '.debug_time_override // "null"' 2>/dev/null)
        
        echo "  $node_name: office_hours=$is_office_hours, interval=${current_interval}s, debug=$debug_override"
        
        if [ "$is_office_hours" != "true" ] || [ "$current_interval" != "30" ]; then
            all_healthy=false
        fi
    done
    
    if [ "$all_healthy" = true ]; then
        print_success "All nodes report office hours active with 30s intervals"
    else
        print_failure "Some nodes have incorrect office hours or interval settings"
    fi
}

# Build or prepare runtime environment
prepare_runtime() {
    if [ "$RUNTIME" = "docker" ]; then
        echo -e "${BLUE}Preparing Docker Compose environment...${NC}"
        
        # Check if Docker is running
        if ! docker info >/dev/null 2>&1; then
            echo -e "${RED}Error: Docker is not running${NC}"
            exit 1
        fi
        
        # Check if docker-compose is available
        if ! command -v docker-compose >/dev/null 2>&1; then
            echo -e "${RED}Error: docker-compose is not installed${NC}"
            exit 1
        fi
        
        # Build Docker images via Docker Compose
        echo -e "${BLUE}Building Docker images with Docker Compose...${NC}"
        cd "$PROJECT_ROOT"
        if ! docker-compose -f docker-compose.test.yml build --quiet; then
            echo -e "${RED}Failed to build Docker images${NC}"
            exit 1
        fi
        echo -e "${GREEN}✓ Docker images built successfully${NC}"
    else
        echo -e "${BLUE}Building project for native execution...${NC}"
        cd "$PROJECT_ROOT"
        cargo build --release
        echo -e "${GREEN}✓ Native build completed${NC}"
    fi
}

# Main execution with timeout
main() {
    echo -e "${BLUE}Starting multi-node test with $RUNTIME runtime...${NC}"
    
    # Pre-flight checks
    check_ports
    
    # Prepare runtime environment
    prepare_runtime
    
    # Start all nodes
    echo -e "\n${BLUE}Starting nodes with office hours enabled...${NC}"
    if ! start_nodes; then
        print_failure "Failed to start nodes"
        return 1
    fi
    
    # Run tests
    test_health_endpoints
    test_vci_data_fetching
    test_gossip_communication
    
    # Print final results
    echo -e "\n${BLUE}=== Test Results ===${NC}"
    echo -e "Runtime: $RUNTIME"
    echo -e "Total Tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}✅ SUCCESS: All multi-node tests passed with $RUNTIME runtime! 🎉${NC}"
        echo "Verified:"
        echo "1. ✓ All 3 nodes started on different ports"
        echo "2. ✓ Office hours detection and interval management"
        echo "3. ✓ VCI API integration and data fetching"
        echo "4. ✓ Gossip communication between nodes"
        echo "5. ✓ $RUNTIME runtime execution working correctly"
        exit 0
    else
        echo -e "\n${RED}❌ FAILURE: Some tests failed with $RUNTIME runtime. Check the output above for details.${NC}"
        exit 1
    fi
}

# Run the main function
main "$@"