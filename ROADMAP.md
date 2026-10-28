# Roadmap

This file lists open work for the Economic Simulation Framework.
The list uses priorities. Priority 1 is the highest priority.
Do the work in priority order, unless the maintainer gives a different order.

Rules for this file:

- Write in ASD-STE100 Simplified Technical English.
- Add one entry for each open item. Do not give an entry a status. Each entry in this file is open.
- Remove an entry in the pull request that completes it.
- Describe the finished feature in `FEATURES.md`, if users can use it.
- Each entry names the code location, the goal, and the acceptance test.
- Add new ideas to the lowest fitting priority. Use the criteria in "Rules for new features".

## Priority 1: Integrate existing fields into the engine

Each item below is an integration gap. The engine does not use the field, or a user cannot reach it. Tests and validators read the fields, but they do not change the simulation.
Finish this work before you add new fields. See "Integration of New Fields and Types" in `AGENTS.md`.

### `Resource.degradation_factor`

- **Code:** `src/environment.rs` (`Resource::degradation_factor`).
- **Problem:** Only `validate()` and tests call the function. Sustainability scores ignore the factor.
- **Goal:** Apply the factor in the sustainability score or in the resource consumption.
- **Decision needed:** Choose one of the two options.
  - Option A: `score = 1 - consumed * factor / reserves`. Consumption values stay the same.
  - Option B: Multiply each consumption by the factor. This changes `total_consumption` and `remaining_reserves`.
- **Acceptance test:** An engine run with `enable_environment` gives a lower `Energy` score than a `Land` score for equal consumption.
- **Note:** Update the existing tests in `src/environment.rs` that expect the old score values.

### `InvestmentType.maturity_period` and the investment system

- **Code:** `src/investment.rs`, `src/engine.rs` (`investment_statistics: None`).
- **Problem:** The engine does not create, pay, or complete investments. `maturity_period` has no effect.
- **Goal:** Add the investment phase to `step()`. Persons create investments, receive returns, and complete them.
- **Goal:** Pay no return before `maturity_period` ends.
- **Config:** `enable_investments` and the related parameters exist in `src/config.rs`. Use them.
- **Acceptance test:** An engine run with `enable_investments` pays the first return only after `maturity_period` steps.
- **Acceptance test:** `investment_statistics` is `Some` in the result.

### Recipe fields in the interactive mode and in the config

- **Code:** `src/production.rs` (`generate_default_recipes`), `src/engine.rs`, `src/config.rs`.
- **Problem:** The engine only uses the default recipes. A user cannot set `batch_size`, `execution_time`, or other fields from a config file.
- **Goal:** Add a config option for custom recipes with `#[serde(default)]`. Validate each recipe with `Recipe::validate`.
- **Acceptance test:** A TOML config with a custom recipe changes the production result.

## Priority 2: High research value, short term

These items have a high research value and a clear design. Do them after Priority 1.

### Information asymmetry and signaling

- **Source:** `issues/01_informationsasymmetrie_und_signaling.md`.
- **Dependency:** `Recipe.quality_tier` (now a price factor).
- **Goal:** Sellers know the true quality. Buyers see only the market average (lemons problem).
- **Goal:** Add signals, for example certificates and guarantees. Signals cost money.
- **Acceptance test:** Without signals, high-quality sellers leave the market in a controlled scenario.

### Heuristics and bounded rationality

- **Code:** `src/person.rs`, `src/plugin.rs` (`AgentStrategy`).
- **Dependency:** The existing reinforcement learning.
- **Goal:** Add rule-of-thumb strategies, for example "buy from the cheapest seller", "trust friends", and "follow the majority".
- **Goal:** Compare the strategies with optimizing agents.
- **Acceptance test:** A scenario with both agent types shows a measurable difference in average wealth.

### Futures and hedging

- **Source:** `issues/02_futures_and_hedging_contracts.md`.
- **Code:** `src/contract.rs`, `src/engine.rs`.
- **Dependency:** Loans and investments.
- **Goal:** Add a `FuturesContract` with a delivery step, a fixed price, and long and short positions.
- **Acceptance test:** A hedger has lower income variance than a person without a contract.

### Real-time dashboard and WASM playground

- **Source:** `issues/03_realtime_monitoring_dashboard.md`, `issues/04_plugin_system_wasm_architecture.md`.
- **Code:** `web/`, `src/wasm.rs`.
- **Goal:** Add live charts and parameter controls to the playground.
- **Goal:** Stream the metrics over a WebSocket. Use configurable widget layouts.
- **Acceptance test:** The playground shows the Gini coefficient and the average price live during a run.

## Priority 3: Medium term, extend existing features

