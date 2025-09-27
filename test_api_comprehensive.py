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
        self.last_success_rate = 0.0

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

    def validate_enhanced_data(self, response) -> str:
        """Check if response has enhanced data with calculations"""
        try:
            data = response.json()
            if 'meta' in data and 'data' in data:
                # Check if enhanced calculations exist
                sample_ticker = next(iter(data['data'].values()), [])
                if sample_ticker and len(sample_ticker) > 0:
                    sample_point = sample_ticker[0]
                    has_mf = 'money_flow' in sample_point and sample_point['money_flow'] is not None
                    has_ma = 'ma10' in sample_point and sample_point['ma10'] is not None
                    has_scores = 'score10' in sample_point and sample_point['score10'] is not None

                    if has_mf and has_ma and has_scores:
                        return "Enhanced data with mf, ma, and scores"
                    elif has_mf or has_ma or has_scores:
                        return "Partial enhanced data (some calculations missing)"
                    else:
                        return "Enhanced structure but no calculations"
                return "Enhanced data with meta but no ticker data"
            else:
                return "Fallback OHLCV data (no calculations yet)"
        except:
            return "Invalid enhanced data format"

    def validate_enhanced_calculations(self, response) -> bool:
        """Validate that enhanced calculations exist (mf, ma, scores)"""
        try:
            data = response.json()
            if 'data' in data:
                for ticker_data in data['data'].values():
                    if ticker_data and len(ticker_data) > 0:
                        sample_point = ticker_data[0]
                        has_mf = 'money_flow' in sample_point and sample_point['money_flow'] is not None
                        has_ma = 'ma10' in sample_point and sample_point['ma10'] is not None
                        has_scores = 'score10' in sample_point and sample_point['score10'] is not None
                        return has_mf and has_ma and has_scores
            return False
        except:
            return False

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

        # Store success rate for debugging
        self.last_success_rate = success_rate
        result = success_rate >= 75
        print(f"🔧 DEBUG: About to return result = {result}, success_rate = {success_rate}")
        return result

    def run_enhanced_tests(self):
        """Run enhanced data specific tests after calculations complete"""
        print(f"🧪 Starting enhanced data tests for: {self.base_url}")
        print("=" * 80)

        # Skip running basic tests again since they were already run in phase 1
        # basic_success = self.run_all_tests()

        # Reset results for enhanced-specific tests
        enhanced_results = []

        print("\n🔬 ENHANCED DATA SPECIFIC TESTS")
        print("-" * 40)

        # Test enhanced data presence and calculations
        result = self.test_request(
            "Enhanced Data - Money Flow Present",
            "/tickers?symbol=VCB",
            validate_func=lambda r: "money_flow" in str(r.text) and '"money_flow":null' not in str(r.text)
        )
        enhanced_results.append(result)
        self.print_result(result)

        result = self.test_request(
            "Enhanced Data - MA Scores Present",
            "/tickers?symbol=VCB",
            validate_func=lambda r: "ma10" in str(r.text) and '"ma10":null' not in str(r.text)
        )
        enhanced_results.append(result)
        self.print_result(result)

        result = self.test_request(
            "Enhanced Data - Technical Scores Present",
            "/tickers?symbol=VCB",
            validate_func=lambda r: "score10" in str(r.text) and '"score10":null' not in str(r.text)
        )
        enhanced_results.append(result)
        self.print_result(result)

        result = self.test_request(
            "Enhanced Data - Full Calculations",
            "/tickers?symbol=VCB",
            validate_func=self.validate_enhanced_calculations
        )
        enhanced_results.append(result)
        self.print_result(result)

        # CSV format with enhanced data
        result = self.test_request(
            "Enhanced CSV Format",
            "/tickers?symbol=VCB&format=csv",
            validate_func=lambda r: "money_flow" in r.text and "ma10" in r.text
        )
        enhanced_results.append(result)
        self.print_result(result)

        # Calculate enhanced test results
        enhanced_passed = sum(1 for r in enhanced_results if r.success)
        enhanced_total = len(enhanced_results)
        enhanced_rate = (enhanced_passed / enhanced_total) * 100 if enhanced_total > 0 else 0

        print("\n" + "=" * 80)
        print("📋 ENHANCED TEST SUMMARY")
        print("=" * 80)
        print(f"Enhanced Tests:  {enhanced_passed}/{enhanced_total} passed ({enhanced_rate:.1f}%)")

        # Combine all results
        all_results = self.results + enhanced_results
        total_passed = sum(1 for r in all_results if r.success)
        total_tests = len(all_results)
        overall_rate = (total_passed / total_tests) * 100 if total_tests > 0 else 0

        print(f"Overall Tests:   {total_passed}/{total_tests} passed ({overall_rate:.1f}%)")

        if enhanced_rate >= 90:
            print("🎉 EXCELLENT: Enhanced calculations are working perfectly!")
        elif enhanced_rate >= 75:
            print("👍 GOOD: Enhanced calculations are mostly working")
        elif enhanced_rate >= 50:
            print("⚠️  WARNING: Enhanced calculations have significant issues")
        else:
            print("🚨 CRITICAL: Enhanced calculations are not working")

        return overall_rate >= 75

