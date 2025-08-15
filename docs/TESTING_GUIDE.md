# Testing Guide for aipriceaction-proxy

This guide explains how to test the aipriceaction-proxy system using the provided test scripts. The system includes comprehensive test suites for multi-node operations and office hours functionality.

## Overview

The aipriceaction-proxy is a distributed system for collecting and sharing Vietnamese stock market data. It features:

- **Multi-node architecture**: Supports both core nodes (data fetchers) and public nodes (data consumers)
- **Office hours detection**: Adjusts data fetching intervals based on Vietnamese market hours
- **Gossip protocol**: Enables data sharing between nodes
- **VCI API integration**: Fetches real-time stock data from Vietnamese Capital Securities
- **Rate limiting**: Prevents API abuse
- **Health monitoring**: Provides system status and metrics

## Test Scripts Overview

### 1. `scripts/test-multi-node.sh`
Comprehensive integration test that validates the entire multi-node system including:
- Node startup and health checks
- VCI API data fetching
- Inter-node gossip communication
- Office hours detection and interval management

### 2. `scripts/test-office-hours.sh`
Focused unit tests for office hours functionality including:
- Time zone calculations (Asia/Ho_Chi_Minh)
- Debug time override capabilities
- Production safety features
- Edge cases and error handling

## Prerequisites

Before running tests, ensure you have:

```bash
# Required tools
- Rust and Cargo (latest stable)
- curl (for HTTP requests)
- jq (for JSON parsing)
- Docker and docker-compose (for container tests)
- lsof (for port checking)

# Install dependencies
cargo build --release

# Verify tools are available
curl --version
jq --version
docker --version
docker-compose --version
```

## Multi-Node Testing (`test-multi-node.sh`)

### Purpose
This script validates the complete multi-node system by starting 3 nodes with different configurations and testing their interactions.

### Usage

```bash
# Test with native Rust processes (default)
./scripts/test-multi-node.sh

# Test with Docker containers
./scripts/test-multi-node.sh --runtime docker

# Get help
./scripts/test-multi-node.sh --help
```

### Test Architecture

The script creates a 3-node test cluster:

| Node | Port | Config | Role | Log Level |
|------|------|---------|------|-----------|
| Node1 | 8888 | `examples/configs/node1.yml` | Primary | DEBUG |
| Node2 | 8889 | `examples/configs/node2.yml` | Secondary | INFO |
| Node3 | 8890 | `examples/configs/node3.yml` | Tertiary | INFO |

### Test Phases

#### Phase 1: Pre-flight Checks
- **Port availability**: Ensures ports 8888, 8889, 8890 are free
- **Runtime preparation**: Builds Docker images or native binaries
- **Dependency validation**: Checks Docker/cargo availability

#### Phase 2: Node Startup
Depending on runtime mode:

**Native Mode:**
- Starts each node as a background Rust process
- Uses `DEBUG_SYSTEM_TIME="2025-08-15T02:00:00Z"` to force office hours
- Monitors startup with curl health checks
- Waits up to 15s per node for readiness

**Docker Mode:**
- Uses `docker-compose.test.yml` for orchestration
- Builds images with multi-stage Alpine-based Dockerfile
- Implements health checks with `curl -f http://localhost:PORT/health`
- Waits up to 60s for all containers to be healthy

#### Phase 3: Functional Testing

**Test 1: Health Endpoints and Office Hours**
```bash
# Validates each node's health endpoint
curl -s "http://localhost:8888/health"

# Expected response structure:
{
  "is_office_hours": true,
  "current_interval_secs": 30,
  "debug_time_override": "2025-08-15T02:00:00Z",
  "build_date": "2025-08-15T14:55:00Z",
  "git_commit": "abc123def456",
  "status": "healthy"
}
```

**Test 2: VCI Data Fetching**
- Polls `/tickers` endpoint every 5 seconds for up to 90 seconds
- Requires ≥10 symbols per node for success
- Validates Vietnamese stock data (VCB, TCB, FPT, etc.)
- Implements fail-fast logic (fails after 45s with no progress)

**Test 3: Gossip Communication**
```bash
# Sends test data to Node2
curl -X POST "http://localhost:8889/gossip" \
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
  }'

# Verifies Node2 received and stored the data
curl -s "http://localhost:8889/tickers" | jq '.VND'
```

