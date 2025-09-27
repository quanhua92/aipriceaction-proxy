#!/usr/bin/env python3

import requests
import time
import json
import sys
import argparse
from dataclasses import dataclass
from typing import Optional, Callable, Dict, Any
from datetime import timedelta
import concurrent.futures

@dataclass
class TestResult:
    test_name: str
    success: bool
    response_time: float
    status_code: int
    error_message: Optional[str] = None

class APITester:
    def __init__(self, base_url: str, timeout: float = 10.0):
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.results = []
        self.last_success_rate = 0.0

    def make_request(self, endpoint: str, expected_status: int = 200) -> requests.Response:
        """Make HTTP request with timing"""
        url = f"{self.base_url}{endpoint}"
        start_time = time.time()
        response = requests.get(url, timeout=self.timeout)
        response.elapsed = timedelta(seconds=time.time() - start_time)
        return response

    def test_request(self, test_name: str, endpoint: str, expected_status: int = 200, validate_func: Optional[Callable] = None) -> TestResult:
        """Execute a single test request"""
        try:
            response = self.make_request(endpoint, expected_status)
            response_time = response.elapsed.total_seconds()

            # Check status code
            status_ok = response.status_code == expected_status

            # Run validation function if provided
            validation_ok = True
            error_msg = None
            if validate_func and status_ok:
                try:
                    validation_ok = validate_func(response)
                except Exception as e:
                    validation_ok = False
                    error_msg = f"Validation error: {str(e)}"

            success = status_ok and validation_ok
            if not status_ok:
                error_msg = f"Expected status {expected_status}, got {response.status_code}"

            return TestResult(
                test_name=test_name,
                success=success,
                response_time=response_time,
                status_code=response.status_code,
                error_message=error_msg
            )

        except Exception as e:
            return TestResult(
                test_name=test_name,
                success=False,
                response_time=0.0,
                status_code=0,
                error_message=str(e)
            )

    def print_result(self, result: TestResult):
        """Print test result"""
        status = "✅ PASS" if result.success else "❌ FAIL"
        print(f"{status:8} | {result.test_name:30} | {result.response_time:6.3f}s | {result.error_message or 'OK'}")

    # Validation functions
    def validate_health_response(self, response: requests.Response) -> bool:
        try:
            data = response.json()
            return 'memory_usage_mb' in data and 'active_tickers_count' in data
        except:
            return False

    def validate_ticker_groups(self, response: requests.Response) -> bool:
        try:
            data = response.json()
            return isinstance(data, dict) and len(data) > 0
        except:
            return False

    def validate_json_response(self, response: requests.Response) -> bool:
        try:
            data = response.json()
            return isinstance(data, (dict, list))
        except:
            return False

    def validate_csv_response(self, response: requests.Response) -> bool:
        content_type = response.headers.get('content-type', '')
        return 'text/csv' in content_type and 'date,symbol' in response.text[:100]

    def validate_single_symbol(self, response: requests.Response) -> bool:
        try:
            data = response.json()
            if isinstance(data, dict):
                # Check for enhanced format
                if 'data' in data:
                    return len(data['data']) == 1
                # Check for direct format
                return len(data) == 1
            return False
        except:
            return False

    def validate_multiple_symbols(self, expected_count: int):
        def validator(response: requests.Response) -> bool:
            try:
                data = response.json()
                if isinstance(data, dict):
                    # Check for enhanced format
                    if 'data' in data:
                        return len(data['data']) == expected_count
                    # Check for direct format
                    return len(data) == expected_count
                return False
            except:
                return False
        return validator

    def validate_empty_response(self, response: requests.Response) -> bool:
        try:
            data = response.json()
            if isinstance(data, dict):
                # Check for enhanced format
                if 'data' in data:
                    return len(data['data']) == 0
                # Check for direct format
                return len(data) == 0
            return isinstance(data, list) and len(data) == 0
        except:
            return False

    def display_enhanced_data_samples(self):
        """Display sample enhanced data values for major tickers"""
        major_tickers = ["VCB", "VIX", "VNINDEX"]

        print("\n📊 ENHANCED DATA SAMPLES (Last 3 Data Points)")
        print("=" * 80)

        for ticker in major_tickers:
            try:
                response = requests.get(f"{self.base_url}/tickers?symbol={ticker}&all=true", timeout=10.0)
                if response.status_code == 200:
                    data = response.json()

                    # Check if it's enhanced data format
                    if 'data' in data and ticker in data['data']:
                        ticker_data = data['data'][ticker]
                        if len(ticker_data) > 0:
                            print(f"\n🔹 {ticker}:")
                            # Show last 3 data points
                            for i, point in enumerate(ticker_data[-3:]):
                                date = point.get('date', 'N/A')
                                close = point.get('close', 'N/A')
                                mf = point.get('money_flow', 'N/A')
                                ma10 = point.get('ma10', 'N/A')
                                ma20 = point.get('ma20', 'N/A')
                                score10 = point.get('score10', 'N/A')

                                # Format values
                                close_str = f"{close:.2f}" if isinstance(close, (int, float)) else str(close)
                                mf_str = f"{mf:.2f}" if isinstance(mf, (int, float)) else str(mf)
                                ma10_str = f"{ma10:.2f}" if isinstance(ma10, (int, float)) else str(ma10)
                                ma20_str = f"{ma20:.2f}" if isinstance(ma20, (int, float)) else str(ma20)
                                score10_str = f"{score10:.2f}" if isinstance(score10, (int, float)) else str(score10)

                                print(f"   {date}: Close={close_str}, MF={mf_str}, MA10={ma10_str}, MA20={ma20_str}, Score10={score10_str}")
                        else:
                            print(f"\n🔹 {ticker}: No data available")
                    elif isinstance(data, dict) and ticker in data:
                        # Fallback OHLCV format
                        ticker_data = data[ticker]
                        if len(ticker_data) > 0:
                            print(f"\n🔹 {ticker} (OHLCV only - no enhanced calculations):")
                            for point in ticker_data[-3:]:
                                time_str = point.get('time', 'N/A')
                                close = point.get('close', 'N/A')
                                close_str = f"{close:.2f}" if isinstance(close, (int, float)) else str(close)
                                print(f"   {time_str}: Close={close_str}, MF=N/A, MA=N/A, Score=N/A")
                        else:
                            print(f"\n🔹 {ticker}: No data available")
                else:
                    print(f"\n🔹 {ticker}: HTTP {response.status_code}")
            except Exception as e:
                print(f"\n🔹 {ticker}: Error - {e}")

    # Test categories broken into small methods
    def run_health_status_tests(self):
        """Run health and status endpoint tests"""
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

    def run_symbol_filtering_tests(self):
        """Run symbol filtering tests"""
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

    def run_format_tests(self):
        """Run format tests (JSON/CSV)"""
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

    def run_date_range_tests(self):
        """Run date range filtering tests"""
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

    def run_performance_tests(self):
        """Run performance tests"""
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

    def run_edge_case_tests(self):
        """Run edge case tests"""
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

    def run_error_handling_tests(self):
        """Run error handling tests"""
        print("\n❌ ERROR HANDLING TESTS")
        print("-" * 40)

        # Non-existent endpoint
        result = self.test_request(
            "Non-existent Endpoint",
            "/nonexistent",
            expected_status=404
        )
        self.results.append(result)
        self.print_result(result)

    def run_load_tests(self):
        """Run simple load test"""
        print("\n⚡ SIMPLE LOAD TEST")
        print("-" * 40)

        # Load test with concurrent requests
        start_time = time.time()
        test_url = "/tickers?symbol=VCB"
        concurrent_requests = 10
        success_count = 0
        total_response_time = 0

        with concurrent.futures.ThreadPoolExecutor(max_workers=concurrent_requests) as executor:
            future_to_request = {executor.submit(self.make_request, test_url): i for i in range(concurrent_requests)}
            for future in concurrent.futures.as_completed(future_to_request):
                try:
                    response = future.result()
                    if response.status_code == 200:
                        success_count += 1
                        total_response_time += getattr(response, 'elapsed', timedelta()).total_seconds()
                except Exception as e:
                    pass

        avg_response_time = total_response_time / max(success_count, 1)
        load_test_result = TestResult(
            test_name="Load Test (10 requests)",
            success=success_count == concurrent_requests,
            response_time=time.time() - start_time,
            status_code=200 if success_count == concurrent_requests else 500,
            error_message=None if success_count == concurrent_requests else f"Only {success_count}/{concurrent_requests} succeeded"
        )
        self.results.append(load_test_result)
        self.print_result(load_test_result)

        if success_count == concurrent_requests:
            print(f"   ✅ {success_count}/{concurrent_requests} successful, avg: {avg_response_time:.3f}s")
        else:
            print(f"   ❌ Only {success_count}/{concurrent_requests} successful")

    def calculate_and_print_summary(self):
        """Calculate and print test summary"""
        print("\n" + "=" * 80)
        print("📋 TEST SUMMARY")
        print("=" * 80)

        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.success)
        failed_tests = total_tests - passed_tests
        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0

        avg_response_time = sum(r.response_time for r in self.results) / len(self.results) if self.results else 0

        print(f"Total Tests:     {total_tests}")
        print(f"Passed:          {passed_tests} ✅")
        print(f"Failed:          {failed_tests} ❌")
        print(f"Success Rate:    {success_rate:.1f}%")
        print(f"Avg Response:    {avg_response_time:.3f}s")

        print("\n" + "=" * 80)
        if success_rate >= 90:
            print("🎉 EXCELLENT: API is performing very well!")
        elif success_rate >= 75:
            print("👍 GOOD: API is mostly functional with minor issues")
        elif success_rate >= 50:
            print("⚠️  WARNING: API has significant issues")
        else:
            print("🚨 CRITICAL: API is not functioning properly")

        return success_rate

    def run_all_tests(self):
        """Run comprehensive test suite - CLEAN REFACTORED VERSION"""
        print(f"🧪 Starting comprehensive API tests for: {self.base_url}")
        print("=" * 80)

        # Clear previous results
        self.results = []

        # Run all test categories
        self.run_health_status_tests()
        self.run_symbol_filtering_tests()
        self.run_format_tests()
        self.run_date_range_tests()
        self.run_performance_tests()
        self.run_edge_case_tests()
        self.run_error_handling_tests()
        self.run_load_tests()

        # Calculate and display results
        success_rate = self.calculate_and_print_summary()

        # Store for debugging and return result
        self.last_success_rate = success_rate
        result = success_rate >= 75
        print(f"🔧 DEBUG: Returning {result} (success_rate={success_rate:.1f}%)")
        return result

    def run_enhanced_tests(self):
        """Run enhanced data specific tests after calculations complete"""
        print(f"🧪 Starting enhanced data tests for: {self.base_url}")
        print("=" * 80)

        # Clear previous results for enhanced tests
        self.results = []

        print("\n🔬 ENHANCED DATA CALCULATION TESTS")
        print("-" * 40)

        # Test enhanced data with calculations
        result = self.test_request(
            "Enhanced Data with Calculations",
            "/tickers?symbol=VCB&all=true",
            validate_func=self.validate_enhanced_data
        )
        self.results.append(result)
        self.print_result(result)

        if result.success:
            # Display sample data if enhanced calculations are available
            self.display_enhanced_data_samples()

        # Test CSV format with enhanced data
        result = self.test_request(
            "Enhanced CSV Format",
            "/tickers?symbol=VCB&symbol=VIX&format=csv&all=true",
            validate_func=self.validate_enhanced_csv
        )
        self.results.append(result)
        self.print_result(result)

        # Calculate results
        success_rate = self.calculate_and_print_summary()
        self.last_success_rate = success_rate
        return success_rate >= 75

    def validate_enhanced_data(self, response: requests.Response) -> bool:
        """Validate that response contains enhanced calculations"""
        try:
            data = response.json()

            # Check for enhanced format with metadata
            if 'data' in data and 'meta' in data:
                meta = data['meta']
                if meta.get('calculated', False):
                    # Check if enhanced calculations exist
                    ticker_data = data['data']
                    for symbol, points in ticker_data.items():
                        if points:
                            first_point = points[0]
                            # Check for enhanced fields
                            has_enhanced = any(field in first_point for field in
                                             ['money_flow', 'ma10', 'ma20', 'score10', 'score20'])
                            if has_enhanced:
                                return True
            return False
        except:
            return False

    def validate_enhanced_csv(self, response: requests.Response) -> bool:
        """Validate enhanced CSV format"""
        try:
            content_type = response.headers.get('content-type', '')
            if 'text/csv' not in content_type:
                return False

            # Check for enhanced CSV headers
            text = response.text
            header_line = text.split('\n')[0] if text else ""
            enhanced_fields = ['ma10', 'ma20', 'money_flow', 'score10']
            return any(field in header_line for field in enhanced_fields)
        except:
            return False

