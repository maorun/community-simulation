# Economic Simulation Framework

This repository contains a configurable economic simulation written in Rust. It simulates a small economy of individuals, each with unique skills and needs, engaging in trade within a dynamic market. The simulation is designed to explore concepts like price formation, wealth distribution, and market equilibrium.

## Features

- **Agent-Based Simulation:** Simulates individual persons with money, unique skills, and randomly generated needs for other skills.
- **Geographic Locations:** Each person has a 2D location (x, y coordinates) in a virtual space. When enabled via `--distance-cost-factor`, trade costs increase based on the Euclidean distance between buyer and seller, simulating transportation costs and geographic barriers to trade. The distance multiplier is applied as: `final_cost = base_cost * (1 + distance * distance_cost_factor)`. For example, with a factor of 0.01 and a distance of 50 units, costs increase by 50%. Location data is included in JSON output for spatial economic analysis.
- **Multiple Skills Per Person:** Each person can possess and offer multiple skills in the market, creating more realistic labor dynamics with skill redundancy and competition. Configurable via `--skills-per-person` parameter (default: 1). When set to values > 1, skills are distributed across persons using a round-robin approach, allowing multiple providers per skill and more complex market interactions.
- **Multiple Pricing Scenarios:** Choose from different market price mechanisms to study their effects:
  - **Original** (default): Supply/demand-based pricing with random volatility - prices adjust based on the ratio of buyers to sellers
  - **DynamicPricing**: Sales-based pricing - prices increase 5% when sold, decrease 5% when not sold
  - **AdaptivePricing**: Gradual price adaptation using exponential moving average with 20% learning rate for smooth convergence
  - **AuctionPricing**: Competitive bidding mechanism where prices increase aggressively when multiple buyers compete for the same skill (simulating auction psychology), with gentler decreases when demand is low. Uses quadratic competition factor to model bidding war intensity. Ideal for studying price spikes in competitive markets and auction-like dynamics.
- **Dynamic Market:** Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).
- **Demand Generation Strategies:** Configurable strategies for determining how many skills each person needs per step. Three strategies available:
  - **Uniform** (default): Random 2-5 needs with equal probability, creating balanced markets
  - **Concentrated**: Pareto-like distribution (70% low demand, 30% high demand), simulating consumption inequality
  - **Cyclical**: Time-varying demand in cycles, simulating business cycle dynamics with expansion and contraction phases
  
  Controlled via `--demand-strategy` parameter. Enables studying how demand patterns affect market behavior, wealth distribution, and economic resilience. Interacts with pricing mechanisms and seasonal effects to create complex market dynamics.
