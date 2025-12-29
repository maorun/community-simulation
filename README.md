# Economic Simulation Framework

This repository contains a configurable economic simulation written in Rust. It simulates a small economy of individuals, each with unique skills and needs, engaging in trade within a dynamic market. The simulation is designed to explore concepts like price formation, wealth distribution, and market equilibrium.

## Features

- **Agent-Based Simulation:** Simulates individual persons with money, unique skills, and randomly generated needs for other skills.
- **Dynamic Market:** Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).
- **Trading System:** Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.
- **Reputation System:** Each person has a reputation score (starting at 1.0) that increases with successful trades. Higher reputation leads to better prices (up to 10% discount), while lower reputation results in price premiums. Reputation slowly decays toward neutral over time, encouraging ongoing positive behavior.
- **Urgency-Based Decisions:** Persons prioritize buying skills based on a randomly assigned urgency level.
- **Price Volatility:** Skill prices include a configurable random volatility component.
- **Configurable Parameters:** Allows customization of simulation parameters via command-line arguments or configuration files (YAML/TOML). CLI arguments override config file values.
- **Configuration Files:** Support for YAML and TOML configuration files to easily define complex simulation scenarios without lengthy command lines.
- **Progress Bar:** Visual progress indicator with real-time statistics during long simulations (can be disabled with `--no-progress` flag).
- **Structured Logging:** Configurable logging system for debugging and monitoring using standard Rust logging infrastructure (`log` + `env_logger`).
- **Wealth Inequality Analysis:** Automatic calculation of the Gini coefficient to measure wealth inequality in the simulated economy.
- **JSON Output:** Outputs detailed simulation results, including final wealth distribution, reputation statistics, skill valuations, and skill price history over time (suitable for graphing), to a JSON file.
- **Performance:** Leverages Rust and Rayon for potential parallelism in parts of the simulation (though current critical paths like trading are largely sequential for N=100).

## Getting Started

### Prerequisites

- Rust Toolchain (see [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) for installation instructions)