def check_server_connectivity(url: str, timeout: float = 5.0) -> bool:
    """Check if server is running"""
    try:
        response = requests.get(f"{url}/health", timeout=timeout)
        return response.status_code == 200
    except:
        return False

def main():
    parser = argparse.ArgumentParser(description='Comprehensive API test suite')
    parser.add_argument('url', help='Base URL of the API to test')
    parser.add_argument('--timeout', type=float, default=10.0, help='Request timeout in seconds')
    parser.add_argument('--single-run', action='store_true', help='Run tests once without waiting for enhanced calculations')

    args = parser.parse_args()

    # Print current time
    from datetime import datetime
    current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"🕐 Test started at: {current_time}")

    # Check server connectivity first
    print(f"🔗 Checking server connectivity to {args.url}...")
    if not check_server_connectivity(args.url, timeout=5.0):
        print(f"\n❌ FAILED: Cannot connect to server at {args.url}")
        print("\n💡 To start the server, run one of these commands:")
        print("   # For development (with tokens):")
        print('   PRIMARY_TOKEN="test-token-1" SECONDARY_TOKEN="test-token-2" INTERNAL_PEER_URLS="" PORT=9000 RUST_LOG=info cargo run')
        print("\n   # For production:")
        print('   PRIMARY_TOKEN="your-primary-token" SECONDARY_TOKEN="your-secondary-token" PORT=9000 cargo run --release')
        sys.exit(1)

    # Run startup tests (basic functionality)
    startup_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"\n🚀 PHASE 1: STARTUP TESTS (Basic API functionality) - {startup_time}")
    print("=" * 80)

    tester1 = APITester(args.url)
    try:
        success1 = tester1.run_all_tests()
        print(f"\n🔍 DEBUG: Phase 1 result = {success1}, type = {type(success1)}")
    except Exception as e:
        print(f"\n❌ ERROR during startup tests: {e}")
        sys.exit(1)

    # Exit immediately if startup tests failed
    if not success1:
        print("\n❌ STARTUP TESTS FAILED - Exiting without running enhanced tests")
        print("   Please fix the issues above before proceeding")
        sys.exit(1)

    print("\n✅ STARTUP TESTS PASSED - Proceeding to enhanced tests")

    # Skip enhanced tests if single-run mode
    if args.single_run:
        print("\n⏭️  SINGLE-RUN MODE: Skipping enhanced data tests")
        print(f"\nOverall startup success rate: {tester1.last_success_rate:.1f}%")
        sys.exit(0 if success1 else 1)

    # Check if enhanced calculations are already available
    check_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"\n🔍 PHASE 2: Checking for enhanced calculations... - {check_time}")

    def check_enhanced_data_available():
        """Check if enhanced calculations (MF, MA) are already available"""
        try:
            test_response = requests.get(f"{args.url}/tickers?symbol=VCB&all=true", timeout=5.0)
            if test_response.status_code == 200:
                data = test_response.json()
                # Check for enhanced format with metadata
                if 'data' in data and 'meta' in data:
                    meta = data['meta']
                    if meta.get('calculated', False):
                        # Check if enhanced calculations exist
                        ticker_data = data['data']
                        for symbol, points in ticker_data.items():
                            if points:
                                first_point = points[0]
                                # Check for enhanced fields
                                has_enhanced = any(field in first_point for field in
                                                 ['money_flow', 'ma10', 'ma20', 'score10'])
                                if has_enhanced:
                                    return True
            return False
        except:
            return False

    enhanced_available = check_enhanced_data_available()

    if enhanced_available:
        ready_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"✅ Enhanced calculations already available! Skipping wait - {ready_time}")
        print("   Found: Money flow, moving averages, technical scores")
    else:
        # Wait for enhanced calculations to complete
        wait_start_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"⏳ Enhanced calculations not ready yet. Waiting 90 seconds... - {wait_start_time}")
        print("   (Money flow, moving averages, technical scores)")

        # Show progress every 5 seconds and check for enhanced data availability
        for i in range(18):  # 90 seconds / 5 seconds = 18 intervals
            time.sleep(5)
            elapsed = (i + 1) * 5
            remaining = 90 - elapsed
            current_time = datetime.now().strftime("%H:%M:%S")

            # Check if enhanced data is now available
            if check_enhanced_data_available():
                print(f"   ✅ {current_time} - Enhanced data detected after {elapsed}s! Stopping wait early.")
                break

            print(f"   ⏱️  {current_time} - {elapsed}s elapsed, {remaining}s remaining... (checking for data)")

            if remaining <= 0:
                break

    # Run enhanced tests
    enhanced_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"\n🚀 PHASE 2: ENHANCED DATA TESTS (After calculations) - {enhanced_time}")
    print("=" * 80)

    tester2 = APITester(args.url)
    try:
        success2 = tester2.run_enhanced_tests()
        print(f"\n🔍 DEBUG: Phase 2 result = {success2}, type = {type(success2)}")
    except Exception as e:
        print(f"\n❌ ERROR during enhanced tests: {e}")
        success2 = False

    # Final summary
    completion_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"\n" + "=" * 80)
    print(f"🏁 FINAL SUMMARY - {completion_time}")
    print("=" * 80)
    print(f"Startup Tests:   {'✅ PASSED' if success1 else '❌ FAILED'} ({tester1.last_success_rate:.1f}%)")
    print(f"Enhanced Tests:  {'✅ PASSED' if success2 else '❌ FAILED'} ({tester2.last_success_rate:.1f}%)")

    overall_success = success1 and success2
    print(f"Overall Result:  {'✅ SUCCESS' if overall_success else '❌ FAILURE'}")

    sys.exit(0 if overall_success else 1)

if __name__ == "__main__":
    main()