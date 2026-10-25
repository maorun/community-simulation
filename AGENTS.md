# Project Overview

- Treat this repository as the canonical source code for the Economic Simulation Framework (`community-simulation`).
- The project implements an agent-based economic simulation in Rust with high performance and deterministic execution.
- Build targets include a command-line interface binary and dual library outputs (`rlib` and `cdylib`) with optional WebAssembly support.
- The project targets Rust edition 2024 with a minimum supported Rust version of 1.88.
- Default to `master` as the primary integration and release branch.

# Important Paths

- `Cargo.toml` — package manifest, dependencies, feature flags, benchmark definitions, and release profile.
- `.cargo/config.toml` — Cargo aliases for partitioned test execution (`test-unit`, `test-integration`, `test-golden`, `test-doc`).
- `scripts/run_tests.sh` — executable test runner script for partitioned local and continuous integration testing.
- `src/main.rs` — command-line entry point, command argument parsing with `clap`, and interactive wizard launcher.
- `src/lib.rs` — root library file exporting public simulation modules and extension traits.
- `src/engine.rs` — `SimulationEngine` containing the primary simulation loop, step execution, and parallel iteration.
- `src/config.rs` — `SimulationConfig`, scenario configurations, configuration loaders, and parameter validation.
- `src/person.rs` — `Person` struct, spending strategies (`Conservative`, `Balanced`, `Aggressive`, `Frugal`), needs, and transactions.
- `src/market.rs` — `Market` struct, price discovery, order books, historical statistics, volatility, and transaction fees.
- `src/nepotism.rs` — nepotism mechanics, social connections, preferential trading, and cronyism tracking.
- `src/lending_marketplace.rs` — peer-to-peer lending marketplace, credit underwriting, collateral, and interest rates.
- `src/plugin.rs` — public extension traits (`Plugin`, `PricingStrategy`, `AgentStrategy`) for modular enhancements.
- `src/result.rs` — `SimulationResult` collection, statistical summaries, and data export to JSON, CSV, and Parquet.
- `src/database.rs` — SQLite database persistence for simulation history, entities, and transactions.
- `src/scenario.rs` — scenario definitions, market shocks, policy interventions, and external conditions.
- `src/scenario_comparison.rs` — multi-scenario evaluation, metric comparisons, and confidence scoring.
- `src/invariant.rs` — simulation invariant validation rules, `InvariantViolation`, and `InvariantSeverity` tracking.
- `src/error.rs` — `SimulationError` enum, error category classification, and structured error handling.
- `src/tests/` — modular integration test suites, scenario integration tests, and coverage tests.
- `tests/golden_regression.rs` — golden-file regression test suite ensuring deterministic simulation output.
- `tests/golden/` — committed reference baseline outputs for regression verification.
- `.github/workflows/rust.yml` — continuous integration workflow with formatting, clippy, build, partitioned tests, and release build.
- `.github/workflows/coverage.yml` — manual workflow for generating test coverage reports.
- `.clippy.toml` — Clippy complexity limits and lint rules.
- `.rustfmt.toml` — code formatting rules for `cargo fmt`.

# Development Commands

- Format code: `cargo fmt --all`
- Verify code formatting: `cargo fmt --all -- --check`
- Run linter checks: `cargo clippy --all-targets --all-features -- -D warnings -A deprecated`
- Build debug binary: `cargo build --verbose`
- Build fast release binary: `CARGO_PROFILE_RELEASE_LTO=thin CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 cargo build --release`
- Build production release binary: `cargo build --release`
- Run unit tests: `cargo test-unit` or `./scripts/run_tests.sh unit`
- Run scenario integration tests: `cargo test-integration-scenarios` or `./scripts/run_tests.sh scenarios`
- Run coverage stress tests: `cargo test-integration-coverage` or `./scripts/run_tests.sh coverage-suites`
- Run documentation tests: `cargo test-doc` or `./scripts/run_tests.sh doc`
- Run golden regression tests: `cargo test-golden` or `./scripts/run_tests.sh golden`
- Run complete test suite: `cargo test` or `./scripts/run_tests.sh all`

# Architecture and Code Conventions

- Write idiomatic Rust using Rust 2024 edition conventions and strict typing.
- Use `SimulationError` for error conditions. Do not introduce unstructured panics or raw `unwrap` calls in core engine loops.
- Preserve backward compatibility for Serde deserialization.
- Add `#[serde(default)]` to every newly added struct field in serialized structures.
- Ensure deterministic simulation behavior. Fixed random seeds must generate identical simulation runs.
- Use `Rayon` parallel iterators for heavy agent loops where state mutability permits.
- Implement extension points from `src/plugin.rs` instead of modifying core loops for custom strategies.
- Enforce simulation invariants via `src/invariant.rs` with appropriate `InvariantSeverity` levels.
- Apply defensive numerical clamping to probabilities (`0.0..=1.0`), price multipliers, and tax rates.
- Prevent floating-point underflows, overflows, or `NaN` values in financial balance calculations.

# Testing and Verification Conventions

- Add a focused regression test for every bug fix or behavioral change.
- Verify that the regression test fails before you apply the code fix.
- Partition tests appropriately. Place unit tests near module code and scenario tests in `src/tests/`.
- Express expected test failures with explicit assertions. Do not rely on unhandled crashes.
- Verify golden-file regression tests with `cargo test-golden` when modifying core engine output.
- Run `git diff --check` to identify whitespace errors or conflict markers before completion.
- Report local test results with the exact test count and pass status.

# Pull Request and Git Workflow

- Perform implementation work only in dedicated branches.
- Do not commit directly to `master`.
- Ensure the branch is up to date with `origin/master` before opening a pull request.
- Write commit titles and pull request descriptions using ASD-STE100 rules.
- Write commit titles in the imperative mood with a maximum length of 72 characters.
- Merge each pull request to `master` as a single commit using squash and merge.
