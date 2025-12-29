# GitHub Copilot Instructions for Community Simulation Repository

## Repository Overview

This is an **economic simulation framework** written in Rust that models a small economy of individuals (persons) with unique skills who engage in trade within a dynamic market. The simulation explores price formation, wealth distribution, and market equilibrium through an agent-based model.

**Repository Stats:**
- **Language:** Rust (edition 2021)
- **Size:** ~1,200 lines of Rust code (excluding dependencies)
- **Project Type:** Binary application with library crate
- **Package Name:** `simulation-framework` (despite the repo name `community-simulation`)
- **Rust Version:** Tested with 1.92.0 (but should work with stable Rust toolchain)

**Key Dependencies:**
- `rayon` (1.8) - Parallelism support
- `serde` + `serde_json` (1.0) - Serialization for configuration and output
- `rand` (0.9) - Random number generation
- `clap` (4.0) - CLI argument parsing
- `strum` + `strum_macros` (0.27) - Enum utilities

## Build, Test, and Validation Commands

### Prerequisites
- Rust toolchain installed (https://www.rust-lang.org/tools/install)
- No other system dependencies required

### Build Commands

**ALWAYS run these commands from the repository root directory.**

**Debug Build (fast, for development):**
```bash
cargo build --verbose
```
- Build time: ~18 seconds for clean build, ~2-5 seconds for incremental
- Output: `target/debug/simulation-framework` (binary name matches package name, not repo name)
- Use for development and testing

**Release Build (optimized, for production):**
```bash
cargo build --release
```
- Build time: ~29 seconds for clean build
- Output: `target/release/simulation-framework`
- Use for performance-critical testing and benchmarking
- Profile settings include: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`

**Known Build Warnings:**
- The build currently produces 3 deprecation warnings about `rand::Rng::gen_range` (renamed to `random_range` in rand 0.9)
- These warnings do not prevent compilation and can be safely ignored OR fixed by replacing `gen_range` with `random_range`
- Warnings appear in: `src/engine.rs` (lines ~172, ~185) and `src/scenario.rs` (line ~171)

### Test Commands

**Run all tests:**
```bash
cargo test --verbose
```
- Test time: ~4 seconds
- All 14 tests should pass
- Tests include: unit tests in `src/tests/mod.rs`, scenario tests in `src/scenario.rs`, and result tests in `src/result.rs`

**Test Structure:**
- Unit tests are in `src/tests/mod.rs` - tests for `SimulationEngine`
- Module tests are inline in source files using `#[cfg(test)] mod tests { ... }`
- Tests use `tempfile` (dev-dependency) for file I/O testing

### Linting and Formatting

**Format code (applies changes):**
```bash
cargo fmt
```
- ALWAYS run this before committing code changes
- Uses default rustfmt configuration (no custom `.rustfmt.toml`)

**Check formatting (without applying changes):**
```bash
cargo fmt -- --check
```
- Returns exit code 1 if formatting is needed
- Used in CI to enforce code style

**Run Clippy (linter):**
```bash
cargo clippy --all-targets --all-features -- -D warnings -A deprecated
```
- **REQUIRED** step before completing development
- Treats all warnings as errors, except deprecated warnings
- Must pass without errors before submitting PR
- Current known deprecation warnings (ignored with `-A deprecated`):
  - 3 deprecation warnings about `rand::Rng::gen_range`
- To run clippy without treating warnings as errors: `cargo clippy`

### Running the Application

**Binary location:**
- Debug: `./target/debug/simulation-framework`
- Release: `./target/release/simulation-framework`

**Basic execution:**
```bash
./target/release/simulation-framework -o results.json
```

**With custom parameters:**
```bash
./target/release/simulation-framework --steps 1000 --persons 50 --initial-money 200 --base-price 15 --output results.json --seed 123
```

**CLI Arguments:**
- `-s, --steps <STEPS>` - Number of simulation steps (default: 500)
- `-p, --persons <PERSONS>` - Number of persons (default: 100)
- `--initial-money <AMOUNT>` - Initial money per person (default: 100.0)
- `--base-price <PRICE>` - Base price for skills (default: 10.0)
- `-o, --output <PATH>` - JSON output file path (optional)
- `--threads <NUM>` - Number of threads for Rayon (optional, defaults to CPU cores)
- `--seed <SEED>` - RNG seed for reproducibility (default: 42)
- `--scenario <SCENARIO>` - Scenario type: "Original" or "DynamicPricing" (default: Original)

**Quick test run:**
```bash
cargo run --release -- -s 10 -p 5 -o /tmp/test.json
```

## Project Layout and Architecture

### Directory Structure

```
.
├── .github/
│   ├── workflows/
│   │   └── rust.yml          # CI/CD workflow (build + test on push/PR)
│   └── copilot-instructions.md  # This file
├── src/
│   ├── main.rs               # CLI entry point, argument parsing
│   ├── lib.rs                # Library crate, module exports
│   ├── config.rs             # SimulationConfig struct
│   ├── engine.rs             # SimulationEngine - main simulation loop
│   ├── entity.rs             # Entity wrapper for Person
│   ├── person.rs             # Person struct, Transaction types
│   ├── skill.rs              # Skill struct and generation
│   ├── market.rs             # Market struct, price mechanisms
│   ├── scenario.rs           # Scenario enum, PriceUpdater trait/impls
│   ├── result.rs             # SimulationResult, JSON output, statistics
│   └── tests/
│       └── mod.rs            # Integration tests
├── Cargo.toml                # Package manifest and dependencies
├── Cargo.lock                # Locked dependency versions
├── README.md                 # User documentation
├── features.md               # German-language feature wishlist
├── LICENSE                   # MIT License
├── renovate.json             # Renovate bot configuration
└── .gitignore                # Git ignore rules (target/, debug/, *.rs.bk, etc.)
```

### Key Files and Their Purpose

**Configuration:**
- `Cargo.toml` - Dependencies, build profile settings, package metadata
- No `.rustfmt.toml` or `clippy.toml` - uses Rust defaults

**Core Logic:**
- `src/engine.rs` - Contains `SimulationEngine` with `new()`, `step()`, and `run()` methods
  - Manages the simulation loop
  - Initializes entities and market
  - Handles trading logic and price updates
- `src/person.rs` - Defines `Person`, `Transaction`, `TransactionType`, `NeededSkillItem`
  - Each person has: id, money, own_skill, needed_skills, transaction_history
- `src/market.rs` - Defines `Market` with skill prices, demand/supply tracking, price history
  - Contains `update_prices()` method that delegates to scenario-specific price updaters
- `src/scenario.rs` - Defines `Scenario` enum and `PriceUpdater` implementations
  - `Original` - Supply/demand-based pricing with volatility
  - `DynamicPricing` - Sales-based pricing (increase if sold, decrease if not)
- `src/result.rs` - Defines `SimulationResult` with statistics and JSON serialization
  - Calculates money statistics (average, median, std_dev, min, max)
  - Tracks skill price history over time
  - Provides `print_summary()` and `save_to_file()` methods

**Entry Points:**
- `src/main.rs` - CLI application entry point
- `src/lib.rs` - Library crate entry point (exports public API)

### GitHub Actions CI/CD

**Workflow:** `.github/workflows/rust.yml`
- **Triggers:** Push to `master` branch, pull requests to `master`
- **Jobs:**
  1. Checkout code (`actions/checkout@v6`)
  2. Build: `cargo build --verbose`
  3. Test: `cargo test --verbose`
- **No linting step** in CI (clippy/fmt not enforced automatically)
- **Runs on:** `ubuntu-latest`
- **Environment variable:** `CARGO_TERM_COLOR: always`

**To replicate CI locally:**
```bash
cargo build --verbose && cargo test --verbose
```

## Common Issues and Workarounds

### Issue 1: Deprecation Warnings from `rand` crate
**Symptoms:** Warnings about `gen_range` being renamed to `random_range`  
**Impact:** Build succeeds but with warnings; clippy with `-D warnings` fails  
**Workaround:** Either:
1. Ignore the warnings (they don't affect functionality)
2. Replace `rng.gen_range(a..=b)` with `rng.random_range(a..=b)` in:
   - `src/engine.rs` lines 172, 185
   - `src/scenario.rs` line 171

### Issue 2: Code Formatting Required
**Symptoms:** `cargo fmt -- --check` fails  
**Impact:** Code style inconsistencies  
**Workaround:** ALWAYS run `cargo fmt` before committing

### Issue 3: Clippy Warnings About Default-Constructed Unit Structs
**Symptoms:** Clippy suggests removing `.default()` calls on unit structs  
**Impact:** Style warnings, not functional issues  
**Workaround:** In `src/scenario.rs`, replace:
- `OriginalPriceUpdater::default()` with `OriginalPriceUpdater`
- `DynamicPricingUpdater::default()` with `DynamicPricingUpdater`

### Issue 4: Package Name vs Repository Name Mismatch
**Symptoms:** Binary is named `simulation-framework`, not `community-simulation`  
**Impact:** Confusion when referencing the binary  
**Workaround:** ALWAYS use `simulation-framework` when referring to the binary name

### Issue 5: Long Release Build Times
**Symptoms:** Release builds take ~29 seconds even for small changes  
**Impact:** Slow iteration for performance testing  
**Workaround:** Use debug builds (`cargo build`) for development; reserve release builds for final testing

## Additional Validation Steps

**Performance check:**
```bash
./target/release/simulation-framework -s 500 -p 100 -o /tmp/perf-test.json
# Should complete in < 1 second and output "Performance: X steps/second"
```

**JSON output validation:**
```bash
./target/release/simulation-framework -s 10 -p 5 -o /tmp/output.json
cat /tmp/output.json | jq '.total_steps'  # Should output: 10
cat /tmp/output.json | jq '.active_persons'  # Should output: 5
```

**Memory and thread behavior:**
- The simulation uses Rayon for parallelism
- Default thread count: number of logical CPU cores
- Memory usage is proportional to: persons × steps (for history tracking)

## Quick Reference for Common Tasks

**Add a new dependency:**
1. Edit `Cargo.toml`
2. Run `cargo build` (downloads and compiles new dependency)
3. Import in code: `use new_dependency::*;`

**Add a new test:**
1. Add test function in appropriate test module (e.g., `src/tests/mod.rs`)
2. Use `#[test]` attribute
3. Run with `cargo test`

**Add a new CLI argument:**
1. Edit `Args` struct in `src/main.rs`
2. Add `#[arg(...)]` attribute
3. Update `SimulationConfig` initialization in `main()`

**Modify simulation logic:**
1. Core loop: `src/engine.rs` - `SimulationEngine::step()`
2. Price updates: `src/scenario.rs` - `PriceUpdater` implementations
3. Trading logic: `src/engine.rs` - within `step()` method

**Add a new scenario:**
1. Add variant to `Scenario` enum in `src/scenario.rs`
2. Implement new `XyzUpdater` struct
3. Add implementation to `PriceUpdater::update_prices()` match statement
4. Update `PriceUpdater::from(Scenario)` implementation

## Important Notes for Coding Agents

1. **ALWAYS run `cargo fmt` before committing** - The codebase uses standard Rust formatting
2. **The binary name is `simulation-framework`, not `community-simulation`** - Don't get confused by the repo name
3. **Tests must pass** - Run `cargo test` to verify changes don't break existing functionality
4. **Deprecation warnings are acceptable** - The rand 0.9 warnings don't need to be fixed unless specifically requested
5. **Release builds are slow** - Use debug builds for development unless performance testing
6. **CI only runs build + test** - No automatic linting enforcement, but should still follow Rust style guidelines
7. **JSON output structure is part of the API** - Changes to `SimulationResult` affect downstream consumers
8. **The simulation is deterministic with fixed seed** - Use `--seed` parameter for reproducible results

## Trust These Instructions

These instructions have been validated by running all commands and examining the outputs. If something doesn't work as described:
1. Check that you're in the repository root directory
2. Verify your Rust toolchain version (`rustc --version`)
3. Try `cargo clean` and rebuild
4. Only if the instructions appear incorrect, perform additional investigation
