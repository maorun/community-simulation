# Code Coverage Guide

This document explains how to generate and view code coverage reports for the simulation framework.

## Coverage Goals

This project aims for **100% code coverage** as the ultimate goal, following a progressive improvement approach:

- **Current baseline:** 56% (as of 2026-02)
- **Next milestone:** 70%
- **Short-term target:** 80%
- **Ultimate goal:** 100%

### Why 100% Coverage?

While 100% code coverage doesn't guarantee bug-free code, it ensures:
- All code paths are exercised by tests
- Edge cases and error handling are validated
- Refactoring is safer with comprehensive test coverage
- Code quality and maintainability are improved
- Better documentation through test examples

### Progressive Approach

We follow a pragmatic, progressive approach:
1. **Maintain minimum baseline** (56%): CI fails if coverage drops below this
2. **Improve incrementally**: Each PR should ideally increase or maintain coverage
3. **Target high-value areas first**: Focus on business logic, algorithms, and public APIs
4. **Achieve milestones**: 70% → 80% → 90% → 100%

## Prerequisites

Install `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin
```

## Generating Coverage Reports

### Basic Coverage Report

Generate a basic coverage report in the terminal:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300
```

This will:
- Run all tests with coverage instrumentation
- Show coverage percentages for each file
- Display overall project coverage

### HTML Coverage Report

Generate an HTML report for detailed, browsable coverage:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out html --output-dir ./coverage
```

Then open `coverage/index.html` in your browser to explore:
- Line-by-line coverage for each file
- Uncovered code paths highlighted in red
- Partially covered branches shown in yellow
- Fully covered code shown in green

### XML Coverage Report (for CI)

Generate an XML report compatible with coverage services:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out xml --output-dir ./coverage
```

This creates `coverage/cobertura.xml` which can be uploaded to Codecov or similar services.

## Coverage Threshold

The project follows a progressive coverage improvement strategy:

**Current Requirements:**
- **Minimum threshold:** 56% (CI fails below this)
- **Target:** 80% (recommended for all new features)
- **Goal:** 100% (ultimate objective)

You can check if your changes meet the minimum threshold:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --fail-under 56
```

For new features, aim for the target threshold:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --fail-under 80
```

These commands will:
- Exit with code 0 if coverage meets the threshold
- Exit with code 1 if coverage is below the threshold

### Progressive Improvement Strategy

As the codebase coverage improves, the minimum threshold will be gradually increased:
- When overall coverage reaches 70%, minimum will increase to 65%
- When overall coverage reaches 80%, minimum will increase to 75%
- When overall coverage reaches 90%, minimum will increase to 85%
- Final goal: 100% coverage with 95% minimum threshold

## Continuous Integration

Coverage is automatically tracked and enforced in CI:
- Every push to `master` and every PR generates a coverage report
- **CI will fail** if coverage drops below the minimum threshold (56%)
- Reports are uploaded to Codecov with strict requirements (no coverage decreases allowed)
- Coverage badges in README.md show current coverage status
- Coverage summaries are automatically added to PR checks
- **New code** should aim for 80%+ coverage (enforced by Codecov patch checks)
- **Caching:** The CI workflow caches cargo-tarpaulin binary and cargo dependencies to significantly speed up subsequent runs (typically 2-3 minutes faster)

## Understanding Coverage Metrics

### Line Coverage
Percentage of executable lines that were run during tests. This is the primary metric.

### Branch Coverage
Percentage of conditional branches (if/else, match arms) that were executed.

### Function Coverage
Percentage of functions that were called at least once during tests.

## Improving Coverage

To find uncovered code and reach our 100% coverage goal:

1. Generate HTML report: `cargo tarpaulin --out html --output-dir ./coverage`
2. Open `coverage/index.html` in browser
3. Navigate to files with low coverage
4. Red-highlighted lines show uncovered code
5. Write tests to exercise those code paths

### Priority Areas for Coverage Improvement

Focus on these areas to maximize impact:

**High Priority (Critical Business Logic):**
- Core simulation engine (`src/engine.rs`) - currently 57%
- Configuration handling (`src/config.rs`) - currently 55%
- Market mechanisms (`src/market.rs`) - currently 98% ✓
- Person behavior (`src/person.rs`) - currently 72%

**Medium Priority (Features and Modules):**
- Scenario implementations (`src/scenario.rs`) - currently 62%
- Result analysis (`src/result.rs`) - currently 81%
- Error handling (`src/error.rs`) - currently 21%
- Plugin system (`src/plugin.rs`) - currently 93%

**Lower Priority (UI and Utilities):**
- Main entry point (`src/main.rs`) - currently 0%
- Wizard/CLI (`src/wizard.rs`) - currently 0%
- Database operations (`src/database.rs`) - currently 100% ✓

### Coverage Improvement Strategies

Focus on:
- Error handling paths (match Err branches)
- Edge cases (empty collections, zero values)
- Conditional logic (if/else branches)
- Public API functions (higher priority than internal helpers)

## Excluding Code from Coverage

To exclude code from coverage metrics, use `#[cfg(not(tarpaulin_include))]`:

```rust
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // This won't count against coverage
}
```

Or exclude entire modules:

```rust
#[cfg(not(tarpaulin_include))]
mod debugging_utilities {
    // Entire module excluded
}
```

Use this sparingly and only for:
- Debug-only code
- Platform-specific code that can't be tested in CI
- Generated code

## Troubleshooting

### Timeout Errors

If tests timeout during coverage collection, increase the timeout:

```bash
cargo tarpaulin --timeout 600  # 10 minutes
```

### Out of Memory

For large test suites, run coverage on a subset:

```bash
cargo tarpaulin --lib  # Only library code
cargo tarpaulin --bin simulation-framework  # Only binary
```

### Inaccurate Results

Some coverage results may be misleading:
- Proc macros and generated code may show as uncovered
- Inline functions may be attributed to wrong files
- Trait implementations might not count properly

These are known limitations of coverage tools.

## References

- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.com/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
