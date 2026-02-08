# Development Guide

This document provides comprehensive information for developers working on the Economic Simulation Framework.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building the Project](#building-the-project)
- [Code Structure](#code-structure)
- [Testing](#testing)
- [Code Coverage](#code-coverage)
- [Benchmarks](#benchmarks)
- [Continuous Integration](#continuous-integration)
- [Linting and Code Quality](#linting-and-code-quality)
- [Fuzz Testing](#fuzz-testing)
- [Contributing](#contributing)

## Prerequisites

- Rust Toolchain (see [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) for installation instructions)
- No other system dependencies required

## Building the Project

### Debug Build (fast, for development)

```bash
cargo build --verbose
```
- Build time: ~18 seconds for clean build, ~2-5 seconds for incremental
- Output: `target/debug/community-simulation` (binary name matches package name, not repo name)
- Use for development and testing

### Release Build (optimized, for production)

```bash
cargo build --release
```
- Build time: ~29 seconds for clean build
- Output: `target/release/community-simulation`
- Use for performance-critical testing and benchmarking
- Profile settings include: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`

### Known Build Warnings

The build may produce deprecation warnings about `rand::Rng::gen_range` depending on the version of the rand crate used. These warnings do not prevent compilation and can be safely ignored. The warnings typically appear in:
- `src/engine.rs` (lines ~172, ~185)
- `src/scenario.rs` (line ~171)

## Code Structure

The project is organized as follows:

*   **`src/main.rs`**: Handles command-line arguments and initializes the simulation.
*   **`src/lib.rs`**: Main library crate, exporting core modules.
*   **`src/config.rs`**: Defines `SimulationConfig` for simulation parameters.
*   **`src/engine.rs`**: Contains `SimulationEngine` which runs the main simulation loop and step-by-step logic.
*   **`src/person.rs`**: Defines the `Person` struct, `Transaction`, `NeededSkillItem`, `Strategy` enum, and related types. The `Strategy` enum defines four behavioral strategies (Conservative, Balanced, Aggressive, Frugal) that affect how persons make spending decisions.
*   **`src/skill.rs`**: Defines the `Skill` struct.
*   **`src/market.rs`**: Defines the `Market` struct and its logic for price adjustments and history.
*   **`src/entity.rs`**: Defines the `Entity` struct which wraps a `Person` for compatibility with the engine structure.
*   **`src/result.rs`**: Defines `SimulationResult` and helper structs (`MoneyStats`, `SkillPriceInfo`) for structuring and outputting simulation results. It also includes `print_summary` and `save_to_file` methods.
*   **`src/scenario.rs`**: Defines scenario types and price update mechanisms.
*   **`src/tests/mod.rs`**: Contains integration tests for the simulation engine.

## Testing

The project includes a comprehensive test suite to ensure code quality and correctness.

### Running Tests

Run all tests with:
```bash
cargo test
```

For verbose output with detailed test information:
```bash
cargo test --verbose
```

Run tests for a specific module:
```bash
cargo test result
cargo test scenario
cargo test engine
```

### Test Structure

The test suite includes:

1. **Unit Tests** (`src/tests/mod.rs`): Core simulation engine tests
   - `test_simulation_engine_new()`: Verifies engine initialization
   - `test_simulation_engine_step()`: Tests single simulation step execution
   - `test_simulation_engine_run()`: Tests complete simulation runs

2. **Property-Based Tests** (`src/tests/proptest_tests.rs`): Uses `proptest` to verify invariants across random inputs
   - Person reputation bounds (always 0.0 to 2.0)
   - Market price bounds enforcement
   - Gini coefficient correctness
   - Transaction recording integrity
   - Skill price non-negativity

3. **Integration Tests** (`src/tests/scenario_integration_tests.rs`): Complete simulation scenarios
   - Different scenarios (Original, DynamicPricing)
   - Various population sizes (5 to 100 persons)
   - Extreme conditions testing
   - Reputation and trade volume tracking
   - Result statistics validation

4. **Module Tests** (inline in source files):
   - `src/result.rs`: Tests for result calculation, statistics, and JSON/CSV output
   - `src/scenario.rs`: Tests for price update mechanisms in different scenarios
   - `src/config.rs`: Tests for configuration file loading (YAML/TOML)
   - `src/person.rs`: Tests for person behavior and reputation system
   - `src/skill.rs`: Tests for skill generation and management

### Writing New Tests

Tests follow Rust's standard testing conventions. Here's an example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Arrange
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            // ... other config
        };
        
        // Act
        let engine = SimulationEngine::new(config);
        
        // Assert
        assert_eq!(engine.get_active_entity_count(), 10);
    }
}
```

## Code Coverage

The project includes comprehensive code coverage tracking and reporting. Coverage is automatically collected in CI and uploaded to Codecov for tracking over time.

### Generating Coverage Locally

Install `cargo-tarpaulin` (one-time setup):

```bash
cargo install cargo-tarpaulin
```

Generate a basic terminal coverage report:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300
```

Generate an HTML report for detailed line-by-line analysis:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out html --output-dir ./coverage
# Open coverage/index.html in your browser
```

### Coverage Goals

The project aims for **at least 70% code coverage**. You can verify your changes meet this threshold:

```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --fail-under 70
```

### More Information

For detailed coverage instructions, troubleshooting, and best practices, see [COVERAGE.md](COVERAGE.md).

## Benchmarks

The project includes performance benchmarks using `criterion`:

```bash
cargo bench
```

Benchmarks cover:
- Engine initialization with different population sizes
- Single step execution performance
- Full simulation runs
- Scenario comparison (Original vs DynamicPricing)

Results are saved in `target/criterion/` with detailed HTML reports.

## Continuous Integration

The project uses GitHub Actions for continuous integration, automatically running on every push and pull request to the `master` branch. Workflow configurations are located in `.github/workflows/`.

### Main CI Pipeline (`rust.yml`)

The main CI pipeline enforces code quality through:
1. **Code Formatting**: Checks that all code follows Rust formatting standards with `cargo fmt --all -- --check`
2. **Linting**: Runs Clippy to catch common mistakes and enforce best practices with `cargo clippy --all-targets --all-features -- -D warnings -A deprecated`
3. **Build**: Compiles the project with `cargo build --verbose`
4. **Tests**: Runs all tests with `cargo test --verbose`
5. **Release Build**: Verifies that release builds succeed with `cargo build --release --verbose`

### Code Coverage Pipeline (`coverage.yml`)

A separate workflow collects code coverage metrics:
1. **Coverage Generation**: Uses `cargo-tarpaulin` to instrument tests and measure coverage
2. **Codecov Upload**: Automatically uploads coverage reports to Codecov for tracking
3. **Threshold Check**: Verifies coverage meets the 70% threshold (warns but doesn't fail)

Coverage badges are displayed in the README.md header, and detailed coverage reports are available on Codecov.

All PRs must pass these checks before merging, ensuring consistent code style and quality across the project.

## Linting and Code Quality

The project uses custom linting configurations to maintain high code quality. For detailed information, see [LINTING.md](LINTING.md).

**Quick Overview:**

**`.clippy.toml`** - Configures Clippy with additional lints:
- **Complexity Checks**: Functions are checked for cognitive complexity (threshold: 25), excessive arguments (threshold: 7), and excessive lines (threshold: 150)
- **Type Complexity**: Warns about overly complex type definitions (threshold: 500)
- **Performance**: Detects large arrays (threshold: 16KB) and enum variants (threshold: 200 bytes) that might impact performance
- **Style**: Enforces consistent naming conventions and documentation standards

**`.rustfmt.toml`** - Configures code formatting:
- **Maximum Line Width**: 100 characters
- **Import Organization**: Automatically reorders imports
- **Consistent Style**: Enforces use of field init shorthand, try shorthand, and explicit ABI
- **Code Layout**: Standardizes function call width (80 chars), struct literal width (80 chars), and other formatting rules

To run linting locally:
```bash
# Run clippy with project configuration
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# Format code with project configuration
cargo fmt --all
```

These configurations help identify:
- Overly complex functions that should be refactored
- Functions with too many parameters
- Large data structures that might cause stack overflow
- Type definitions that are hard to understand
- Inconsistent code formatting

**For comprehensive documentation on linting rules, best practices, and examples, see [LINTING.md](LINTING.md).**

## Fuzz Testing

The project includes fuzz testing to find edge cases, crashes, and security vulnerabilities through automated random input generation. Fuzz tests use `cargo-fuzz` and require the Rust nightly toolchain.

### Quick Start

```bash
# Install nightly toolchain (if not already installed)
rustup toolchain install nightly

# Install cargo-fuzz
cargo install cargo-fuzz

# Run a fuzz target for 60 seconds
cargo +nightly fuzz run fuzz_config_yaml -- -max_total_time=60
```

### Available Fuzz Targets

1. **fuzz_config_yaml**: Tests YAML configuration parsing robustness
2. **fuzz_config_toml**: Tests TOML configuration parsing robustness  
3. **fuzz_simulation_init**: Tests simulation engine initialization with arbitrary numeric inputs

### Documentation

For detailed information on running fuzz tests, interpreting results, and adding new fuzz targets, see [`fuzz/FUZZ_TESTING.md`](fuzz/FUZZ_TESTING.md).

### Why Fuzz Testing?

Fuzz testing automatically discovers:
- Crashes and panics from unexpected inputs
- Edge cases not covered by traditional tests
- Security vulnerabilities (buffer overflows, integer overflows, etc.)
- Input validation issues
- Robustness problems with malformed configuration files

Run fuzzing regularly (especially before releases) to catch potential issues early.

## Contributing

We welcome contributions to the Economic Simulation Framework! Here are some guidelines:

### Before You Start

1. Check existing issues and pull requests to avoid duplicate work
2. For major changes, open an issue first to discuss your ideas
3. Ensure you have a working Rust development environment

### Development Workflow

1. **Fork and Clone**: Fork the repository and clone your fork locally
   ```bash
   git clone https://github.com/YOUR_USERNAME/community-simulation.git
   cd community-simulation
   ```

2. **Create a Branch**: Create a feature branch for your changes
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**: Implement your changes following the code style
   - Follow Rust naming conventions
   - Add tests for new functionality
   - Update documentation as needed
   - Keep functions focused and avoid high complexity (cognitive complexity < 25)
   - Limit function parameters to 7 or fewer when possible

4. **Format and Lint**: Ensure code quality
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings -A deprecated
   ```
   The project uses `.clippy.toml` and `.rustfmt.toml` for consistent linting and formatting.
   Clippy will warn about:
   - Functions with high cognitive complexity
   - Functions with too many parameters or lines
   - Large data structures that might impact performance

5. **Test**: Run all tests to ensure nothing breaks
   ```bash
   cargo test --verbose
   cargo build --release
   ```

6. **Commit**: Write clear, descriptive commit messages
   ```bash
   git add .
   git commit -m "Add feature: description of your changes"
   ```

7. **Push and PR**: Push to your fork and create a pull request
   ```bash
   git push origin feature/your-feature-name
   ```

### Code Style Guidelines

- **Formatting**: Use `cargo fmt` for consistent formatting
- **Comments**: Add comments for complex logic, but prefer self-documenting code
- **Documentation**: Use doc comments (`///`) for public APIs
- **Error Handling**: Use `Result` types for fallible operations
- **Testing**: Aim for comprehensive test coverage

### Pull Request Guidelines

- Provide a clear description of the changes
- Reference any related issues
- Ensure CI checks pass
- Be responsive to review feedback
- Keep PRs focused and reasonably sized

### Reporting Issues

When reporting bugs:
- Use the provided issue templates
- Include steps to reproduce
- Provide system information (OS, Rust version)
- Include relevant error messages or logs

### Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help create a welcoming environment

Thank you for contributing to make this project better!
