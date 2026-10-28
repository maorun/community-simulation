# Roadmap

This file lists open work for the Economic Simulation Framework.
The list uses priorities. Priority 1 is the highest priority.
Do the work in priority order, unless the maintainer gives a different order.

Rules for this file:

- Write in ASD-STE100 Simplified Technical English.
- Add one entry for each open item. Give the entry a status.
- Remove an entry when the pull request that completes it is merged.
- Each entry names the code location, the goal, and the acceptance test.

Status values: `open`, `in progress`, `blocked`.

## Priority 1: Integrate existing fields into the engine

Each item below is an integration gap. The engine does not use the field, or a user cannot reach it. Tests and validators read the fields, but they do not change the simulation.
Finish this work before you add new fields. See "Integration of New Fields and Types" in `AGENTS.md`.

### 1.1 `Resource.degradation_factor` — status: open

- **Code:** `src/environment.rs` (`Resource::degradation_factor`).
- **Problem:** Only `validate()` and tests call the function. Sustainability scores ignore the factor.
- **Goal:** Apply the factor in the sustainability score or in the resource consumption.
- **Decision needed:** Choose one of the two options.
  - Option A: `score = 1 - consumed * factor / reserves`. Consumption values stay the same.
  - Option B: Multiply each consumption by the factor. This changes `total_consumption` and `remaining_reserves`.
- **Acceptance test:** An engine run with `enable_environment` gives a lower `Energy` score than a `Land` score for equal consumption.
- **Note:** Update the existing tests in `src/environment.rs` that expect the old score values.

### 1.2 `InvestmentType.maturity_period` and the investment system — status: open

- **Code:** `src/investment.rs`, `src/engine.rs` (`investment_statistics: None`).
- **Problem:** The engine does not create, pay, or complete investments. `maturity_period` has no effect.
- **Goal:** Add the investment phase to `step()`. Persons create investments, receive returns, and complete them.
- **Goal:** Pay no return before `maturity_period` ends.
- **Config:** `enable_investments` and the related parameters exist in `src/config.rs`. Use them.
- **Acceptance test:** An engine run with `enable_investments` pays the first return only after `maturity_period` steps.
- **Acceptance test:** `investment_statistics` is `Some` in the result.

### 1.3 Recipe fields in the interactive mode and in the config — status: open

- **Code:** `src/production.rs` (`generate_default_recipes`), `src/engine.rs`, `src/config.rs`.
- **Problem:** The engine only uses the default recipes. A user cannot set `batch_size`, `execution_time`, or other fields from a config file.
- **Goal:** Add a config option for custom recipes with `#[serde(default)]`. Validate each recipe with `Recipe::validate`.
- **Acceptance test:** A TOML config with a custom recipe changes the production result.

## Priority 2: Add quality and market features

### 2.1 Information asymmetry and signaling — status: open

- **Source:** `issues/01_informationsasymmetrie_und_signaling.md`.
- **Dependency:** `Recipe.quality_tier` (now a price factor).
- **Goal:** Sellers know the true quality. Buyers see only the market average (lemons problem).
- **Goal:** Add signals, for example certificates and guarantees. Signals cost money.
- **Acceptance test:** Without signals, high-quality sellers leave the market in a controlled scenario.

### 2.2 Market power and cartels — status: open

- **Source:** `issues/07_marktmacht_und_kartellbildung.md`.
- **Dependency:** `per_skill_seller_volumes` in `src/engine.rs` and the Herfindahl index in `src/result.rs`.
- **Goal:** Sellers with a high market share set prices. Cartel members can cheat.
- **Goal:** Add an anti-trust intervention to `src/scenario.rs`.
- **Acceptance test:** A cartel raises the price of one skill above the competitive price. An intervention reduces the price.

### 2.3 Bank runs and liquidity shortages — status: open

- **Source:** `TODO.md`, heading "Liquiditätsengpässe & Bankruns".
- **Dependency:** `src/lending_marketplace.rs`, `src/trust_network.rs`, `src/crisis.rs`.
- **Goal:** A confidence shock causes mass withdrawals. Deposit insurance and circuit breakers limit the effect.
- **Acceptance test:** A shock without insurance causes more failed loans than a shock with insurance.

### 2.4 Arbitrage and intermediaries — status: open

- **Source:** `issues/06_arbitrage_handel_und_zwischenhaendler.md`.
- **Code:** `src/market.rs`, `src/person.rs` (new strategy `Arbitrageur`).
- **Goal:** Add market segments or regions. An arbitrageur buys cheap and sells at a higher price.
- **Acceptance test:** Arbitrage reduces the price difference between two segments.

### 2.5 Futures and hedging — status: open