- **Trading System:** Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.
- **Panic Recovery:** Robust error handling with graceful degradation - if a panic occurs during simulation step execution, it is caught and logged, allowing the simulation to continue. Failed steps are tracked and reported in the results.
- **Reputation System:** Each person has a reputation score (starting at 1.0) that increases with successful trades. Higher reputation leads to better prices (up to 10% discount), while lower reputation results in price premiums. Reputation slowly decays toward neutral over time, encouraging ongoing positive behavior.
- **Quality Rating System:** Skills have quality ratings (0.0-5.0 scale) that evolve over time and affect prices. Higher quality skills command higher prices, creating quality competition alongside price competition. Quality improves through successful trades (practice makes perfect) at a configurable rate (default: +0.1 per trade, capped at 5.0). Unused skills decay in quality over time (default: -0.05 per step, minimum 0.0), simulating skill rust. Price adjustment formula: `price * (1.0 + (quality - 3.0) * 0.1)` means quality 5.0 gives +20% price, quality 3.0 (average) gives base price, and quality 1.0 gives -20% price. Quality statistics (average, median, min/max, skills at extremes) are tracked and reported. Enable via `--enable-quality` flag or configuration file with parameters `quality_improvement_rate` (default: 0.1), `quality_decay_rate` (default: 0.05), and `initial_quality` (default: 3.0). This system enables studying product differentiation, the relationship between experience and value, and the importance of maintaining skills through regular use. Works well with reputation system and education system for comprehensive skill value modeling.
- **Friendship System:** Social network dynamics through friendship formation between trading partners. When enabled, persons who successfully trade together have a configurable probability of becoming friends (bidirectional relationships). Friends receive price discounts when trading with each other (default: 10% discount), simulating trust and social capital. The system tracks comprehensive friendship statistics including total friendships formed, average/min/max friends per person, and network density (ratio of actual to possible friendships, ranging from 0.0 to 1.0). Enable via configuration file with `enable_friendships: true`, then configure `friendship_probability` (0.0-1.0, default: 0.1 or 10% chance per trade) and `friendship_discount` (0.0-1.0, default: 0.1 or 10% discount). Friendship discounts stack with reputation-based price adjustments. Ideal for studying social networks, trust in markets, and the economic impact of social relationships.
- **Trade Agreements:** Bilateral trade agreements between persons providing mutual price discounts on trades between agreement partners. When enabled, persons with existing friendships can form trade agreements at a configurable probability each simulation step (default: 5%). Agreements have a limited duration (default: 100 steps) and automatically expire, requiring renewal. Agreement partners receive an additional price discount (default: 15%) that stacks with friendship discounts, enabling study of preferential trading relationships and regional economic blocks. The system tracks comprehensive statistics including total agreements formed, active/expired counts, bilateral vs multilateral agreements, trade volume under agreements, and average discount rates. Enable via configuration file with `enable_trade_agreements: true`, then configure `trade_agreement_probability` (0.0-1.0, default: 0.05 or 5% chance per step), `trade_agreement_discount` (0.0-1.0, default: 0.15 or 15% discount), and `trade_agreement_duration` (in steps, default: 100). Requires friendships to be enabled for realistic behavior. Perfect for studying trade policy, economic integration, and the impact of preferential trade relationships on wealth distribution and market dynamics.
- **Technological Progress:** Skills become more efficient over time through a configurable technology growth rate, simulating productivity improvements. More efficient skills effectively cost less, enabling increased trade and economic growth over the simulation period.
- **Seasonal Demand Effects:** Configurable seasonal fluctuations in skill demand using cyclical patterns. Different skills experience peak demand at different times, creating realistic market dynamics and economic cycles. Controlled via `--seasonal-amplitude` and `--seasonal-period` parameters.
- **Transaction Fees:** Configurable marketplace transaction fees that are deducted from seller proceeds on each trade. Simulates realistic trading costs (e.g., platform fees, payment processing) and allows studying the impact of fees on market liquidity, wealth distribution, and economic activity. Total fees collected are tracked and reported. Controlled via `--transaction-fee` parameter (0.0-1.0 range representing 0-100% fee rate).
- **Savings System:** Persons can save a configurable percentage of their money each simulation step. Saved money is moved from available cash to a separate savings account, affecting spending capacity while enabling wealth accumulation studies. Configurable via `--savings-rate` parameter (0.0-1.0 range representing 0-100% savings rate). Savings statistics (total, average, median, min, max) are tracked and reported in results.
- **Tax System:** Configurable income tax on trade proceeds with optional redistribution. The system collects taxes from sellers' proceeds after transaction fees and can redistribute collected taxes equally among all persons at the end of each step. This simulates government taxation and wealth redistribution policies, allowing study of their effects on wealth inequality and economic activity. Controlled via `--tax-rate` parameter (0.0-1.0 range representing 0-100% tax rate) and `--enable-tax-redistribution` flag. Tax statistics (total collected, total redistributed) are tracked and reported in results.
- **Loan System:** Persons can borrow and lend money with interest and repayment schedules. When enabled, the system tracks loans between persons, processes scheduled repayments each step, and provides statistics on loan activity. Loans have configurable interest rates and repayment periods. Enable via `--enable-loans` flag or configuration file, with parameters `--loan-interest-rate` (0.0-1.0, default: 0.01 or 1% per step), `--loan-repayment-period` (in steps, default: 20), and `--min-money-to-lend` (minimum threshold for lending, default: 50.0). Loan statistics (total issued, repaid, active) are included in simulation results. Note: With loans enabled, persons can accumulate debt (negative money balances) when unable to make payments.
- **Investment System (Infrastructure):** Foundation for investment-based capital allocation allowing persons to invest money with expectations of future returns. The system includes complete data structures (`Investment` struct with investor, target, principal, return rate, duration), configuration parameters (`enable_investments`, `investment_return_rate`, `investment_duration`, `investment_probability`, `min_money_to_invest`), and investment portfolio tracking per person. Investment types include education investments (funding another person's skill learning) and production investments (enhancing production capacity). Returns are calculated as principal plus profit based on return rate and duration (e.g., 100 invested at 2% per step for 20 steps returns 120 total). Statistics tracking infrastructure (`InvestmentStats`) captures total investments created, completed, active count, total invested amount, total returns paid, and average ROI percentage. The investment creation and execution logic in the simulation engine is ready for future implementation. This enables research on capital allocation, risk-return trade-offs, and economic growth through investment.
- **Credit Rating System:** FICO-like credit scoring (300-850 scale) that evaluates person creditworthiness based on financial behavior. When enabled alongside loans, each person's credit score dynamically updates based on: payment history (35% weight - successful vs. missed payments), debt level (30% weight - debt-to-money ratio), credit history length (15% weight - duration with loans), new credit activity (10% weight - recent loans), and credit mix (10% weight - variety of credit types). Credit scores directly affect loan interest rates: Excellent (800+) gets 50% discount, Very Good (740-799) gets 30% discount, Good (670-739) pays base rate, Fair (580-669) pays 50% premium, Poor (300-579) pays 150% premium. This simulates realistic lending risk assessment and creates differentiated credit markets. Enable via configuration file with `enable_credit_rating: true` (requires `enable_loans: true`). Credit score statistics (averages, distribution by rating category, payment history) are tracked and included in simulation results. Perfect for studying credit access inequality, lending discrimination, and the impact of credit history on economic mobility.
- **Contract System:** Long-term agreements for stable trading relationships. When enabled, persons can form contracts that lock in prices for multiple simulation steps, providing price stability and predictable income/expenses for both parties. Contracts have configurable duration bounds and offer a price discount to incentivize formation. Enable via `--enable-contracts` flag or configuration file, with parameters `max_contract_duration`, `min_contract_duration`, and `contract_price_discount` (default: 5% discount). Contract statistics (total created, completed, active, average duration, total value) are tracked and included in simulation results. Ideal for studying long-term economic relationships, price stability mechanisms, and the effects of contractual obligations on market dynamics.
- **Black Market:** Parallel informal market with different pricing rules. When enabled, a configurable percentage of trades are routed to an alternative market that operates with different prices (typically cheaper), simulating informal economy dynamics. Configured via `enable_black_market`, `black_market_price_multiplier` (e.g., 0.8 for 20% discount), and `black_market_participation_rate` (e.g., 0.2 for 20% of trades). Black market statistics (trades, volume, percentages) are tracked separately and included in simulation results.
- **Behavioral Strategies:** Persons are assigned different behavioral strategies that affect their spending decisions, creating heterogeneous agent behavior. Four strategy types are supported:
  - **Conservative** (0.7x spending multiplier): Risk-averse agents who prefer saving and only spend when they have ample reserves. Willing to spend up to 70% of their money on needed skills.
  - **Balanced** (1.0x spending multiplier): Standard agents with normal spending behavior. This is the default strategy.
  - **Aggressive** (1.3x spending multiplier): Risk-taking agents who prioritize acquiring skills and are willing to spend beyond their immediate means. Can afford skills up to 130% of their current money.
  - **Frugal** (0.5x spending multiplier): Extremely cautious agents who minimize spending and maximize savings. Only willing to spend up to 50% of their money.
  Strategies are distributed equally across the population using round-robin assignment, ensuring balanced representation. The strategy system enables studying how different agent behaviors affect market dynamics, wealth distribution, and economic activity.
- **Adaptive Strategies:** Persons can dynamically adapt their behavioral strategies based on performance through reinforcement learning. When enabled, agents track their wealth growth and adjust their spending behavior accordingly:
  - **Positive growth:** Agents become more aggressive (higher spending multiplier), reinforcing successful behavior
  - **Negative growth:** Agents become more conservative (lower spending multiplier), adapting to poor outcomes
  - **Exploration:** Random adjustments (ε-greedy approach) enable discovering new strategies
  - **Bounded adaptation:** Adjustment factor stays within 0.5-2.0x to prevent extreme behavior
  
  The system tracks success metrics (wealth growth rate, trade counts) and uses a simple learning rule: agents that are doing well increase their risk-taking, while struggling agents become more cautious. This creates emergent behavior patterns and evolutionary dynamics where successful strategies spread through learning rather than fixed assignment. Enable via configuration file with `enable_adaptive_strategies: true`, then configure `adaptation_rate` (0.0-1.0, default: 0.1 or 10% adaptation rate) and `exploration_rate` (0.0-1.0, default: 0.05 or 5% exploration). Perfect for studying agent learning, strategy evolution, market adaptation, and how behavioral flexibility affects economic outcomes.
- **Per-Skill Price Controls:** Skill-specific price floors and ceilings for regulatory intervention studies. While global `min_skill_price` and `max_skill_price` apply to all skills, per-skill price limits enable targeted regulations. Configure via YAML/TOML files using `per_skill_price_limits` (e.g., `"Programming": [25.0, 100.0]` sets min 25 and max 100 for the Programming skill). Per-skill limits override global limits when set, allowing mixed regulatory regimes where some skills are regulated and others follow free-market dynamics. Use `null` for no limit on a side (e.g., `[null, 75.0]` sets only a maximum). This enables studying:
  - Skill-specific minimum wages (professional licensing requirements)
  - Price caps on essential services
  - Mixed regulatory approaches and their effects on market equilibrium
  - Comparative analysis between regulated and unregulated skills
  Statistics on price enforcement and limit violations are tracked per skill. Configuration is via YAML/TOML only (not CLI due to complexity). Perfect for regulatory economics research, studying unintended consequences of price controls, and analyzing optimal intervention design.
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
- **Interactive Mode (REPL):** Step-by-step simulation control through an interactive command-line interface. Enable with `--interactive` flag to enter a read-eval-print loop (REPL) where you can execute individual simulation steps, inspect current state, and save checkpoints on demand. Available commands:
  - `step` - Execute one simulation step
  - `run N` - Execute N simulation steps
  - `status` - Show current step, progress, active persons, and scenario
  - `stats` - Display comprehensive statistics snapshot (money, reputation, savings, trade volume, etc.)
  - `save <path>` - Save current state to checkpoint file
  - `help` - Show all available commands
  - `exit`/`quit` - Exit interactive mode
  
  Features include command history (navigate with arrow keys), graceful handling of Ctrl+C and Ctrl+D, color-coded output, and real-time performance metrics. Perfect for debugging, exploring simulation behavior, teaching, demonstrations, and iterative testing of parameter changes. Example: `./simulation-framework --interactive -s 100 -p 10`
- **Colored Terminal Output:** Enhanced terminal output with color-coded statistics and messages for improved readability. Automatically detects terminal capabilities and can be disabled with `--no-color` flag.
- **Wealth Inequality Analysis:** Comprehensive wealth distribution analysis including:
  - **Gini Coefficient:** Measures overall inequality (0 = perfect equality, 1 = perfect inequality)
  - **Wealth Concentration Ratios:** Intuitive metrics showing what percentage of total wealth is held by:
    - Top 10% wealthiest persons (indicates upper class concentration)
    - Top 1% wealthiest persons (indicates extreme wealth concentration)
    - Bottom 50% of persons (indicates poverty/lower class share)
  - **ASCII Histogram Visualization:** Real-time terminal visualization of wealth distribution across 10 percentile buckets with color-coded bars (green for lower percentiles, yellow for middle, red for upper). Can be disabled with `--no-histogram` flag.
  - These metrics provide actionable insights into wealth distribution patterns and are included in all output formats (JSON, CSV, terminal summary).
- **Market Concentration Analysis:** Calculates the Herfindahl-Hirschman Index (HHI) to measure wealth concentration among participants. HHI values indicate market structure: < 1,500 (competitive), 1,500-2,500 (moderate concentration), > 2,500 (high concentration/oligopoly).
- **Per-Skill Market Power Analysis:** Comprehensive market concentration metrics for each individual skill, enabling detection of monopolies and oligopolies. For each skill, calculates:
  - **Herfindahl-Hirschman Index (HHI)**: Market concentration on a 0-10,000 scale
  - **CR4 (Concentration Ratio 4)**: Market share of top 4 sellers (0.0-1.0 range, where 0.8+ indicates oligopoly)
  - **CR8 (Concentration Ratio 8)**: Market share of top 8 sellers (0.0-1.0 range)
  - **Market Structure Classification**: Automatic categorization as Competitive (HHI < 1,500), Moderate Concentration (HHI 1,500-2,500), or High Concentration/Oligopoly (HHI > 2,500)
  - **Number of Active Sellers**: Counts unique providers per skill
  - **Total Trading Volume**: Aggregate value traded for the skill
  
  Results are included in JSON output under `skill_market_concentration`, sorted by HHI (most concentrated first) for easy identification of monopolistic skills. This feature enables research on market power, price-setting behavior, barriers to entry, and the effectiveness of competition policies. Perfect for studying how different skills evolve from competitive to oligopolistic markets and identifying skills that may require regulatory intervention.
- **Per-Skill Trade Analytics:** Detailed trade statistics for each skill type, tracking trade count, total volume, and average price per skill. Enables identification of the most traded and valuable skills in the market. Results are sorted by total trading volume and included in JSON output for easy analysis.
- **Business Cycle Detection:** Automatic identification of economic expansion and contraction phases from trade volume patterns. Uses a simple peak/trough detection algorithm with moving average smoothing to identify cyclical behavior in the simulated economy. For each detected cycle, tracks phase type (expansion or contraction), start/end steps, duration, average volume, and peak/trough volumes. Calculates aggregate statistics including total cycles detected, average cycle duration, average expansion duration, and average contraction duration. Requires at least 10 simulation steps for meaningful detection. Results are included in JSON output under `business_cycle_statistics` when cycles are detected. Perfect for studying endogenous economic cycles, boom-bust patterns, market volatility, and the natural oscillations that emerge from agent interactions without external shocks. Enables research on cycle amplitudes, periodicity, and asymmetry between expansions and contractions.
- **Per-Step Wealth Distribution Statistics:** Time-series tracking of wealth distribution metrics at each simulation step. Captures comprehensive statistics including average/median/std dev, min/max money, Gini coefficient, Herfindahl index, and wealth concentration ratios (top 10%, top 1%, bottom 50%). Enables analysis of how economic inequality evolves over time. Available in both JSON output (`wealth_stats_history` array) and CSV format (`{prefix}_wealth_stats_history.csv`). Perfect for studying inequality dynamics, comparing policy scenarios, and generating time-series plots for research.
- **Trading Partner Statistics:** Comprehensive analysis of trading relationships and network structure. For each person, tracks unique trading partners, buyer/seller trade counts, and top partners by trade frequency and value. Network-level metrics include average unique partners per person, network density (0.0-1.0 indicating connectivity), and identification of the most active trading pairs. This feature enables social network analysis without complex graph structures, helping understand market dynamics, trading patterns, and relationship formation. All statistics are automatically calculated and included in JSON output for further analysis.
- **Network Centrality Analysis:** Advanced network analysis identifying key market participants and their roles in the trading network. Calculates four centrality metrics for each person: **Degree Centrality** (number of trading partners, normalized 0.0-1.0 indicating connectivity), **Betweenness Centrality** (how often a person lies on shortest paths between others, identifying brokers and bridges with values 0.0-1.0), **Eigenvector Centrality** (influence based on connections to other well-connected traders, normalized 0.0-1.0), and **PageRank** (importance based on weighted connections, normalized 0.0-1.0). Network-level metrics include number of connected components (separate trading groups), average centrality scores, and network density. The analysis identifies top 5 traders in each category: most connected (degree), best brokers (betweenness), most influential (eigenvector), and highest importance (PageRank). Automatically calculated from trading relationships and included in JSON output under `centrality_analysis`. Ideal for identifying market hubs, understanding power dynamics, detecting isolated trading communities, and analyzing the structure of economic networks. Uses the petgraph library for efficient graph algorithms.
- **Failed Trade Attempt Tracking:** Monitors and reports trade attempts that fail due to insufficient funds, providing insight into unmet demand and market accessibility. Tracks total failed attempts, failure rate (percentage of total attempts), and patterns over time. High failure rates indicate economic stress where persons want to buy skills but cannot afford them, revealing inefficiencies in wealth distribution and market accessibility. Statistics include total failures, failure rate, average/min/max failures per step. Failure rates are color-coded in terminal output (green < 10%, yellow < 30%, red ≥ 30%) to quickly identify market health issues. Available in JSON output and CSV exports for detailed analysis of economic accessibility over time.
- **Monte Carlo Simulations:** Run multiple parallel simulations with different random seeds to achieve statistical significance. Automatically aggregates results across runs with mean, standard deviation, min, max, and median statistics for key metrics (average money, Gini coefficient, trade volume, reputation). Ideal for research, parameter sensitivity analysis, and understanding simulation variability.
- **Parameter Sweep Analysis:** Automated sensitivity analysis through systematic parameter sweeps (grid search). Test a parameter across a range of values with multiple runs per value to understand how parameter choices affect simulation outcomes. Supports sweeping initial_money, base_price, savings_rate, and transaction_fee. Results include aggregated statistics and identification of optimal parameter values for different objectives. Perfect for research, parameter tuning, and understanding system robustness.
- **Scenario Comparison:** Compare multiple simulation scenarios side-by-side to analyze the effects of different economic policies. Run A/B testing on pricing mechanisms (Original, DynamicPricing, AdaptivePricing) with multiple runs per scenario for statistical robustness. Automatically determines winners based on different criteria: highest average wealth, lowest inequality, highest trade volume, and highest reputation. Results are saved in JSON format with detailed statistics and winner analysis. Ideal for policy evaluation, economic research, and understanding the impact of different market mechanisms on outcomes.
- **Checkpoint System:** Save and resume simulation state at any point. Automatically save checkpoints at regular intervals during long simulations. Resume from saved checkpoints to continue interrupted simulations without starting from scratch. Useful for multi-hour simulations, distributed computing, incremental analysis, and crash recovery. Checkpoints are stored in JSON format with complete simulation state including entities, market data, loans, and statistics.
- **Replay and Debugging System:** Comprehensive debugging capabilities for bug reproduction and deterministic testing. Combines checkpoints, streaming output, and detailed logging to enable exact reproduction of simulation behavior. Fixed seeds ensure identical results across runs, enabling regression testing and bug investigation. Load checkpoints to inspect exact simulation state at any point, use streaming output to identify problematic steps, and leverage trace-level logging to understand decision-making. The system provides action log infrastructure for detailed event tracking when needed. Perfect for troubleshooting, validating changes, and understanding complex simulation dynamics without specialized replay tools.
- **Education System:** Persons can learn new skills over time by investing money in education. Each simulation step, persons have a configurable probability of attempting to learn a skill they don't currently possess. The cost to learn a skill is based on the current market price multiplied by a learning cost multiplier (default: 3x). This simulates human capital formation and skill development, allowing persons to become more versatile and participate in multiple markets. Education statistics (total skills learned, average per person, total spending) are tracked and reported. Enable via `--enable-education` flag or configuration file with parameters `learning_cost_multiplier` and `learning_probability`. Learned skills allow persons to provide those services in the market, increasing their earning potential.
- **Mentorship System:** Experienced persons can mentor others, reducing learning costs and accelerating skill acquisition. When enabled alongside education, persons with high-quality skills (quality >= 3.5 by default) can mentor others learning that skill. Mentored learners pay a reduced cost (default: 50% of normal learning cost), simulating the efficiency gain from having an experienced teacher. Mentors receive reputation bonuses (+0.05 by default) for successful mentoring, incentivizing knowledge transfer. The system tracks comprehensive mentorship statistics including total mentorships formed, successful mentored learnings, total cost savings, and counts of unique mentors and mentees. Enable via `--enable-mentorship` flag (requires `--enable-education` and works best with `--enable-quality`) with configurable parameters: `--mentorship-cost-reduction` (0.0-1.0, default: 0.5), `--min-mentor-quality` (0.0-5.0, default: 3.5), and `--mentor-reputation-bonus` (default: 0.05). Mentorship statistics are automatically included in JSON output. Perfect for studying knowledge transfer, educational efficiency, and the value of experience in human capital development.
- **Certification System:** Professional credentialing system that validates skill quality and increases market trust. When enabled, persons can invest money to get their skills certified by a central authority. Certifications have levels (1-5) based on skill quality (if quality system is enabled) or randomly assigned, with higher levels commanding greater price premiums (+5% per level, so level 5 = +25% price). Certification cost is calculated as: `skill_base_price × certification_cost_multiplier × certification_level` (default multiplier: 2.0). Certifications expire after a configurable duration (default: 200 steps) and must be renewed to maintain the price premium, simulating real-world credential renewal requirements. Each simulation step, persons have a configurable probability (default: 5%) of attempting certification if they can afford it and their skill isn't already certified. The system tracks comprehensive statistics including total certifications issued, expired certifications, active certifications, and total money spent on certification. Enable via `--enable-certification` flag or configuration file with configurable parameters: `--certification-cost-multiplier` (0.1-10.0, default: 2.0), `--certification-duration` (in steps, default: 200, set to 0 for non-expiring certifications), and `--certification-probability` (0.0-1.0, default: 0.05 or 5% chance per step). Certification statistics are automatically included in JSON output. Perfect for studying professional licensing, quality signaling, credential markets, and the economic impact of standardization and certification programs. Works synergistically with quality and reputation systems to create multi-dimensional skill value assessment.
- **Environmental Resource Tracking:** Track resource consumption and sustainability metrics throughout the simulation. When enabled, each transaction consumes environmental resources (Energy, Water, Materials, Land) proportional to its value. The system tracks total consumption by resource type, remaining reserves, and calculates sustainability scores (1.0 = sustainable, 0.0 = depleted, <0 = overconsumed). Environmental statistics include per-resource and overall sustainability scores, remaining reserves, and a boolean sustainability flag. This enables modeling ecological economics, studying the environmental impact of different trading behaviors, and analyzing resource depletion patterns. Configure via `enable_environment: true` in configuration file, with parameters `resource_cost_per_transaction` (default: 1.0, resource units consumed per dollar traded) and optional `custom_resource_reserves` (custom starting reserves per resource type). Default reserves: Energy 100,000, Water 100,000, Materials 100,000, Land 10,000 units. Environmental data is included in JSON output under `environment_statistics` with detailed per-resource breakdowns. Ideal for sustainability research, environmental policy analysis, and studying the trade-offs between economic growth and resource conservation.
- **Production System:** Persons can combine two skills they possess to produce new, more valuable skills through predefined recipes. When enabled, persons have a configurable probability of attempting production each step. If they have the required input skills and can afford the production cost (based on input skill prices and a recipe cost multiplier), a new skill is learned and added to the market. The system includes 14 predefined recipes such as: Programming + DataAnalysis → MachineLearning, Marketing + GraphicDesign → DigitalMarketing, and Engineering + Programming → SoftwareEngineering. This simulates supply chains, skill composition, and economic specialization, enabling study of how advanced skills emerge from basic building blocks. Enable via `--enable-production` flag or configuration file with parameter `production_probability` (default: 0.05 or 5% chance per step). Produced skills are priced higher than their inputs (reflecting value added) and are automatically added to the market for trading. Works well in combination with the education system to create dynamic skill ecosystems where basic skills are learned and combined to create specialized advanced skills.
- **Crisis Events:** Random economic shocks that test the resilience of the simulated economy. When enabled, crisis events can occur randomly during the simulation, creating unexpected challenges such as:
  - **Market crashes:** Sudden price drops across all skills (20-40%)
  - **Demand shocks:** Reduced overall consumption (30-50%)
  - **Supply shocks:** Reduced availability (20-40%)
  - **Currency devaluations:** Wealth destruction (10-30%)
  - **Technology shocks:** Technological disruption making certain skills obsolete with massive value loss (50-80%), simulating automation and paradigm shifts
  
  Each crisis type has distinct effects on the economy with configurable severity levels. Enable via `--enable-crisis-events` flag with parameters `--crisis-probability` (frequency, default: 2% per step) and `--crisis-severity` (impact level 0.0-1.0, default: 0.5). The crisis scenario preset (`--preset crisis_scenario`) demonstrates this feature with higher crisis probability (5%) and severity (0.7) to create a challenging economic environment. Ideal for studying economic resilience, shock recovery, technological disruption, and the effectiveness of stabilization mechanisms like price floors and redistribution policies.
- **Group/Organization System:** Persons can be assigned to groups or organizations for collective behavior analysis. When enabled via `--num-groups` parameter (or configuration file), persons are distributed across groups using round-robin assignment at simulation start. Each group tracks member count, average/total money, and average reputation. Overall statistics include total groups, average/min/max group size, and per-group breakdowns. Groups remain static during simulation but enable studying economic dynamics at the collective level, such as wealth distribution between organizations, group-based inequality, and comparative performance. Statistics are included in JSON output under `group_statistics`. Useful for analyzing team dynamics, organizational economics, and group-level wealth accumulation patterns. Valid range: 1 to number of persons.
- **Community Resource Pools:** Groups can maintain shared resource pools for collective support and mutual aid. When enabled with `--enable-resource-pools` (requires `--num-groups`), each group maintains a pool where members contribute a configurable percentage of their money each step (`--pool-contribution-rate`, default: 2%). Members with money below a threshold (`--pool-withdrawal-threshold`, default: 30.0) receive equal distributions from their group's pool, simulating needs-based mutual aid. The system tracks pool balance, total contributions, and total withdrawals for each group and across all groups. This enables studying cooperative economics, solidarity economies, mutual aid societies, and alternative approaches to social insurance. Perfect for modeling community-based resource sharing, informal savings groups (ROSCAs), and collective security mechanisms. Pool statistics are included in JSON output under `group_statistics` with per-group and aggregate metrics. Configure via configuration file with `enable_resource_pools: true`, `pool_contribution_rate` (0.0-0.5), and `pool_withdrawal_threshold` (0.0-1000.0). Example: `./simulation-framework --num-groups 3 --enable-resource-pools --pool-contribution-rate 0.05 --pool-withdrawal-threshold 50.0`
- **Streaming Output (JSONL):** Real-time streaming of step-by-step simulation data to a JSON Lines (JSONL) file. Each simulation step appends one JSON object containing key metrics (trades, volume, money statistics, Gini coefficient, reputation) to the output file. Enables real-time monitoring of long-running simulations, reduces memory footprint by not storing all step data in memory, and allows progressive analysis. Each line is a complete JSON object that can be parsed independently, making it ideal for streaming analysis tools and real-time dashboards.
- **Trading Network Export:** Export the trading network graph for visualization and analysis. The simulation automatically exports trading relationships as a network graph in both JSON and CSV formats. JSON output is compatible with vis.js, D3.js, NetworkX (Python), Gephi, and Cytoscape for creating interactive network visualizations. CSV export provides separate node and edge files for import into spreadsheet tools, network analysis packages (igraph, NetworkX), or graph databases. Network nodes include person attributes (money, reputation, trade count, unique partners) and edges capture relationship strength (number of trades, total value exchanged). Exported automatically when using `--csv-output` flag, or programmatically via `save_trading_network_json()` and `save_trading_network_csv()` methods. Ideal for social network analysis, identifying trading hubs, visualizing market structure, and studying economic relationships without complex graph libraries.
- **JSON Output:** Outputs detailed simulation results, including final wealth distribution, reputation statistics, skill valuations, and skill price history over time (suitable for graphing), to a JSON file.
- **Compressed Output:** Optional gzip compression for JSON output files, reducing file sizes by 10-20x while maintaining full data fidelity. Ideal for large-scale simulations and batch processing.
- **CSV Export:** Export simulation results to multiple CSV files for easy analysis in Excel, pandas, R, or other data analysis tools. Includes summary statistics, per-person distributions, skill prices, time-series price history, and trading network data (nodes and edges).
- **SQLite Database Export:** Export simulation results to a SQLite database for long-term storage, querying, and analysis. Creates tables for money distribution, reputation distribution, skill prices, and summary statistics. Ideal for integration with data analysis tools, business intelligence platforms, and automated reporting systems.
- **Performance:** Leverages Rust and Rayon for potential parallelism in parts of the simulation (though current critical paths like trading are largely sequential for N=100).
- **Memory Pooling:** Optional object pooling infrastructure (`VecPool<T>`) for reusing vector allocations to reduce allocation overhead. The pool maintains cleared vectors that can be reused, minimizing pressure on the system allocator and reducing memory fragmentation. Particularly beneficial in simulation loops with frequent allocations and deallocations. The pool is designed as a general-purpose utility available for integration throughout the codebase.
- **Plugin System:** Extensible trait-based plugin architecture for custom simulation extensions without modifying core code. Plugins can hook into simulation lifecycle events (start, step start/end, completion) to:
  - Collect custom metrics and statistics
  - Monitor and log simulation behavior
  - Modify simulation results with additional data
  - Implement custom algorithms and analysis
  
  The plugin system uses a feature-flags approach for compile-time plugin selection, ensuring zero runtime overhead for unused plugins. Plugins are type-safe, thread-safe (`Send + Sync`), and support downcasting for accessing plugin-specific methods. Perfect for research extensions, custom metrics collection, and experimental features without forking the codebase. See the `plugin` module documentation for implementation details and the `Plugin` trait for available lifecycle hooks.
- **Event System:** Comprehensive event tracking system for detailed simulation analysis and debugging. The event system captures all key simulation events in real-time:
  - **TradeExecuted**: Records every successful trade with buyer, seller, skill, and price
  - **PriceUpdated**: Tracks all price changes in the market with before/after values
  - **ReputationChanged**: Monitors reputation adjustments for all persons
  - **StepCompleted**: Marks the end of each simulation step with trade count and volume
  
  Enable via `--enable-events` flag or `enable_events: true` in configuration file. When enabled, all events are collected by the EventBus and included in the JSON output under the `events` array. Each event is timestamped with its simulation step and contains detailed contextual information. Zero performance overhead when disabled (events are not collected). Perfect for timeline analysis, debugging complex behaviors, understanding market dynamics, and conducting detailed research into economic phenomena. Events can be filtered, analyzed, and visualized using the JSON output.
  
  Example event:
  ```json
  {
    "step": 42,
    "event_type": {
      "type": "TradeExecuted",
      "buyer_id": 5,
      "seller_id": 12,
      "skill_id": "Programming",
      "price": 25.50
    }
  }
  ```

- **Voting System (Political Simulation):** Democratic governance and collective decision-making system for studying how different voting mechanisms affect economic outcomes. When enabled, persons can create and vote on proposals that affect simulation parameters. 
  
  The system supports **three voting methods**:
  - **SimpleMajority**: One person, one vote (pure democracy) - equal voting power for all participants
  - **WeightedByWealth**: Voting power proportional to wealth (plutocracy) - wealthier persons have more influence
  - **QuadraticVoting**: Square root of wealth (balanced influence) - reduces wealth's impact while still considering it
  
  **Proposal types** include:
  - **TaxRateChange**: Adjust the tax rate through democratic vote
  - **BasePriceChange**: Change base skill prices collectively  
  - **TransactionFeeChange**: Modify marketplace transaction fees
  - **Generic**: Custom proposals for governance experiments
  
  Features include automatic proposal expiration, duplicate vote prevention, and comprehensive voting statistics (participation rates, pass/fail ratios, voter turnout). 
  
  **Configuration**: Enable via configuration file with `enable_voting: true`, then configure:
  - `voting_method`: SimpleMajority/WeightedByWealth/QuadraticVoting
  - `proposal_duration`: Voting period in steps (default: 20)
  - `proposal_probability`: Chance of random proposal per step (default: 0.05)
  - `voting_participation_rate`: Voter turnout probability (default: 0.3)
  
  Voting results and statistics are included in JSON output. Ideal for studying democratic governance, policy formation, wealth-based political influence, and the interaction between economic and political systems. Perfect for research on voting theory, collective choice, and institutional economics.

## Getting Started

### Prerequisites

- Rust Toolchain (see [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) for installation instructions)

### Quick Start

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd community-simulation
    ```
    (Replace `<repository-url>` with the actual repository URL)

2.  Build the project in release mode for optimal performance:
    ```bash
    cargo build --release
    ```

3.  Run a basic simulation:
    ```bash
    ./target/release/simulation-framework -o results.json
    ```

### Running the Simulation

After building, the executable will be located at `target/release/simulation-framework`.

**Basic Execution (using default parameters):**

```bash
./target/release/simulation-framework -o results.json
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
*   `--sqlite-output <PATH>`:
    *   Path to SQLite database file for exporting simulation results.
    *   When enabled, the simulation exports final results to a SQLite database with the following tables:
        *   `summary_statistics` - Overall simulation statistics (steps, duration, money stats, Gini coefficient, trade volumes)
        *   `money_distribution` - Final money distribution per person
        *   `reputation_distribution` - Final reputation distribution per person
        *   `skill_prices` - Final skill prices for all skills
    *   **Use cases:**
        *   Long-term storage and historical analysis of simulation results
        *   Integration with business intelligence tools and reporting systems
        *   SQL-based querying and analysis of simulation data
        *   Combining results from multiple simulation runs for comparative analysis
    *   Example: `--sqlite-output results.db`
    *   The database file is created or overwritten if it exists
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
*   `--price-elasticity <FACTOR>`:
    *   Price elasticity factor controlling sensitivity to supply/demand imbalances (0.0-1.0, default: 0.1). This determines how dramatically prices change when supply doesn't match demand. Higher values create more volatile markets with rapid price swings, while lower values create more price stability but slower market adjustment.
    *   **Interpretation:**
        *   `0.05` - Very inelastic, stable prices (similar to utilities, healthcare, essentials)
        *   `0.1` - Moderate elasticity (default, balanced markets)
        *   `0.2` - High elasticity, volatile prices (similar to fashion, tech products, luxury goods)
    *   **Use cases:**
        *   Model different types of markets with varying price responsiveness
        *   Study how elasticity affects market convergence and stability
        *   Compare outcomes between rigid (regulated) and flexible (free) markets
        *   Test economic resilience to supply/demand shocks
    *   Example: `--price-elasticity 0.15` for moderately responsive markets
*   `--volatility <PERCENTAGE>`:
    *   Volatility percentage for random price fluctuations (0.0-0.5, default: 0.02). Adds random noise to prices each simulation step to model unpredictable market forces, news events, sentiment changes, and other real-world uncertainties. The value represents the range of random variation as a percentage of the current price.
    *   **Interpretation:**
        *   `0.0` - No volatility, completely deterministic price evolution
        *   `0.02` - Low volatility (default, stable markets like bonds, blue-chip stocks)
        *   `0.05` - Moderate volatility (typical commodities, mid-cap stocks)
        *   `0.1` - High volatility (cryptocurrency, speculative assets, emerging markets)
    *   **Use cases:**
        *   Simulate different levels of market uncertainty and risk
        *   Study how volatility affects wealth distribution and economic stability
        *   Model crisis scenarios with high market chaos
        *   Test the robustness of economic policies under uncertain conditions
    *   Example: `--volatility 0.08` for a highly volatile market environment
*   `--demand-strategy <STRATEGY>`:
    *   Specifies the demand generation strategy. This determines how many skills each person needs per simulation step.
    *   Available strategies:
        *   `Uniform` (default) - Random uniform distribution: each person needs 2-5 skills with equal probability. This is the baseline strategy that maintains current simulation behavior, creating a balanced market.
        *   `Concentrated` - Pareto-like distribution: 70% of persons have low demand (2-3 skills), while 30% have high demand (4-5 skills). This simulates markets with unequal consumption patterns, useful for studying demand inequality alongside wealth inequality.
        *   `Cyclical` - Time-varying cyclical demand: demand oscillates between 2 and 5 skills over a 100-step cycle, with phase offsets per person for variety. This simulates business cycles and creates periodic expansion/contraction phases in market activity.
    *   **Use cases:**
        *   Study how demand patterns affect market dynamics and wealth distribution
        *   Test market resilience to concentrated vs. distributed demand
        *   Simulate economic cycles and business cycle dynamics
        *   Analyze the interaction between demand patterns and pricing mechanisms
    *   Example: `--demand-strategy Concentrated`
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
*   `--no-histogram`:
    *   Disable ASCII histogram visualization of wealth distribution in terminal output. By default, the simulation displays a color-coded histogram showing how wealth is distributed across 10 percentile buckets (deciles). The histogram uses green bars for lower percentiles (0-30%), yellow for middle (30-70%), and red for upper (70-100%), making it easy to spot inequality patterns at a glance. Use this flag when you want a more compact terminal output or when redirecting output to files.
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
./target/release/simulation-framework --list-presets

# Use a preset for quick testing
./target/release/simulation-framework --preset quick_test -o quick_results.json

# Use a preset and override some parameters
./target/release/simulation-framework --preset crisis_scenario --steps 2000 --seed 999 -o crisis_results.json
```

**Example with Custom Parameters:**

```bash
./target/release/simulation-framework --steps 1000 --persons 50 --initial-money 200 --base-price 15 --output custom_results.json --seed 123
```
This runs the simulation for 1000 steps with 50 persons, each starting with 200 money, skills having a base price of 15, and saves results to `custom_results.json` using RNG seed 123.

**Example with Seasonal Effects:**

```bash
./target/release/simulation-framework --steps 500 --persons 100 --seasonal-amplitude 0.3 --seasonal-period 50 --output seasonal_results.json
```
This runs the simulation with seasonal demand fluctuations. The `--seasonal-amplitude 0.3` parameter creates ±30% variation in demand, and `--seasonal-period 50` means the seasonal cycle repeats every 50 steps. Different skills will have their peak demand at different times due to phase offsets, creating realistic market dynamics.

**Example with Price Floor:**

```bash
# Crisis scenario with price floor to prevent market collapse
./target/release/simulation-framework --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 3 --scenario DynamicPricing --output price_floor_results.json

# Compare with and without price floor
./target/release/simulation-framework --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 1 --scenario DynamicPricing --output no_floor.json
./target/release/simulation-framework --steps 500 --persons 100 --initial-money 50 --base-price 15 --min-skill-price 5 --scenario DynamicPricing --output with_floor.json
```

The price floor feature is particularly useful in crisis scenarios or with dynamic pricing that can drive prices down. By setting `--min-skill-price 3`, you ensure that no skill price falls below $3, preventing deflationary spirals and maintaining minimum market viability. This models real-world economic policies like minimum wage laws or regulatory price controls.

**Example with Demand Strategies:**

```bash
# Default uniform demand (baseline)
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Uniform --output uniform_demand.json

# Concentrated demand (inequality)
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Concentrated --output concentrated_demand.json

# Cyclical demand (business cycles)
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Cyclical --output cyclical_demand.json

# Compare demand strategies side-by-side
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Uniform --output uniform.json
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Concentrated --seed 42 --output concentrated.json
./target/release/simulation-framework --steps 500 --persons 100 --demand-strategy Cyclical --seed 42 --output cyclical.json
```

Demand strategies usage:
- **Uniform (default):** Each person randomly needs 2-5 skills with equal probability. This creates a balanced market where all demand levels are equally likely, providing a baseline for comparison.
- **Concentrated:** 70% of persons have low demand (2-3 needs), while 30% have high demand (4-5 needs). This simulates markets with unequal consumption patterns, useful for studying how demand inequality interacts with wealth inequality and affects market dynamics.
- **Cyclical:** Demand varies over time in a sine wave pattern with a 100-step period. Each person has a phase offset, creating diverse cyclical patterns. This simulates business cycles with expansion and contraction phases, testing how markets adapt to changing aggregate demand levels over time.

The demand strategy interacts with other features:
- Combine with `--scenario` to see how different pricing mechanisms respond to demand patterns
- Use with `--seasonal-amplitude` to create layered demand dynamics (cyclical baseline + seasonal fluctuations)
- Pair with wealth inequality metrics (Gini coefficient) to study demand-side vs supply-side inequality

**Example with Transaction Fees:**

```bash
./target/release/simulation-framework --steps 500 --persons 100 --transaction-fee 0.05 --output fees_results.json
```
This runs the simulation with a 5% transaction fee on all trades. The fee is deducted from the seller's proceeds (e.g., if a skill sells for $100, the buyer pays $100 but the seller receives $95, with $5 collected as fees). This simulates realistic marketplace costs and allows studying the impact of trading fees on market liquidity, wealth distribution, and economic activity. The total fees collected are reported in the JSON output.

**Example with Tax System:**

```bash
# Simulation with 10% income tax (no redistribution)
./target/release/simulation-framework --steps 500 --persons 100 --tax-rate 0.10 --output tax_results.json

# Simulation with 15% income tax and redistribution
./target/release/simulation-framework --steps 500 --persons 100 --tax-rate 0.15 --enable-tax-redistribution --output tax_redistribution_results.json

# Combined: transaction fees + taxes + redistribution
./target/release/simulation-framework --steps 500 --persons 100 \
  --transaction-fee 0.05 --tax-rate 0.20 --enable-tax-redistribution \
  --output combined_policy.json
```

Tax system usage:
- **Without redistribution:** Taxes are collected from seller proceeds and removed from the economy, simulating government spending on public goods outside the simulation. This reduces overall money supply and can affect economic activity.
- **With redistribution:** Taxes collected each step are redistributed equally to all persons at the end of the step, simulating basic income or welfare programs. This can reduce wealth inequality while maintaining total money supply.
- The total taxes collected and (if enabled) redistributed are reported in the JSON output for analysis.
- Taxes are calculated on net seller proceeds (after transaction fees): `tax = (price - transaction_fee) * tax_rate`

**Example with Education System:**

```bash
# Enable education with default parameters (3x market price, 10% learning probability per step)
./target/release/simulation-framework --steps 500 --persons 100 --enable-education --output education_results.json

# Custom education parameters: cheaper learning (2x price) and higher probability (50%)
./target/release/simulation-framework --steps 1000 --persons 50 \
  --enable-education --learning-cost-multiplier 2.0 --learning-probability 0.5 \
  --initial-money 500 --output education_custom.json

# Study skill acquisition in a wealthy economy
./target/release/simulation-framework --steps 1500 --persons 100 \
  --enable-education --learning-cost-multiplier 1.5 --learning-probability 0.3 \
  --initial-money 1000 --output wealthy_education.json
```

Education system usage:
- When enabled, persons can invest money to learn new skills they don't currently possess.
- Each simulation step, each person has a `learning_probability` chance (default: 10%) of attempting to learn a random skill.
- The cost to learn a skill is calculated as: `current_market_price * learning_cost_multiplier` (default multiplier: 3.0).
- Persons can only learn skills they don't already have (either as own_skill or previously learned).
- Once learned, skills become part of the person's repertoire and can be provided to others in the market, increasing earning potential.
- Education statistics are tracked and reported in the JSON output:
  - Total skills learned across all persons
  - Average number of learned skills per person
  - Maximum number of skills learned by any individual
  - Total money spent on education
- **Use cases:**
  - Simulate human capital formation and workforce development
  - Study how skill acquisition affects wealth distribution over time
  - Model economies where workers can retrain and adapt to market demands
  - Analyze the relationship between education investment and economic mobility
- Learned skills persist through the simulation and count toward a person's available skills for trading.

**Example with Certification System:**

```bash
# Enable certification with default parameters (2x cost multiplier, 200 step duration, 5% probability)
./target/release/simulation-framework --steps 500 --persons 100 --enable-certification --output cert_results.json

# Custom certification parameters: cheaper certification (1.5x price) and higher probability (20%)
./target/release/simulation-framework --steps 1000 --persons 50 \
  --enable-certification --certification-cost-multiplier 1.5 --certification-probability 0.2 \
  --certification-duration 300 --initial-money 200 --output cert_custom.json

# Combine with quality system for quality-based certification levels
./target/release/simulation-framework --steps 1500 --persons 100 \
  --enable-quality --enable-certification \
  --certification-duration 250 --certification-probability 0.15 \
  --output quality_cert.json

# Study certification economics with education and mentorship
./target/release/simulation-framework --steps 2000 --persons 100 \
  --enable-education --enable-quality --enable-mentorship --enable-certification \
  --certification-cost-multiplier 2.5 --certification-duration 0 \
  --initial-money 300 --output full_education_economy.json
```

Certification system usage:
- When enabled, persons can invest money to get their skills certified by a central authority.
- Each simulation step, each person has a `certification_probability` chance (default: 5%) of attempting to certify an uncertified skill.
- Certification cost: `skill_base_price × certification_cost_multiplier × certification_level` (default multiplier: 2.0)
- Certification levels (1-5) are based on skill quality if quality system is enabled, otherwise randomly assigned.
- Certified skills receive a price premium: +5% per level (level 1 = +5%, level 5 = +25%)
- Certifications expire after `certification_duration` steps (default: 200), requiring renewal to maintain the premium.
- Set `certification_duration` to 0 for non-expiring (permanent) certifications.
- Certification statistics are tracked and reported in JSON output:
  - Total certifications issued during simulation
  - Total certifications that expired
  - Currently active (non-expired) certifications at simulation end
  - Total money spent on obtaining certifications
- **Use cases:**
  - Model professional licensing and credentialing markets
  - Study the economic impact of quality signaling through certification
  - Analyze the cost-benefit trade-offs of professional credentials
  - Simulate certification renewal markets and credential inflation
  - Research the effects of standardization on market trust and pricing

**Example with CSV Export:**

```bash
./target/release/simulation-framework --steps 500 --persons 100 --csv-output ./output/analysis
```
This runs the simulation and creates CSV files (`analysis_summary.csv`, `analysis_money.csv`, etc.) in the `./output/` directory for easy data analysis.

**Example with SQLite Database Export:**

```bash
# Export to SQLite database
./target/release/simulation-framework --steps 500 --persons 100 --sqlite-output results.db

# Query the database using sqlite3 command-line tool
sqlite3 results.db "SELECT * FROM summary_statistics;"
sqlite3 results.db "SELECT AVG(money) FROM money_distribution;"
sqlite3 results.db "SELECT skill_id, price FROM skill_prices ORDER BY price DESC LIMIT 5;"

# Use with multiple output formats simultaneously
./target/release/simulation-framework --steps 1000 --persons 100 \
  --output results.json \
  --csv-output ./analysis \
  --sqlite-output results.db
```

The SQLite database export provides:
- **Structured storage** for long-term archival of simulation results
- **SQL querying** for flexible data analysis and reporting
- **Integration** with business intelligence tools (Tableau, Power BI, Metabase)
- **Combination** of results from multiple simulation runs for comparative analysis

**Example with Compressed Output:**

```bash
./target/release/simulation-framework --steps 1000 --persons 100 --output results.json --compress
```
This runs the simulation and saves compressed results to `results.json.gz`, achieving significant space savings (typically 10-20x smaller file size) while preserving all simulation data. The compressed file can be decompressed with `gunzip results.json.gz` or opened directly by many data analysis tools.

**Example with Monte Carlo Simulations:**

```bash
# Run 10 parallel simulations for statistical significance
./target/release/simulation-framework --monte-carlo-runs 10 -s 500 -p 100 -o mc_results.json

# With custom seed for reproducibility
./target/release/simulation-framework --monte-carlo-runs 5 -s 1000 -p 50 --seed 12345 -o mc_analysis.json

# Combine with other features (compressed output, custom parameters)
./target/release/simulation-framework --monte-carlo-runs 20 -s 500 -p 100 \
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
./target/release/simulation-framework -s 500 -p 100 \
  --compare-scenarios "Original,DynamicPricing" \
  --comparison-runs 5 \
  -o comparison_results.json

# Compare all three available scenarios with custom parameters
./target/release/simulation-framework -s 1000 -p 50 \
  --compare-scenarios "Original,DynamicPricing,AdaptivePricing" \
  --comparison-runs 10 \
  --initial-money 150 --base-price 12 \
  -o full_comparison.json

# Scenario comparison with economic features enabled
./target/release/simulation-framework -s 500 -p 100 \
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
./target/release/simulation-framework -s 500 -p 100 \
  --parameter-sweep "initial_money:50:200:7" \
  --sweep-runs 5 \
  -o sweep_initial_money.json

# Test the impact of transaction fees on market activity
./target/release/simulation-framework -s 500 -p 100 \
  --parameter-sweep "transaction_fee:0.0:0.15:6" \
  --sweep-runs 3 \
  -o sweep_transaction_fee.json

# Analyze savings rate effects on wealth distribution
./target/release/simulation-framework -s 500 -p 100 \
  --parameter-sweep "savings_rate:0.0:0.2:5" \
  --sweep-runs 4 \
  -o sweep_savings_rate.json

# Test base price sensitivity
./target/release/simulation-framework -s 500 -p 100 \
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
./target/release/simulation-framework --steps 5000 --persons 100 \
  --checkpoint-interval 500 \
  --checkpoint-file ./checkpoints/long_run.json \
  --output results.json

# If the simulation is interrupted, resume from the last checkpoint
./target/release/simulation-framework --resume \
  --checkpoint-file ./checkpoints/long_run.json \
  --output continued_results.json

# You can also manually save checkpoints at specific intervals
# Run first 1000 steps with checkpoint every 250 steps
./target/release/simulation-framework --steps 1000 --persons 50 \
  --checkpoint-interval 250 \
  --checkpoint-file ./step1.json

# Resume and run another 1000 steps
./target/release/simulation-framework --resume \
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
./target/release/simulation-framework --steps 1000 --persons 100 \
  --stream-output ./stream/simulation.jsonl \
  --output results.json

# Monitor the simulation in real-time (in another terminal)
tail -f ./stream/simulation.jsonl | jq '.step, .trades, .avg_money'

# Stream without final output (for pure streaming mode)
./target/release/simulation-framework --steps 5000 --persons 200 \
  --stream-output simulation_progress.jsonl

# Combine streaming with other features
./target/release/simulation-framework --steps 2000 --persons 150 \
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

**Debugging and Replay System:**

The simulation framework includes a comprehensive debugging and replay capability built on checkpoints and streaming output. This system enables bug reproduction, deterministic testing, and detailed analysis of simulation behavior.

**Replay System Features:**

1. **Deterministic Execution**: Using fixed seeds ensures identical results across runs
2. **State Checkpointing**: Save complete simulation state at any point for later inspection
3. **Step-by-Step Logs**: Stream output provides detailed progression data
4. **Action Logging**: The framework includes action log infrastructure for detailed event tracking

**Reproducing Bugs:**

```bash
# Step 1: Record the problematic simulation with detailed logging
RUST_LOG=debug ./target/release/simulation-framework \
  --steps 1000 --persons 100 --seed 42 \
  --stream-output debug_stream.jsonl \
  --checkpoint-interval 100 \
  --checkpoint-file debug_checkpoint.json \
  --output debug_results.json 2> debug.log

# Step 2: Replay from checkpoint to investigate specific state
./target/release/simulation-framework --resume \
  --checkpoint-file debug_checkpoint.json \
  --steps 50 \
  --output replay_results.json

# Step 3: Compare results for reproducibility
# The same seed ensures deterministic behavior
./target/release/simulation-framework \
  --steps 1000 --persons 100 --seed 42 \
  --output verify_results.json

# Use diff or jq to compare JSON outputs
jq -S '.money_statistics' debug_results.json > stats1.json
jq -S '.money_statistics' verify_results.json > stats2.json
diff stats1.json stats2.json
```

**Replay System Benefits:**

- **Bug Reproduction**: Fixed seeds and checkpoints enable exact reproduction of issues
- **Deterministic Testing**: Verify that changes don't alter simulation outcomes
- **State Inspection**: Load checkpoints to examine exact simulation state at failure points
- **Progressive Debugging**: Use streaming output to identify the step where issues occur
- **Regression Testing**: Compare output files to detect unintended behavior changes

**Advanced Debugging Workflow:**

```bash
# 1. Run with detailed logging to capture all events
RUST_LOG=trace ./target/release/simulation-framework \
  --steps 500 --persons 50 --seed 123 \
  --stream-output trace_stream.jsonl \
  --checkpoint-interval 50 \
  --output trace_results.json 2> trace.log

# 2. Analyze the stream to find problematic steps
grep "step.*25[0-9]" trace_stream.jsonl

# 3. Resume from checkpoint just before the problem
./target/release/simulation-framework --resume \
  --checkpoint-file debug_checkpoint.json \
  --steps 10 \
  --no-progress 2> detailed_debug.log

# 4. Extract specific step data for analysis
jq 'select(.step == 255)' trace_stream.jsonl
```

The replay system leverages existing infrastructure (checkpoints, streaming, logging) to provide powerful debugging capabilities without requiring specialized replay tools.

**Example with Trading Network Export:**

The simulation automatically exports trading network data when using the `--csv-output` flag, creating files for network visualization and analysis.

```bash
# Export simulation results including trading network
./target/release/simulation-framework --steps 500 --persons 100 --csv-output results
# Creates network files:
#   results_network_nodes.csv (person attributes: id, money, reputation, trade_count, unique_partners)
#   results_network_edges.csv (trading relationships: source, target, weight, total_value)
```

Network visualization examples:

```bash
# Small network for visualization
./target/release/simulation-framework --steps 200 --persons 20 --csv-output network_small -o network.json

# Large network for analysis
./target/release/simulation-framework --steps 1000 --persons 200 --csv-output network_large --compress
```

**Using the Network Data:**

The exported network can be visualized using various tools:

- **Python (NetworkX):**
  ```python
  import pandas as pd
  import networkx as nx
  import matplotlib.pyplot as plt
  
  # Load network data
  nodes = pd.read_csv('results_network_nodes.csv')
  edges = pd.read_csv('results_network_edges.csv')
  
  # Create graph
  G = nx.from_pandas_edgelist(edges, source='source', target='target', 
                                edge_attr=['weight', 'total_value'])
  
  # Add node attributes
  nx.set_node_attributes(G, nodes.set_index('id')['money'].to_dict(), 'money')
  nx.set_node_attributes(G, nodes.set_index('id')['reputation'].to_dict(), 'reputation')
  
  # Visualize
  pos = nx.spring_layout(G)
  nx.draw(G, pos, node_color=[G.nodes[n]['money'] for n in G.nodes()], 
          with_labels=True, cmap='coolwarm')
  plt.show()
  ```

- **Gephi:** Import `results_network_nodes.csv` as nodes table and `results_network_edges.csv` as edges table

- **D3.js/vis.js:** Use the JSON format (programmatic export via `result.save_trading_network_json("network.json")`)

- **R (igraph):**
  ```r
  library(igraph)
  library(readr)
  
  nodes <- read_csv("results_network_nodes.csv")
  edges <- read_csv("results_network_edges.csv")
  
  g <- graph_from_data_frame(edges, directed=FALSE, vertices=nodes)
  plot(g, vertex.size=V(g)$trade_count/5, vertex.color=V(g)$money)
  ```

The network data reveals:
- **Trading hubs:** Persons with many unique partners (high degree centrality)
- **Market structure:** Density and clustering patterns
- **Economic relationships:** Strong ties (high trade volume) vs. weak ties
- **Wealth distribution:** Correlation between network position and money

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
./target/release/simulation-framework --config my_config.yaml -o results.json
```

CLI arguments override config file values:
```bash
# Use config file but override steps and persons
./target/release/simulation-framework --config my_config.yaml --steps 2000 --persons 100 -o results.json
```

See `config.example.yaml` and `config.example.toml` in the repository for complete examples with all available options and comments.

### Logging and Trace Mode

The simulation uses structured logging to provide insights into its operation. The **Trace Mode** feature enables comprehensive debug logging for detailed problem diagnosis.

**Basic Logging via Environment Variable:**
```bash
# Default (info level) - High-level progress
./target/release/simulation-framework -s 100 -p 10 -o results.json

# Debug level - Detailed execution information
RUST_LOG=debug ./target/release/simulation-framework -s 100 -p 10 -o results.json

# Trace level - Extremely detailed output
RUST_LOG=trace ./target/release/simulation-framework -s 100 -p 10 -o results.json
```

**Module-Specific Logging:**
```bash
# Only debug engine operations
RUST_LOG=simulation_framework::engine=debug ./target/release/simulation-framework -s 100 -p 10 -o results.json

# Only debug scenario/pricing
RUST_LOG=simulation_framework::scenario=debug ./target/release/simulation-framework -s 100 -p 10 -o results.json

# Multiple modules
RUST_LOG=simulation_framework::engine=debug,simulation_framework::scenario=trace ./target/release/simulation-framework -s 100 -p 10 -o results.json
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
RUST_LOG=debug ./target/release/simulation-framework -s 50 -p 5 -o debug.json --no-progress

# Trace all affordability decisions
RUST_LOG=trace ./target/release/simulation-framework -s 10 -p 5 -o trace.json --no-progress 2>&1 | grep "cannot afford"
```

Analyze economic behavior:
```bash
# Watch reputation changes
RUST_LOG=debug ./target/release/simulation-framework -s 100 -p 10 -o results.json 2>&1 | grep "reputation"

# Track tax redistribution
RUST_LOG=debug ./target/release/simulation-framework -s 100 -p 10 --tax-rate 0.15 --enable-tax-redistribution -o results.json 2>&1 | grep "Redistributing"
```

**Tips:**
- Use `info` (default) for normal operations
- Use `debug` when investigating simulation behavior, understanding trade dynamics, or troubleshooting
- Use `trace` for deep analysis of individual agent decisions
- Use `warn` or `error` for minimal output in production/batch scenarios
- Combine with `--no-progress` flag to disable the progress bar when using debug/trace logging
- Redirect stderr to a file for large logs: `RUST_LOG=debug ./target/release/simulation-framework ... 2> debug.log`

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
*   `trading_partner_statistics`: Comprehensive trading relationship analysis including:
    *   `per_person`: Array of per-person trading statistics, each containing:
        *   `person_id`: Unique identifier for the person
        *   `unique_partners`: Number of unique trading partners
        *   `total_trades_as_buyer`: Total trades where this person was the buyer
        *   `total_trades_as_seller`: Total trades where this person was the seller
        *   `top_partners`: Top 5 trading partners sorted by trade count, each with:
            *   `partner_id`: ID of the trading partner
            *   `trade_count`: Number of trades with this partner
            *   `total_value`: Total money exchanged with this partner
    *   `network_metrics`: Network-level connectivity metrics:
        *   `avg_unique_partners`: Average number of unique partners per person
        *   `network_density`: Ratio of actual connections to possible connections (0.0-1.0)
        *   `most_active_pair`: Tuple of (person1_id, person2_id, trade_count) for the most active trading pair
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
*   `{prefix}_wealth_stats_history.csv`: **Wealth distribution statistics over time** (if available)
*   `{prefix}_trade_volume.csv`: **Trade volume history showing trades count and money exchanged per step**
*   `{prefix}_network_nodes.csv`: **Trading network nodes** (persons with money, reputation, trade count, unique partners)
*   `{prefix}_network_edges.csv`: **Trading network edges** (trading relationships with weight and total value)

The trade volume CSV provides time-series data perfect for analyzing market activity and economic vitality trends. The wealth stats history CSV contains comprehensive inequality metrics at each step, ideal for studying how wealth distribution evolves over the course of the simulation. The network CSVs enable graph analysis and visualization of trading relationships using tools like NetworkX, igraph, Gephi, or Cytoscape.

## Development

For information about developing, testing, and contributing to this project, please see the [Development Guide](DEVELOPMENT.md).

### Quick Links for Developers

- [Building the Project](DEVELOPMENT.md#building-the-project)
- [Code Structure](DEVELOPMENT.md#code-structure)
- [Testing](DEVELOPMENT.md#testing)
- [Contributing Guidelines](DEVELOPMENT.md#contributing)

## License

This project is licensed under the terms of the MIT license. See the `LICENSE` file for details.
