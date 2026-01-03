# Economic Simulation Framework

This repository contains a configurable economic simulation written in Rust. It simulates a small economy of individuals, each with unique skills and needs, engaging in trade within a dynamic market. The simulation is designed to explore concepts like price formation, wealth distribution, and market equilibrium.

## Features

- **Agent-Based Simulation:** Simulates individual persons with money, unique skills, and randomly generated needs for other skills.
- **Dynamic Market:** Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).
- **Trading System:** Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.
- **Panic Recovery:** Robust error handling with graceful degradation - if a panic occurs during simulation step execution, it is caught and logged, allowing the simulation to continue. Failed steps are tracked and reported in the results.
- **Reputation System:** Each person has a reputation score (starting at 1.0) that increases with successful trades. Higher reputation leads to better prices (up to 10% discount), while lower reputation results in price premiums. Reputation slowly decays toward neutral over time, encouraging ongoing positive behavior.
- **Technological Progress:** Skills become more efficient over time through a configurable technology growth rate, simulating productivity improvements. More efficient skills effectively cost less, enabling increased trade and economic growth over the simulation period.
- **Seasonal Demand Effects:** Configurable seasonal fluctuations in skill demand using cyclical patterns. Different skills experience peak demand at different times, creating realistic market dynamics and economic cycles. Controlled via `--seasonal-amplitude` and `--seasonal-period` parameters.
- **Transaction Fees:** Configurable marketplace transaction fees that are deducted from seller proceeds on each trade. Simulates realistic trading costs (e.g., platform fees, payment processing) and allows studying the impact of fees on market liquidity, wealth distribution, and economic activity. Total fees collected are tracked and reported. Controlled via `--transaction-fee` parameter (0.0-1.0 range representing 0-100% fee rate).
- **Savings System:** Persons can save a configurable percentage of their money each simulation step. Saved money is moved from available cash to a separate savings account, affecting spending capacity while enabling wealth accumulation studies. Configurable via `--savings-rate` parameter (0.0-1.0 range representing 0-100% savings rate). Savings statistics (total, average, median, min, max) are tracked and reported in results.
- **Urgency-Based Decisions:** Persons prioritize buying skills based on a randomly assigned urgency level.
- **Price Volatility:** Skill prices include a configurable random volatility component.
- **Configurable Parameters:** Allows customization of simulation parameters via command-line arguments or configuration files (YAML/TOML). CLI arguments override config file values.
- **Input Validation:** Comprehensive validation of all configuration parameters with clear error messages. Ensures parameters are within acceptable ranges (e.g., positive values for steps/persons, valid ranges for rates/amplitudes) to prevent crashes and provide immediate feedback on configuration errors.
- **Configuration Files:** Support for YAML and TOML configuration files to easily define complex simulation scenarios without lengthy command lines.
- **Progress Bar:** Visual progress indicator with real-time statistics during long simulations (can be disabled with `--no-progress` flag).
- **Structured Logging:** Configurable logging system for debugging and monitoring using standard Rust logging infrastructure (`log` + `env_logger`).
- **Colored Terminal Output:** Enhanced terminal output with color-coded statistics and messages for improved readability. Automatically detects terminal capabilities and can be disabled with `--no-color` flag.
- **Wealth Inequality Analysis:** Automatic calculation of the Gini coefficient to measure wealth inequality in the simulated economy.
- **Market Concentration Analysis:** Calculates the Herfindahl-Hirschman Index (HHI) to measure wealth concentration among participants. HHI values indicate market structure: < 1,500 (competitive), 1,500-2,500 (moderate concentration), > 2,500 (high concentration/oligopoly).
- **JSON Output:** Outputs detailed simulation results, including final wealth distribution, reputation statistics, skill valuations, and skill price history over time (suitable for graphing), to a JSON file.
- **Compressed Output:** Optional gzip compression for JSON output files, reducing file sizes by 10-20x while maintaining full data fidelity. Ideal for large-scale simulations and batch processing.
- **CSV Export:** Export simulation results to multiple CSV files for easy analysis in Excel, pandas, R, or other data analysis tools. Includes summary statistics, per-person distributions, skill prices, and time-series price history.
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

