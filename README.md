# Economic Simulation Framework

This repository contains a configurable economic simulation written in Rust. It simulates a small economy of individuals, each with unique skills and needs, engaging in trade within a dynamic market. The simulation is designed to explore concepts like price formation, wealth distribution, and market equilibrium.

## Features

- **Agent-Based Simulation:** Simulates individual persons with money, unique skills, and randomly generated needs for other skills.
- **Multiple Skills Per Person:** Each person can possess and offer multiple skills in the market, creating more realistic labor dynamics with skill redundancy and competition. Configurable via `--skills-per-person` parameter (default: 1). When set to values > 1, skills are distributed across persons using a round-robin approach, allowing multiple providers per skill and more complex market interactions.
- **Dynamic Market:** Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).
- **Trading System:** Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.
- **Panic Recovery:** Robust error handling with graceful degradation - if a panic occurs during simulation step execution, it is caught and logged, allowing the simulation to continue. Failed steps are tracked and reported in the results.
- **Reputation System:** Each person has a reputation score (starting at 1.0) that increases with successful trades. Higher reputation leads to better prices (up to 10% discount), while lower reputation results in price premiums. Reputation slowly decays toward neutral over time, encouraging ongoing positive behavior.
- **Technological Progress:** Skills become more efficient over time through a configurable technology growth rate, simulating productivity improvements. More efficient skills effectively cost less, enabling increased trade and economic growth over the simulation period.
- **Seasonal Demand Effects:** Configurable seasonal fluctuations in skill demand using cyclical patterns. Different skills experience peak demand at different times, creating realistic market dynamics and economic cycles. Controlled via `--seasonal-amplitude` and `--seasonal-period` parameters.
- **Transaction Fees:** Configurable marketplace transaction fees that are deducted from seller proceeds on each trade. Simulates realistic trading costs (e.g., platform fees, payment processing) and allows studying the impact of fees on market liquidity, wealth distribution, and economic activity. Total fees collected are tracked and reported. Controlled via `--transaction-fee` parameter (0.0-1.0 range representing 0-100% fee rate).
- **Savings System:** Persons can save a configurable percentage of their money each simulation step. Saved money is moved from available cash to a separate savings account, affecting spending capacity while enabling wealth accumulation studies. Configurable via `--savings-rate` parameter (0.0-1.0 range representing 0-100% savings rate). Savings statistics (total, average, median, min, max) are tracked and reported in results.
- **Tax System:** Configurable income tax on trade proceeds with optional redistribution. The system collects taxes from sellers' proceeds after transaction fees and can redistribute collected taxes equally among all persons at the end of each step. This simulates government taxation and wealth redistribution policies, allowing study of their effects on wealth inequality and economic activity. Controlled via `--tax-rate` parameter (0.0-1.0 range representing 0-100% tax rate) and `--enable-tax-redistribution` flag. Tax statistics (total collected, total redistributed) are tracked and reported in results.
- **Loan System:** Persons can borrow and lend money with interest and repayment schedules. When enabled, the system tracks loans between persons, processes scheduled repayments each step, and provides statistics on loan activity. Loans have configurable interest rates and repayment periods. Enable via configuration file with `enable_loans: true`, then configure `loan_interest_rate`, `loan_repayment_period`, and `min_money_to_lend` parameters. Loan statistics (total issued, repaid, active) are included in simulation results.
- **Contract System:** Long-term agreements for stable trading relationships. When enabled, persons can form contracts that lock in prices for multiple simulation steps, providing price stability and predictable income/expenses for both parties. Contracts have configurable duration bounds and offer a price discount to incentivize formation. Enable via `--enable-contracts` flag or configuration file, with parameters `max_contract_duration`, `min_contract_duration`, and `contract_price_discount` (default: 5% discount). Contract statistics (total created, completed, active, average duration, total value) are tracked and included in simulation results. Ideal for studying long-term economic relationships, price stability mechanisms, and the effects of contractual obligations on market dynamics.
- **Black Market:** Parallel informal market with different pricing rules. When enabled, a configurable percentage of trades are routed to an alternative market that operates with different prices (typically cheaper), simulating informal economy dynamics. Configured via `enable_black_market`, `black_market_price_multiplier` (e.g., 0.8 for 20% discount), and `black_market_participation_rate` (e.g., 0.2 for 20% of trades). Black market statistics (trades, volume, percentages) are tracked separately and included in simulation results.
- **Behavioral Strategies:** Persons are assigned different behavioral strategies that affect their spending decisions, creating heterogeneous agent behavior. Four strategy types are supported:
  - **Conservative** (0.7x spending multiplier): Risk-averse agents who prefer saving and only spend when they have ample reserves. Willing to spend up to 70% of their money on needed skills.
  - **Balanced** (1.0x spending multiplier): Standard agents with normal spending behavior. This is the default strategy.
  - **Aggressive** (1.3x spending multiplier): Risk-taking agents who prioritize acquiring skills and are willing to spend beyond their immediate means. Can afford skills up to 130% of their current money.
  - **Frugal** (0.5x spending multiplier): Extremely cautious agents who minimize spending and maximize savings. Only willing to spend up to 50% of their money.
  Strategies are distributed equally across the population using round-robin assignment, ensuring balanced representation. The strategy system enables studying how different agent behaviors affect market dynamics, wealth distribution, and economic activity.
