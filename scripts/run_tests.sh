#!/usr/bin/env bash
# scripts/run_tests.sh
# Partitioned test runner for community-simulation
# Allows running specific test suites or partitions locally and in CI for faster feedback.

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

usage() {
    cat << EOF
Usage: $(basename "$0") [SUITE|PARTITION] [OPTIONS]

Available test suites:
  unit                    Run all module unit tests (~790+ tests, ~1-2s)
  integration             Run all integration & scenario tests (~740 tests, ~10s)
  scenarios               Run scenario and engine integration tests (~320 tests, ~3-4s)
  coverage-suites         Run coverage breakthrough & stress suites (~410 tests, ~7s)
  doc                     Run documentation tests (~120 tests, ~2s)
  golden                  Run golden-file regression tests (7 tests, ~1.5s)
  all                     Run all tests sequentially (default)
  partition <idx> <total> Run 1-of-N partition of integration tests (e.g. partition 1 2)

Cargo aliases available:
  cargo test-unit                 (runs unit suite)
  cargo test-integration          (runs all integration tests)
  cargo test-integration-scenarios (runs scenario partition)
  cargo test-integration-coverage  (runs coverage partition)
  cargo test-doc                  (runs doctests)
  cargo test-golden               (runs golden tests)
  cargo test-fast                 (runs unit tests)

Options:
  --verbose, -v           Pass --verbose to cargo test
  --help, -h              Show this help message

Examples:
  ./scripts/run_tests.sh unit
  ./scripts/run_tests.sh scenarios
  ./scripts/run_tests.sh -v unit
  ./scripts/run_tests.sh partition 1 2
  ./scripts/run_tests.sh -v partition 1 2
EOF
}

TARGET=""
POSITIONAL_ARGS=()
CARGO_VERBOSE_FLAGS=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --verbose|-v)
            CARGO_VERBOSE_FLAGS=("--verbose")
            shift
            ;;
        --help|-h|help)
            usage
            exit 0
            ;;
        -*)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            if [ -z "${TARGET}" ]; then
                TARGET="$1"
            else
                POSITIONAL_ARGS+=("$1")
            fi
            shift
            ;;
    esac
done

TARGET="${TARGET:-all}"

run_timed() {
    local name="$1"
    shift
    local start_time
    start_time="$(date +%s)"
    printf "${BLUE}==>${NC} Running ${YELLOW}%s${NC}...\n" "${name}"
    
    if "$@"; then
        local end_time
        end_time="$(date +%s)"
        local elapsed=$((end_time - start_time))
        printf "${GREEN}✔${NC} %s finished successfully in ${GREEN}%ds${NC}\n\n" "${name}" "${elapsed}"
    else
        local end_time
        end_time="$(date +%s)"
        local elapsed=$((end_time - start_time))
        printf "${RED}✘${NC} %s failed after %ds\n\n" "${name}" "${elapsed}"
        return 1
    fi
}

run_cargo_test() {
    cargo ${CARGO_VERBOSE_FLAGS[@]+"${CARGO_VERBOSE_FLAGS[@]}"} "$@"
}

case "${TARGET}" in
    unit|test-unit)
        run_timed "Unit Tests" run_cargo_test test --lib -- --skip integration_tests
        ;;
    integration|test-integration)
        run_timed "Integration Tests" run_cargo_test test --lib -- integration_tests
        ;;
    scenarios|integration-scenarios|test-integration-scenarios)
        run_timed "Scenario & Engine Integration Tests" run_cargo_test test --lib -- integration_tests:: \
            --skip integration_tests::coverage \
            --skip integration_tests::final \
            --skip integration_tests::ultimate \
            --skip integration_tests::ultra \
            --skip integration_tests::laser
        ;;
    coverage-suites|integration-coverage|test-integration-coverage)
        run_timed "Coverage & Stress Integration Tests" run_cargo_test test --lib -- \
            integration_tests::coverage \
            integration_tests::final \
            integration_tests::ultimate \
            integration_tests::ultra \
            integration_tests::laser
        ;;
    doc|test-doc)
        run_timed "Documentation Tests" run_cargo_test test --doc
        ;;
    golden|test-golden)
        run_timed "Golden-File Regression Tests" run_cargo_test test --test golden_regression -- --nocapture
        ;;
    partition)
        PART_IDX="${POSITIONAL_ARGS[0]:-1}"
        PART_TOTAL="${POSITIONAL_ARGS[1]:-2}"
        if [ "${PART_TOTAL}" -eq 2 ]; then
            if [ "${PART_IDX}" -eq 1 ]; then
                run_timed "Partition 1/2 (Scenario & Engine Tests)" run_cargo_test test --lib -- integration_tests:: \
                    --skip integration_tests::coverage \
                    --skip integration_tests::final \
                    --skip integration_tests::ultimate \
                    --skip integration_tests::ultra \
                    --skip integration_tests::laser
            elif [ "${PART_IDX}" -eq 2 ]; then
                run_timed "Partition 2/2 (Coverage & Stress Tests)" run_cargo_test test --lib -- \
                    integration_tests::coverage \
                    integration_tests::final \
                    integration_tests::ultimate \
                    integration_tests::ultra \
                    integration_tests::laser
            else
                echo "Error: Invalid partition index '${PART_IDX}'. Must be between 1 and ${PART_TOTAL}."
                exit 1
            fi
        else
            echo "Error: Only 2-way partitions are currently configured for integration tests."
            exit 1
        fi
        ;;
    all|test-all)
        printf "${BLUE}Running all test suites partitioned...${NC}\n\n"
        run_timed "1. Unit Tests" run_cargo_test test --lib -- --skip integration_tests
        run_timed "2. Scenario & Engine Tests" run_cargo_test test --lib -- integration_tests:: \
            --skip integration_tests::coverage \
            --skip integration_tests::final \
            --skip integration_tests::ultimate \
            --skip integration_tests::ultra \
            --skip integration_tests::laser
        run_timed "3. Coverage & Stress Tests" run_cargo_test test --lib -- \
            integration_tests::coverage \
            integration_tests::final \
            integration_tests::ultimate \
            integration_tests::ultra \
            integration_tests::laser
        run_timed "4. Golden Regression Tests" run_cargo_test test --test golden_regression -- --nocapture
        run_timed "5. Documentation Tests" run_cargo_test test --doc
        printf "${GREEN}✔ All test suites passed!${NC}\n"
        ;;
    *)
        echo "Unknown suite: ${TARGET}"
        usage
        exit 1
        ;;
esac
