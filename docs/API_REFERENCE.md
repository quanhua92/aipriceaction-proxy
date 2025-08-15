# API Reference

This document provides comprehensive documentation for all available API endpoints in the aipriceaction-proxy service.

## Base URL

```
http://localhost:8888
```

## 🐳 Quick Start with Docker

```bash
# Run the API server (single-node setup)
docker run -p 8888:8888 \
  -e PRIMARY_TOKEN="secret-token-A-12345" \
  -e SECONDARY_TOKEN="secret-token-B-67890" \
  -e INTERNAL_PEER_URLS="" \
  quanhua92/aipriceaction-proxy:latest

# Test the API
curl http://localhost:8888/health | jq .
curl "http://localhost:8888/tickers?symbol=VCB&symbol=TCB" | jq .
```

## Endpoints

### 1. Get Tickers Data

Retrieve OHLCV (Open, High, Low, Close, Volume) data for stocks.

**Endpoint:** `GET /tickers`

**Query Parameters:**
- `symbol` (optional): Filter results to specific ticker symbols. Can be provided multiple times to fetch multiple symbols.

**Examples:**

```bash
# Get all available ticker data
curl "http://localhost:8888/tickers"

# Get data for a single ticker
curl "http://localhost:8888/tickers?symbol=VNINDEX"

# Get data for multiple tickers
curl "http://localhost:8888/tickers?symbol=VNINDEX&symbol=VIX&symbol=BMP"
```

**Response Format:**
```json
{
  "SYMBOL": [
    {
      "time": "2025-08-15T13:14:01.650095Z",
      "open": 1234.56,
      "high": 1250.00,
      "low": 1230.00,
      "close": 1245.78,
      "volume": 123456,
      "symbol": "SYMBOL"
    }
  ]
}
```

**Response Codes:**
- `200 OK`: Successfully retrieved ticker data (returns empty object `{}` if no matching symbols found)

**Use Cases:**
- Real-time market data monitoring
- Historical price analysis
- Portfolio tracking
- Financial dashboard display

---

### 2. Get Ticker Groups

Retrieve predefined groups of tickers organized by categories.

**Endpoint:** `GET /tickers/group`

**Examples:**

```bash
curl "http://localhost:8888/tickers/group"
```

**Response Format:**
```json
{
  "group_name": ["SYMBOL1", "SYMBOL2", "SYMBOL3"],
  "another_group": ["SYMBOL4", "SYMBOL5"]
}
```

**Response Codes:**
- `200 OK`: Successfully retrieved ticker groups

**Use Cases:**
- Market sector analysis
- Portfolio organization
- Bulk ticker management
- Category-based filtering

---

### 3. Internal Gossip (Node Communication)

Internal endpoint for trusted nodes to share market data updates.

**Endpoint:** `POST /gossip`

**Authentication:** Required
- Header: `Authorization: Bearer <token>`
- Accepts primary or secondary tokens configured in the system

**Request Body:**
```json
{
  "time": "2025-08-15T13:14:01.650095Z",
  "open": 1234.56,
  "high": 1250.00,
  "low": 1230.00,
  "close": 1245.78,
  "volume": 123456,
  "symbol": "SYMBOL"
}
```

**Examples:**

```bash
curl -X POST "http://localhost:8888/gossip" \
  -H "Authorization: Bearer your-token-here" \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-15T13:14:01.650095Z",
    "open": 1234.56,
    "high": 1250.00,
    "low": 1230.00,
    "close": 1245.78,
    "volume": 123456,
    "symbol": "VNINDEX"
  }'
```

**Response Codes:**
- `200 OK`: Data successfully processed
- `401 Unauthorized`: Invalid or missing authentication token
- `400 Bad Request`: Invalid data format

**Use Cases:**
- Multi-node data synchronization
- Distributed system communication
- Real-time data propagation

---

### 4. Public Gossip (Community Contributions)

Public endpoint for external contributors to submit market data with reputation tracking and validation.

**Endpoint:** `POST /public/gossip`

**Rate Limiting:** 10 requests per second, burst size of 20

**Request Body:**
```json
{
  "time": "2025-08-15T13:14:01.650095Z",
  "open": 1234.56,
  "high": 1250.00,
  "low": 1230.00,
  "close": 1245.78,
  "volume": 123456,
  "symbol": "SYMBOL"
}
```

**Examples:**

```bash
curl -X POST "http://localhost:8888/public/gossip" \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-15T13:14:01.650095Z",
    "open": 1234.56,
    "high": 1250.00,
    "low": 1230.00,
    "close": 1245.78,
    "volume": 123456,
    "symbol": "VNINDEX"
  }'
```

**Response Codes:**
- `200 OK`: Data successfully processed and accepted
- `400 Bad Request`: Implausible price change detected (>10% change)
- `403 Forbidden`: Source IP is banned due to repeated bad data
- `503 Service Unavailable`: System running on untrusted data for too long (>5 minutes)
- `429 Too Many Requests`: Rate limit exceeded