*   `--preset <PRESET_NAME>`:
    *   Use a predefined configuration preset for quick setup. Available presets:
        *   `default` - Standard economy (100 persons, 500 steps, $100 initial money)
        *   `small_economy` (alias: `small`) - Quick testing (20 persons, 100 steps)
        *   `large_economy` (alias: `large`) - Detailed analysis (500 persons, 2000 steps, $200 initial money)
        *   `crisis_scenario` (alias: `crisis`) - Economic crisis (100 persons, 1000 steps, $50 initial money, $25 base price)
        *   `high_inflation` (alias: `inflation`) - Dynamic pricing scenario (100 persons, 1000 steps, DynamicPricing scenario)
        *   `tech_growth` (alias: `tech`) - Technology growth (150 persons, 1500 steps, $250 initial money, $8 base price)
        *   `quick_test` (alias: `quick`) - Rapid testing (10 persons, 50 steps)
    *   CLI arguments can override preset values when explicitly provided.
    *   Example: `--preset small_economy --steps 200` uses the small economy preset but overrides steps to 200.
*   `--list-presets`:
    *   Display all available preset configurations with their parameters and exit.
*   `--config <CONFIG>` or `-c <CONFIG>`:
    *   Path to a configuration file (YAML or TOML format). When provided, configuration is loaded from the file first, then any CLI arguments override those values.
    *   See `config.example.yaml` and `config.example.toml` for example configuration files.
*   `--steps <STEPS>` or `-s <STEPS>`:
    *   Total number of simulation steps. If not specified, uses default (500) or preset value.
*   `--persons <PERSONS>` or `-p <PERSONS>`:
    *   Number of persons to simulate. Each person will have one unique skill. If not specified, uses default (100) or preset value.
*   `--initial-money <AMOUNT>`:
    *   Initial amount of money each person starts with. If not specified, uses default (100.0) or preset value.
*   `--base-price <PRICE>`:
    *   Initial base price for all skills. If not specified, uses default (10.0) or preset value.
*   `--output <FILEPATH>` or `-o <FILEPATH>`:
    *   Specifies the path to save the simulation results in JSON format. If not provided, results are printed to console only (summary).
*   `--compress`:
    *   Compress the JSON output using gzip compression. When enabled, a `.gz` extension is automatically added to the output filename.
    *   Example: `--output results.json --compress` creates `results.json.gz`
    *   Achieves significant file size reduction (typically 10-20x smaller) while maintaining full data fidelity.
    *   Compressed files can be decompressed with standard tools like `gunzip` or opened directly by many analysis tools.
*   `--csv-output <PATH_PREFIX>`:
    *   Specifies the path prefix for CSV output files. Creates multiple CSV files with this prefix for easy analysis in Excel, pandas, R, etc.
    *   Generated files:
        *   `{prefix}_summary.csv` - Summary statistics and metrics
        *   `{prefix}_money.csv` - Money distribution per person
        *   `{prefix}_reputation.csv` - Reputation distribution per person
        *   `{prefix}_skill_prices.csv` - Final skill prices
        *   `{prefix}_price_history.csv` - Skill price history over time
    *   Example: `--csv-output results` creates `results_summary.csv`, `results_money.csv`, etc.
*   `--threads <NUM_THREADS>`:
    *   (Optional) Number of threads for Rayon to use. Defaults to Rayon's choice (usually number of logical cores).
*   `--seed <SEED>`:
    *   Seed for the random number generator for reproducible simulations. If not specified, uses default (42) or preset value.
*   `--scenario <SCENARIO>`:
    *   Scenario type: `Original` (supply/demand pricing) or `DynamicPricing` (sales-based pricing). If not specified, uses default (Original) or preset value.
*   `--tech-growth-rate <RATE>`:
    *   Technology growth rate per simulation step (e.g., 0.001 = 0.1% growth per step). Simulates productivity improvements over time where skills become more efficient, effectively reducing their cost. Higher efficiency enables more trade and economic growth. Set to 0.0 to disable (default). If not specified, uses default (0.0) or preset value.
*   `--seasonal-amplitude <AMPLITUDE>`:
    *   Seasonal demand amplitude controlling the strength of seasonal fluctuations in skill demand (0.0 = no seasonality, 0.0-1.0 = variation strength). A value of 0.5 means demand can vary ±50% from the base level. Set to 0.0 to disable seasonal effects (default). If not specified, uses default (0.0) or preset value.