def main():
    parser = argparse.ArgumentParser(description='Comprehensive API test suite for aipriceaction-proxy')
    parser.add_argument('url', help='Base URL of the API (e.g., http://localhost:9000)')
    parser.add_argument('--timeout', type=float, default=10.0, help='Request timeout in seconds (default: 10)')
    parser.add_argument('--single-run', action='store_true', help='Run single test only (skip 30s wait)')

    args = parser.parse_args()

    # Validate URL format
    if not args.url.startswith(('http://', 'https://')):
        print("❌ Error: URL must start with http:// or https://")
        sys.exit(1)

    print(f"🔗 Testing API at: {args.url}")
    print(f"⏱️  Request timeout: {args.timeout}s")

    if args.single_run:
        # Single test run only
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
    else:
        # Default: startup test sequence
        print("🚀 Running startup test sequence...")

        # Quick server connectivity check first
        print("\n🔍 Checking server connectivity...")
        try:
            test_response = requests.get(f"{args.url}/health", timeout=3)
            print("✅ Server is responding")
        except requests.exceptions.ConnectionError:
            print("❌ ERROR: Cannot connect to server - server may not be running")
            print(f"   Please make sure the server is running at {args.url}")
            sys.exit(1)
        except requests.exceptions.Timeout:
            print("❌ ERROR: Server response timeout - server may be overloaded")
            sys.exit(1)
        except Exception as e:
            print(f"❌ ERROR: Unexpected connection error: {e}")
            sys.exit(1)

        # First test run during startup
        print("\n" + "="*80)
        print("📊 PHASE 1: STARTUP TESTS (Immediate - Fallback Mode Expected)")
        print("="*80)

        tester1 = APITester(args.url)
        try:
            success1 = tester1.run_all_tests()
        except Exception as e:
            print(f"\n❌ ERROR during startup tests: {e}")
            sys.exit(1)

        # Debug: Print actual return value
        print(f"\n🔍 DEBUG: success1 = {success1}, type = {type(success1)}, tester1.last_success_rate = {tester1.last_success_rate}")

        # Exit immediately if startup tests failed
        if not success1:
            print("\n❌ STARTUP TESTS FAILED - Exiting without running enhanced tests")
            print("   Please fix the issues above before proceeding")
            sys.exit(1)

        print("\n✅ STARTUP TESTS PASSED - Proceeding to enhanced tests")

        # Wait 90 seconds for calculations to complete
        print(f"\n⏳ Waiting 90 seconds for enhanced calculations to complete...")
        for i in range(90, 0, -1):
            print(f"\r   {i:2d}s remaining...", end="", flush=True)
            time.sleep(1)
        print("\r   ✅ Wait complete!    ")

        # Second test run after delay
        print("\n" + "="*80)
        print("📊 PHASE 2: POST-STARTUP TESTS (After 30s - Enhanced Mode Expected)")
        print("="*80)

        tester2 = APITester(args.url)
        success2 = tester2.run_all_tests()

        # Compare results
        print("\n" + "="*80)
        print("📈 COMPARISON SUMMARY")
        print("="*80)

        phase1_passed = sum(1 for r in tester1.results if r.success)
        phase1_total = len(tester1.results)
        phase2_passed = sum(1 for r in tester2.results if r.success)
        phase2_total = len(tester2.results)

        print(f"Phase 1 (Startup):     {phase1_passed}/{phase1_total} passed ({phase1_passed/phase1_total*100:.1f}%)")
        print(f"Phase 2 (Enhanced):    {phase2_passed}/{phase2_total} passed ({phase2_passed/phase2_total*100:.1f}%)")
        print(f"Improvement:           +{phase2_passed-phase1_passed} tests passed")

        # Show specific improvements
        phase1_failed = {r.name for r in tester1.results if not r.success}
        phase2_failed = {r.name for r in tester2.results if not r.success}
        improvements = phase1_failed - phase2_failed

        if improvements:
            print(f"\n✅ Tests that improved after 30s:")
            for test_name in improvements:
                print(f"   • {test_name}")

        regressions = phase2_failed - phase1_failed
        if regressions:
            print(f"\n❌ Tests that regressed after 30s:")
            for test_name in regressions:
                print(f"   • {test_name}")

        # Final result
        overall_success = success2 and (phase2_passed >= phase1_passed)
        sys.exit(0 if overall_success else 1)

if __name__ == "__main__":
    main()