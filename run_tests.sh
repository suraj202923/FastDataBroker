#!/bin/bash
# Cross-platform test runner wrapper
# Usage: ./run_tests.sh [--category all|unit|python|go|java|javascript] [-v] [--pattern PATTERN]

cd "$(dirname "$0")"
python3 scripts/run_tests.py "$@"
