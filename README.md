# Economic Simulation Framework

[![Rust CI](https://github.com/maorun/community-simulation/actions/workflows/rust.yml/badge.svg)](https://github.com/maorun/community-simulation/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/maorun/community-simulation/branch/master/graph/badge.svg)](https://codecov.io/gh/maorun/community-simulation)

A configurable economic simulation written in Rust that models a small economy of individuals with unique skills engaging in trade within a dynamic market. Perfect for exploring price formation, wealth distribution, market equilibrium, and economic policy effects.

## What Can This Project Do?

This framework enables you to simulate and analyze:

- **Economic Systems**: Market dynamics, price formation, supply/demand, wealth distribution, trade flows
- **Social Networks**: Friendship formation, influence dynamics, reputation systems, social capital
- **Policy Analysis**: Tax systems, redistribution, insurance, automation impacts, crisis management
- **Market Mechanisms**: Multiple pricing strategies (supply/demand, auctions, adaptive pricing, climate effects)
- **Capital & Investment**: Loans, credit ratings, assets (property, equipment, stocks), savings
- **Technology Effects**: Gradual progress, breakthrough innovations, automation and unemployment
- **Advanced Dynamics**: Geographic distance costs, seasonal demand, quality ratings, social classes

**Use cases**: Economic research, policy simulation, market behavior studies, agent-based modeling, educational demonstrations.

## Core Features

### Agent-Based Simulation
- Autonomous persons with money, skills, and needs
- Geographic locations with distance-based trade costs
- Multiple skills per person for realistic labor dynamics
- Transaction history and behavioral patterns
- **Satisficing Behavior**: Agents can use "good enough" heuristics instead of optimal decision-making (bounded rationality)
- **Reinforcement Learning**: Agents adapt spending strategies through Q-learning, using epsilon-greedy exploration to balance exploitation vs exploration

### Economic Mechanisms
- **Pricing Scenarios**: Supply/demand, dynamic, adaptive, auction, climate change
- **Demand Strategies**: Uniform, concentrated (Pareto), cyclical (business cycles)
- **Market Systems**: Dynamic price adjustments, seasonal effects, transaction fees
- **Market Segmentation**: Budget/Mittelklasse/Luxury segments based on wealth with differentiated price-quality preferences
- **Auction Infrastructure**: English auction mechanism with bid collection and winner selection (configurable via `--enable-auctions`)
- **Wealth Building**: Savings, loans with interest, assets (property/equipment/stocks)

### Social & Network Systems
- **Reputation**: Scores that affect prices and trade opportunities
- **Friendships**: Social networks with trading benefits and influence tracking
- **Social Classes**: Automatic classification with mobility tracking (lower/middle/upper/elite)
- **Trade Agreements**: Bilateral agreements with preferential pricing

### Risk & Financial Systems
- **Insurance**: Crisis, income, and credit insurance with risk-based pricing
- **Credit Rating**: FICO-like scoring (300-850) affecting loan rates
- **Loans**: Borrowing/lending with configurable interest and repayment
- **P2P Lending Marketplace**: Decentralized peer-to-peer lending with automatic matching based on credit scores and risk preferences
- **Crisis Events**: Random economic shocks to test system resilience

### Technology & Progress
- **Technological Progress**: Gradual efficiency improvements
- **Technology Breakthroughs**: Sudden innovation events with major impacts
- **Automation**: Skill displacement modeling for labor market studies
- **Quality System**: Skill quality ratings that affect prices and evolve over time

### Analysis & Research Tools
- **Externality Analysis**: Track social costs/benefits and optimal policy interventions
- **Investment Tracking**: Monitor capital allocation and returns
- **Comprehensive Statistics**: Wealth distribution, transaction patterns, mobility metrics
- **Multiple Output Formats**: JSON, CSV, time-series exports
- **Terminal Visualization**: ASCII-based price history charts showing skill price evolution over time (`--show-price-chart`)

📖 **For detailed feature explanations**, see [FEATURES.md](FEATURES.md).

## Quick Start

### Prerequisites

- Rust Toolchain ([installation guide](https://www.rust-lang.org/tools/install))

### Installation & Basic Usage

1. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd community-simulation
   cargo build --release
   ```

2. **Run a basic simulation:**
   ```bash
   ./target/release/community-simulation run -o results.json
   ```

3. **Try preset scenarios:**
   ```bash
   # Gig Economy (platform with ratings, fees, surge pricing)
   ./target/release/community-simulation run --preset gig_economy -o results.json
   
   # Economic crisis scenario
   ./target/release/community-simulation run --preset crisis_scenario -o results.json
   
   # List all available presets
   ./target/release/community-simulation list presets
   ```

4. **Interactive configuration wizard:**
   ```bash
   ./target/release/community-simulation wizard
   ```

### Example: Custom Simulation

```bash
# Simulation with social networks, automation, and crisis events
./target/release/community-simulation run \
  --steps 1000 \
  --persons 100 \
  --initial-money 150 \
  --scenario AdaptivePricing \
  --enable-friendships \
  --enable-influence \
  --enable-automation \
  --enable-crisis-events \
  --crisis-probability 0.02 \
  -o custom_results.json

# Market segmentation simulation
./target/release/community-simulation run \
  --steps 500 \
  --persons 100 \
  --enable-market-segments \
  --enable-quality \
  --enable-reputation \
  -o market_segments.json

# Satisficing (bounded rationality) simulation
# Agents accept "good enough" options rather than always seeking optimal
./target/release/community-simulation run \
  --steps 500 \
  --persons 100 \
  --enable-satisficing \
  --satisficing-threshold 0.6 \
  -o satisficing.json
```

**Note:** Reinforcement Learning is currently configurable via YAML/TOML config files only. Example configuration:

```yaml
enable_reinforcement_learning: true
rl_learning_rate: 0.1              # Learning speed (alpha)
rl_discount_factor: 0.9            # Future reward importance (gamma)  
rl_epsilon: 0.1                    # Initial exploration rate (10%)
rl_epsilon_decay: 0.995            # Exploration decay per step
rl_reward_success_multiplier: 1.0  # Reward scaling for successful trades
rl_reward_failure_multiplier: 0.5  # Penalty scaling (reserved for future use)
```

### Using Configuration Files

For complex scenarios, use YAML or TOML configuration files:

```bash
# Run with configuration file
./target/release/community-simulation run --config config.gig_economy.yaml -o results.json

# Available preset configs:
# - config.example.yaml/toml - Basic configuration example
# - config.gig_economy.yaml - Gig economy simulation
# - config.strategy_evolution.yaml - Strategy evolution scenario
# - config.comprehensive.yaml/toml - All features enabled
```

### Output & Analysis

The simulation outputs detailed JSON results including:
- Per-person statistics (money, transactions, skills, reputation, assets)
- Market data (price history, supply/demand)
- Social networks (friendships, influence, class mobility)
- Economic metrics (wealth distribution, Gini coefficient, trade volumes)
- System statistics (taxes, loans, insurance, automation effects)

Export to multiple formats for analysis:
```bash
# Export to CSV for spreadsheet analysis
./target/release/community-simulation export results.json --format csv --output results.csv

# Time-series export for plotting
./target/release/community-simulation export results.json --format timeseries --output timeseries.csv

# NEW: Export directly to Parquet for big-data analytics
./target/release/community-simulation run -s 500 -p 100 --parquet-output results.parquet
```

**Parquet Export Benefits:**
- **10x smaller** file size compared to JSON/CSV (with compression)
- **Columnar format** - Fast aggregations and filtering
- **Type-safe** - Schema enforcement and strong typing
- **Interoperability** - Works with pandas, DuckDB, Spark, Apache Arrow

Analyze Parquet data:
```python
# Python with pandas
import pandas as pd
df = pd.read_parquet('results.parquet')
trade_volume = df[df['metric'] == 'trade_count']
print(trade_volume.groupby('step')['value'].sum())
```

```sql
-- DuckDB
SELECT step, value FROM 'results.parquet'
WHERE metric = 'avg_money'
ORDER BY step;
```


## Documentation

- **[FEATURES.md](FEATURES.md)** - Comprehensive feature documentation with examples and configuration details
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build instructions, code structure, testing, and contribution guidelines
- **[LINTING.md](LINTING.md)** - Code quality and linting information
- **[COVERAGE.md](COVERAGE.md)** - Code coverage reports and testing metrics

### Quick Links

**For Users:**
- [All available features](FEATURES.md)
- [Configuration file reference](FEATURES.md#configuration-parameters)
- [CLI commands](DEVELOPMENT.md#cli-structure)

**For Developers:**
- [Building the project](DEVELOPMENT.md#building-the-project)
- [Running tests](DEVELOPMENT.md#testing)
- [Code structure](DEVELOPMENT.md#code-structure)
- [Contributing guidelines](DEVELOPMENT.md#contributing)

## CLI Command Structure

```bash
community-simulation <COMMAND> [OPTIONS]

Commands:
  run        Run a simulation
  export     Export results to different formats
  wizard     Interactive configuration wizard
  list       List available presets and scenarios
  help       Print help information

Run 'community-simulation help <COMMAND>' for command-specific help.
```

### Common CLI Parameters

Essential parameters for customizing simulations:

```bash
-s, --steps <STEPS>              Number of simulation steps (default: 500)
-p, --persons <PERSONS>          Number of persons (default: 100)
--initial-money <AMOUNT>         Starting money per person (default: 100.0)
--base-price <PRICE>            Base price for skills (default: 10.0)
--scenario <SCENARIO>           Pricing scenario (default: Original)
                                Options: Original, DynamicPricing, AdaptivePricing,
                                         AuctionPricing, ClimateChange
-o, --output <FILE>             Output JSON file path
--csv-output <PREFIX>           Export CSV files with prefix
--parquet-output <FILE>         Export Parquet file for big-data analytics
--timeseries-output <FILE>      Export time-series CSV for analysis
--config <FILE>                 Load configuration from YAML/TOML file
--seed <SEED>                   RNG seed for reproducibility (default: 42)
--show-price-chart              Display ASCII price history chart in terminal
--no-histogram                  Disable wealth distribution histogram
```

For the complete parameter list, see [FEATURES.md](FEATURES.md) or run:
```bash
./target/release/community-simulation run --help
```

## Performance

- **Typical runtime**: 500 steps with 100 persons completes in < 1 second (release build)
- **Parallelization**: Uses Rayon for parallel processing (configurable with `--threads`)
- **Memory**: Proportional to persons × steps (for transaction history)
- **Optimization**: Release builds use LTO and are 10-20x faster than debug builds

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [DEVELOPMENT.md](DEVELOPMENT.md) for:
- Build and test instructions
- Code structure overview
- Development workflow
- Style guidelines

For questions or issues, please open a GitHub issue.