**Validation Rules:**
- Price changes >10% from last known value are rejected
- IPs with >5 failed updates are banned
- Data must include valid symbol and price information

**Use Cases:**
- Community-driven data collection
- Crowdsourced market information
- External data provider integration
- Public data validation

---

### 5. Health Check

Retrieve system health status and operational metrics.

**Endpoint:** `GET /health`

**Examples:**

```bash
curl "http://localhost:8888/health"
```

**Response Format:**
```json
{
  "is_office_hours": false,
  "current_interval_secs": 300,
  "office_hours_enabled": true,
  "timezone": "Asia/Ho_Chi_Minh",
  "office_start_hour": 9,
  "office_end_hour": 16,
  "environment": "development",
  "node_name": "aipriceaction-proxy",
  "uptime_secs": 120,
  "total_tickers_count": 288,
  "active_tickers_count": 50,
  "internal_peers_count": 1,
  "public_peers_count": 1,
  "iteration_count": 5,
  "last_update_timestamp": "2025-08-15T13:14:01.137445+00:00",
  "current_system_time": "2025-08-15T13:14:01.137441+00:00",
  "debug_time_override": null,
  "build_date": "2025-08-15T14:55:00Z",
  "git_commit": "abc123def456"
}
```

**Response Codes:**
- `200 OK`: System is healthy and operational

**Use Cases:**
- System monitoring and alerting
- Load balancer health checks
- Operational dashboards
- Performance tracking

---

## Data Models

### OhlcvData

Standard financial market data structure:

```json
{
  "time": "2025-08-15T13:14:01.650095Z",    // UTC timestamp
  "open": 1234.56,                          // Opening price
  "high": 1250.00,                          // Highest price
  "low": 1230.00,                           // Lowest price
  "close": 1245.78,                         // Closing price
  "volume": 123456,                         // Trading volume
  "symbol": "SYMBOL"                        // Ticker symbol
}
```

### HealthStats

System operational metrics:

```json
{
  "is_office_hours": false,                 // Whether market is in office hours
  "current_interval_secs": 300,             // Current update interval
  "office_hours_enabled": true,             // Office hours feature enabled
  "timezone": "Asia/Ho_Chi_Minh",           // System timezone
  "office_start_hour": 9,                   // Market open hour
  "office_end_hour": 16,                    // Market close hour
  "environment": "development",             // Deployment environment
  "node_name": "aipriceaction-proxy",       // Node identifier
  "uptime_secs": 120,                       // System uptime in seconds
  "total_tickers_count": 288,               // Total configured tickers
  "active_tickers_count": 50,               // Currently active tickers
  "internal_peers_count": 1,                // Internal peer nodes
  "public_peers_count": 1,                  // Public peer nodes
  "iteration_count": 5,                     // Processing iterations
  "last_update_timestamp": "...",           // Last data update time
  "current_system_time": "...",             // Current system time
  "debug_time_override": null,              // Debug time override (if any)
  "build_date": "2025-08-15T14:55:00Z",     // Build timestamp from Docker
  "git_commit": "abc123def456"              // Git commit hash
}
```

## Error Handling

All endpoints return appropriate HTTP status codes and error messages:

- `200 OK`: Request successful
- `400 Bad Request`: Invalid request format or data
- `401 Unauthorized`: Authentication required or failed
- `403 Forbidden`: Access denied (e.g., banned IP)
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `503 Service Unavailable`: Service temporarily unavailable

## Rate Limiting

- **Public Gossip Endpoint**: Limited to 10 requests per second with a burst capacity of 20 requests
- **Other Endpoints**: No explicit rate limiting (subject to system capacity)

## Security Features

1. **Token Authentication**: Internal gossip endpoint requires Bearer token authentication
2. **IP Reputation System**: Public gossip endpoint tracks contributor reputation
3. **Data Validation**: Price change validation prevents implausible data
4. **Automatic Banning**: Repeated bad actors are automatically banned
5. **System Trust Monitoring**: Service becomes unavailable if running on untrusted data too long

## Office Hours

The system operates in different modes based on market hours:
- **Office Hours**: Faster update intervals (30 seconds)
- **Non-Office Hours**: Slower update intervals (300 seconds)
- **Timezone**: Configurable (default: Asia/Ho_Chi_Minh)
- **Hours**: Configurable (default: 9 AM - 4 PM)

## Docker Usage Examples

### Single Node Deployment

```bash
# Basic deployment
docker run -d --name aipriceaction-proxy \
  -p 8888:8888 \
  -e NODE_NAME="api-server" \
  -e PRIMARY_TOKEN="secure-token-123" \
  -e SECONDARY_TOKEN="secure-token-456" \
  -e ENVIRONMENT="production" \
  quanhua92/aipriceaction-proxy:latest

# Check logs
docker logs aipriceaction-proxy

# Test API endpoints
curl http://localhost:8888/health
curl http://localhost:8888/tickers | jq 'keys | length'
```