*   `--seasonal-period <STEPS>`:
    *   Seasonal cycle period in simulation steps (default: 100). Determines how many steps it takes for demand to complete one seasonal cycle. For example, a value of 100 means demand patterns repeat every 100 steps. Only used when seasonal-amplitude > 0.0. If not specified, uses default (100) or preset value.
*   `--transaction-fee <RATE>`:
    *   Transaction fee rate as a percentage of the transaction value (0.0-1.0, e.g., 0.05 = 5% fee). The fee is deducted from the seller's proceeds on each transaction, simulating realistic market costs. For example, if a skill sells for $100 with a 5% fee, the buyer pays $100 but the seller receives only $95, with $5 collected as fees. Set to 0.0 to disable transaction fees (default). If not specified, uses default (0.0) or preset value.
    *   **Use cases:** 
        *   Simulate marketplace transaction costs (e.g., platform fees, payment processing)
        *   Study the impact of trading costs on market efficiency and liquidity
        *   Model wealth extraction by intermediaries or governments
    *   The total fees collected across all transactions are reported in the simulation results.
*   `--savings-rate <RATE>`:
    *   Savings rate as a percentage of current money to save each simulation step (0.0-1.0, e.g., 0.05 = 5% savings rate). Each step, persons save this percentage of their current available money, which is moved from cash to a separate savings account. This affects spending capacity while enabling wealth accumulation. For example, if a person has $100 and savings-rate is 0.05, they will save $5 and have $95 available for trading. Set to 0.0 to disable savings (default). If not specified, uses default (0.0) or preset value.
    *   **Use cases:** 
        *   Model realistic wealth accumulation behavior
        *   Study the impact of savings rates on market liquidity and economic activity
        *   Explore wealth distribution with different savings patterns
    *   Savings statistics (total, average, median, min, max) are tracked and reported in the simulation results.
*   `--no-progress`:
    *   Disable the progress bar during simulation. Useful for non-interactive environments or when redirecting output.
*   `--no-color`:
    *   Disable colored terminal output. By default, the simulation uses colors to improve readability of terminal output (e.g., green for success messages, yellow for warnings, color-coded statistics). Use this flag in non-interactive environments, when redirecting output to files, or if your terminal doesn't support colors.
*   `--log-level <LOG_LEVEL>`:
    *   Set the logging level for the simulation. Valid values: `error`, `warn`, `info`, `debug`, `trace`. Default: `info`.
    *   Can also be set via the `RUST_LOG` environment variable (e.g., `RUST_LOG=debug`).
    *   Use `info` for high-level progress messages, `debug` for detailed step-by-step information, or `warn`/`error` for minimal output.

**Example with Preset:**

```bash
# List all available presets
./target/release/economic_simulation --list-presets

# Use a preset for quick testing
./target/release/economic_simulation --preset quick_test -o quick_results.json

# Use a preset and override some parameters
./target/release/economic_simulation --preset crisis_scenario --steps 2000 --seed 999 -o crisis_results.json
```

**Example with Custom Parameters:**

```bash
./target/release/economic_simulation --steps 1000 --persons 50 --initial-money 200 --base-price 15 --output custom_results.json --seed 123
```
This runs the simulation for 1000 steps with 50 persons, each starting with 200 money, skills having a base price of 15, and saves results to `custom_results.json` using RNG seed 123.

**Example with Seasonal Effects:**

```bash
./target/release/economic_simulation --steps 500 --persons 100 --seasonal-amplitude 0.3 --seasonal-period 50 --output seasonal_results.json
```
This runs the simulation with seasonal demand fluctuations. The `--seasonal-amplitude 0.3` parameter creates ±30% variation in demand, and `--seasonal-period 50` means the seasonal cycle repeats every 50 steps. Different skills will have their peak demand at different times due to phase offsets, creating realistic market dynamics.

**Example with Transaction Fees:**

```bash
./target/release/economic_simulation --steps 500 --persons 100 --transaction-fee 0.05 --output fees_results.json
```
This runs the simulation with a 5% transaction fee on all trades. The fee is deducted from the seller's proceeds (e.g., if a skill sells for $100, the buyer pays $100 but the seller receives $95, with $5 collected as fees). This simulates realistic marketplace costs and allows studying the impact of trading fees on market liquidity, wealth distribution, and economic activity. The total fees collected are reported in the JSON output.