### Market power and cartels

- **Source:** `issues/07_marktmacht_und_kartellbildung.md`.
- **Dependency:** `per_skill_seller_volumes` in `src/engine.rs`, the Herfindahl index in `src/result.rs`, and the black market system.
- **Goal:** Sellers with a high market share set prices. Cartel members can cheat.
- **Goal:** Add an anti-trust intervention to `src/scenario.rs`.
- **Acceptance test:** A cartel raises the price of one skill above the competitive price. An intervention reduces the price.

### Social capital erosion

- **Code:** `src/person.rs` (friendships), `src/trust_network.rs`.
- **Goal:** Give each friendship a strength value that can fall. Loan defaults, broken contracts, and competition lower it.
- **Goal:** End the friendship when the strength falls below a threshold. Add a repair mechanism.
- **Acceptance test:** A loan default lowers the strength of the friendship between lender and borrower.

### Demographic change and aging

- **Source:** `issues/05_demografischer_wandel_und_alterung.md`.
- **Dependency:** `src/tests/demographics_tests.rs`, `src/person.rs`. The work needs a generation model.
- **Goal:** Persons age. Productivity and skills change with age. Births and deaths change the population structure.
- **Goal:** Add pension payments and care demand. Show the old-age dependency ratio.
- **Acceptance test:** A run with `enable_demographics` shows a rising share of old persons and a falling average productivity.

### Risk-seeking and risk-averse profiles

- **Code:** `src/person.rs`, credit rating and insurance modules.
- **Goal:** Add a `risk_aversion` coefficient for each person. Use it in investment, loan, and insurance decisions.
- **Acceptance test:** Risk-averse persons buy more insurance than risk-seeking persons in the same scenario.

### Time-series database integration

- **Code:** `src/database.rs`, `src/result.rs`.
- **Goal:** Write time series to a time-series database, for example InfluxDB or TimescaleDB, during the run.
- **Goal:** Add a query API for multi-run comparison.
- **Acceptance test:** A long run writes its series without a rise in memory use.

### Network graph visualization

- **Dependency:** `src/trust_network.rs`, `web/`.
- **Goal:** Show the social network with friendship and trust links. Use a force-directed layout, zoom, and node details.
- **Goal:** Color the nodes by community.
- **Acceptance test:** The page shows all persons and links of a 100-person run.

### Bank runs and liquidity shortages

- **Dependency:** `src/lending_marketplace.rs`, `src/trust_network.rs`, `src/crisis.rs`.
- **Goal:** A confidence shock causes mass withdrawals. Deposit insurance and circuit breakers limit the effect.
- **Acceptance test:** A shock without insurance causes more failed loans than a shock with insurance.

### Arbitrage and intermediaries

- **Source:** `issues/06_arbitrage_handel_und_zwischenhaendler.md`.
- **Code:** `src/market.rs`, `src/person.rs` (new strategy `Arbitrageur`).
- **Goal:** Add market segments or regions. An arbitrageur buys cheap and sells at a higher price.
- **Acceptance test:** Arbitrage reduces the price difference between two segments.

### Social mobility and class barriers

- **Code:** `src/person.rs`, `src/config.rs`.
- **Goal:** Give each person a class at birth (lower, middle, upper). Make education cost and loan access depend on the class.
- **Goal:** Let network effects strengthen the class separation.
- **Acceptance test:** With class barriers on, fewer persons from the lower class reach the upper class than with barriers off.

### Cultural and language barriers

- **Code:** `src/person.rs`, `src/nepotism.rs`.
- **Goal:** Add culture tags. Cross-culture trade has higher transaction costs and slower trust building.
- **Goal:** Add language knowledge as a skill.
- **Acceptance test:** With barriers on, the share of trades inside the own culture is higher than with barriers off.

### Reputation systems with manipulation

- **Code:** `src/reputation.rs` (if the module is missing, add it), `src/trust_network.rs`.
- **Goal:** Persons can buy fake reviews, run Sybil attacks, or coordinate ratings.
- **Goal:** Add a detection probability and a penalty.
- **Acceptance test:** Manipulation raises the reputation of the attacker. Detection removes the gain.

### Resource scarcity and peak oil

- **Code:** `src/environment.rs`.
- **Dependency:** The item "`Resource.degradation_factor`" in this file.
- **Goal:** Add finite resource pools with extraction rates. Extraction cost rises when the pool runs low.
- **Goal:** Add alternative energy sources with higher start costs.
- **Acceptance test:** In a long run, the price of the scarce resource rises and demand moves to the alternative.

### Migration dynamics

