# Code Coverage Guide

This document explains how to generate and view code coverage reports for the simulation framework.

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

The project aims for at least **70% code coverage**. You can check if your changes meet this threshold:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --fail-under 70
```

This command will:
- Exit with code 0 if coverage is ≥70%
- Exit with code 1 if coverage is <70%

## Continuous Integration

Coverage is automatically tracked in CI:
- Every push to `master` and every PR generates a coverage report
- Reports are uploaded to Codecov
- Coverage badges in README.md show current coverage status
- CI will warn (but not fail) if coverage drops below 70%

## Understanding Coverage Metrics

### Line Coverage
Percentage of executable lines that were run during tests. This is the primary metric.

### Branch Coverage
Percentage of conditional branches (if/else, match arms) that were executed.

### Function Coverage
Percentage of functions that were called at least once during tests.

## Improving Coverage

To find uncovered code:

1. Generate HTML report: `cargo tarpaulin --out html --output-dir ./coverage`
2. Open `coverage/index.html` in browser
3. Navigate to files with low coverage
4. Red-highlighted lines show uncovered code
5. Write tests to exercise those code paths

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