### Troubleshooting Multi-Node Tests

#### Common Issues

**Port conflicts:**
```bash
# Check what's using the ports
lsof -i :8888 -i :8889 -i :8890

# Kill conflicting processes
pkill -f aipriceaction-proxy
```

**VCI API timeout:**
- Tests may fail if VCI API is unreachable
- Check network connectivity to Vietnamese trading APIs
- VCI endpoint: `https://trading.vietcap.com.vn`

**Docker build failures:**
```bash
# Check Docker daemon
docker info

# Rebuild images
docker-compose -f docker-compose.test.yml build --no-cache

# Check logs
docker-compose -f docker-compose.test.yml logs
```

**Insufficient data:**
- Test requires ≥10 symbols per node
- Increase wait time if network is slow
- Check VCI API response: `curl -s "http://localhost:8888/tickers" | jq 'keys | length'`

## Office Hours Testing (`test-office-hours.sh`)

### Purpose
This script provides comprehensive testing of the office hours detection system, which is critical for optimizing API request intervals based on Vietnamese market hours.

### Usage

```bash
# Run all office hours tests
./scripts/test-office-hours.sh

# The script runs automatically with no parameters
# All test scenarios are executed sequentially
```

### Office Hours Logic

The system implements the following business rules:

**Vietnamese Stock Market Hours (Asia/Ho_Chi_Minh timezone):**
- **Office hours**: Monday-Friday, 9:00 AM to 4:00 PM
- **Fast interval**: 30 seconds during office hours  
- **Slow interval**: 300 seconds (5 minutes) outside office hours
- **Weekend**: Always slow interval regardless of time

**Debug Time Override:**
- Development/staging: `DEBUG_SYSTEM_TIME` environment variable allows time simulation
- Production: Debug time override is **disabled** for safety
- Invalid format: Falls back to system time with warning

### Test Scenarios

The script runs 9 comprehensive test scenarios:

#### Test 1: Normal Operation (System Time)
```bash
# Environment: Default settings
# Expected: Uses current system time, no debug override
# Interval: 300s (assuming run outside office hours)
```

#### Test 2: Office Hours Active (11am Vietnam)
```bash
# Environment: DEBUG_SYSTEM_TIME="2025-08-15T02:00:00Z" (11am Vietnam)
# Expected: Office hours = true, interval = 30s
# UTC Conversion: 11am Vietnam = 2am UTC (UTC+7)
```

#### Test 3: Non-Office Hours (7pm Vietnam)  
```bash
# Environment: DEBUG_SYSTEM_TIME="2025-08-15T12:00:00Z" (7pm Vietnam)
# Expected: Office hours = false, interval = 300s
# UTC Conversion: 7pm Vietnam = 12pm UTC
```

#### Test 4: Weekend Detection (Saturday 11am)
```bash
# Environment: DEBUG_SYSTEM_TIME="2025-08-16T02:00:00Z" (Saturday)
# Expected: Office hours = false, interval = 300s
# Reason: Weekend override regardless of time
```

#### Test 5: Office Hours Disabled
```bash
# Environment: ENABLE_OFFICE_HOURS="false"
# Expected: Office hours = false, interval = 30s (core interval)
# Reason: Feature disabled, uses core worker interval
```

#### Test 6: Production Safety
```bash
# Environment: ENVIRONMENT="production", DEBUG_SYSTEM_TIME set
# Expected: Debug time ignored, office hours = false, debug_override = null
# Reason: Production safety prevents time manipulation
```

#### Test 7: Invalid Debug Time Format
```bash
# Environment: DEBUG_SYSTEM_TIME="invalid-time-format"
# Expected: Falls back to system time, debug_override = null
# Reason: Invalid format gracefully handled
```

#### Test 8: Office Start Time Edge Case (9am Vietnam)
```bash
# Environment: DEBUG_SYSTEM_TIME="2025-08-15T02:00:00Z" (exactly 9am)
# Expected: Office hours = true, interval = 30s
# Reason: Start time is inclusive
```

#### Test 9: Office End Time Edge Case (4pm Vietnam)
```bash
# Environment: DEBUG_SYSTEM_TIME="2025-08-15T09:00:00Z" (exactly 4pm)
# Expected: Office hours = false, interval = 300s  
# Reason: End time is exclusive
```

### Health Endpoint Validation