- **Dependency:** The item "Arbitrage and intermediaries" in this file (regions).
- **Goal:** Add a multi-region simulation. Persons move because of economic chances, quality of life, and crises.
- **Goal:** Add migration costs and access to the home market.
- **Acceptance test:** A crisis in one region moves persons to the other region.

### Altruism and charity

- **Code:** `src/person.rs`.
- **Goal:** Add a utility function with a weighted average of the well-being of others.
- **Goal:** Add voluntary transfers to poorer persons.
- **Acceptance test:** Altruistic persons give money, and the Gini coefficient falls in a controlled scenario.

### Emotional and irrational strategies

- **Code:** `src/person.rs`.
- **Goal:** Add the states fear, greed, and anger. Add panic sales, fear of missing out, and revenge.
- **Goal:** Add feedback loops that strengthen the states.
- **Acceptance test:** A scenario with emotional agents shows higher price volatility than the same scenario without them.

## Priority 4: Improve the code structure

### Split large files

- **Code:** `src/engine.rs` (about 7000 lines), `src/result.rs` (about 7000 lines), `src/config.rs` (about 4800 lines).
- **Goal:** Move the phases of `step()` to submodules, for example `engine/production.rs`, `engine/trade.rs`, `engine/lending.rs`.
- **Goal:** Split `result.rs` into statistics and export modules.
- **Rule:** Do not change behavior in the same pull request. The golden tests must pass without a baseline update.

### Group feature configuration

- **Code:** `src/config.rs`.
- **Problem:** `SimulationConfig` has flat fields, for example `enable_nepotism` and `nepotism_bias`.
- **Goal:** Group the fields in sub-structures, for example `nepotism: NepotismConfig`.
- **Rule:** Keep the old flat format readable with serde aliases.

### Use the `Plugin` trait for new mechanics

- **Code:** `src/plugin.rs`.
- **Goal:** Implement new phases as plugins. Do not add more code to `step()`.

### Recipe builders

- **Code:** `src/production.rs`.
- **Problem:** The module has one `with_*` constructor for each field combination. The names reach 90 characters.
- **Goal:** Replace the constructors with a `RecipeBuilder`. Keep the old constructors as deprecated wrappers.
- **Acceptance test:** All existing recipe tests pass without change.

### State machine for the simulation lifecycle

- **Code:** `src/engine.rs`.
- **Goal:** Define the phases pre-step, trading, post-trading, and update. Add hooks at each transition.
- **Acceptance test:** A test plugin receives the hooks in a fixed order.

### Command pattern and replay

- **Goal:** Model actions as commands with `execute` and `undo`. Use snapshots for undo.
- **Acceptance test:** A replay of the command log gives the same result as the original run.

### Dependency injection

- **Goal:** Use trait abstractions instead of hard-coded component links. Inject mocks in tests.
- **Note:** Do this item only if the items "Use the `Plugin` trait for new mechanics" and "State machine for the simulation lifecycle" do not give enough testability.

## Priority 5: Improve tests and tooling

### Rename and merge test files

- **Code:** `src/tests/`.
- **Problem:** Names such as `coverage_80_breakthrough.rs`, `final_80_percent_push.rs`, and `ultra_final_tests.rs` do not name a domain.
- **Goal:** Rename the files by domain, for example `market_tests.rs`. Remove duplicate tests.

### Clean the repository root

- **Files:** `COVERAGE_80_SUCCESS.md`, `COVERAGE_87_ACHIEVED.md`, `COVERAGE_ACHIEVEMENT.md`, `COVERAGE_SUMMARY.md`, `FINAL_80_PERCENT_SUCCESS.md`, `FINAL_SUMMARY.md`.
- **Goal:** Merge the content into `COVERAGE.md`. Delete the other files.

### Property tests for invariants

- **Code:** `src/invariant.rs`, `src/tests/proptest_tests.rs`.
- **Goal:** For random seeds and random configs, check that the money supply stays constant and that no value is `NaN`.
- **Goal:** Add proptest strategies for config generation.

### Determinism test for Rayon

- **Code:** `src/tests/`, `tests/golden_regression.rs`.
- **Goal:** Run the same seed with `RAYON_NUM_THREADS=1` and with several threads.
- **Acceptance test:** Both runs give identical results.

### Benchmark regression in CI

- **Code:** `benches/simulation_benchmarks.rs`, `.github/workflows/rust.yml`.
- **Goal:** Compare Criterion results with a stored baseline. Fail the job at a defined slowdown.
- **Acceptance test:** A deliberate slowdown of 20 percent in `step()` fails the CI job.

### Profile `step()`

- **Goal:** Create a flamegraph with 10000 persons. Remove the hot spots.
- **Example:** `attempt_production` allocates a `Vec<SkillId>` for each person in each step.