- **Priority-Based Buying Decisions:** Sophisticated multi-factor decision-making system for purchase prioritization. Each purchase option is scored based on four weighted factors:
  - **Urgency** (default weight: 0.5): Need urgency level (1-3 scale, randomly assigned)
  - **Affordability** (default weight: 0.3): Cost relative to available money (lower cost = higher priority)
  - **Efficiency** (default weight: 0.1): Technological progress factor (more efficient skills prioritized)
  - **Reputation** (default weight: 0.1): Seller reputation score (higher reputation = higher priority)
  
  All weights are configurable (0.0-1.0 range), allowing experimentation with different decision strategies. The system combines these factors into a single priority score for each potential purchase, then sorts options by priority (highest first). This enables realistic, heterogeneous agent behavior that considers multiple objectives simultaneously rather than simple urgency-only sorting.
- **Price Volatility:** Skill prices include a configurable random volatility component.
- **Configurable Parameters:** Allows customization of simulation parameters via command-line arguments or configuration files (YAML/TOML). CLI arguments override config file values.
- **Input Validation:** Comprehensive validation of all configuration parameters with clear error messages. Ensures parameters are within acceptable ranges (e.g., positive values for steps/persons, valid ranges for rates/amplitudes) to prevent crashes and provide immediate feedback on configuration errors.
- **Configuration Files:** Support for YAML and TOML configuration files to easily define complex simulation scenarios without lengthy command lines.
- **Progress Bar:** Visual progress indicator with real-time statistics during long simulations (can be disabled with `--no-progress` flag).
- **Structured Logging:** Configurable logging system for debugging and monitoring using standard Rust logging infrastructure (`log` + `env_logger`).
- **Trace Mode:** Comprehensive debug logging for problem diagnosis. Enable detailed logging of all simulation actions including trade attempts, price updates, reputation changes, loan payments, and tax redistribution. Use environment variable `RUST_LOG=debug` for detailed logs or `RUST_LOG=trace` for extremely detailed output. Ideal for debugging simulation behavior, understanding agent decision-making, and diagnosing unexpected results.
- **Colored Terminal Output:** Enhanced terminal output with color-coded statistics and messages for improved readability. Automatically detects terminal capabilities and can be disabled with `--no-color` flag.
- **Wealth Inequality Analysis:** Automatic calculation of the Gini coefficient to measure wealth inequality in the simulated economy.
- **Market Concentration Analysis:** Calculates the Herfindahl-Hirschman Index (HHI) to measure wealth concentration among participants. HHI values indicate market structure: < 1,500 (competitive), 1,500-2,500 (moderate concentration), > 2,500 (high concentration/oligopoly).
- **Per-Skill Trade Analytics:** Detailed trade statistics for each skill type, tracking trade count, total volume, and average price per skill. Enables identification of the most traded and valuable skills in the market. Results are sorted by total trading volume and included in JSON output for easy analysis.
- **Monte Carlo Simulations:** Run multiple parallel simulations with different random seeds to achieve statistical significance. Automatically aggregates results across runs with mean, standard deviation, min, max, and median statistics for key metrics (average money, Gini coefficient, trade volume, reputation). Ideal for research, parameter sensitivity analysis, and understanding simulation variability.
- **Parameter Sweep Analysis:** Automated sensitivity analysis through systematic parameter sweeps (grid search). Test a parameter across a range of values with multiple runs per value to understand how parameter choices affect simulation outcomes. Supports sweeping initial_money, base_price, savings_rate, and transaction_fee. Results include aggregated statistics and identification of optimal parameter values for different objectives. Perfect for research, parameter tuning, and understanding system robustness.
- **Scenario Comparison:** Compare multiple simulation scenarios side-by-side to analyze the effects of different economic policies. Run A/B testing on pricing mechanisms (Original, DynamicPricing, AdaptivePricing) with multiple runs per scenario for statistical robustness. Automatically determines winners based on different criteria: highest average wealth, lowest inequality, highest trade volume, and highest reputation. Results are saved in JSON format with detailed statistics and winner analysis. Ideal for policy evaluation, economic research, and understanding the impact of different market mechanisms on outcomes.
- **Checkpoint System:** Save and resume simulation state at any point. Automatically save checkpoints at regular intervals during long simulations. Resume from saved checkpoints to continue interrupted simulations without starting from scratch. Useful for multi-hour simulations, distributed computing, incremental analysis, and crash recovery. Checkpoints are stored in JSON format with complete simulation state including entities, market data, loans, and statistics.
- **Streaming Output (JSONL):** Real-time streaming of step-by-step simulation data to a JSON Lines (JSONL) file. Each simulation step appends one JSON object containing key metrics (trades, volume, money statistics, Gini coefficient, reputation) to the output file. Enables real-time monitoring of long-running simulations, reduces memory footprint by not storing all step data in memory, and allows progressive analysis. Each line is a complete JSON object that can be parsed independently, making it ideal for streaming analysis tools and real-time dashboards.
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
*   `--min-skill-price <PRICE>`:
    *   Minimum price floor for skills. Prevents skill prices from dropping below this threshold, modeling real-world price controls like minimum wages or regulatory price floors. Must be positive and less than or equal to base_price. Useful for preventing market crashes and maintaining economic stability. If not specified, uses default (1.0) or preset value.
    *   **Use cases:**
        *   Model minimum wage policies in labor markets
        *   Prevent deflationary spirals and market collapse
        *   Study the effects of price floor regulations
        *   Maintain market liquidity during economic crises
    *   Example: `--base-price 10.0 --min-skill-price 2.0` ensures no skill price falls below $2
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
*   `--stream-output <FILEPATH>`:
    *   Path to stream step-by-step simulation data in JSONL (JSON Lines) format.
    *   When enabled, the simulation appends one JSON object per line to this file after each step.
    *   Each line contains step metrics: step number, trades count, volume exchanged, average money, Gini coefficient, average reputation, and top 5 skill prices.
    *   **Use cases:**
        *   Real-time monitoring of long-running simulations
        *   Reduced memory footprint (doesn't store all history in RAM)
        *   Progressive analysis with streaming tools (e.g., `tail -f`, data pipelines)
        *   Real-time dashboards and visualization
    *   JSONL format: Each line is a complete, independent JSON object that can be parsed separately
    *   Example: `--stream-output simulation_stream.jsonl`
    *   Can be used alongside `--output` for both real-time and final results
*   `--threads <NUM_THREADS>`:
    *   (Optional) Number of threads for Rayon to use. Defaults to Rayon's choice (usually number of logical cores).
*   `--seed <SEED>`:
    *   Seed for the random number generator for reproducible simulations. If not specified, uses default (42) or preset value.
*   `--scenario <SCENARIO>`:
    *   Specifies which simulation scenario to run. This determines the price adjustment mechanism used in the market.
    *   Available scenarios:
        *   `Original` (default) - Supply and demand based pricing with volatility. Prices adjust based on the ratio of demand to supply, with random fluctuations for market realism.
        *   `DynamicPricing` - Sales-based pricing. If a skill is sold, its price increases by 5%; if not sold, it decreases by 5%. This creates rapid price adjustments based on immediate market feedback.
        *   `AdaptivePricing` - Gradual adaptive pricing using exponential moving average. Prices smoothly converge toward targets based on sales activity (±10% targets with 20% learning rate). This creates more stable price movements than DynamicPricing while still responding to market conditions.
    *   Example: `--scenario AdaptivePricing`
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
*   `--tax-rate <RATE>`:
    *   Tax rate as a percentage of seller trade income (0.0-1.0, e.g., 0.10 = 10% income tax). Each time a seller completes a trade, this percentage is deducted from their proceeds (after transaction fees) and collected as taxes. For example, if a seller receives $90 after fees and tax-rate is 0.10, they pay $9 in taxes and keep $81. Set to 0.0 to disable taxation (default). If not specified, uses default (0.0) or preset value.
    *   **Use cases:** 
        *   Simulate government taxation policies
        *   Study the impact of income taxes on economic activity and wealth distribution
        *   Model wealth extraction by central authorities
        *   Test progressive vs flat tax systems (flat tax in current implementation)
    *   Tax statistics (total collected, total redistributed) are tracked and reported in the simulation results.
*   `--enable-tax-redistribution`:
    *   Enable equal redistribution of collected taxes to all persons at the end of each simulation step. When enabled, taxes collected during a step are distributed equally among all active persons, simulating basic income or welfare programs. This flag only has effect when `--tax-rate` is greater than 0. Without this flag, collected taxes are removed from the economy (simulating government spending outside the simulation). Set to false to collect taxes without redistribution (default).
    *   **Use cases:** 
        *   Simulate basic income / universal basic income (UBI) policies
        *   Study wealth redistribution and inequality reduction
        *   Model social welfare programs
        *   Compare taxation with and without redistribution effects
    *   The total amount redistributed is tracked separately and reported in the simulation results.
*   `--skills-per-person <COUNT>`:
    *   Number of skills each person can provide (default: 1). Higher values create more versatile persons who can participate in multiple markets, introducing skill redundancy and increased competition. Skills are distributed using a round-robin approach across all persons. For example, with 10 persons and `--skills-per-person 2`, each skill will have 2 providers. Valid range: 1 to entity_count. If not specified, uses default (1) or preset value.
    *   **Use cases:** 
        *   Model labor markets with multi-skilled workers
        *   Study the impact of skill redundancy on market dynamics and prices
        *   Simulate economies with cross-training and skill diversification
        *   Analyze how market competition changes with multiple providers per skill
    *   **Example:** `--persons 20 --skills-per-person 3` creates 20 persons, each with 3 different skills from a pool of 20 unique skills, resulting in 3 providers per skill.
*   `--no-progress`:
    *   Disable the progress bar during simulation. Useful for non-interactive environments or when redirecting output.
*   `--no-color`:
    *   Disable colored terminal output. By default, the simulation uses colors to improve readability of terminal output (e.g., green for success messages, yellow for warnings, color-coded statistics). Use this flag in non-interactive environments, when redirecting output to files, or if your terminal doesn't support colors.
*   `--log-level <LOG_LEVEL>`:
    *   Set the logging level for the simulation. Valid values: `error`, `warn`, `info`, `debug`, `trace`. Default: `info`.
    *   Can also be set via the `RUST_LOG` environment variable (e.g., `RUST_LOG=debug`).
    *   Use `info` for high-level progress messages, `debug` for detailed step-by-step information, or `warn`/`error` for minimal output.
*   `--monte-carlo-runs <NUM_RUNS>`:
    *   Run multiple simulations in parallel with different random seeds for statistical significance.
    *   Each run uses a sequential seed: `seed`, `seed+1`, `seed+2`, etc.
    *   Results are aggregated with statistics (mean, std dev, min, max, median) for key metrics:
        *   Average money per person
        *   Gini coefficient (wealth inequality)
        *   Total trades (economic activity)
        *   Average reputation
    *   Both individual run results and aggregated statistics are saved to the JSON output.
    *   Runs execute in parallel using Rayon for maximum performance.
    *   **Use cases:**
        *   Research: Establish statistical significance of results
        *   Sensitivity analysis: Understand variability across random seeds
        *   Parameter tuning: Identify stable configurations
    *   Example: `--monte-carlo-runs 10` runs 10 parallel simulations
    *   Minimum value: 2 runs
*   `--checkpoint-interval <STEPS>`:
    *   Interval (in steps) between automatic checkpoint saves. Set to 0 to disable auto-checkpointing (default).
    *   When enabled, the simulation automatically saves its complete state every N steps to the checkpoint file.
    *   Useful for long-running simulations that may be interrupted or for incremental progress tracking.
    *   Example: `--checkpoint-interval 100` saves a checkpoint every 100 steps.
    *   **Use cases:**
        *   Resume interrupted simulations without starting from scratch
        *   Save progress during very long simulations (e.g., 10,000+ steps)
        *   Create snapshots for analysis at specific intervals
        *   Recover from system crashes or errors
*   `--checkpoint-file <PATH>`:
    *   Path to the checkpoint file for saving/loading simulation state.
    *   Defaults to `checkpoint.json` if not specified.
    *   The checkpoint file stores the complete simulation state in JSON format, including:
        *   Current step number
        *   All entities (persons) with their money, skills, transactions, and reputation
        *   Market state with prices and history
        *   Loan system state (if enabled)
        *   Trade volume statistics
    *   Example: `--checkpoint-file ./checkpoints/simulation_1.json`
*   `--resume`:
    *   Resume the simulation from a previously saved checkpoint.
    *   When enabled, the simulation loads its state from the checkpoint file instead of initializing from scratch.
    *   The checkpoint file must exist (use --checkpoint-file to specify the path).
    *   After resuming, the simulation continues from where it left off and runs for the configured number of steps.
    *   **Note:** The RNG is reseeded based on the checkpoint's step number for reproducible behavior.
    *   Example: `--resume --checkpoint-file ./checkpoints/simulation_1.json`
*   `--parameter-sweep <SPEC>`:
    *   Run parameter sweep analysis over a parameter range (sensitivity analysis).
    *   Format: `"parameter:min:max:steps"` where:
        *   `parameter`: Name of the parameter to sweep (see below for available parameters)
        *   `min`: Minimum value to test
        *   `max`: Maximum value to test
        *   `steps`: Number of evenly-spaced values to test between min and max
    *   Available parameters:
        *   `initial_money` - Initial money per person (e.g., `"initial_money:50:150:5"`)
        *   `base_price` - Base skill price (e.g., `"base_price:5:25:5"`)
        *   `savings_rate` - Savings rate percentage (e.g., `"savings_rate:0:0.2:5"`)
        *   `transaction_fee` - Transaction fee percentage (e.g., `"transaction_fee:0:0.1:6"`)
    *   Runs multiple simulations at each parameter value (controlled by `--sweep-runs`)
    *   Results include aggregated statistics and optimal parameter identification
    *   Example: `--parameter-sweep "initial_money:80:120:5" --sweep-runs 3`
*   `--sweep-runs <NUM>`:
    *   Number of simulation runs per parameter value in parameter sweep (default: 3).
    *   Each run uses a different random seed (seed, seed+1, seed+2, etc.) for statistical robustness.
    *   Higher values provide more reliable statistics but increase computation time.
    *   Example: `--sweep-runs 5` (run 5 simulations at each parameter value)
*   `--compare-scenarios <SCENARIOS>`:
    *   Compare multiple simulation scenarios side-by-side to analyze the effects of different economic policies.
    *   Format: Comma-separated list of scenario names (e.g., `"Original,DynamicPricing,AdaptivePricing"`)
    *   Available scenarios:
        *   `Original` - Supply/demand-based pricing with volatility
        *   `DynamicPricing` - Sales-based pricing (increase if sold, decrease if not)
        *   `AdaptivePricing` - Gradual price adjustments using exponential moving average
    *   Runs multiple simulations for each scenario (controlled by `--comparison-runs`)
    *   Results include:
        *   Aggregated statistics for each scenario (mean, std dev, min, max, median)
        *   Winner determination for different criteria (wealth, inequality, trade volume, reputation)
        *   Side-by-side comparison data for analysis
    *   **Use cases:**
        *   A/B testing of economic policies
        *   Comparing the effectiveness of different pricing mechanisms
        *   Understanding trade-offs between scenarios (e.g., efficiency vs. equality)
        *   Research on market dynamics under different rules
    *   Example: `--compare-scenarios "Original,DynamicPricing" --comparison-runs 5`
    *   Minimum: 2 different scenarios required
*   `--comparison-runs <NUM>`:
    *   Number of simulation runs per scenario in scenario comparison mode (default: 3).
    *   Each run uses a different random seed (seed, seed+1, seed+2, etc.) for statistical robustness.
    *   Higher values provide more reliable comparison results but increase computation time.
    *   Example: `--comparison-runs 5` (run 5 simulations for each scenario being compared)

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

**Example with Price Floor:**

```bash
# Crisis scenario with price floor to prevent market collapse
./target/release/economic_simulation --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 3 --scenario DynamicPricing --output price_floor_results.json

# Compare with and without price floor
./target/release/economic_simulation --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 1 --scenario DynamicPricing --output no_floor.json
./target/release/economic_simulation --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 5 --scenario DynamicPricing --output with_floor.json
```

The price floor feature is particularly useful in crisis scenarios or with dynamic pricing that can drive prices down. By setting `--min-skill-price 3`, you ensure that no skill price falls below $3, preventing deflationary spirals and maintaining minimum market viability. This models real-world economic policies like minimum wage laws or regulatory price controls.

**Example with Transaction Fees:**

```bash
./target/release/economic_simulation --steps 500 --persons 100 --transaction-fee 0.05 --output fees_results.json
```
This runs the simulation with a 5% transaction fee on all trades. The fee is deducted from the seller's proceeds (e.g., if a skill sells for $100, the buyer pays $100 but the seller receives $95, with $5 collected as fees). This simulates realistic marketplace costs and allows studying the impact of trading fees on market liquidity, wealth distribution, and economic activity. The total fees collected are reported in the JSON output.

**Example with Tax System:**

```bash
# Simulation with 10% income tax (no redistribution)
./target/release/economic_simulation --steps 500 --persons 100 --tax-rate 0.10 --output tax_results.json

# Simulation with 15% income tax and redistribution
./target/release/economic_simulation --steps 500 --persons 100 --tax-rate 0.15 --enable-tax-redistribution --output tax_redistribution_results.json

# Combined: transaction fees + taxes + redistribution
./target/release/economic_simulation --steps 500 --persons 100 \
  --transaction-fee 0.05 --tax-rate 0.20 --enable-tax-redistribution \
  --output combined_policy.json
```

Tax system usage:
- **Without redistribution:** Taxes are collected from seller proceeds and removed from the economy, simulating government spending on public goods outside the simulation. This reduces overall money supply and can affect economic activity.
- **With redistribution:** Taxes collected each step are redistributed equally to all persons at the end of the step, simulating basic income or welfare programs. This can reduce wealth inequality while maintaining total money supply.
- The total taxes collected and (if enabled) redistributed are reported in the JSON output for analysis.
- Taxes are calculated on net seller proceeds (after transaction fees): `tax = (price - transaction_fee) * tax_rate`

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

**Example with Monte Carlo Simulations:**

```bash
# Run 10 parallel simulations for statistical significance
./target/release/economic_simulation --monte-carlo-runs 10 -s 500 -p 100 -o mc_results.json

# With custom seed for reproducibility
./target/release/economic_simulation --monte-carlo-runs 5 -s 1000 -p 50 --seed 12345 -o mc_analysis.json

# Combine with other features (compressed output, custom parameters)
./target/release/economic_simulation --monte-carlo-runs 20 -s 500 -p 100 \
  --seasonal-amplitude 0.3 --transaction-fee 0.05 \
  -o mc_seasonal_fees.json --compress
```

Monte Carlo output includes:
- Individual results from each run (all simulation data preserved)
- Aggregated statistics across runs:
  - **Average Money**: Mean, std dev, min, max, median across all runs
  - **Gini Coefficient**: Distribution of wealth inequality across runs
  - **Total Trades**: Economic activity variation across runs
  - **Average Reputation**: Reputation dynamics across runs

This is ideal for:
- **Research**: Establishing statistical significance of economic phenomena
- **Sensitivity Analysis**: Understanding how random variation affects outcomes
- **Parameter Tuning**: Finding robust configurations that work across multiple seeds
- **Publication**: Providing mean ± std dev statistics for academic papers

**Example with Scenario Comparison:**

```bash
# Compare Original and DynamicPricing scenarios with 5 runs each
./target/release/economic_simulation -s 500 -p 100 \
  --compare-scenarios "Original,DynamicPricing" \
  --comparison-runs 5 \
  -o comparison_results.json

# Compare all three available scenarios with custom parameters
./target/release/economic_simulation -s 1000 -p 50 \
  --compare-scenarios "Original,DynamicPricing,AdaptivePricing" \
  --comparison-runs 10 \
  --initial-money 150 --base-price 12 \
  -o full_comparison.json

# Scenario comparison with economic features enabled
./target/release/economic_simulation -s 500 -p 100 \
  --compare-scenarios "Original,DynamicPricing" \
  --comparison-runs 5 \
  --transaction-fee 0.05 --tax-rate 0.1 --enable-tax-redistribution \
  -o comparison_with_policies.json
```

Scenario comparison output includes:
- Individual results from each run for each scenario
- Aggregated statistics per scenario:
  - **Average Money**: Mean wealth distribution across runs
  - **Gini Coefficient**: Wealth inequality comparison
  - **Total Trades**: Economic activity levels
  - **Average Reputation**: Trust dynamics
- Winner determination for each criterion:
  - **Highest Avg Wealth**: Which scenario produces the most wealth
  - **Lowest Inequality**: Which scenario is most equitable
  - **Highest Trade Volume**: Which scenario is most economically active
  - **Highest Reputation**: Which scenario builds the most trust

This is ideal for:
- **Policy Evaluation**: Comparing the effects of different economic rules
- **A/B Testing**: Determining which pricing mechanism works best for specific goals
- **Research**: Understanding trade-offs between efficiency and equity
- **Decision Making**: Choosing the right scenario for your simulation objectives

**Example with Parameter Sweep Analysis:**

```bash
# Sweep initial money from 50 to 200 with 7 test points, 5 runs each
./target/release/economic_simulation -s 500 -p 100 \
  --parameter-sweep "initial_money:50:200:7" \
  --sweep-runs 5 \
  -o sweep_initial_money.json

# Test the impact of transaction fees on market activity
./target/release/economic_simulation -s 500 -p 100 \
  --parameter-sweep "transaction_fee:0.0:0.15:6" \
  --sweep-runs 3 \
  -o sweep_transaction_fee.json

# Analyze savings rate effects on wealth distribution
./target/release/economic_simulation -s 500 -p 100 \
  --parameter-sweep "savings_rate:0.0:0.2:5" \
  --sweep-runs 4 \
  -o sweep_savings_rate.json

# Test base price sensitivity
./target/release/economic_simulation -s 500 -p 100 \
  --parameter-sweep "base_price:5:25:5" \
  --sweep-runs 3 \
  -o sweep_base_price.json
```

Parameter sweep output includes:
- Results for each parameter value tested
- Multiple runs per value for statistical robustness
- Aggregated statistics (mean, std dev, min, max, median) for:
  - **Average Money**: How parameter affects wealth levels
  - **Gini Coefficient**: Impact on wealth inequality
  - **Total Trades**: Effects on economic activity
  - **Average Reputation**: Influence on reputation dynamics
- Optimal parameter values identified for different objectives:
  - Highest average money
  - Lowest inequality (best Gini coefficient)
  - Highest trade volume (most economic activity)

This is ideal for:
- **Sensitivity Analysis**: Systematically understand how parameters affect outcomes
- **Parameter Optimization**: Find parameter values that maximize desired objectives
- **Robustness Testing**: Identify parameter ranges where the system behaves stably
- **Research**: Generate publication-quality parameter sensitivity plots
- **Policy Analysis**: Compare economic policies (fees, taxes, regulations) quantitatively

**Example with Checkpoint System:**

```bash
# Run a long simulation with automatic checkpoints every 500 steps
./target/release/economic_simulation --steps 5000 --persons 100 \
  --checkpoint-interval 500 \
  --checkpoint-file ./checkpoints/long_run.json \
  --output results.json

# If the simulation is interrupted, resume from the last checkpoint
./target/release/economic_simulation --resume \
  --checkpoint-file ./checkpoints/long_run.json \
  --output continued_results.json

# You can also manually save checkpoints at specific intervals
# Run first 1000 steps with checkpoint every 250 steps
./target/release/economic_simulation --steps 1000 --persons 50 \
  --checkpoint-interval 250 \
  --checkpoint-file ./step1.json

# Resume and run another 1000 steps
./target/release/economic_simulation --resume \
  --checkpoint-file ./step1.json \
  --steps 1000 \
  --checkpoint-interval 250 \
  --checkpoint-file ./step2.json
```

Checkpoint system benefits:
- **Resume Long Simulations**: Save progress and resume after interruptions or crashes
- **Incremental Analysis**: Save simulation state at different stages for comparison
- **Distributed Computing**: Run simulations in stages across different machines
- **Debugging**: Examine specific simulation states by loading checkpoints

The checkpoint file stores:
- Complete simulation state (entities, market, loans, statistics)
- Current step number and configuration
- All transaction history and price data up to that point
- JSON format for easy inspection and debugging

**Example with Streaming Output:**

```bash
# Stream step-by-step data to a JSONL file for real-time monitoring
./target/release/economic_simulation --steps 1000 --persons 100 \
  --stream-output ./stream/simulation.jsonl \
  --output results.json

# Monitor the simulation in real-time (in another terminal)
tail -f ./stream/simulation.jsonl | jq '.step, .trades, .avg_money'

# Stream without final output (for pure streaming mode)
./target/release/economic_simulation --steps 5000 --persons 200 \
  --stream-output simulation_progress.jsonl

# Combine streaming with other features
./target/release/economic_simulation --steps 2000 --persons 150 \
  --stream-output stream.jsonl \
  --output final.json \
  --compress \
  --seasonal-amplitude 0.3 \
  --transaction-fee 0.05
```

Streaming output benefits:
- **Real-Time Monitoring**: Watch simulation progress as it runs using `tail -f` or similar tools
- **Memory Efficiency**: Doesn't store all step data in RAM, ideal for very long simulations
- **Progressive Analysis**: Analyze data while the simulation is still running
- **Dashboards**: Feed the JSONL stream into real-time visualization tools
- **JSONL Format**: Each line is a complete JSON object (step number, trades, volume, money stats, Gini coefficient, reputation, top skill prices)

Example JSONL line (one per step):
```json
{"step":42,"trades":18,"volume":234.56,"avg_money":102.34,"gini_coefficient":0.15,"avg_reputation":1.23,"top_skill_prices":[{"id":"Skill5","price":25.67},...]}
```

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

### Logging and Trace Mode

The simulation uses structured logging to provide insights into its operation. The **Trace Mode** feature enables comprehensive debug logging for detailed problem diagnosis.

**Basic Logging via Environment Variable:**
```bash
# Default (info level) - High-level progress
./target/release/economic_simulation -s 100 -p 10 -o results.json

# Debug level - Detailed execution information
RUST_LOG=debug ./target/release/economic_simulation -s 100 -p 10 -o results.json

# Trace level - Extremely detailed output
RUST_LOG=trace ./target/release/economic_simulation -s 100 -p 10 -o results.json
```

**Module-Specific Logging:**
```bash
# Only debug engine operations
RUST_LOG=simulation_framework::engine=debug ./target/release/economic_simulation -s 100 -p 10 -o results.json

# Only debug scenario/pricing
RUST_LOG=simulation_framework::scenario=debug ./target/release/economic_simulation -s 100 -p 10 -o results.json

# Multiple modules
RUST_LOG=simulation_framework::engine=debug,simulation_framework::scenario=trace ./target/release/economic_simulation -s 100 -p 10 -o results.json
```

**Log Levels:**
- `error` - Only critical errors that prevent operation
- `warn` - Warnings about potential issues (file I/O errors, invalid configurations)
- `info` - High-level progress information (default) - initialization, completion, performance metrics
- `debug` - **Detailed step-by-step execution:**
  - Trade scheduling and execution with amounts and participants
  - Reputation changes for buyers and sellers
  - Price updates with demand/supply ratios
  - Tax collection and redistribution
  - Loan payments and completion
- `trace` - **Extremely detailed logging:**
  - Individual affordability checks
  - Need satisfaction tracking
  - Savings calculations per person
  - Detailed balance changes

**Trace Mode Examples:**

Debug a specific simulation issue:
```bash
# See all trades and price updates
RUST_LOG=debug ./target/release/economic_simulation -s 50 -p 5 -o debug.json --no-progress

# Trace all affordability decisions
RUST_LOG=trace ./target/release/economic_simulation -s 10 -p 5 -o trace.json --no-progress 2>&1 | grep "cannot afford"
```

Analyze economic behavior:
```bash
# Watch reputation changes
RUST_LOG=debug ./target/release/economic_simulation -s 100 -p 10 -o results.json 2>&1 | grep "reputation"

# Track tax redistribution
RUST_LOG=debug ./target/release/economic_simulation -s 100 -p 10 --tax-rate 0.15 --enable-tax-redistribution -o results.json 2>&1 | grep "Redistributing"
```

**Tips:**
- Use `info` (default) for normal operations
- Use `debug` when investigating simulation behavior, understanding trade dynamics, or troubleshooting
- Use `trace` for deep analysis of individual agent decisions
- Use `warn` or `error` for minimal output in production/batch scenarios
- Combine with `--no-progress` flag to disable the progress bar when using debug/trace logging
- Redirect stderr to a file for large logs: `RUST_LOG=debug ./target/release/economic_simulation ... 2> debug.log`

## Code Structure

*   `src/main.rs`: Handles command-line arguments and initializes the simulation.
*   `src/lib.rs`: Main library crate, exporting core modules.
*   `src/config.rs`: Defines `SimulationConfig` for simulation parameters.
*   `src/engine.rs`: Contains `SimulationEngine` which runs the main simulation loop and step-by-step logic.
*   `src/person.rs`: Defines the `Person` struct, `Transaction`, `NeededSkillItem`, `Strategy` enum, and related types. The `Strategy` enum defines four behavioral strategies (Conservative, Balanced, Aggressive, Frugal) that affect how persons make spending decisions.
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