### Building the Project

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd economic-simulation-framework
    ```
    (Replace `<repository-url>` and `economic-simulation-framework` with actual values)
2.  Build the project in release mode for optimal performance:
    ```bash
    cargo build --release
    ```

### Running the Simulation

After building, the executable will be located at `target/release/economic_simulation` (the exact name depends on your `Cargo.toml` package name, assuming it's `economic_simulation`).

**Basic Execution (using default parameters):**

```bash
./target/release/economic_simulation -o results.json
```
This runs the simulation with default settings (e.g., 100 persons, 500 steps, 100 initial money, 10 base skill price) and saves the output to `results.json`.

**Command-Line Arguments:**

The simulation accepts the following CLI arguments:

*   `--config <CONFIG>` or `-c <CONFIG>`:
    *   Path to a configuration file (YAML or TOML format). When provided, configuration is loaded from the file first, then any CLI arguments override those values.
    *   See `config.example.yaml` and `config.example.toml` for example configuration files.
*   `--steps <STEPS>` or `-s <STEPS>`:
    *   Total number of simulation steps. Default: `500`.
*   `--persons <PERSONS>` or `-p <PERSONS>`:
    *   Number of persons to simulate. Each person will have one unique skill. Default: `100`.
*   `--initial-money <AMOUNT>`:
    *   Initial amount of money each person starts with. Default: `100.0`.
*   `--base-price <PRICE>`:
    *   Initial base price for all skills. Default: `10.0`.
*   `--output <FILEPATH>` or `-o <FILEPATH>`:
    *   Specifies the path to save the simulation results in JSON format. If not provided, results are printed to console only (summary).
*   `--threads <NUM_THREADS>`:
    *   (Optional) Number of threads for Rayon to use. Defaults to Rayon's choice (usually number of logical cores).
*   `--seed <SEED>`:
    *   Seed for the random number generator for reproducible simulations. Default: `42`.
*   `--no-progress`:
    *   Disable the progress bar during simulation. Useful for non-interactive environments or when redirecting output.
*   `--log-level <LOG_LEVEL>`:
    *   Set the logging level for the simulation. Valid values: `error`, `warn`, `info`, `debug`, `trace`. Default: `info`.
    *   Can also be set via the `RUST_LOG` environment variable (e.g., `RUST_LOG=debug`).
    *   Use `info` for high-level progress messages, `debug` for detailed step-by-step information, or `warn`/`error` for minimal output.

**Example with Custom Parameters:**

```bash
./target/release/economic_simulation --steps 1000 --persons 50 --initial-money 200 --base-price 15 --output custom_results.json --seed 123
```
This runs the simulation for 1000 steps with 50 persons, each starting with 200 money, skills having a base price of 15, and saves results to `custom_results.json` using RNG seed 123.

**Using Configuration Files:**

Configuration files provide an easier way to manage complex simulation scenarios without lengthy command lines. Both YAML and TOML formats are supported.

Example YAML configuration (`my_config.yaml`):
```yaml
max_steps: 1000
entity_count: 50
seed: 123
initial_money_per_person: 200.0
base_skill_price: 15.0
time_step: 1.0
scenario: Original
```

Example TOML configuration (`my_config.toml`):
```toml
max_steps = 1000
entity_count = 50
seed = 123
initial_money_per_person = 200.0
base_skill_price = 15.0
time_step = 1.0
scenario = "Original"
```

Run with a configuration file:
```bash
./target/release/economic_simulation --config my_config.yaml -o results.json
```

CLI arguments override config file values:
```bash
# Use config file but override steps and persons
./target/release/economic_simulation --config my_config.yaml --steps 2000 --persons 100 -o results.json
```

See `config.example.yaml` and `config.example.toml` in the repository for complete examples with all available options and comments.

### Logging

The simulation uses structured logging to provide insights into its operation. You can control the logging level to see more or less detail:

**Via CLI flag:**
```bash
./target/release/economic_simulation -s 100 -p 10 --log-level debug -o results.json
```

**Via environment variable:**
```bash
RUST_LOG=debug ./target/release/economic_simulation -s 100 -p 10 -o results.json
```

**Log Levels:**
- `error` - Only critical errors that prevent operation
- `warn` - Warnings about potential issues (minimal output)
- `info` - High-level progress information (default) - initialization, completion, performance metrics
- `debug` - Detailed step-by-step progress - useful for understanding simulation behavior
- `trace` - Very detailed logging (not currently used, reserved for future detailed instrumentation)

**Tips:**
- Use `info` (default) for normal operations
- Use `debug` when investigating simulation behavior or troubleshooting
- Use `warn` or `error` for minimal output in production/batch scenarios
- Combine with `--no-progress` flag to disable the progress bar when using debug logging

## Code Structure

*   `src/main.rs`: Handles command-line arguments and initializes the simulation.
*   `src/lib.rs`: Main library crate, exporting core modules.
*   `src/config.rs`: Defines `SimulationConfig` for simulation parameters.
*   `src/engine.rs`: Contains `SimulationEngine` which runs the main simulation loop and step-by-step logic.
*   `src/person.rs`: Defines the `Person` struct, `Transaction`, `NeededSkillItem`, and related types.
*   `src/skill.rs`: Defines the `Skill` struct.
*   `src/market.rs`: Defines the `Market` struct and its logic for price adjustments and history.
*   `src/entity.rs`: Defines the `Entity` struct which wraps a `Person` for compatibility with the engine structure.
*   `src/result.rs`: Defines `SimulationResult` and helper structs (`MoneyStats`, `SkillPriceInfo`) for structuring and outputting simulation results. It also includes `print_summary` and `save_to_file` methods.
*   `src/physics.rs`: (Removed) This module from the original framework is no longer used.
*   `src/tests/mod.rs`: Contains integration tests for the simulation engine.

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

2. **Module Tests** (inline in source files):
   - `src/result.rs`: Tests for result calculation, statistics, and JSON output
   - `src/scenario.rs`: Tests for price update mechanisms in different scenarios

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

### Continuous Integration

Tests are automatically run via GitHub Actions on every push and pull request to the `master` branch. The workflow configuration is located at `.github/workflows/rust.yml`.

The CI pipeline:
1. Checks out the code
2. Builds the project with `cargo build --verbose`
3. Runs all tests with `cargo test --verbose`

## Output Format (`results.json`)

The JSON output file contains a comprehensive summary of the simulation, including:

*   `total_steps`, `total_duration`, `active_persons`: General simulation metrics.
*   `final_money_distribution`: A list of final money amounts for each active person.
*   `money_statistics`: An object with:
    *   `average`: Average money across all persons
    *   `median`: Median money value
    *   `std_dev`: Standard deviation of money distribution
    *   `min_money`, `max_money`: Minimum and maximum money values
    *   `gini_coefficient`: Measure of wealth inequality (0 = perfect equality, 1 = perfect inequality). Values above 1 can occur when negative money (debt) exists.
*   `final_reputation_distribution`: A list of final reputation scores for each active person.
*   `reputation_statistics`: An object with:
    *   `average`: Average reputation across all persons
    *   `median`: Median reputation value
    *   `std_dev`: Standard deviation of reputation distribution
    *   `min_reputation`, `max_reputation`: Minimum and maximum reputation values
*   `final_skill_prices`: A list of all skills sorted by their final price (descending), including `id` and `price`.
*   `most_valuable_skill`, `least_valuable_skill`: Information on the skills with the highest and lowest final prices.
*   `skill_price_history`: A map where keys are `SkillId`s and values are lists of prices for that skill at each step of the simulation. This data can be used for plotting price trends.
*   `final_persons_data`: A list of all person data at the end of the simulation, including their full transaction history and reputation scores.

## License

This project is licensed under the terms of the MIT license. See the `LICENSE` file for details.