- **Source:** `issues/02_futures_and_hedging_contracts.md`.
- **Code:** `src/contract.rs`, `src/engine.rs`.
- **Goal:** Add a `FuturesContract` with a delivery step, a fixed price, and long and short positions.
- **Acceptance test:** A hedger has lower income variance than a person without a contract.

### 2.6 Demographic change and aging — status: open

- **Source:** `issues/05_demografischer_wandel_und_alterung.md`.
- **Dependency:** `src/tests/demographics_tests.rs`, `src/person.rs`.
- **Goal:** Persons age. Productivity falls with age. Births and deaths change the population structure.
- **Acceptance test:** A run with `enable_demographics` shows a rising share of old persons and a falling average productivity.

## Priority 3: Improve the code structure

### 3.1 Split large files — status: open

- **Code:** `src/engine.rs` (about 7000 lines), `src/result.rs` (about 7000 lines), `src/config.rs` (about 4800 lines).
- **Goal:** Move the phases of `step()` to submodules, for example `engine/production.rs`, `engine/trade.rs`, `engine/lending.rs`.
- **Goal:** Split `result.rs` into statistics and export modules.
- **Rule:** Do not change behavior in the same pull request. The golden tests must pass without a baseline update.

### 3.2 Group feature configuration — status: open

- **Code:** `src/config.rs`.
- **Problem:** `SimulationConfig` has flat fields, for example `enable_nepotism` and `nepotism_bias`.
- **Goal:** Group the fields in sub-structures, for example `nepotism: NepotismConfig`.
- **Rule:** Keep the old flat format readable with serde aliases.

### 3.3 Use the `Plugin` trait for new mechanics — status: open

- **Code:** `src/plugin.rs`.
- **Goal:** Implement new phases as plugins. Do not add more code to `step()`.

### 3.4 Recipe builders — status: open

- **Code:** `src/production.rs`.
- **Problem:** The module has one `with_*` constructor for each field combination. The names reach 90 characters.
- **Goal:** Replace the constructors with a `RecipeBuilder`. Keep the old constructors as deprecated wrappers.
- **Acceptance test:** All existing recipe tests pass without change.

## Priority 4: Improve tests and tooling

### 4.1 Rename and merge test files — status: open

- **Code:** `src/tests/`.
- **Problem:** Names such as `coverage_80_breakthrough.rs`, `final_80_percent_push.rs`, and `ultra_final_tests.rs` do not name a domain.
- **Goal:** Rename the files by domain, for example `market_tests.rs`. Remove duplicate tests.

### 4.2 Clean the repository root — status: open

- **Files:** `COVERAGE_80_SUCCESS.md`, `COVERAGE_87_ACHIEVED.md`, `COVERAGE_ACHIEVEMENT.md`, `COVERAGE_SUMMARY.md`, `FINAL_80_PERCENT_SUCCESS.md`, `FINAL_SUMMARY.md`.
- **Goal:** Merge the content into `COVERAGE.md`. Delete the other files.

### 4.3 Property tests for invariants — status: open

- **Code:** `src/invariant.rs`, `src/tests/proptest_tests.rs`.
- **Goal:** For random seeds, check that the money supply stays constant and that no value is `NaN`.

### 4.4 Determinism test for Rayon — status: open

- **Code:** `src/tests/`, `tests/golden_regression.rs`.
- **Goal:** Run the same seed with `RAYON_NUM_THREADS=1` and with several threads.
- **Acceptance test:** Both runs give identical results.

### 4.5 Benchmark regression in CI — status: open

- **Code:** `benches/simulation_benchmarks.rs`, `.github/workflows/rust.yml`.
- **Goal:** Compare Criterion results with a stored baseline. Fail the job at a defined slowdown.
- **Acceptance test:** A deliberate slowdown of 20 percent in `step()` fails the CI job.

### 4.6 Profile `step()` — status: open

- **Goal:** Create a flamegraph with 10000 persons. Remove the hot spots.
- **Example:** `attempt_production` allocates a `Vec<SkillId>` for each person in each step.

### 4.7 Real-time dashboard and WASM playground — status: open

- **Source:** `issues/03_realtime_monitoring_dashboard.md`, `issues/04_plugin_system_wasm_architecture.md`.
- **Code:** `web/`, `src/wasm.rs`.
- **Goal:** Add live charts and parameter controls to the playground.

## Further ideas

The file `TODO.md` lists more ideas. Examples: "Soziale Mobilität & Klassenbarrieren", "Kulturelle/Sprachliche Barrieren", "Reputationssysteme mit Manipulation", and "Sozialkapital-Erosion".
Move an idea from `TODO.md` to this file when work on it starts.