Each test validates the `/health` endpoint response:

```json
{
  "is_office_hours": true|false,
  "current_interval_secs": 30|300,
  "debug_time_override": "2025-08-15T02:00:00Z"|null,
  "status": "healthy",
  "current_time": "2025-08-15T02:00:00Z",
  "timezone": "Asia/Ho_Chi_Minh"
}
```

### Log Message Validation

The script also validates specific log messages:

**Debug Time Warning (Development):**
```
⚠️  DEBUG TIME OVERRIDE ACTIVE - Using custom time instead of system time! ⚠️
```

**Production Safety Warning:**
```
DEBUG_SYSTEM_TIME ignored in production environment for safety
```

### Troubleshooting Office Hours Tests

#### Common Issues

**Port 8899 conflicts:**
```bash
# Check what's using port 8899
lsof -i :8899

# The script automatically kills conflicting processes
# But manual cleanup may be needed:
pkill -f "PORT=8899"
```

**Build failures:**
```bash
# Ensure project builds successfully
cargo build --quiet

# Check for compilation errors
cargo check
```

**Time zone issues:**
```bash
# Verify system time zone support
timedatectl status

# Check if Asia/Ho_Chi_Minh is available
timedatectl list-timezones | grep Ho_Chi_Minh
```

**jq parsing errors:**
```bash
# Verify jq is installed and working
echo '{"test": true}' | jq '.test'

# Check health endpoint manually
curl -s "http://localhost:8899/health" | jq '.'
```

### Expected Output

Successful test run should show:
```
=== Office Hours Testing Script ===

🧪 TEST 1: Normal operation (current system time)
ℹ️  INFO: Office hours: false, Interval: 300s, Debug: null
✅ SUCCESS: All checks passed

🧪 TEST 2: Office hours active (11am Vietnam time)  
ℹ️  INFO: Office hours: true, Interval: 30s, Debug: 2025-08-15T02:00:00Z
✅ SUCCESS: All checks passed
✅ SUCCESS: Debug warning found in logs

... (additional tests)

=== Test Results ===
Total Tests: 9
Passed: 18  
Failed: 0
✅ SUCCESS: All tests passed! 🎉
```

## Advanced Testing Scenarios

### Custom Test Configurations

You can create custom test scenarios by modifying environment variables:

```bash
# Test specific time periods
DEBUG_SYSTEM_TIME="2025-12-25T05:00:00Z" ./scripts/test-office-hours.sh

# Test with custom intervals  
CORE_WORKER_INTERVAL="60" ./scripts/test-office-hours.sh

# Test with different log levels
RUST_LOG="debug" ./scripts/test-multi-node.sh --runtime native
```

### Performance Testing

Monitor system performance during tests:

```bash
# Monitor resource usage
htop

# Monitor network connections
ss -tuln | grep -E "888[89]|8899"

# Check VCI API response times
time curl -s "http://localhost:8888/tickers" > /dev/null
```

### CI/CD Integration

Both test scripts are designed for automated testing:

```bash
# Exit codes:
# 0 = All tests passed
# 1 = One or more tests failed

# Example CI usage
if ./scripts/test-multi-node.sh --runtime docker; then
  echo "Multi-node tests passed"
else
  echo "Multi-node tests failed"
  exit 1
fi

if ./scripts/test-office-hours.sh; then
  echo "Office hours tests passed"  
else
  echo "Office hours tests failed"
  exit 1
fi
```

## Test Coverage Summary

### Multi-Node Tests Cover:
- ✅ Node startup and health monitoring
- ✅ Docker containerization 
- ✅ VCI API integration and data fetching
- ✅ Inter-node gossip communication
- ✅ Office hours detection in multi-node context
- ✅ Error handling and fail-fast behavior
- ✅ Port management and cleanup

### Office Hours Tests Cover:
- ✅ Time zone conversions (UTC ↔ Asia/Ho_Chi_Minh)
- ✅ Business hour detection (9am-4pm weekdays)
- ✅ Weekend detection and handling
- ✅ Debug time override functionality
- ✅ Production safety mechanisms
- ✅ Edge case handling (start/end times)
- ✅ Configuration validation
- ✅ Log message verification
- ✅ API response validation

Both test suites provide comprehensive coverage of the aipriceaction-proxy system's core functionality and edge cases.
