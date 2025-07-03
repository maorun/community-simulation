# Economic Simulation Framework

This repository contains a configurable economic simulation written in Rust. It simulates a small economy of individuals, each with unique skills and needs, engaging in trade within a dynamic market. The simulation is designed to explore concepts like price formation, wealth distribution, and market equilibrium.

## Features

- **Agent-Based Simulation:** Simulates individual persons with money, unique skills, and randomly generated needs for other skills.
- **Dynamic Market:** Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).
- **Trading System:** Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.
- **Urgency-Based Decisions:** Persons prioritize buying skills based on a randomly assigned urgency level.
- **Price Volatility:** Skill prices include a configurable random volatility component.
- **Configurable Parameters:** Allows customization of simulation parameters via command-line arguments (number of persons, steps, initial money, etc.).
- **JSON Output:** Outputs detailed simulation results, including final wealth distribution, skill valuations, and skill price history over time (suitable for graphing), to a JSON file.
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

**Example with Custom Parameters:**

```bash
./target/release/economic_simulation --steps 1000 --persons 50 --initial-money 200 --base-price 15 --output custom_results.json --seed 123
```
This runs the simulation for 1000 steps with 50 persons, each starting with 200 money, skills having a base price of 15, and saves results to `custom_results.json` using RNG seed 123.

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

## Output Format (`results.json`)

The JSON output file contains a comprehensive summary of the simulation, including:

*   `total_steps`, `total_duration`, `active_persons`: General simulation metrics.
*   `final_money_distribution`: A list of final money amounts for each active person.
*   `money_statistics`: An object with `average`, `median`, `std_dev`, `min_money`, `max_money`.
*   `final_skill_prices`: A list of all skills sorted by their final price (descending), including `id` and `price`.
*   `most_valuable_skill`, `least_valuable_skill`: Information on the skills with the highest and lowest final prices.
*   `skill_price_history`: A map where keys are `SkillId`s and values are lists of prices for that skill at each step of the simulation. This data can be used for plotting price trends.
*   `final_persons_data`: A list of all person data at the end of the simulation, including their full transaction history.

## License

This project is licensed under the terms of the MIT license. See the `LICENSE` file for details.
