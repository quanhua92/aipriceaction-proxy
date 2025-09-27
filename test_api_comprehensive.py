#!/usr/bin/env python3
"""
Comprehensive API Test Suite for aipriceaction-proxy
Tests all endpoints, parameters, and edge cases
"""

import requests
import json
import time
import sys
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import argparse

@dataclass
class TestResult:
    name: str
    success: bool
    response_time: float
    details: str
    status_code: int

class APITester:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
        self.results: List[TestResult] = []

    def test_request(self, name: str, endpoint: str, expected_status: int = 200,
                    validate_func=None, timeout: float = 10.0) -> TestResult:
        """Make a test request and validate the response"""
        url = f"{self.base_url}{endpoint}"
        start_time = time.time()

        try:
            response = requests.get(url, timeout=timeout)
            response_time = time.time() - start_time

            # Basic status code check
            if response.status_code != expected_status:
                return TestResult(
                    name=name,
                    success=False,
                    response_time=response_time,
                    details=f"Expected status {expected_status}, got {response.status_code}",
                    status_code=response.status_code
                )

            # Custom validation if provided
            if validate_func:
                try:
                    validation_result = validate_func(response)
                    if validation_result is not True:
                        return TestResult(
                            name=name,
                            success=False,
                            response_time=response_time,
                            details=f"Validation failed: {validation_result}",
                            status_code=response.status_code
                        )
                except Exception as e:
                    return TestResult(
                        name=name,
                        success=False,
                        response_time=response_time,
                        details=f"Validation error: {str(e)}",
                        status_code=response.status_code
                    )

            return TestResult(
                name=name,
                success=True,
                response_time=response_time,
                details="OK",
                status_code=response.status_code
            )

        except requests.exceptions.Timeout:
            return TestResult(
                name=name,
                success=False,
                response_time=timeout,
                details="Request timeout",
                status_code=0
            )
        except requests.exceptions.ConnectionError:
            return TestResult(
                name=name,
                success=False,
                response_time=time.time() - start_time,
                details="Connection error - server may not be running",
                status_code=0
            )
        except Exception as e:
            return TestResult(
                name=name,
                success=False,
                response_time=time.time() - start_time,
                details=f"Request error: {str(e)}",
                status_code=0
            )

    def validate_json_response(self, response) -> bool:
        """Validate that response is valid JSON"""
        try:
            response.json()
            return True
        except:
            return "Invalid JSON response"

    def validate_health_response(self, response) -> bool:
        """Validate health endpoint response structure"""
        try:
            data = response.json()
            required_fields = ['active_tickers_count', 'memory_usage_mb', 'is_office_hours']
            for field in required_fields:
                if field not in data:
                    return f"Missing field: {field}"
            return True
        except:
            return "Invalid health response format"

    def validate_ticker_groups(self, response) -> bool:
        """Validate ticker groups response"""
        try:
            data = response.json()
            if not isinstance(data, dict):
                return "Expected object with ticker groups"
            if len(data) == 0:
                return "No ticker groups found"
            return True
        except:
            return "Invalid ticker groups format"

    def validate_single_symbol(self, response) -> bool:
        """Validate single symbol response"""
        try:
            data = response.json()
            if isinstance(data, dict) and len(data) == 1:
                return True
            return f"Expected 1 symbol, got {len(data) if isinstance(data, dict) else 'invalid'}"
        except:
            return "Invalid single symbol response"

    def validate_multiple_symbols(self, expected_count: int):
        """Return validator for multiple symbols"""
        def validator(response) -> bool:
            try:
                data = response.json()
                if isinstance(data, dict) and len(data) == expected_count:
                    return True
                return f"Expected {expected_count} symbols, got {len(data) if isinstance(data, dict) else 'invalid'}"
            except:
                return "Invalid multiple symbols response"
        return validator

    def validate_empty_response(self, response) -> bool:
        """Validate empty response for non-existent symbols"""
        try:
            data = response.json()
            if isinstance(data, dict) and len(data) == 0:
                return True
            return f"Expected empty response, got {len(data) if isinstance(data, dict) else 'invalid'}"
        except:
            return "Invalid empty response"

    def validate_csv_response(self, response) -> bool:
        """Validate CSV response format"""
        content_type = response.headers.get('content-type', '')
        text = response.text

        # CSV format is only available when enhanced data is ready
        # During fallback mode, it returns JSON (which is expected behavior)
        if 'text/csv' in content_type:
            # True CSV response - check for CSV headers
            if 'date,symbol,open,high,low,close,volume' in text:
                return True
            return "CSV content-type but invalid CSV format"
        elif 'application/json' in content_type:
            # Fallback mode - CSV requested but enhanced data not ready
            try:
                data = response.json()
                if isinstance(data, dict):
                    return "CSV requested but in fallback mode (JSON returned)"
                return "Invalid JSON in fallback mode"
            except:
                return "Invalid JSON in fallback mode"
        else:
            return f"Unexpected content-type: {content_type}"

    def validate_enhanced_data(self, response) -> bool:
        """Check if response has enhanced data with calculations"""
        try:
            data = response.json()
            if 'meta' in data:
                return "Enhanced data with meta"
            else:
                return "Fallback OHLCV data (no calculations yet)"
        except:
            return "Invalid enhanced data format"

    def run_all_tests(self):
        """Run comprehensive test suite"""
        print(f"🧪 Starting comprehensive API tests for: {self.base_url}")
        print("=" * 80)

        # 1. Basic Health Checks
        print("\n📊 HEALTH & STATUS TESTS")
        print("-" * 40)

        result = self.test_request(
            "Health Endpoint",
            "/health",
            validate_func=self.validate_health_response
        )
        self.results.append(result)
        self.print_result(result)

        result = self.test_request(
            "Ticker Groups",
            "/tickers/group",
            validate_func=self.validate_ticker_groups
        )
        self.results.append(result)
        self.print_result(result)

        # 2. Symbol Filtering Tests
        print("\n🎯 SYMBOL FILTERING TESTS")
        print("-" * 40)

        # Single symbol
        result = self.test_request(
            "Single Symbol (VCB)",
            "/tickers?symbol=VCB",
            validate_func=self.validate_single_symbol
        )
        self.results.append(result)
        self.print_result(result)

        # Multiple symbols
        result = self.test_request(
            "Multiple Symbols (3)",
            "/tickers?symbol=VCB&symbol=VIX&symbol=VNINDEX",
            validate_func=self.validate_multiple_symbols(3)
        )
        self.results.append(result)
        self.print_result(result)

        # Non-existent symbol
        result = self.test_request(
            "Non-existent Symbol",
            "/tickers?symbol=NONEXISTENT",
            validate_func=self.validate_empty_response
        )
        self.results.append(result)
        self.print_result(result)

        # Mixed existing/non-existing
        result = self.test_request(
            "Mixed Symbols (2 valid, 1 invalid)",
            "/tickers?symbol=VCB&symbol=NONEXISTENT&symbol=VIX",
            validate_func=self.validate_multiple_symbols(2)
        )
        self.results.append(result)
        self.print_result(result)

        # 3. Format Tests
        print("\n📄 FORMAT TESTS")
        print("-" * 40)

        # JSON format (default)
        result = self.test_request(
            "JSON Format (default)",
            "/tickers?symbol=VCB&format=json",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # CSV format
        result = self.test_request(
            "CSV Format",
            "/tickers?symbol=VCB&format=csv",
            validate_func=self.validate_csv_response
        )
        self.results.append(result)
        self.print_result(result)

        # 4. Date Range Tests
        print("\n📅 DATE RANGE TESTS")
        print("-" * 40)

        # All data
        result = self.test_request(
            "All Historical Data",
            "/tickers?symbol=VCB&all=true",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # Date range
        result = self.test_request(
            "Date Range Filter",
            "/tickers?symbol=VCB&start_date=2025-09-01&end_date=2025-09-30",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # Start date only
        result = self.test_request(
            "Start Date Only",
            "/tickers?symbol=VCB&start_date=2025-09-20",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # End date only
        result = self.test_request(
            "End Date Only",
            "/tickers?symbol=VCB&end_date=2025-09-25",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # 5. Enhanced Data Tests
        print("\n⚡ ENHANCED DATA TESTS")
        print("-" * 40)

        result = self.test_request(
            "Check Enhanced Data Status",
            "/tickers?symbol=VCB",
            validate_func=self.validate_enhanced_data
        )
        self.results.append(result)
        self.print_result(result)

        # 6. Performance Tests
        print("\n🚀 PERFORMANCE TESTS")
        print("-" * 40)

        # Response time test
        result = self.test_request(
            "Response Time Test",
            "/tickers?symbol=VCB&symbol=VIX",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        if result.response_time > 1.0:
            print(f"   ⚠️  Warning: Response time {result.response_time:.3f}s is > 1s")
        elif result.response_time < 0.1:
            print(f"   ✨ Excellent: Response time {result.response_time:.3f}s")

        # 7. Edge Cases
        print("\n🧩 EDGE CASE TESTS")
        print("-" * 40)

        # No parameters
        result = self.test_request(
            "No Parameters (should return all)",
            "/tickers",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # Invalid format
        result = self.test_request(
            "Invalid Format Parameter",
            "/tickers?symbol=VCB&format=xml",
            validate_func=self.validate_json_response  # Should fallback to JSON
        )
        self.results.append(result)
        self.print_result(result)

        # Invalid date format
        result = self.test_request(
            "Invalid Date Format",
            "/tickers?symbol=VCB&start_date=invalid-date",
            validate_func=self.validate_json_response  # Should ignore invalid date
        )
        self.results.append(result)
        self.print_result(result)

        # Empty symbol parameter
        result = self.test_request(
            "Empty Symbol Parameter",
            "/tickers?symbol=",
            validate_func=self.validate_json_response
        )
        self.results.append(result)
        self.print_result(result)

        # 8. Error Handling Tests
        print("\n❌ ERROR HANDLING TESTS")
        print("-" * 40)

        # Non-existent endpoint
        result = self.test_request(
            "Non-existent Endpoint",
            "/invalid-endpoint",
            expected_status=404
        )
        self.results.append(result)
        self.print_result(result)

        # Invalid HTTP method test would require different approach

        # 9. Load Test (simple)
        print("\n⚡ SIMPLE LOAD TEST")
        print("-" * 40)

        start_time = time.time()
        success_count = 0
        total_requests = 10

        for i in range(total_requests):
            result = self.test_request(
                f"Load Test Request {i+1}",
                "/tickers?symbol=VCB",
                validate_func=self.validate_json_response
            )
            if result.success:
                success_count += 1

        load_test_time = time.time() - start_time
        avg_time = load_test_time / total_requests

        load_result = TestResult(
            name=f"Load Test ({total_requests} requests)",
            success=success_count == total_requests,
            response_time=avg_time,
            details=f"{success_count}/{total_requests} successful, avg: {avg_time:.3f}s",
            status_code=200 if success_count == total_requests else 500
        )
        self.results.append(load_result)
        self.print_result(load_result)

        # Print final summary
        self.print_summary()

    def print_result(self, result: TestResult):
        """Print a single test result"""
        status = "✅ PASS" if result.success else "❌ FAIL"
        print(f"{status:8} | {result.name:30} | {result.response_time:6.3f}s | {result.details}")

    def print_summary(self):
        """Print final test summary"""
        print("\n" + "=" * 80)
        print("📋 TEST SUMMARY")
        print("=" * 80)

        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.success)
        failed_tests = total_tests - passed_tests

        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0
        avg_response_time = sum(r.response_time for r in self.results) / total_tests if total_tests > 0 else 0

        print(f"Total Tests:     {total_tests}")
        print(f"Passed:          {passed_tests} ✅")
        print(f"Failed:          {failed_tests} ❌")
        print(f"Success Rate:    {success_rate:.1f}%")
        print(f"Avg Response:    {avg_response_time:.3f}s")

        if failed_tests > 0:
            print(f"\n❌ FAILED TESTS:")
            for result in self.results:
                if not result.success:
                    print(f"   • {result.name}: {result.details}")

        print("\n" + "=" * 80)

        if success_rate >= 90:
            print("🎉 EXCELLENT: API is performing very well!")
        elif success_rate >= 75:
            print("👍 GOOD: API is mostly functional with minor issues")
        elif success_rate >= 50:
            print("⚠️  WARNING: API has significant issues")
        else:
            print("🚨 CRITICAL: API is not functioning properly")

        return success_rate >= 75

def main():
    parser = argparse.ArgumentParser(description='Comprehensive API test suite for aipriceaction-proxy')
    parser.add_argument('url', help='Base URL of the API (e.g., http://localhost:9000)')
    parser.add_argument('--timeout', type=float, default=10.0, help='Request timeout in seconds (default: 10)')

    args = parser.parse_args()

    # Validate URL format
    if not args.url.startswith(('http://', 'https://')):
        print("❌ Error: URL must start with http:// or https://")
        sys.exit(1)

    print(f"🔗 Testing API at: {args.url}")
    print(f"⏱️  Request timeout: {args.timeout}s")

    tester = APITester(args.url)

    try:
        success = tester.run_all_tests()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\n⏹️  Tests interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n💥 Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()