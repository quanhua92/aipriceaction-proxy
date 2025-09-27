# API Test Suite

Comprehensive test suite for the aipriceaction-proxy API.

## Usage

```bash
# Basic usage (runs startup test sequence by default)
python3 test_api_comprehensive.py http://localhost:9000

# With custom timeout
python3 test_api_comprehensive.py http://localhost:9000 --timeout 30

# Test production API
python3 test_api_comprehensive.py https://api.aipriceaction.com

# Single run only (skip 30s wait)
python3 test_api_comprehensive.py http://localhost:9000 --single-run

# Show help
python3 test_api_comprehensive.py --help
```

## Test Coverage

The script tests the following scenarios:

### 📊 Health & Status Tests
- `/health` endpoint functionality
- `/tickers/group` ticker groups

### 🎯 Symbol Filtering Tests
- Single symbol requests (`?symbol=VCB`)
- Multiple symbols (`?symbol=VCB&symbol=VIX&symbol=VNINDEX`)
- Non-existent symbols (should return empty)
- Mixed valid/invalid symbols

### 📄 Format Tests
- JSON format (default)
- CSV format (`?format=csv`)

### 📅 Date Range Tests
- All historical data (`?all=true`)
- Date range filtering (`?start_date=2025-09-01&end_date=2025-09-30`)
- Start date only
- End date only

### ⚡ Enhanced Data Tests
- Checks if enhanced calculations are available
- Falls back to OHLCV data during startup

### 🚀 Performance Tests
- Response time validation (<1s expected)
- Load testing (10 concurrent requests)

### 🧩 Edge Case Tests
- No parameters (returns all data)
- Invalid format parameters
- Invalid date formats
- Empty symbol parameters

### ❌ Error Handling Tests
- Non-existent endpoints (404 expected)

## Success Criteria

- **90%+ pass rate**: Excellent performance
- **75%+ pass rate**: Good performance
- **<75% pass rate**: Issues need attention

## Sample Output

```
🧪 Starting comprehensive API tests for: http://localhost:9000
================================================================================

📊 HEALTH & STATUS TESTS
----------------------------------------
✅ PASS   | Health Endpoint                |  0.006s | OK
✅ PASS   | Ticker Groups                  |  0.002s | OK

...

================================================================================
📋 TEST SUMMARY
================================================================================
Total Tests:     20
Passed:          18 ✅
Failed:          2 ❌
Success Rate:    90.0%
Avg Response:    0.001s

🎉 EXCELLENT: API is performing very well!
```

## Dependencies

- Python 3.7+
- `requests` library

Install dependencies:
```bash
pip install requests
```

## Exit Codes

- `0`: Tests passed (75%+ success rate)
- `1`: Tests failed or critical errors