### Fuzz tests for configurations

- **Goal:** Generate random configs with `cargo-fuzz`. Check for panics.
- **Acceptance test:** The fuzz target runs for 10 minutes without a crash.

### Mutation testing

- **Goal:** Run `cargo-mutants` in CI. Track the mutation score.
- **Acceptance test:** CI reports the score for `src/market.rs` and `src/engine.rs`.

### Differential testing

- **Goal:** Compare simulation results with a reference implementation or with analytical solutions.
- **Acceptance test:** A simple market with a known equilibrium converges to the analytical price.

### Chaos tests

- **Goal:** Inject faults, for example agent failures and corrupt checkpoints. Measure the recovery.
- **Acceptance test:** The engine finishes the run or returns a `SimulationError`. It does not panic.

## Priority 6: Low priority, long term

These items have a high effort or a narrow use. Start them only after a maintainer decision.

### Technological singularity

- **Goal:** Add a fast rise of the automation risk and sudden skill devaluation. Add a basic income experiment.
- **Acceptance test:** A run shows a sharp rise of unemployment. A basic income lowers the Gini coefficient.

### Pandemic and health crises

- **Goal:** Add an infection status that spreads over trade contacts. Add lower productivity, treatment costs, deaths, and quarantine.
- **Acceptance test:** Quarantine lowers the number of infections and the economic output.

### Plugin system with WASM or dynamic libraries

- **Code:** `src/plugin.rs`, `src/wasm.rs`.
- **Goal:** Load plugins at run time. Add a plugin registry and a sandbox.
- **Note:** Start this item when the community asks for it.

### GPU acceleration

- **Goal:** Move network analysis and matrix work to the GPU.
- **Note:** The gain starts at about 10000 persons.

### Performance ideas

- **Adaptive sampling:** Update unimportant agents less often.
- **Incremental network updates:** Recompute centrality and clustering only when the graph changes.
- **Memory pooling:** Reuse objects instead of allocation in each step.
- **Rule:** Measure with the item "Profile `step()`" before you start any of them.

### Data management ideas

- **Incremental checkpointing:** Save only the changed data.
- **Distributed simulation database:** Shard the data over several nodes.
- **Note:** Parquet export exists in `src/result.rs`. Do not add it again.

### Developer tools

- **Time-travel debugger:** Step forward and backward with breakpoints. Depends on the item "Command pattern and replay".
- **Agent profiler:** Measure the CPU time of each agent.
- **Causality tracer:** Answer "why did agent X lose money?" with an event dependency graph.
- **Assertion DSL:** Let users define their own invariants.
- **Visual scenario builder:** Create scenarios in a web editor and export them to YAML.

### Visualization ideas

- **Lorenz curves:** Show animated curves for wealth and income over time.
- **Heatmaps:** Show trade activity by grid position.

### Documentation and education

- **Interactive tutorials:** Use mdBook with runnable snippets.
- **Economics glossary:** Link each concept to its implementation.
- **Scenario library:** Collect scenarios with tags, difficulty, and expected results.

### Formal verification

- **Goal:** Prove key invariants, for example money conservation, with a verification tool.

## Rules for new features

- Keep backward compatibility. Add `#[serde(default)]` to each new serialized field.
- Write unit tests and integration tests.
- Update `README.md` and the code comments.
- Think about the performance effect.
- Put each feature behind a config flag or a compile-time feature.
- Connect each feature to three to five existing features. Make the strength of each link configurable.
- Make each feature produce data that other features can use.

Use these criteria to rank a new idea:

1. **Harmony score (0 to 10):** How many existing features does the idea use or extend?
2. **Effort:** S (1 to 2 weeks), M (1 month), L (2 to 3 months), XL (more than 3 months).
3. **Research relevance (0 to 10):** Does an established economic theory exist?
4. **Practical relevance (0 to 10):** How often does the effect occur in real markets?
5. **Teaching value (0 to 10):** Is the idea easy to understand?

| Feature | Harmony | Effort | Research | Practice | Teaching | Total |
|---|---|---|---|---|---|---|
| Information asymmetry | 9 | M | 10 | 9 | 10 | 38/50 |
| Heuristics | 9 | S | 8 | 8 | 9 | 34/50 |
| Market power and cartels | 7 | L | 9 | 10 | 8 | 34/50 |
| Futures | 8 | M | 8 | 10 | 7 | 33/50 |
| Demographic change | 6 | XL | 9 | 10 | 7 | 32/50 |
| Real-time dashboard | 5 | M | 5 | 6 | 8 | 24/50 |
| GPU acceleration | 3 | XL | 4 | 5 | 3 | 15/50 |
