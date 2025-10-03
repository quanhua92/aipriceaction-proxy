#!/bin/bash
# health-check.sh

# Default to localhost, allow override via parameter
BASE_URL="${1:-http://localhost:8888}"

echo "=== aipriceaction-proxy Health Check ==="
echo "Testing: $BASE_URL"
echo

# Basic connectivity
echo "1. Testing basic connectivity..."
if curl -s $BASE_URL/health > /dev/null; then
    echo "✅ Service is responding"
else
    echo "❌ Service is not responding"
    exit 1
fi

# System status
echo -e "\n2. System status:"
curl -s $BASE_URL/health | jq '{
    node: .node_name,
    environment: .environment,
    uptime_seconds: .uptime_secs,
    office_hours: .is_office_hours,
    total_tickers: .total_tickers_count,
    active_tickers: .active_tickers_count
}'

# Data availability
echo -e "\n3. Data availability:"
TICKER_COUNT=$(curl -s $BASE_URL/tickers | jq 'keys | length')
echo "Available tickers: $TICKER_COUNT"

if [ "$TICKER_COUNT" -gt 0 ]; then
    echo "✅ Market data is available"
else
    echo "⚠️  No market data available (may be normal outside office hours)"
fi

echo -e "\n4. API endpoints test:"
curl -s "$BASE_URL/tickers?symbol=VCB" | jq 'keys' && echo "✅ Ticker filtering works"
curl -s $BASE_URL/tickers/group | jq 'keys | length' > /dev/null && echo "✅ Ticker groups available"

echo -e "\n5. Raw data proxy test:"
RAW_RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/raw/market_data/AAA.csv)
RAW_HTTP_CODE=$(echo "$RAW_RESPONSE" | tail -n1)
RAW_CONTENT=$(echo "$RAW_RESPONSE" | head -n1)

if [ "$RAW_HTTP_CODE" = "200" ] && echo "$RAW_CONTENT" | grep -q "ticker,time,open"; then
    echo "✅ Raw data proxy works (/raw/market_data/AAA.csv)"
else
    echo "❌ Raw data proxy failed (HTTP $RAW_HTTP_CODE)"
fi

echo -e "\n=== Health check complete ==="