### Multi-Node with Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  core-node:
    image: quanhua92/aipriceaction-proxy:latest
    container_name: aipriceaction-core
    ports:
      - "8888:8888"
    environment:
      - NODE_NAME=core-node-01
      - PRIMARY_TOKEN=secure-internal-token-ABC123
      - SECONDARY_TOKEN=secure-internal-token-DEF456
      - ENVIRONMENT=production
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  api-gateway:
    image: quanhua92/aipriceaction-proxy:latest
    container_name: aipriceaction-gateway
    ports:
      - "8889:8888"
    environment:
      - NODE_NAME=api-gateway-01
      - PRIMARY_TOKEN=secure-internal-token-ABC123
      - SECONDARY_TOKEN=secure-internal-token-DEF456
      - INTERNAL_PEER_URLS=http://core-node:8888
      - ENVIRONMENT=production
      - RUST_LOG=info
    depends_on:
      - core-node
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  default:
    name: aipriceaction-network
```

```bash
# Deploy the stack
docker-compose up -d

# Scale the API gateway
docker-compose up -d --scale api-gateway=3

# View all endpoints
echo "Core Node: http://localhost:8888/health"
echo "Gateway: http://localhost:8889/health"

# Stop the stack
docker-compose down
```

### Production Deployment with Custom Configuration

```bash
# Create production config
cat > production.yml << 'EOF'
node_name: "prod-api-01"
environment: "production"
port: 8888

tokens:
  primary: "prod-secure-token-primary-xyz789"
  secondary: "prod-secure-token-secondary-abc123"

enable_office_hours: true
office_hours_config:
  default_office_hours:
    timezone: "Asia/Ho_Chi_Minh"
    start_hour: 9
    end_hour: 16

core_worker_interval_secs: 30
non_office_worker_interval_secs: 300

internal_peers:
  - "https://node2.yourcompany.com:8888"
  - "https://node3.yourcompany.com:8888"

public_peers:
  - "https://api.yourcompany.com"
EOF

# Deploy with custom configuration
docker run -d --name aipriceaction-prod \
  -p 8888:8888 \
  -v $(pwd)/production.yml:/app/production.yml \
  -e CONFIG_FILE=production.yml \
  -e RUST_LOG=info \
  --restart unless-stopped \
  quanhua92/aipriceaction-proxy:latest

# Monitor the deployment
docker logs -f aipriceaction-prod
```

### Kubernetes Deployment

```yaml
# k8s-deployment.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aipriceaction-config
data:
  config.yml: |
    node_name: "k8s-node"
    environment: "production"
    port: 8888
    enable_office_hours: true
    office_hours_config:
      default_office_hours:
        timezone: "Asia/Ho_Chi_Minh"
        start_hour: 9
        end_hour: 16

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aipriceaction-proxy
  labels:
    app: aipriceaction-proxy
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
        - name: CONFIG_FILE
          value: "/config/config.yml"
        - name: PRIMARY_TOKEN
          valueFrom:
            secretKeyRef:
              name: aipriceaction-secrets
              key: primary-token
        - name: SECONDARY_TOKEN
          valueFrom:
            secretKeyRef:
              name: aipriceaction-secrets
              key: secondary-token
        - name: NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: config-volume
          mountPath: /config
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
      volumes:
      - name: config-volume
        configMap:
          name: aipriceaction-config

---
apiVersion: v1
kind: Service
metadata:
  name: aipriceaction-service
spec:
  selector:
    app: aipriceaction-proxy
  ports:
  - port: 80
    targetPort: 8888
    name: http
  type: LoadBalancer

---
apiVersion: v1
kind: Secret
metadata:
  name: aipriceaction-secrets
type: Opaque
data:
  primary-token: <base64-encoded-primary-token>
  secondary-token: <base64-encoded-secondary-token>
```

```bash
# Deploy to Kubernetes
kubectl apply -f k8s-deployment.yaml

# Check deployment status
kubectl get pods -l app=aipriceaction-proxy
kubectl get service aipriceaction-service

# Access the API
kubectl port-forward service/aipriceaction-service 8888:80
curl http://localhost:8888/health

# Scale the deployment
kubectl scale deployment aipriceaction-proxy --replicas=5
```

### Monitoring and Maintenance

```bash
# Health monitoring script
#!/bin/bash
while true; do
  echo "=== $(date) ==="
  curl -s http://localhost:8888/health | jq '{
    node: .node_name,
    uptime: .uptime_secs,
    tickers: .total_tickers_count,
    office_hours: .is_office_hours
  }'
  echo
  sleep 30
done

# Log aggregation
docker run -d --name log-viewer \
  -v /var/lib/docker/containers:/var/lib/docker/containers:ro \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  --label=io.portainer.accesscontrol.public \
  dozzle/dozzle:latest

# Performance monitoring
docker stats aipriceaction-proxy

# Backup configuration
docker run --rm \
  -v aipriceaction_config:/config \
  -v $(pwd):/backup \
  alpine tar czf /backup/config-backup-$(date +%Y%m%d).tar.gz -C /config .
```