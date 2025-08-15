# aipriceaction-proxy

A high-performance Vietnamese stock market data distribution system with hybrid trust architecture.

## Overview

aipriceaction-proxy is a distributed system for collecting, processing, and distributing Vietnamese stock market data through a novel hybrid trust model that combines authenticated internal peers with reputation-based public peers.

**Key Features:**
- **Hybrid Trust Model**: Internal cryptographic authentication + public reputation system
- **Complete Market Coverage**: 291 Vietnamese stock symbols across 27 sectors
- **Multi-Node Architecture**: Core data fetchers and public data consumers
- **Office Hours Optimization**: Adjusts API intervals based on Vietnamese market hours (9AM-4PM ICT)
- **VCI API Integration**: Real-time data from Vietnamese Capital Investment API
- **Gossip Protocol**: Efficient peer-to-peer data distribution
- **Rate Limiting**: Anti-abuse protection with exponential backoff
- **Health Monitoring**: System status and performance metrics

## Architecture

**Technology Stack:**
- Runtime: Rust with Tokio async runtime
- Web Framework: Axum with tower middleware
- Data Source: VCI (Vietnamese Capital Investment) API
- Rate Limiting: Tower-governor middleware
- Serialization: Serde JSON/YAML

**Core Components:**
- `src/main.rs` - Application entry point and server setup
- `src/config.rs` - YAML/environment configuration management
- `src/worker.rs` - Background data processing and gossip distribution
- `src/vci.rs` - Vietnamese market API integration with anti-detection
- `src/api.rs` - REST endpoints and authentication
- `src/data_structures.rs` - OHLCV data types and reputation system

## Quick Start

### 🐳 Docker (Recommended)

The fastest way to get started:

```bash
# Pull and run the latest version (single-node setup)
docker run -p 8888:8888 \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  quanhua92/aipriceaction-proxy:latest

# Or with custom node name
docker run -p 8888:8888 \
  -e NODE_NAME="my-node" \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  quanhua92/aipriceaction-proxy:latest
```

**Verify it's working:**
```bash
curl http://localhost:8888/health | jq .
curl "http://localhost:8888/tickers?symbol=VCB&symbol=TCB" | jq .
```

📖 **[Complete Getting Started Guide](docs/GETTING_STARTED.md)** - Docker Compose, Kubernetes, configuration options, and troubleshooting.

### 🦀 Local Development

For development or building from source:

```bash
# Prerequisites: Rust (latest stable), curl, jq
cargo build --release
cargo run

# Or with custom configuration
CONFIG_FILE="examples/configs/node1.yml" cargo run
```

### API Endpoints
```bash
# Get all market data (public access)
curl http://localhost:8888/tickers

# Get ticker groups configuration
curl http://localhost:8888/tickers/group

# Health check
curl http://localhost:8888/health

# Internal gossip (requires authentication)
curl -X POST http://localhost:8888/gossip \
  -H "Authorization: Bearer secret-token-A-12345" \
  -H "Content-Type: application/json" \
  -d '{"time":"2025-08-14T09:30:00Z","open":85.0,"high":86.0,"low":84.5,"close":85.5,"volume":1000000,"symbol":"VCB"}'
```

## Configuration

### YAML Configuration (Production)
```yaml
# examples/configs/node1.yml
node_name: "node-01"
tokens:
  primary: "secret-token-A-12345"
  secondary: "secret-token-B-67890"
internal_peers:
  - "http://localhost:8889"
  - "http://localhost:8890"
public_peers:
  - "https://api.aipriceaction.com"
core_network_url: null  # null = core node mode
core_worker_interval_secs: 30
environment: "development"
port: 8888
```

### Environment Variables (Docker/Testing)
```bash
export NODE_NAME="node-dev-01"
export PORT="8888"
export PRIMARY_TOKEN="secret-token-A-12345"
export SECONDARY_TOKEN="secret-token-B-67890"
export INTERNAL_PEER_URLS="http://node2:8889,http://node3:8890"
export CORE_WORKER_INTERVAL="30"
export ENVIRONMENT="production"
```

## Multi-Node Deployment

### Core Node Setup
Set `core_network_url: null` to enable VCI data fetching and gossip distribution.

### Public Node Setup
Set `core_network_url: "http://core-node:8888"` to sync from core network.

### Network Types
- **Internal Network**: Bearer token authentication, immediate trust
- **Public Network**: IP-based reputation, price validation, progressive trust

## Testing

Comprehensive test suites are provided for validation:

```bash
# Multi-node integration tests (native)
./scripts/test-multi-node.sh

# Multi-node tests with Docker
./scripts/test-multi-node.sh --runtime docker

# Office hours functionality tests
./scripts/test-office-hours.sh
```

**Test Coverage:**
- Node startup and health monitoring
- VCI API integration and data fetching
- Inter-node gossip communication
- Office hours detection and interval management
- Time zone calculations (UTC ↔ Asia/Ho_Chi_Minh)
- Debug time override capabilities
- Production safety features
- Error handling and edge cases

## Performance Characteristics

**Core Worker (291 symbols):**
- Complete cycle: 30 batches (29×10 + 1×1 symbols)
- Processing time: ~6-15 seconds per cycle
- Memory usage: ~20.9KB per cycle
- API rate limiting: 30 requests/minute with exponential backoff

**Network Efficiency:**
- Batch processing: 10 symbols per API call
- Rate limit compliance: 1-2 second sleep between batches
- Anti-detection: 5 rotating browser user agents
- Timeout management: 30-second request timeouts

## Market Coverage

**Vietnamese Stock Market (291 symbols across 27 sectors):**
- Banking (NGAN_HANG): 17 major banks (VCB, TCB, ACB, etc.)
- Real Estate (BAT_DONG_SAN): 26 property companies
- Technology (CONG_NGHE): 9 tech companies including FPT
- Oil & Gas (DAU_KHI): 8 energy companies
- Steel (THEP): 13 steel and metal companies
- And 22 additional sectors

## Documentation

For detailed technical documentation and implementation details:
- [Complete Technical Documentation](docs/README.md) - Full system architecture, implementation details, and API reference
- [Testing Guide](docs/TESTING_GUIDE.md) - Comprehensive testing procedures and troubleshooting

## Security

**Trust Model:**
- Internal peers: Cryptographic bearer token authentication
- Public peers: IP-based reputation with price validation (10% threshold)
- Rate limiting: Tower-governor middleware protection
- Production safety: Debug time override disabled in production
- Zero-downtime: Primary/secondary token rotation support

## License

Licensed under the MIT License. See LICENSE file for details.