**Example with CSV Export:**

```bash
./target/release/economic_simulation --steps 500 --persons 100 --csv-output ./output/analysis
```
This runs the simulation and creates CSV files (`analysis_summary.csv`, `analysis_money.csv`, etc.) in the `./output/` directory for easy data analysis.

**Example with Compressed Output:**

```bash
./target/release/economic_simulation --steps 1000 --persons 100 --output results.json --compress
```
This runs the simulation and saves compressed results to `results.json.gz`, achieving significant space savings (typically 10-20x smaller file size) while preserving all simulation data. The compressed file can be decompressed with `gunzip results.json.gz` or opened directly by many data analysis tools.

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

### Benchmarks

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
*   `failed_steps`: Number of steps that failed due to panics but were recovered gracefully. The simulation uses panic recovery to continue execution even if an individual step encounters an unexpected error. Failed steps are logged and counted, but do not halt the simulation. In normal operation, this value should be 0.
*   `final_money_distribution`: A list of final money amounts for each active person.
*   `money_statistics`: An object with:
    *   `average`: Average money across all persons
    *   `median`: Median money value
    *   `std_dev`: Standard deviation of money distribution
    *   `min_money`, `max_money`: Minimum and maximum money values
    *   `gini_coefficient`: Measure of wealth inequality (0 = perfect equality, 1 = perfect inequality). Values above 1 can occur when negative money (debt) exists.
    *   `herfindahl_index`: Herfindahl-Hirschman Index measuring wealth concentration (0 = perfect competition, 10,000 = monopoly). Values < 1,500 indicate competitive distribution, 1,500-2,500 moderate concentration, > 2,500 high concentration.
*   `final_reputation_distribution`: A list of final reputation scores for each active person.
*   `reputation_statistics`: An object with:
    *   `average`: Average reputation across all persons
    *   `median`: Median reputation value
    *   `std_dev`: Standard deviation of reputation distribution
    *   `min_reputation`, `max_reputation`: Minimum and maximum reputation values
*   `trade_volume_statistics`: An object with comprehensive trade activity metrics:
    *   `total_trades`: Total number of successful trades across all steps
    *   `total_volume`: Total money exchanged across all trades
    *   `avg_trades_per_step`: Average number of trades per simulation step
    *   `avg_volume_per_step`: Average money exchanged per simulation step
    *   `avg_transaction_value`: Average transaction value (total volume / total trades)
    *   `min_trades_per_step`: Minimum trades in a single step
    *   `max_trades_per_step`: Maximum trades in a single step
*   `trades_per_step`: An array tracking the number of trades at each simulation step
*   `volume_per_step`: An array tracking the total money exchanged at each simulation step
*   `total_fees_collected`: Total transaction fees collected across all trades when a non-zero transaction fee is configured. This represents the cumulative cost of trading in the market.
*   `final_skill_prices`: A list of all skills sorted by their final price (descending), including `id` and `price`.
*   `most_valuable_skill`, `least_valuable_skill`: Information on the skills with the highest and lowest final prices.
*   `skill_price_history`: A map where keys are `SkillId`s and values are lists of prices for that skill at each step of the simulation. This data can be used for plotting price trends.
*   `final_persons_data`: A list of all person data at the end of the simulation, including their full transaction history and reputation scores.

### CSV Export

When using the `--csv-output` flag, the simulation generates multiple CSV files for easy analysis:

*   `{prefix}_summary.csv`: Summary statistics including money distribution, reputation, skill prices, and **trade volume metrics**
*   `{prefix}_money.csv`: Money distribution per person
*   `{prefix}_reputation.csv`: Reputation distribution per person
*   `{prefix}_skill_prices.csv`: Final skill prices
*   `{prefix}_price_history.csv`: Skill price history over time (if available)
*   `{prefix}_trade_volume.csv`: **Trade volume history showing trades count and money exchanged per step**

The trade volume CSV provides time-series data perfect for analyzing market activity and economic vitality trends.

## License

This project is licensed under the terms of the MIT license. See the `LICENSE` file for details.
