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