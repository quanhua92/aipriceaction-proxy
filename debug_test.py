#!/usr/bin/env python3

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from test_api_comprehensive import APITester

def test_return_value():
    print("=== Testing APITester return value ===")

    tester = APITester("http://localhost:9000")
    print(f"Created tester: {tester}")

    try:
        print("Calling run_all_tests()...")
        result = tester.run_all_tests()
        print(f"Result: {result}, type: {type(result)}")
        print(f"Last success rate: {tester.last_success_rate}")
        return result
    except Exception as e:
        print(f"Exception caught: {e}")
        import traceback
        traceback.print_exc()
        return None

if __name__ == "__main__":
    test_return_value()