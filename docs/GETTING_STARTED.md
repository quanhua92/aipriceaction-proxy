# Getting Started with aipriceaction-proxy

This guide will help you quickly get started with aipriceaction-proxy, a high-performance Vietnamese stock market data distribution system.

## Table of Contents

- [Quick Start with Docker](#quick-start-with-docker)
- [Local Development Setup](#local-development-setup)
- [Configuration Options](#configuration-options)
- [API Usage Examples](#api-usage-examples)
- [Multi-Node Deployment](#multi-node-deployment)
- [Troubleshooting](#troubleshooting)

## Quick Start with Docker

The fastest way to get started is using our pre-built Docker image.

> **⚠️ Important**: The container requires three mandatory environment variables:
> - `PRIMARY_TOKEN` - Primary authentication token
> - `SECONDARY_TOKEN` - Secondary authentication token  
> - `INTERNAL_PEER_URLS` - Peer URLs (use empty string `""` for single-node setups)

### 1. Run with Docker

```bash
# Basic single-node setup (minimal required environment variables)
docker run -p 8888:8888 \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  quanhua92/aipriceaction-proxy:latest

# Or run a specific version
docker run -p 8888:8888 \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  quanhua92/aipriceaction-proxy:0.1.0
```

### 2. Run with Custom Configuration

```bash
# Create a configuration file
cat > node.yml << 'EOF'
node_name: "my-node"
tokens:
  primary: "my-secret-token-12345"
  secondary: "my-backup-token-67890"
environment: "development"
port: 8888
enable_office_hours: true
office_hours_config:
  default_office_hours:
    timezone: "Asia/Ho_Chi_Minh"
    start_hour: 9
    end_hour: 16
core_worker_interval_secs: 30
non_office_worker_interval_secs: 300
EOF

# Run with custom config
docker run -p 8888:8888 \
  -v $(pwd)/node.yml:/app/node.yml \
  -e CONFIG_FILE=node.yml \
  quanhua92/aipriceaction-proxy:latest
```

### 3. Run with Environment Variables

```bash
# Single-node setup with custom configuration
docker run -p 8888:8888 \
  -e NODE_NAME="docker-node-01" \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  -e ENVIRONMENT="development" \
  -e RUST_LOG="info" \
  quanhua92/aipriceaction-proxy:latest

# Multi-node setup (requires peer URLs)
docker run -p 8888:8888 \
  -e NODE_NAME="peer-node-01" \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="http://core-node:8888,http://other-peer:8889" \
  -e ENVIRONMENT="production" \
  -e RUST_LOG="info" \
  quanhua92/aipriceaction-proxy:latest
```

### 4. Verify Installation

```bash
# Check health status
curl http://localhost:8888/health | jq .

# Get available tickers
curl http://localhost:8888/tickers | jq 'keys | length'

# Get specific tickers
curl "http://localhost:8888/tickers?symbol=VCB&symbol=TCB" | jq .
```

## Local Development Setup

For development or when you want to build from source.

### Prerequisites

- **Rust** (latest stable): [Install Rust](https://rustup.rs/)
- **Git**: For cloning the repository
- **curl & jq**: For testing API endpoints

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/quanhua92/aipriceaction-proxy.git
cd aipriceaction-proxy

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### 2. Development Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with custom port
PORT=9999 cargo run

# Run with environment variables
export NODE_NAME="dev-node"
export PRIMARY_TOKEN="dev-token-123"
export SECONDARY_TOKEN="dev-token-456"
cargo run
```

### 3. Building Docker Image Locally

```bash
# Build your own Docker image
./scripts/docker-build-and-tag.sh dev-local

# Run your local build
docker run -p 8888:8888 aipriceaction-proxy:dev-local
```

## Configuration Options

### Environment Variables

| Variable | Description | Default | Example | Required |
|----------|-------------|---------|---------|----------|
| `NODE_NAME` | Unique node identifier | `"aipriceaction-proxy"` | `"prod-node-01"` | No |
| `PORT` | HTTP server port | `8888` | `3000` | No |
| `PRIMARY_TOKEN` | Primary authentication token | N/A | `"secret-ABC-123"` | **Yes** |
| `SECONDARY_TOKEN` | Secondary authentication token | N/A | `"secret-DEF-456"` | **Yes** |
| `INTERNAL_PEER_URLS` | Comma-separated peer URLs | N/A | `"http://node1:8888,http://node2:8889"` or `""` for single-node | **Yes** |
| `ENVIRONMENT` | Deployment environment | `"development"` | `"production"` | No |
| `CONFIG_FILE` | Path to YAML config file | `""` | `"config/prod.yml"` | No |
| `RUST_LOG` | Logging level | `"info"` | `"debug"` | No |

### YAML Configuration

```yaml
# config/production.yml
node_name: "prod-node-01"
environment: "production"
port: 8888

# Authentication tokens for internal gossip
tokens:
  primary: "secure-primary-token-12345"
  secondary: "secure-secondary-token-67890"

# Peer network configuration
internal_peers:
  - "https://node2.example.com:8888"
  - "https://node3.example.com:8888"

public_peers:
  - "https://public-api.example.com"

# Core node settings (null = this is a core node)
core_network_url: null

# Performance tuning
core_worker_interval_secs: 30
non_office_worker_interval_secs: 300

# Office hours configuration
enable_office_hours: true
office_hours_config:
  default_office_hours:
    timezone: "Asia/Ho_Chi_Minh"
    start_hour: 9    # 9 AM
    end_hour: 16     # 4 PM
```

## API Usage Examples

### Basic Data Retrieval

```bash
# Get all available stock data
curl http://localhost:8888/tickers

# Get specific stocks by symbol
curl "http://localhost:8888/tickers?symbol=VCB&symbol=TCB&symbol=ACB"

# Get ticker groups
curl http://localhost:8888/tickers/group

# System health check
curl http://localhost:8888/health
```

### Advanced Usage with jq

```bash
# Count total available tickers
curl -s http://localhost:8888/tickers | jq 'keys | length'

# Get bank stocks only
curl -s http://localhost:8888/tickers/group | jq '.NGAN_HANG'

# Get latest prices for bank stocks
BANKS=$(curl -s http://localhost:8888/tickers/group | jq -r '.NGAN_HANG | join("&symbol=")')
curl -s "http://localhost:8888/tickers?symbol=$BANKS" | jq 'to_entries | map({symbol: .key, price: .value[-1].close})'

# Monitor system health
watch -n 5 'curl -s http://localhost:8888/health | jq "{status: .environment, tickers: .total_tickers_count, office_hours: .is_office_hours}"'
```

### Internal Node Communication

```bash
# Send data to another node (requires authentication)
curl -X POST http://localhost:8888/gossip \
  -H "Authorization: Bearer your-secret-token" \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-15T09:30:00Z",
    "open": 85000,
    "high": 86500,
    "low": 84200,
    "close": 85800,
    "volume": 1250000,
    "symbol": "VCB"
  }'
```

### Public Data Contribution

```bash
# Contribute data as public peer (no authentication required)
curl -X POST http://localhost:8888/public/gossip \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-15T09:30:00Z",
    "open": 25500,
    "high": 26000,
    "low": 25200,
    "close": 25800,
    "volume": 890000,
    "symbol": "TCB"
  }'
```

## Multi-Node Deployment

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  core-node:
    image: quanhua92/aipriceaction-proxy:latest
    ports:
      - "8888:8888"
    environment:
      - NODE_NAME=core-node-01
      - PRIMARY_TOKEN=secure-token-ABC-123
      - SECONDARY_TOKEN=secure-token-DEF-456
      - INTERNAL_PEER_URLS=http://peer-node-1:8888,http://peer-node-2:8888
      - ENVIRONMENT=development
      - RUST_LOG=info
    restart: unless-stopped

  peer-node-1:
    image: quanhua92/aipriceaction-proxy:latest
    ports:
      - "8889:8888"
    environment:
      - NODE_NAME=peer-node-01
      - PRIMARY_TOKEN=secure-token-ABC-123
      - SECONDARY_TOKEN=secure-token-DEF-456
      - INTERNAL_PEER_URLS=http://core-node:8888,http://peer-node-2:8888
      - ENVIRONMENT=development
      - RUST_LOG=info
    restart: unless-stopped

  peer-node-2:
    image: quanhua92/aipriceaction-proxy:latest
    ports:
      - "8890:8888"
    environment:
      - NODE_NAME=peer-node-02
      - PRIMARY_TOKEN=secure-token-ABC-123
      - SECONDARY_TOKEN=secure-token-DEF-456
      - INTERNAL_PEER_URLS=http://core-node:8888,http://peer-node-1:8888
      - ENVIRONMENT=development
      - RUST_LOG=info
    restart: unless-stopped
```

```bash
# Start the multi-node cluster
docker-compose up -d

# Check all nodes health
curl http://localhost:8888/health | jq .node_name  # core-node-01
curl http://localhost:8889/health | jq .node_name  # peer-node-01
curl http://localhost:8890/health | jq .node_name  # peer-node-02

# Stop the cluster
docker-compose down
```

### Kubernetes Deployment

```yaml
# k8s-deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aipriceaction-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: aipriceaction-proxy
  template:
    metadata:
      labels:
        app: aipriceaction-proxy
    spec:
      containers:
      - name: aipriceaction-proxy
        image: quanhua92/aipriceaction-proxy:latest
        ports:
        - containerPort: 8888
        env:
        - name: NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: ENVIRONMENT
          value: "production"
        - name: PRIMARY_TOKEN
          valueFrom:
            secretKeyRef:
              name: proxy-secrets
              key: primary-token
        - name: SECONDARY_TOKEN
          valueFrom:
            secretKeyRef:
              name: proxy-secrets
              key: secondary-token
        livenessProbe:
          httpGet:
            path: /health
            port: 8888
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8888
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: aipriceaction-proxy-service
spec:
  selector:
    app: aipriceaction-proxy
  ports:
  - port: 80
    targetPort: 8888
  type: LoadBalancer
```

## Troubleshooting

### Common Issues

#### 1. Docker Container Won't Start

```bash
# Check container logs
docker logs $(docker ps -lq)

# Common error: Missing required environment variables
# Error: "PRIMARY_TOKEN must be set: NotPresent"
# Solution: Add required environment variables
docker run -p 8888:8888 \
  -e PRIMARY_TOKEN="your-primary-token" \
  -e SECONDARY_TOKEN="your-secondary-token" \
  -e INTERNAL_PEER_URLS="" \
  -e RUST_LOG=debug \
  quanhua92/aipriceaction-proxy:latest

# Check if port is already in use
lsof -i :8888
```

#### 2. No Data Available

```bash
# Check if VCI API is accessible
curl -s http://localhost:8888/health | jq .total_tickers_count

# Verify office hours settings
curl -s http://localhost:8888/health | jq '{office_hours: .is_office_hours, interval: .current_interval_secs}'

# Force data fetch (if you're debugging)
DEBUG_SYSTEM_TIME="2025-08-15T12:00:00Z" docker run -p 8888:8888 -e RUST_LOG=debug quanhua92/aipriceaction-proxy:latest
```

#### 3. Authentication Issues

```bash
# Test internal gossip endpoint
curl -X POST http://localhost:8888/gossip \
  -H "Authorization: Bearer wrong-token" \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
# Should return 401 Unauthorized

# Test with correct token
curl -X POST http://localhost:8888/gossip \
  -H "Authorization: Bearer your-correct-token" \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-15T09:30:00Z",
    "open": 1000, "high": 1100, "low": 900, "close": 1050,
    "volume": 100000, "symbol": "TEST"
  }'
```

#### 4. Performance Issues

```bash
# Monitor resource usage
docker stats

# Check system health
curl -s http://localhost:8888/health | jq '{
  uptime: .uptime_secs,
  tickers: .total_tickers_count,
  active: .active_tickers_count,
  iterations: .iteration_count
}'

# Adjust worker intervals
docker run -p 8888:8888 \
  -e CORE_WORKER_INTERVAL=60 \
  -e NON_OFFICE_WORKER_INTERVAL=600 \
  quanhua92/aipriceaction-proxy:latest
```

### Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/quanhua92/aipriceaction-proxy/issues)
- **API Reference**: [Complete API documentation](API_REFERENCE.md)
- **Testing Guide**: [Comprehensive testing procedures](TESTING_GUIDE.md)
- **Technical Documentation**: [Full system architecture](README.md)

### Quick Health Check Script

```bash
#!/bin/bash
# health-check.sh

echo "=== aipriceaction-proxy Health Check ==="
echo

# Basic connectivity
echo "1. Testing basic connectivity..."
if curl -s http://localhost:8888/health > /dev/null; then
    echo "✅ Service is responding"
else
    echo "❌ Service is not responding"
    exit 1
fi

# System status
echo -e "\n2. System status:"
curl -s http://localhost:8888/health | jq '{
    node: .node_name,
    environment: .environment,
    uptime_seconds: .uptime_secs,
    office_hours: .is_office_hours,
    total_tickers: .total_tickers_count,
    active_tickers: .active_tickers_count
}'

# Data availability
echo -e "\n3. Data availability:"
TICKER_COUNT=$(curl -s http://localhost:8888/tickers | jq 'keys | length')
echo "Available tickers: $TICKER_COUNT"

if [ "$TICKER_COUNT" -gt 0 ]; then
    echo "✅ Market data is available"
else
    echo "⚠️  No market data available (may be normal outside office hours)"
fi

echo -e "\n4. API endpoints test:"
curl -s "http://localhost:8888/tickers?symbol=VCB" | jq 'keys' && echo "✅ Ticker filtering works"
curl -s http://localhost:8888/tickers/group | jq 'keys | length' > /dev/null && echo "✅ Ticker groups available"

echo -e "\n=== Health check complete ==="
```

## Next Steps

1. **Explore the API**: Try the examples in [API_REFERENCE.md](API_REFERENCE.md)
2. **Set up monitoring**: Use the `/health` endpoint for monitoring
3. **Scale your deployment**: Use Docker Compose or Kubernetes for multi-node setups
4. **Customize configuration**: Adjust office hours, intervals, and peer networks
5. **Contribute**: Submit issues or improvements on GitHub

For production deployments, make sure to:
- Use secure authentication tokens
- Set up proper logging and monitoring
- Configure appropriate resource limits
- Implement backup and recovery procedures
- Follow security best practices