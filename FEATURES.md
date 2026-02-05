# Features

This document provides comprehensive details about all features available in the Economic Simulation Framework.

For a quick overview, see the [README](README.md). For development information, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Table of Contents

- [Core Simulation Features](#core-simulation-features)
- [Market Mechanisms](#market-mechanisms)
- [Social Systems](#social-systems)
- [Economic & Financial Systems](#economic--financial-systems)
- [Insurance & Risk Management](#insurance--risk-management)
- [Technology & Innovation](#technology--innovation)
- [Advanced Market Systems](#advanced-market-systems)
- [Behavioral Systems](#behavioral-systems)
- [Analysis & Research Tools](#analysis--research-tools)
- [Output & Export Formats](#output--export-formats)
- [Configuration & Usability](#configuration--usability)
- [Development & Debugging](#development--debugging)

---

## Core Simulation Features

### Agent-Based Simulation

Simulates individual persons with money, unique skills, and randomly generated needs for other skills.

### Geographic Locations

Each person has a 2D location (x, y coordinates) in a virtual space. When enabled via `--distance-cost-factor`, trade costs increase based on the Euclidean distance between buyer and seller, simulating transportation costs and geographic barriers to trade. The distance multiplier is applied as: `final_cost = base_cost * (1 + distance * distance_cost_factor)`. For example, with a factor of 0.01 and a distance of 50 units, costs increase by 50%. Location data is included in JSON output for spatial economic analysis.

### Multiple Skills Per Person

Each person can possess and offer multiple skills in the market, creating more realistic labor dynamics with skill redundancy and competition. Configurable via `--skills-per-person` parameter (default: 1). When set to values > 1, skills are distributed across persons using a round-robin approach, allowing multiple providers per skill and more complex market interactions.

### Trading System

Persons attempt to buy needed skills from providers if they can afford them, leading to money exchange and transaction logging.

### Panic Recovery

Robust error handling with graceful degradation - if a panic occurs during simulation step execution, it is caught and logged, allowing the simulation to continue. Failed steps are tracked and reported in the results.

### Event System

Comprehensive event tracking system for detailed simulation analysis and debugging. The event system captures all key simulation events in real-time:
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

### Performance

Leverages Rust and Rayon for potential parallelism in parts of the simulation (though current critical paths like trading are largely sequential for N=100).

## Market Mechanisms

### Multiple Pricing Scenarios

Choose from different market price mechanisms to study their effects:
  - **Original** (default): Supply/demand-based pricing with random volatility - prices adjust based on the ratio of buyers to sellers
  - **DynamicPricing**: Sales-based pricing - prices increase 5% when sold, decrease 5% when not sold
  - **AdaptivePricing**: Gradual price adaptation using exponential moving average with 20% learning rate for smooth convergence
  - **AuctionPricing**: Competitive bidding mechanism where prices increase aggressively when multiple buyers compete for the same skill (simulating auction psychology), with gentler decreases when demand is low. Uses quadratic competition factor to model bidding war intensity. Ideal for studying price spikes in competitive markets and auction-like dynamics.
  - **ClimateChange**: Simulates gradual cost increases due to environmental degradation. Prices increase deterministically each step (base rate: 0.2% per step) with acceleration over time (+0.1% per 100 steps), representing the economic impact of climate change, resource scarcity, and adaptation costs. Unlike other scenarios, affects all skills equally to model systemic environmental costs. Ideal for studying long-term economic impacts of climate change and environmental policy.

### Dynamic Market

Features a market mechanism where skill prices are adjusted based on supply (fixed per provider) and demand (generated each step).

### Demand Generation Strategies

Configurable strategies for determining how many skills each person needs per step. Three strategies available:
  - **Uniform** (default): Random 2-5 needs with equal probability, creating balanced markets
  - **Concentrated**: Pareto-like distribution (70% low demand, 30% high demand), simulating consumption inequality
  - **Cyclical**: Time-varying demand in cycles, simulating business cycle dynamics with expansion and contraction phases
  
  Controlled via `--demand-strategy` parameter. Enables studying how demand patterns affect market behavior, wealth distribution, and economic resilience. Interacts with pricing mechanisms and seasonal effects to create complex market dynamics.

### Seasonal Demand Effects

Configurable seasonal fluctuations in skill demand using cyclical patterns. Different skills experience peak demand at different times, creating realistic market dynamics and economic cycles. Controlled via `--seasonal-amplitude` and `--seasonal-period` parameters.

### Transaction Fees

Configurable marketplace transaction fees that are deducted from seller proceeds on each trade. Simulates realistic trading costs (e.g., platform fees, payment processing) and allows studying the impact of fees on market liquidity, wealth distribution, and economic activity. Total fees collected are tracked and reported. Controlled via `--transaction-fee` parameter (0.0-1.0 range representing 0-100% fee rate).

### Price Volatility

Skill prices include a configurable random volatility component.

### Per-Skill Price Controls

Skill-specific price floors and ceilings for regulatory intervention studies. While global `min_skill_price` and `max_skill_price` apply to all skills, per-skill price limits enable targeted regulations. Configure via YAML/TOML files using `per_skill_price_limits` (e.g., `"Programming": [25.0, 100.0]` sets min 25 and max 100 for the Programming skill). Per-skill limits override global limits when set, allowing mixed regulatory regimes where some skills are regulated and others follow free-market dynamics. Use `null` for no limit on a side (e.g., `[null, 75.0]` sets only a maximum). This enables studying:
  - Skill-specific minimum wages (professional licensing requirements)
  - Price caps on essential services
  - Mixed regulatory approaches and their effects on market equilibrium
  - Comparative analysis between regulated and unregulated skills
  Statistics on price enforcement and limit violations are tracked per skill. Configuration is via YAML/TOML only (not CLI due to complexity). Perfect for regulatory economics research, studying unintended consequences of price controls, and analyzing optimal intervention design.

### Priority-Based Buying Decisions

Sophisticated multi-factor decision-making system for purchase prioritization. Each purchase option is scored based on four weighted factors:
  - **Urgency** (default weight: 0.5): Need urgency level (1-3 scale, randomly assigned)
  - **Affordability** (default weight: 0.3): Cost relative to available money (lower cost = higher priority)
  - **Efficiency** (default weight: 0.1): Technological progress factor (more efficient skills prioritized)
  - **Reputation** (default weight: 0.1): Seller reputation score (higher reputation = higher priority)
  
  All weights are configurable (0.0-1.0 range), allowing experimentation with different decision strategies. The system combines these factors into a single priority score for each potential purchase, then sorts options by priority (highest first). This enables realistic, heterogeneous agent behavior that considers multiple objectives simultaneously rather than simple urgency-only sorting.

### Per-Skill Market Power Analysis

Comprehensive market concentration metrics for each individual skill, enabling detection of monopolies and oligopolies. For each skill, calculates:
  - **Herfindahl-Hirschman Index (HHI)**: Market concentration on a 0-10,000 scale
  - **CR4 (Concentration Ratio 4)**: Market share of top 4 sellers (0.0-1.0 range, where 0.8+ indicates oligopoly)
  - **CR8 (Concentration Ratio 8)**: Market share of top 8 sellers (0.0-1.0 range)
  - **Market Structure Classification**: Automatic categorization as Competitive (HHI < 1,500), Moderate Concentration (HHI 1,500-2,500), or High Concentration/Oligopoly (HHI > 2,500)
  - **Number of Active Sellers**: Counts unique providers per skill
  - **Total Trading Volume**: Aggregate value traded for the skill
  
  Results are included in JSON output under `skill_market_concentration`, sorted by HHI (most concentrated first) for easy identification of monopolistic skills. This feature enables research on market power, price-setting behavior, barriers to entry, and the effectiveness of competition policies. Perfect for studying how different skills evolve from competitive to oligopolistic markets and identifying skills that may require regulatory intervention.

### Market Concentration Analysis

Calculates the Herfindahl-Hirschman Index (HHI) to measure wealth concentration among participants. HHI values indicate market structure: < 1,500 (competitive), 1,500-2,500 (moderate concentration), > 2,500 (high concentration/oligopoly).

## Social Systems

### Reputation System

Each person has a reputation score (starting at 1.0) that increases with successful trades. Higher reputation leads to better prices (up to 10% discount), while lower reputation results in price premiums. Reputation slowly decays toward neutral over time, encouraging ongoing positive behavior.

### Friendship System

Social network dynamics through friendship formation between trading partners. When enabled, persons who successfully trade together have a configurable probability of becoming friends (bidirectional relationships). Friends receive price discounts when trading with each other (default: 10% discount), simulating trust and social capital. The system tracks comprehensive friendship statistics including total friendships formed, average/min/max friends per person, and network density (ratio of actual to possible friendships, ranging from 0.0 to 1.0). Enable via configuration file with `enable_friendships: true`, then configure `friendship_probability` (0.0-1.0, default: 0.1 or 10% chance per trade) and `friendship_discount` (0.0-1.0, default: 0.1 or 10% discount). Friendship discounts stack with reputation-based price adjustments. Ideal for studying social networks, trust in markets, and the economic impact of social relationships.

### Influence System

Track and analyze opinion leaders and influential persons based on social network position. When enabled alongside friendships, each person's influence score is automatically calculated using logarithmic scaling based on their friend count: `influence = 1.0 + ln(1 + friend_count)`. This creates realistic influence growth where:
  - Baseline influence: 1.0 (no friends)
  - 10 friends: ~2.6 influence
  - 30 friends: ~3.4 influence
  - 100 friends: ~4.6 influence
  
  The system tracks comprehensive statistics including average/min/max influence scores, standard deviation, count of high-influence persons (score > 2.0), and the top 5 most influential persons with their IDs, influence scores, and friend counts. Influence scores are updated dynamically as friendships form, providing real-time insights into network centrality and social capital distribution. Enable via configuration file with `enable_influence: true` (requires `enable_friendships: true`). Perfect for studying information diffusion, trend adoption patterns, opinion leadership, social influence on economic decisions, and the relationship between network position and market power. Future enhancements may include influence-based trend adoption and viral marketing effects.

### Social Class System

Automatic classification of persons into social classes based on wealth percentiles, with comprehensive mobility tracking. The system divides the population into four classes: Lower Class (bottom 25%), Middle Class (25th-75th percentile), Upper Class (75th-95th percentile), and Elite Class (top 5%). Social classes are updated periodically throughout the simulation based on each person's wealth relative to others, enabling dynamic class mobility. The system tracks comprehensive statistics including current class distribution, total upward and downward movements, a 4x4 transition matrix showing movements between classes, average class changes per person, and mobility rates (percentage of population experiencing upward/downward movement). All class transitions are recorded with timestamps. Always enabled with no configuration required—statistics are automatically calculated and included in JSON output under `social_class_statistics`. Perfect for studying economic mobility, wealth stratification, the effects of policy interventions on class structure, and the relationship between market dynamics and social hierarchy.

### Trade Agreements

Bilateral trade agreements between persons providing mutual price discounts on trades between agreement partners. When enabled, persons with existing friendships can form trade agreements at a configurable probability each simulation step (default: 5%). Agreements have a limited duration (default: 100 steps) and automatically expire, requiring renewal. Agreement partners receive an additional price discount (default: 15%) that stacks with friendship discounts, enabling study of preferential trading relationships and regional economic blocks. The system tracks comprehensive statistics including total agreements formed, active/expired counts, bilateral vs multilateral agreements, trade volume under agreements, and average discount rates. Enable via configuration file with `enable_trade_agreements: true`, then configure `trade_agreement_probability` (0.0-1.0, default: 0.05 or 5% chance per step), `trade_agreement_discount` (0.0-1.0, default: 0.15 or 15% discount), and `trade_agreement_duration` (in steps, default: 100). Requires friendships to be enabled for realistic behavior. Perfect for studying trade policy, economic integration, and the impact of preferential trade relationships on wealth distribution and market dynamics.

### Group/Organization System

Persons can be assigned to groups or organizations for collective behavior analysis. When enabled via `--num-groups` parameter (or configuration file), persons are distributed across groups using round-robin assignment at simulation start. Each group tracks member count, average/total money, and average reputation. Overall statistics include total groups, average/min/max group size, and per-group breakdowns. Groups remain static during simulation but enable studying economic dynamics at the collective level, such as wealth distribution between organizations, group-based inequality, and comparative performance. Statistics are included in JSON output under `group_statistics`. Useful for analyzing team dynamics, organizational economics, and group-level wealth accumulation patterns. Valid range: 1 to number of persons.

### Network Centrality Analysis

Advanced network analysis identifying key market participants and their roles in the trading network. Calculates four centrality metrics for each person: **Degree Centrality** (number of trading partners, normalized 0.0-1.0 indicating connectivity), **Betweenness Centrality** (how often a person lies on shortest paths between others, identifying brokers and bridges with values 0.0-1.0), **Eigenvector Centrality** (influence based on connections to other well-connected traders, normalized 0.0-1.0), and **PageRank** (importance based on weighted connections, normalized 0.0-1.0). Network-level metrics include number of connected components (separate trading groups), average centrality scores, and network density. The analysis identifies top 5 traders in each category: most connected (degree), best brokers (betweenness), most influential (eigenvector), and highest importance (PageRank). Automatically calculated from trading relationships and included in JSON output under `centrality_analysis`. Ideal for identifying market hubs, understanding power dynamics, detecting isolated trading communities, and analyzing the structure of economic networks. Uses the petgraph library for efficient graph algorithms.

### Mentorship System

Experienced persons can mentor others, reducing learning costs and accelerating skill acquisition. When enabled alongside education, persons with high-quality skills (quality >= 3.5 by default) can mentor others learning that skill. Mentored learners pay a reduced cost (default: 50% of normal learning cost), simulating the efficiency gain from having an experienced teacher. Mentors receive reputation bonuses (+0.05 by default) for successful mentoring, incentivizing knowledge transfer. The system tracks comprehensive mentorship statistics including total mentorships formed, successful mentored learnings, total cost savings, and counts of unique mentors and mentees. Enable via `--enable-mentorship` flag (requires `--enable-education` and works best with `--enable-quality`) with configurable parameters: `--mentorship-cost-reduction` (0.0-1.0, default: 0.5), `--min-mentor-quality` (0.0-5.0, default: 3.5), and `--mentor-reputation-bonus` (default: 0.05). Mentorship statistics are automatically included in JSON output. Perfect for studying knowledge transfer, educational efficiency, and the value of experience in human capital development.

### Voting System (Political Simulation)

Democratic governance and collective decision-making system for studying how different voting mechanisms affect economic outcomes. When enabled, persons can create and vote on proposals that affect simulation parameters. 
  
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

## Economic & Financial Systems

### Savings System

Persons can save a configurable percentage of their money each simulation step. Saved money is moved from available cash to a separate savings account, affecting spending capacity while enabling wealth accumulation studies. Configurable via `--savings-rate` parameter (0.0-1.0 range representing 0-100% savings rate). Savings statistics (total, average, median, min, max) are tracked and reported in results.

### Tax System

Configurable income tax on trade proceeds with optional redistribution. The system collects taxes from sellers' proceeds after transaction fees and can redistribute collected taxes equally among all persons at the end of each step. This simulates government taxation and wealth redistribution policies, allowing study of their effects on wealth inequality and economic activity. Controlled via `--tax-rate` parameter (0.0-1.0 range representing 0-100% tax rate) and `--enable-tax-redistribution` flag. Tax statistics (total collected, total redistributed) are tracked and reported in results.

### Loan System

Persons can borrow and lend money with interest and repayment schedules. When enabled, the system tracks loans between persons, processes scheduled repayments each step, and provides statistics on loan activity. Loans have configurable interest rates and repayment periods. Enable via `--enable-loans` flag or configuration file, with parameters `--loan-interest-rate` (0.0-1.0, default: 0.01 or 1% per step), `--loan-repayment-period` (in steps, default: 20), and `--min-money-to-lend` (minimum threshold for lending, default: 50.0). Loan statistics (total issued, repaid, active) are included in simulation results. Note: With loans enabled, persons can accumulate debt (negative money balances) when unable to make payments.

### Credit Rating System

FICO-like credit scoring (300-850 scale) that evaluates person creditworthiness based on financial behavior. When enabled alongside loans, each person's credit score dynamically updates based on: payment history (35% weight - successful vs. missed payments), debt level (30% weight - debt-to-money ratio), credit history length (15% weight - duration with loans), new credit activity (10% weight - recent loans), and credit mix (10% weight - variety of credit types). Credit scores directly affect loan interest rates: Excellent (800+) gets 50% discount, Very Good (740-799) gets 30% discount, Good (670-739) pays base rate, Fair (580-669) pays 50% premium, Poor (300-579) pays 150% premium. This simulates realistic lending risk assessment and creates differentiated credit markets. Enable via configuration file with `enable_credit_rating: true` (requires `enable_loans: true`). Credit score statistics (averages, distribution by rating category, payment history) are tracked and included in simulation results. Perfect for studying credit access inequality, lending discrimination, and the impact of credit history on economic mobility.

### Asset System

Persons can purchase and own long-term assets (Property, Equipment, Stocks) that accumulate wealth beyond liquid money, enabling realistic modeling of wealth inequality and capital accumulation.
  
  **Purchase Mechanism:** When enabled, persons with sufficient money (configurable threshold, default: 200.0) have a configurable probability (default: 2% per step) of purchasing an asset at a cost of `base_skill_price × asset_price_multiplier` (default multiplier: 10.0).
  
  **Asset Types:**
  - **Property**: Real estate that appreciates steadily over time (default: 0.2% per step) and generates rental income (default: 0.1% of asset value per step). Simulates wealth building through real estate investment.
  - **Equipment**: Production equipment that depreciates due to wear and tear (default: 1% per step) with a salvage value floor (10% of purchase price). Represents physical capital that loses value over time.
  - **Stocks**: Financial investments with variable returns based on expected return rate (default: 0.3% per step) plus random market volatility (±2%). Simulates stock market investments with risk and reward.

  **Statistics and Dynamics:** The system tracks comprehensive statistics including total assets purchased, active assets, total asset value, value distribution (average, median, min/max), total income generated, average ROI, breakdown by asset type, and ownership metrics (assets per person, ownership rate). Assets update each simulation step: values appreciate/depreciate according to their type, Property generates rental income added to owner's money, and all changes compound over time. Statistics are included in JSON output under `asset_statistics`.
  
  **Configuration:** Enable via configuration file with `enable_assets: true`. Configurable parameters: `asset_purchase_probability` (0.0-1.0, default: 0.02), `min_money_for_asset_purchase` (default: 200.0), `property_appreciation_rate` (default: 0.002), `equipment_depreciation_rate` (default: 0.01), `rental_income_rate` (default: 0.001), `stock_return_rate` (default: 0.003), and `asset_price_multiplier` (default: 10.0).
  
  **Research Applications:** The asset system enables research on wealth accumulation, capital formation, asset price dynamics, income vs. wealth inequality, and the role of different asset types in economic mobility. Works synergistically with loan system (asset-backed lending potential) and social class system (asset ownership correlates with class mobility).
  
  Example configuration:
  ```yaml
  enable_assets: true
  asset_purchase_probability: 0.05  # 5% chance per step
  min_money_for_asset_purchase: 300.0
  property_appreciation_rate: 0.003  # 0.3% per step
  asset_price_multiplier: 15.0  # Assets cost 150 with base price of 10
  ```
  
  **Known Limitations:** Asset data is not persisted in simulation checkpoints. When resuming from a checkpoint with assets enabled, asset information will be lost. Avoid using checkpoint resume with assets or run complete simulations without interruption.

### Investment System (Infrastructure)

Foundation for investment-based capital allocation allowing persons to invest money with expectations of future returns. The system includes complete data structures (`Investment` struct with investor, target, principal, return rate, duration), configuration parameters (`enable_investments`, `investment_return_rate`, `investment_duration`, `investment_probability`, `min_money_to_invest`), and investment portfolio tracking per person. Investment types include education investments (funding another person's skill learning) and production investments (enhancing production capacity). Returns are calculated as principal plus profit based on return rate and duration (e.g., 100 invested at 2% per step for 20 steps returns 120 total). Statistics tracking infrastructure (`InvestmentStats`) captures total investments created, completed, active count, total invested amount, total returns paid, and average ROI percentage. The investment creation and execution logic in the simulation engine is ready for future implementation. This enables research on capital allocation, risk-return trade-offs, and economic growth through investment.

### Contract System

Long-term agreements for stable trading relationships. When enabled, persons can form contracts that lock in prices for multiple simulation steps, providing price stability and predictable income/expenses for both parties. Contracts have configurable duration bounds and offer a price discount to incentivize formation. Enable via `--enable-contracts` flag or configuration file, with parameters `max_contract_duration`, `min_contract_duration`, and `contract_price_discount` (default: 5% discount). Contract statistics (total created, completed, active, average duration, total value) are tracked and included in simulation results. Ideal for studying long-term economic relationships, price stability mechanisms, and the effects of contractual obligations on market dynamics.

### Production System

Persons can combine two skills they possess to produce new, more valuable skills through predefined recipes. When enabled, persons have a configurable probability of attempting production each step. If they have the required input skills and can afford the production cost (based on input skill prices and a recipe cost multiplier), a new skill is learned and added to the market. The system includes 14 predefined recipes such as: Programming + DataAnalysis → MachineLearning, Marketing + GraphicDesign → DigitalMarketing, and Engineering + Programming → SoftwareEngineering. This simulates supply chains, skill composition, and economic specialization, enabling study of how advanced skills emerge from basic building blocks. Enable via `--enable-production` flag or configuration file with parameter `production_probability` (default: 0.05 or 5% chance per step). Produced skills are priced higher than their inputs (reflecting value added) and are automatically added to the market for trading. Works well in combination with the education system to create dynamic skill ecosystems where basic skills are learned and combined to create specialized advanced skills.

### Community Resource Pools

Groups can maintain shared resource pools for collective support and mutual aid. When enabled with `--enable-resource-pools` (requires `--num-groups`), each group maintains a pool where members contribute a configurable percentage of their money each step (`--pool-contribution-rate`, default: 2%). Members with money below a threshold (`--pool-withdrawal-threshold`, default: 30.0) receive equal distributions from their group's pool, simulating needs-based mutual aid. The system tracks pool balance, total contributions, and total withdrawals for each group and across all groups. This enables studying cooperative economics, solidarity economies, mutual aid societies, and alternative approaches to social insurance. Perfect for modeling community-based resource sharing, informal savings groups (ROSCAs), and collective security mechanisms. Pool statistics are included in JSON output under `group_statistics` with per-group and aggregate metrics. Configure via configuration file with `enable_resource_pools: true`, `pool_contribution_rate` (0.0-0.5), and `pool_withdrawal_threshold` (0.0-1000.0). Example: `./simulation-framework run --num-groups 3 --enable-resource-pools --pool-contribution-rate 0.05 --pool-withdrawal-threshold 50.0`

## Insurance & Risk Management

### Insurance System

Persons can purchase insurance policies to protect against economic risks. When enabled, persons randomly attempt to buy insurance each step based on a configurable probability (default: 5% per step). Three types of insurance are available:
  - **Crisis Insurance:** Protects against economic shocks like market crashes and currency devaluations. When a crisis event occurs, persons with active crisis insurance receive payouts proportional to the crisis severity and their coverage amount, helping them weather the economic storm.
  - **Income Insurance:** Provides safety net when trade income falls below a threshold (50% of base skill price). If a person's earnings from selling skills in a given step are too low, their income insurance compensates for the shortfall, helping maintain minimum living standards.
  - **Credit Insurance:** Protects borrowers against loan default risks. When a person with active loans faces financial distress (very low money relative to debt), credit insurance pays out to help cover their obligations, reducing the risk of default and protecting both the borrower from ruin and the lender from losses.
  
  Premiums are calculated as a percentage of coverage (default: 5%) and are adjusted based on reputation—persons with better reputations (higher scores) receive up to 20% discounts, while those with poor reputations pay up to 20% premiums, simulating risk-based pricing. Policies have configurable durations (default: 100 steps) and expire after one claim is paid or when the duration elapses. Each person can own multiple policies (one of each type), and policies are purchased randomly from available types they don't already own. Statistics tracked include total policies issued, active policies, claims paid, premiums collected, payouts made, net result, and loss ratio (payouts/premiums). Enable via `--enable-insurance` flag or configuration file, with parameters `--insurance-premium-rate` (0.0-1.0, default: 0.05 or 5% of coverage), `--insurance-duration` (in steps, default: 100; set to 0 for indefinite coverage), `--insurance-purchase-probability` (0.0-1.0, default: 0.05 or 5% chance per step), and `--insurance-coverage-amount` (default: 50.0). The system naturally integrates with loans (credit insurance), crisis events (crisis insurance), and trade dynamics (income insurance), enabling research on risk management, insurance market dynamics, moral hazard, adverse selection, and the role of insurance in economic stability. Example:
  ```bash
  ./simulation-framework run --enable-insurance \
    --insurance-purchase-probability 0.1 \
    --enable-crisis-events --crisis-probability 0.05 \
    --enable-loans
  ```

### Crisis Events

Random economic shocks that test the resilience of the simulated economy. When enabled, crisis events can occur randomly during the simulation, creating unexpected challenges such as:
  - **Market crashes:** Sudden price drops across all skills (20-40%)
  - **Demand shocks:** Reduced overall consumption (30-50%)
  - **Supply shocks:** Reduced availability (20-40%)
  - **Currency devaluations:** Wealth destruction (10-30%)
  - **Technology shocks:** Technological disruption making certain skills obsolete with massive value loss (50-80%), simulating automation and paradigm shifts
  
  Each crisis type has distinct effects on the economy with configurable severity levels. Enable via `--enable-crisis-events` flag with parameters `--crisis-probability` (frequency, default: 2% per step) and `--crisis-severity` (impact level 0.0-1.0, default: 0.5). The crisis scenario preset (`--preset crisis_scenario`) demonstrates this feature with higher crisis probability (5%) and severity (0.7) to create a challenging economic environment. Ideal for studying economic resilience, shock recovery, technological disruption, and the effectiveness of stabilization mechanisms like price floors and redistribution policies.

## Technology & Innovation

### Quality Rating System

Skills have quality ratings (0.0-5.0 scale) that evolve over time and affect prices. Higher quality skills command higher prices, creating quality competition alongside price competition. Quality improves through successful trades (practice makes perfect) at a configurable rate (default: +0.1 per trade, capped at 5.0). Unused skills decay in quality over time (default: -0.05 per step, minimum 0.0), simulating skill rust. Price adjustment formula: `price * (1.0 + (quality - 3.0) * 0.1)` means quality 5.0 gives +20% price, quality 3.0 (average) gives base price, and quality 1.0 gives -20% price. Quality statistics (average, median, min/max, skills at extremes) are tracked and reported. Enable via `--enable-quality` flag or configuration file with parameters `quality_improvement_rate` (default: 0.1), `quality_decay_rate` (default: 0.05), and `initial_quality` (default: 3.0). This system enables studying product differentiation, the relationship between experience and value, and the importance of maintaining skills through regular use. Works well with reputation system and education system for comprehensive skill value modeling.

### Technological Progress

Skills become more efficient over time through a configurable technology growth rate, simulating productivity improvements. More efficient skills effectively cost less, enabling increased trade and economic growth over the simulation period.

### Technology Breakthroughs

Sudden positive innovation events that dramatically boost skill efficiency. When enabled, breakthrough events can randomly occur (configurable probability, default 1% per step), suddenly increasing a random skill's efficiency by 20-50% (configurable range). Each breakthrough is tracked with details (skill affected, efficiency boost magnitude, simulation step). Breakthroughs represent disruptive innovations like AI tools boosting programmer productivity, new manufacturing techniques, or major scientific discoveries. Unlike gradual technological progress, breakthroughs create sudden step-changes in productivity. Statistics tracked include total breakthroughs, unique skills affected, average/min/max boost magnitudes, and complete event history. Enable via `--enable-technology-breakthroughs` flag with optional parameters: `--tech-breakthrough-probability` (0.0-1.0, default: 0.01 or 1% per step), `--tech-breakthrough-min-effect` (1.0-2.0, default: 1.2 or 20% minimum boost), and `--tech-breakthrough-max-effect` (1.0-2.0, default: 1.5 or 50% maximum boost). Complements gradual technology growth to model both incremental improvements and revolutionary innovations. Perfect for studying innovation dynamics, technology adoption patterns, and the economic impact of breakthrough discoveries on market structure and wealth distribution.

### Automation and Technological Unemployment

Simulates the gradual displacement of human labor by automation, AI, and digitalization. When enabled, skills with higher automation risk experience declining demand over time as technology replaces human workers. Each skill can have an automation risk (0.0-1.0 scale) where 0.0 indicates no automation risk (e.g., creative arts, human interaction) and 1.0 indicates high risk (e.g., repetitive manual tasks, routine data processing). Demand for automatable skills is probabilistically suppressed based on automation progress: `skip_probability = automation_risk × automation_rate × current_step`. For example, with automation_rate = 0.001 (default), a fully automatable skill (risk = 1.0) has a 10% chance to be skipped after 100 steps, 50% after 500 steps. Statistics tracked include skills at risk, average/min/max automation risk, automation progress, estimated demand reduction percentage, and top 5 most automated skills with current demand. Enable via configuration file (not available as CLI flag) with `enable_automation: true`, then configure `automation_rate` (default: 0.001 = 0.1% skip probability increase per step) and `automation_risks_per_skill` (HashMap mapping skill names to risk values). This enables research on: technology-induced structural unemployment, labor market adaptation patterns, skill obsolescence dynamics, economic inequality from technological change, and effectiveness of retraining/education policies (when combined with education system). Example config:
  ```yaml
  enable_automation: true
  automation_rate: 0.005  # Faster automation for testing
  automation_risks_per_skill:
    "Data Entry": 0.9      # High automation risk
    "Manufacturing": 0.7   # Medium-high risk
    "Programming": 0.3     # Low-medium risk (AI assistance, not full replacement)
    "Counseling": 0.1      # Low risk (requires human empathy)
  ```

### Education System

Persons can learn new skills over time by investing money in education. Each simulation step, persons have a configurable probability of attempting to learn a skill they don't currently possess. The cost to learn a skill is based on the current market price multiplied by a learning cost multiplier (default: 3x). This simulates human capital formation and skill development, allowing persons to become more versatile and participate in multiple markets. Education statistics (total skills learned, average per person, total spending) are tracked and reported. Enable via `--enable-education` flag or configuration file with parameters `learning_cost_multiplier` and `learning_probability`. Learned skills allow persons to provide those services in the market, increasing their earning potential.

### Certification System

Professional credentialing system that validates skill quality and increases market trust. When enabled, persons can invest money to get their skills certified by a central authority. Certifications have levels (1-5) based on skill quality (if quality system is enabled) or randomly assigned, with higher levels commanding greater price premiums (+5% per level, so level 5 = +25% price). Certification cost is calculated as: `skill_base_price × certification_cost_multiplier × certification_level` (default multiplier: 2.0). Certifications expire after a configurable duration (default: 200 steps) and must be renewed to maintain the price premium, simulating real-world credential renewal requirements. Each simulation step, persons have a configurable probability (default: 5%) of attempting certification if they can afford it and their skill isn't already certified. The system tracks comprehensive statistics including total certifications issued, expired certifications, active certifications, and total money spent on certification. Enable via `--enable-certification` flag or configuration file with configurable parameters: `--certification-cost-multiplier` (0.1-10.0, default: 2.0), `--certification-duration` (in steps, default: 200, set to 0 for non-expiring certifications), and `--certification-probability` (0.0-1.0, default: 0.05 or 5% chance per step). Certification statistics are automatically included in JSON output. Perfect for studying professional licensing, quality signaling, credential markets, and the economic impact of standardization and certification programs. Works synergistically with quality and reputation systems to create multi-dimensional skill value assessment.

## Advanced Market Systems

### Black Market

Parallel informal market with different pricing rules. When enabled, a configurable percentage of trades are routed to an alternative market that operates with different prices (typically cheaper), simulating informal economy dynamics. Configured via `enable_black_market`, `black_market_price_multiplier` (e.g., 0.8 for 20% discount), and `black_market_participation_rate` (e.g., 0.2 for 20% of trades). Black market statistics (trades, volume, percentages) are tracked separately and included in simulation results.

### Externality Analysis System

Track positive and negative externalities (costs or benefits affecting third parties not involved in transactions) to study market failures and optimal policy interventions. When enabled, the simulation tracks external impacts from each trade transaction and calculates:
  - **Social Costs vs Private Costs:** Separates private transaction value from external impact on society
  - **Optimal Pigovian Taxes/Subsidies:** Computes the ideal tax (for negative externalities) or subsidy (for positive externalities) to internalize external costs/benefits
  - **Externality Intensity:** Measures external effects as percentage of private transaction value
  - **Per-Skill Analysis:** Tracks externality contributions by individual skills (e.g., education produces positive externalities, manufacturing produces pollution)
  
  Externalities are configured as rates (-1.0 to 1.0) representing the external impact as percentage of transaction value. Positive rates create social benefits (e.g., education at +0.25 means 25% positive externality), negative rates create social costs (e.g., manufacturing at -0.15 means 15% pollution cost). Configure via YAML/TOML file with `enable_externalities: true`, `externality_rate` (default rate for all transactions), and `externality_rates_per_skill` (override default for specific skills). Statistics tracked include: total count, positive/negative split, total private value, total external value (net social impact), total social value (private + external), average externality per transaction, optimal Pigovian tax revenue, optimal subsidy amount, and detailed per-skill breakdown. Perfect for studying market failures, environmental economics, social benefit optimization, and the design of corrective taxes/subsidies. Example:
  ```yaml
  enable_externalities: true
  externality_rates_per_skill:
    "Education": 0.25    # 25% positive externality (informed citizens)
    "Healthcare": 0.20   # 20% positive externality (public health)
    "Manufacturing": -0.15  # 15% negative externality (pollution)
    "Research": 0.30     # 30% positive externality (knowledge spillovers)
  ```

### Welfare Analysis

Comprehensive economic welfare analysis quantifying gains from trade. The simulation automatically calculates consumer surplus (buyers' gains from paying less than maximum willingness to pay), producer surplus (sellers' profits after costs), and deadweight loss (unrealized welfare from failed trades). Welfare metrics are estimated from transaction data: consumer surplus is approximated as 10% of trade volume, producer surplus accounts for transaction fees, taxes, and production costs (assuming 70% profit margin), and deadweight loss is calculated from failed trade attempts at 20% of average trade value. Total welfare (consumer surplus + producer surplus) measures the economic value created by the market, while deadweight loss quantifies inefficiency from market frictions, taxes, or price controls. Per-trade averages enable comparison across different market configurations. These metrics are automatically included in simulation results (JSON output) and help evaluate policy effects on economic efficiency, study trade-offs between efficiency and equality, and optimize tax design. Perfect for welfare economics research, policy analysis, and understanding market efficiency. No configuration required—metrics are calculated automatically when trades occur. Example output includes: `consumer_surplus`, `producer_surplus`, `total_welfare`, `deadweight_loss`, `trades_analyzed`, `avg_consumer_surplus_per_trade`, `avg_producer_surplus_per_trade`.

### Health and Epidemiology System

Simulate disease transmission and economic impacts through trade networks. When enabled, persons can become sick through contact during trades, creating dynamic epidemic patterns that propagate through economic relationships. The system models:
  - **Disease Transmission:** When a healthy person trades with a sick person, there's a configurable probability (default: 5%) that the disease transmits during the transaction, simulating contagion through economic interactions
  - **Recovery Process:** Sick persons automatically recover after a set number of steps (default: 10), returning to full health and productivity
  - **Economic Impact:** Sick persons suffer reduced productivity (50% penalty), receiving only half of their normal trading proceeds when selling skills, simulating reduced work capacity during illness
  - **Seed Infections:** A configurable number of persons (default: 0) start the simulation already sick, allowing study of epidemic dynamics from various starting conditions
  
  The health system enables research on:
  - Economic impacts of pandemics and health crises
  - Disease spread through trade networks and economic activity
  - Productivity losses and GDP reductions from widespread illness
  - Network effects in epidemic propagation (highly connected traders spread disease faster)
  - Resilience of economic systems to health shocks
  - Optimal quarantine and containment strategies (by analyzing transmission patterns)
  
  Configure via `--enable-health` flag with optional parameters: `--disease-transmission-rate` (0.0-1.0, default: 0.05 or 5% per trade), `--disease-recovery-duration` (in steps, default: 10), and `--initial-sick-persons` (count, default: 0). Health statistics (sick persons over time, total infections, recoveries, productivity losses) are tracked and included in simulation results. The system integrates naturally with the trading system—more active traders have higher exposure risk, and epidemics can cause economic slowdowns as sick persons earn less and spend less. Perfect for studying pandemic economics, the intersection of public health and economic policy, and the network dynamics of disease spread in markets. Example: `./simulation-framework run --enable-health --initial-sick-persons 5 --disease-transmission-rate 0.1 --disease-recovery-duration 15 -s 500 -p 100`

### Environmental Resource Tracking

Track resource consumption and sustainability metrics throughout the simulation. When enabled, each transaction consumes environmental resources (Energy, Water, Materials, Land) proportional to its value. The system tracks total consumption by resource type, remaining reserves, and calculates sustainability scores (1.0 = sustainable, 0.0 = depleted, <0 = overconsumed). Environmental statistics include per-resource and overall sustainability scores, remaining reserves, and a boolean sustainability flag. This enables modeling ecological economics, studying the environmental impact of different trading behaviors, and analyzing resource depletion patterns. Configure via `enable_environment: true` in configuration file, with parameters `resource_cost_per_transaction` (default: 1.0, resource units consumed per dollar traded) and optional `custom_resource_reserves` (custom starting reserves per resource type). Default reserves: Energy 100,000, Water 100,000, Materials 100,000, Land 10,000 units. Environmental data is included in JSON output under `environment_statistics` with detailed per-resource breakdowns. Ideal for sustainability research, environmental policy analysis, and studying the trade-offs between economic growth and resource conservation.

## Behavioral Systems

### Behavioral Strategies

Persons are assigned different behavioral strategies that affect their spending decisions, creating heterogeneous agent behavior. Four strategy types are supported:
  - **Conservative** (0.7x spending multiplier): Risk-averse agents who prefer saving and only spend when they have ample reserves. Willing to spend up to 70% of their money on needed skills.
  - **Balanced** (1.0x spending multiplier): Standard agents with normal spending behavior. This is the default strategy.
  - **Aggressive** (1.3x spending multiplier): Risk-taking agents who prioritize acquiring skills and are willing to spend beyond their immediate means. Can afford skills up to 130% of their current money.
  - **Frugal** (0.5x spending multiplier): Extremely cautious agents who minimize spending and maximize savings. Only willing to spend up to 50% of their money.
  Strategies are distributed equally across the population using round-robin assignment, ensuring balanced representation. The strategy system enables studying how different agent behaviors affect market dynamics, wealth distribution, and economic activity.

### Adaptive Strategies

Persons can dynamically adapt their behavioral strategies based on performance through reinforcement learning. When enabled, agents track their wealth growth and adjust their spending behavior accordingly:
  - **Positive growth:** Agents become more aggressive (higher spending multiplier), reinforcing successful behavior
  - **Negative growth:** Agents become more conservative (lower spending multiplier), adapting to poor outcomes
  - **Exploration:** Random adjustments (ε-greedy approach) enable discovering new strategies
  - **Bounded adaptation:** Adjustment factor stays within 0.5-2.0x to prevent extreme behavior
  
  The system tracks success metrics (wealth growth rate, trade counts) and uses a simple learning rule: agents that are doing well increase their risk-taking, while struggling agents become more cautious. This creates emergent behavior patterns and evolutionary dynamics where successful strategies spread through learning rather than fixed assignment. Enable via configuration file with `enable_adaptive_strategies: true`, then configure `adaptation_rate` (0.0-1.0, default: 0.1 or 10% adaptation rate) and `exploration_rate` (0.0-1.0, default: 0.05 or 5% exploration). Perfect for studying agent learning, strategy evolution, market adaptation, and how behavioral flexibility affects economic outcomes.

### Evolutionary Strategies and Replicator Dynamics

Cultural evolution system where successful behavioral strategies spread through the population via imitation and mutation. When enabled, agents periodically observe their friends and can copy the strategies of more successful neighbors, creating replicator dynamics where effective strategies proliferate and ineffective ones die out. The system implements:
  - **Imitation Learning:** Agents with successful friends (higher wealth) may adopt their strategies with configurable probability (default: 30% per evolution step). Agents only imitate friends who are more successful than themselves, creating selection pressure for winning strategies.
  - **Mutation:** Random strategy changes occur with configurable probability (default: 5% per agent per evolution step) to maintain strategic diversity and enable exploration of the strategy space, preventing premature convergence.
  - **Periodic Evolution:** Strategy updates occur at regular intervals (default: every 50 steps) rather than continuously, modeling cultural transmission that happens through observation and social learning over time.
  - **Selection Dynamics:** Strategies that lead to higher wealth accumulation become more common in the population, while unsuccessful strategies gradually disappear, creating evolutionary stable strategies (ESS) in the long run.
  
  The system tracks complete evolution history including: strategy distribution snapshots at each evolution step, total strategy changes over time, breakdown of changes by type (mutations vs. imitations), and final strategy distribution. Statistics enable studying convergence patterns, diversity maintenance, and identification of dominant strategies. Requires `enable_friendships: true` as agents imitate strategies of their friends. Enable via configuration file with `enable_strategy_evolution: true`, then configure `evolution_update_frequency` (steps between updates, default: 50), `imitation_probability` (0.0-1.0, default: 0.3 or 30%), and `mutation_rate` (0.0-1.0, default: 0.05 or 5%). Perfect for studying cultural evolution, strategy diffusion, emergence of cooperation, co-evolution of behaviors, and evolutionarily stable strategies (ESS). Example: `./simulation-framework run --config config.strategy_evolution.yaml`

### Specialization and Diversification Strategies

Persons can adopt different approaches to skill development, strategically choosing between specialization (focusing on mastery of few skills) and diversification (learning many skills). When enabled alongside the quality system, three strategies are available:
  - **Specialist:** Focus on mastering few skills with higher quality (+1.0 quality bonus) and premium pricing (+15% price multiplier). Specialists command higher prices and quality in their chosen fields but face narrower market opportunities.
  - **Balanced:** Standard approach with moderate quality and pricing (no adjustments). Represents typical market participants with neither specialization benefits nor diversification advantages.
  - **Generalist:** Learn many skills with standard quality and pricing. While generalists don't receive quality or price premiums, they have broader market access and greater flexibility in adapting to demand changes.
  
  Strategies are distributed evenly across the population using round-robin assignment, ensuring balanced representation. The quality bonus is applied before quality-to-price conversion, and the price multiplier is applied as a separate factor after quality adjustments. This enables studying trade-offs between expertise and flexibility, the emergence of expert markets, income risk vs. market adaptability, and how specialization affects wealth distribution. Enable via configuration file with `enable_specialization: true`. Note: Quality bonuses require `enable_quality: true` to be visible. Perfect for research on labor market dynamics, skill differentiation, and the economics of expertise vs. versatility.

## Analysis & Research Tools

### Wealth Inequality Analysis

Comprehensive wealth distribution analysis including:
  - **Gini Coefficient:** Measures overall inequality (0 = perfect equality, 1 = perfect inequality)
  - **Lorenz Curve Data:** Visualization-ready coordinate pairs showing the cumulative distribution of wealth across the population. Key features:
    - **Format:** Each point represents (cumulative % of population, cumulative % of wealth), starting at (0.0, 0.0) and ending at (1.0, 1.0)
    - **Interpretation:** The diagonal line y=x represents perfect equality. Curves below the diagonal indicate inequality, with greater distance showing more extreme inequality
    - **Mathematical Relationship:** The area between the Lorenz curve and the perfect equality line is directly related to the Gini coefficient
    - **Output Location:** Automatically included in JSON output under `money_statistics.lorenz_curve`
    - **Visualization:** Use with visualization tools like matplotlib, gnuplot, or web-based charting libraries to create publication-quality inequality visualizations
    - **Use Cases:** Compare wealth distributions across different simulation scenarios, study inequality trends over time, and analyze policy impacts on wealth concentration
  - **Wealth Concentration Ratios:** Intuitive metrics showing what percentage of total wealth is held by:
    - Top 10% wealthiest persons (indicates upper class concentration)
    - Top 1% wealthiest persons (indicates extreme wealth concentration)
    - Bottom 50% of persons (indicates poverty/lower class share)
  - **ASCII Histogram Visualization:** Real-time terminal visualization of wealth distribution across 10 percentile buckets with color-coded bars (green for lower percentiles, yellow for middle, red for upper). Can be disabled with `--no-histogram` flag.
  - These metrics provide actionable insights into wealth distribution patterns and are included in all output formats (JSON, CSV, terminal summary).

### Per-Step Wealth Distribution Statistics

Time-series tracking of wealth distribution metrics at each simulation step. Captures comprehensive statistics including average/median/std dev, min/max money, Gini coefficient, Herfindahl index, and wealth concentration ratios (top 10%, top 1%, bottom 50%). Enables analysis of how economic inequality evolves over time. Available in both JSON output (`wealth_stats_history` array) and CSV format (`{prefix}_wealth_stats_history.csv`). Perfect for studying inequality dynamics, comparing policy scenarios, and generating time-series plots for research.

### Trading Partner Statistics

Comprehensive analysis of trading relationships and network structure. For each person, tracks unique trading partners, buyer/seller trade counts, and top partners by trade frequency and value. Network-level metrics include average unique partners per person, network density (0.0-1.0 indicating connectivity), and identification of the most active trading pairs. This feature enables social network analysis without complex graph structures, helping understand market dynamics, trading patterns, and relationship formation. All statistics are automatically calculated and included in JSON output for further analysis.

### Per-Skill Trade Analytics

Detailed trade statistics for each skill type, tracking trade count, total volume, and average price per skill. Enables identification of the most traded and valuable skills in the market. Results are sorted by total trading volume and included in JSON output for easy analysis.

### Failed Trade Attempt Tracking

Monitors and reports trade attempts that fail due to insufficient funds, providing insight into unmet demand and market accessibility. Tracks total failed attempts, failure rate (percentage of total attempts), and patterns over time. High failure rates indicate economic stress where persons want to buy skills but cannot afford them, revealing inefficiencies in wealth distribution and market accessibility. Statistics include total failures, failure rate, average/min/max failures per step. Failure rates are color-coded in terminal output (green < 10%, yellow < 30%, red ≥ 30%) to quickly identify market health issues. Available in JSON output and CSV exports for detailed analysis of economic accessibility over time.

### Velocity of Money Metric

Tracks how many times money changes hands during the simulation, a key indicator of economic activity and money circulation efficiency. Calculated as Total Transaction Volume / Total Money Supply, this metric reveals whether money is being actively used in trade (high velocity) or hoarded (low velocity). A value of 5.0 means each unit of money was used in transactions 5 times on average. Higher velocity indicates more dynamic economic activity, while lower velocity suggests capital is being accumulated rather than spent. The metric is displayed in terminal output ("Velocity of Money: X.XX (times money changed hands)"), included in JSON output under `trade_volume_statistics.velocity_of_money`, and exported to CSV. Perfect for studying the relationship between money supply and economic output, comparing different pricing scenarios, analyzing the impact of savings rates on circulation, and understanding monetary dynamics. This classic economic indicator enables comparison with real-world velocity trends and research on how different economic policies affect money circulation patterns.

### Price Elasticity Analysis

Comprehensive calculation of price elasticities of demand and supply for every skill in the simulation, measuring how responsive quantities are to price changes. Uses the midpoint method for calculating elasticities to ensure symmetric, unbiased results: `Elasticity = ((Q2 - Q1) / ((Q2 + Q1) / 2)) / ((P2 - P1) / ((P2 + P1) / 2))`. For each skill, tracks demand elasticity (typically negative: higher prices reduce demand), supply elasticity (typically positive: higher prices increase supply), and automatically classifies elasticity as: **Perfectly Inelastic** (E ≈ 0), **Inelastic** (0 < |E| < 1), **Unit Elastic** (|E| = 1), **Elastic** (|E| > 1), or **Perfectly Elastic** (|E| = ∞). Calculates elasticities across all time periods, providing average elasticity per skill with standard deviations to measure consistency. Economy-wide averages across all skills enable macro-level analysis of market responsiveness. Elasticity data is automatically generated for simulations with 2+ steps and included in JSON output under `elasticity_statistics`. Perfect for economic research on price sensitivity, optimal policy design (e.g., tax incidence), market intervention effects, and comparing simulation behavior with empirical economic data. Example output includes per-skill elasticities with classifications, sample sizes, standard deviations, plus aggregate averages and analysis period information.

### Equilibrium Convergence Analysis

Analyzes whether and how quickly markets converge to equilibrium by tracking excess demand (demand minus supply) for each skill over time. In Walrasian equilibrium theory, markets should converge to a state where supply equals demand through price adjustments (tâtonnement process). This feature validates whether the simulation exhibits this theoretical convergence behavior. For each skill, calculates: average excess demand, final excess demand at last step, standard deviation (volatility measure), equilibrium percentage (% of steps at equilibrium), distance to equilibrium, convergence status (whether variance is decreasing over time), and average absolute excess demand (overall market clearing measure). Economy-wide metrics include average excess demand across all skills/periods, average distance to equilibrium, overall equilibrium percentage, number of converging skills, and convergence rate (ratio of final-period to initial-period variance, where < 1.0 indicates convergence). Equilibrium statistics are automatically generated for simulations with 2+ steps and included in JSON output under `equilibrium_statistics`. Perfect for validating simulation against economic theory, measuring convergence speed, identifying chronically unbalanced markets, and assessing how interventions (taxes, price controls, etc.) affect equilibrium dynamics. Enables comparison of different pricing scenarios and demand strategies to understand which market mechanisms lead to efficient clearing.

### Business Cycle Detection

Automatic identification of economic expansion and contraction phases from trade volume patterns. Uses a simple peak/trough detection algorithm with moving average smoothing to identify cyclical behavior in the simulated economy. For each detected cycle, tracks phase type (expansion or contraction), start/end steps, duration, average volume, and peak/trough volumes. Calculates aggregate statistics including total cycles detected, average cycle duration, average expansion duration, and average contraction duration. Requires at least 10 simulation steps for meaningful detection. Results are included in JSON output under `business_cycle_statistics` when cycles are detected. Perfect for studying endogenous economic cycles, boom-bust patterns, market volatility, and the natural oscillations that emerge from agent interactions without external shocks. Enables research on cycle amplitudes, periodicity, and asymmetry between expansions and contractions.

### Causal Analysis Framework

Rigorous statistical framework for evaluating policy interventions and mechanism designs through causal inference. Compare treatment and control groups using Welch's t-tests to determine statistically significant differences in key metrics (average money, Gini coefficient, total trades, average reputation). The framework automatically calculates effect sizes (absolute and relative), confidence intervals, t-statistics, p-values, and significance levels. Results include detailed statistical tests for each metric with clear interpretation of treatment effects. Use this to rigorously evaluate whether a policy intervention (e.g., changing tax rates, enabling loans) has a significant causal effect on economic outcomes. The framework accepts any two sets of `SimulationResult` objects (typically from Monte Carlo runs with different configurations) and produces a comprehensive `CausalAnalysisResult` with statistical comparison. Perfect for A/B testing, policy impact evaluation, and scientific research on economic mechanisms. Access via the `causal_analysis` module with the `CausalAnalysisResult::analyze()` method. Example use cases: testing if savings rates reduce inequality, evaluating impact of transaction fees on trade volume, measuring effect of education system on wealth distribution.

### Parameter Sweep Analysis

Automated sensitivity analysis through systematic parameter sweeps (grid search). Test a parameter across a range of values with multiple runs per value to understand how parameter choices affect simulation outcomes. Supports sweeping initial_money, base_price, savings_rate, and transaction_fee. Results include aggregated statistics and identification of optimal parameter values for different objectives. Perfect for research, parameter tuning, and understanding system robustness.

### Monte Carlo Simulations

Run multiple parallel simulations with different random seeds to achieve statistical significance. Automatically aggregates results across runs with mean, standard deviation, min, max, and median statistics for key metrics (average money, Gini coefficient, trade volume, reputation). Ideal for research, parameter sensitivity analysis, and understanding simulation variability.

### Scenario Comparison

Compare multiple simulation scenarios side-by-side to analyze the effects of different economic policies. Run A/B testing on pricing mechanisms (Original, DynamicPricing, AdaptivePricing) with multiple runs per scenario for statistical robustness. Automatically determines winners based on different criteria: highest average wealth, lowest inequality, highest trade volume, and highest reputation. Results are saved in JSON format with detailed statistics and winner analysis. Ideal for policy evaluation, economic research, and understanding the impact of different market mechanisms on outcomes.

## Output & Export Formats

### JSON Output

Outputs detailed simulation results, including **run metadata** (timestamp, git commit hash, simulation parameters, rust/framework versions for reproducibility), final wealth distribution, reputation statistics, skill valuations, and skill price history over time (suitable for graphing), to a JSON file. Metadata enables scientific reproducibility by capturing exact simulation conditions and code version.

### CSV Export

Export simulation results to multiple CSV files for easy analysis in Excel, pandas, R, or other data analysis tools. Includes summary statistics, per-person distributions, skill prices, time-series price history, and trading network data (nodes and edges).

### SQLite Database Export

Export simulation results to a SQLite database for long-term storage, querying, and analysis. Creates tables for money distribution, reputation distribution, skill prices, and summary statistics. Ideal for integration with data analysis tools, business intelligence platforms, and automated reporting systems.

### Streaming Output (JSONL)

Real-time streaming of step-by-step simulation data to a JSON Lines (JSONL) file. Each simulation step appends one JSON object containing key metrics (trades, volume, money statistics, Gini coefficient, reputation) to the output file. Enables real-time monitoring of long-running simulations, reduces memory footprint by not storing all step data in memory, and allows progressive analysis. Each line is a complete JSON object that can be parsed independently, making it ideal for streaming analysis tools and real-time dashboards.

### Trading Network Export

Export the trading network graph for visualization and analysis. The simulation automatically exports trading relationships as a network graph in both JSON and CSV formats. JSON output is compatible with vis.js, D3.js, NetworkX (Python), Gephi, and Cytoscape for creating interactive network visualizations. CSV export provides separate node and edge files for import into spreadsheet tools, network analysis packages (igraph, NetworkX), or graph databases. Network nodes include person attributes (money, reputation, trade count, unique partners) and edges capture relationship strength (number of trades, total value exchanged). Exported automatically when using `--csv-output` flag, or programmatically via `save_trading_network_json()` and `save_trading_network_csv()` methods. Ideal for social network analysis, identifying trading hubs, visualizing market structure, and studying economic relationships without complex graph libraries.

### Compressed Output

Optional gzip compression for JSON output files, reducing file sizes by 10-20x while maintaining full data fidelity. Ideal for large-scale simulations and batch processing.

## Configuration & Usability

### Configuration Files

Support for YAML and TOML configuration files to easily define complex simulation scenarios without lengthy command lines.

### Interactive Configuration Wizard

User-friendly command-line wizard (`wizard` subcommand) that guides users through creating simulation configurations step-by-step. Features include preset selection with descriptions, customization options for basic parameters (steps, persons, money), pricing scenario selection, advanced feature toggles with automatic dependency checking (e.g., credit rating requires loans), and configuration file export to YAML or TOML formats. The wizard provides help text for each option and validates inputs in real-time. Perfect for new users, teaching, and quickly exploring different simulation configurations without manually editing config files. After configuration, the wizard offers to run the simulation immediately or save the config for later use.

### Configurable Parameters

Allows customization of simulation parameters via command-line arguments or configuration files (YAML/TOML). CLI arguments override config file values.

### Input Validation

Comprehensive validation of all configuration parameters with clear error messages. Ensures parameters are within acceptable ranges (e.g., positive values for steps/persons, valid ranges for rates/amplitudes) to prevent crashes and provide immediate feedback on configuration errors.

### Checkpoint System

Save and resume simulation state at any point. Automatically save checkpoints at regular intervals during long simulations. Resume from saved checkpoints to continue interrupted simulations without starting from scratch. Useful for multi-hour simulations, distributed computing, incremental analysis, and crash recovery. Checkpoints are stored in JSON format with complete simulation state including entities, market data, loans, and statistics.

## Development & Debugging

### Invariant Checking Framework

Built-in validation system for checking simulation correctness during execution. When enabled via `--enable-invariant-checking`, the simulation validates conditions that should always hold true at each step:
  - **Money Conservation Invariant:** Ensures total money in the system remains constant (accounting for fees, taxes, and savings), helping detect bugs in money creation/destruction logic
  - **Non-Negative Wealth Invariant:** Ensures no person has negative money balances (automatically skipped when loans are enabled, allowing debt)
  
  Supports two modes:
  - **Lenient mode** (default): Logs violations as warnings but continues simulation, useful for debugging
  - **Strict mode** (`--strict-invariant-mode`): Panics and aborts on first violation, ensuring perfect correctness
  
  The framework is extensible - custom invariants can be added by implementing the `Invariant` trait. Individual invariants can be toggled via configuration (`check_money_conservation`, `check_non_negative_wealth`). Perfect for development, testing, and ensuring simulation validity. Minimal performance overhead when disabled (default).

### Trace Mode

Comprehensive debug logging for problem diagnosis. Enable detailed logging of all simulation actions including trade attempts, price updates, reputation changes, loan payments, and tax redistribution. Use environment variable `RUST_LOG=debug` for detailed logs or `RUST_LOG=trace` for extremely detailed output. Ideal for debugging simulation behavior, understanding agent decision-making, and diagnosing unexpected results.

### Structured Logging

Configurable logging system for debugging and monitoring using standard Rust logging infrastructure (`log` + `env_logger`).

### Interactive Mode (REPL)

Step-by-step simulation control through an interactive command-line interface. Enable with `--interactive` flag to enter a read-eval-print loop (REPL) where you can execute individual simulation steps, inspect current state, and save checkpoints on demand. Available commands:
  - `step` - Execute one simulation step
  - `run N` - Execute N simulation steps
  - `status` - Show current step, progress, active persons, and scenario
  - `stats` - Display comprehensive statistics snapshot (money, reputation, savings, trade volume, etc.)
  - `save <path>` - Save current state to checkpoint file
  - **Debugging Commands** (added for better simulation inspection):
    - `inspect <id>` - Show detailed state of a specific person (money, skills with quality, reputation, loans, recent transactions)
    - `persons` / `list-persons` - List all persons with summary info (ID, money, savings, reputation, skills, active status)
    - `market` - Display current market state (base price, volatility, top 20 skills by price with supply/demand)
    - `find-rich [N]` - Show top N wealthiest persons (default: 10) with ID for easy inspection
    - `find-poor [N]` - Show bottom N poorest persons (default: 10) with ID for easy inspection
    - `filter-by-skill <name>` - List persons with a specific skill (case-insensitive substring match)
  - `help` - Show all available commands
  - `exit`/`quit` - Exit interactive mode
  
  Features include command history (navigate with arrow keys), graceful handling of Ctrl+C and Ctrl+D, color-coded output, and real-time performance metrics. Perfect for debugging, exploring simulation behavior, teaching, demonstrations, and iterative testing of parameter changes. Example: `./simulation-framework run --interactive -s 100 -p 10`

### Action Recording for Replay

Record all simulation actions (trades, failed trades, price updates, crisis events) to a JSON file for replay analysis, debugging, and regression testing. Enable with `--record-actions <path>` flag. The action log captures:
  - **Successful Trades:** Buyer/seller IDs, skill traded, and final price
  - **Failed Trades:** Trade attempts that failed due to insufficient funds
  - **Price Updates:** All skill price changes (> 1 cent threshold)
  - **Crisis Events:** Crisis type and severity when they occur
  
  The JSON format includes simulation metadata (seed, entity count, max steps) and a chronological list of all actions, enabling deterministic replay and detailed analysis of simulation behavior. Action logs integrate with the checkpoint system for persistence across simulation restarts. Minimal performance overhead when disabled (default). Example: `./simulation-framework run -s 500 -p 100 --record-actions actions.json`

### Replay and Debugging System

Comprehensive debugging capabilities for bug reproduction and deterministic testing. Combines checkpoints, streaming output, and detailed logging to enable exact reproduction of simulation behavior. Fixed seeds ensure identical results across runs, enabling regression testing and bug investigation. Load checkpoints to inspect exact simulation state at any point, use streaming output to identify problematic steps, and leverage trace-level logging to understand decision-making. The system provides action log infrastructure for detailed event tracking when needed. Perfect for troubleshooting, validating changes, and understanding complex simulation dynamics without specialized replay tools.

### Enhanced Progress Bar

Visual progress indicator with live simulation metrics displayed in a compact, information-rich format. The progress bar shows five key real-time statistics that update periodically during simulation:
  - **Active:** Number of active entities currently participating in the economy
  - **$̄ (Average Money):** Mean wealth across all active persons, using mathematical notation for brevity
  - **Trades:** Number of trades executed in the most recent simulation step, indicating market activity levels
  - **P̄ (Average Price):** Mean price across all skills in the market, showing overall price level trends
  - **Gini:** Gini coefficient (0.0-1.0+ scale) measuring wealth inequality, where 0 indicates perfect equality and higher values indicate greater concentration
  
  The enhanced display provides immediate insight into simulation dynamics without interrupting execution. Updates occur at calculated intervals (every 1% of steps or every 10 steps, whichever is less frequent) to balance informativeness with performance. Example display: `Active: 100 | $̄: 250.3 | Trades: 45 | P̄: 15.2 | Gini: 0.345`. Can be disabled with `--no-progress` flag. Perfect for monitoring long-running simulations, detecting anomalies early, and understanding economic trends as they emerge.
  
  **Note:** The mathematical notation symbols (P̄, $̄) use Unicode combining characters (macron) which require a terminal with Unicode support. Most modern terminals support these characters, but if they display incorrectly as garbled text or question marks, this indicates limited Unicode support in your terminal environment.

### Colored Terminal Output

Enhanced terminal output with color-coded statistics and messages for improved readability. Automatically detects terminal capabilities and can be disabled with `--no-color` flag.

### Memory Pooling

Optional object pooling infrastructure (`VecPool<T>`) for reusing vector allocations to reduce allocation overhead. The pool maintains cleared vectors that can be reused, minimizing pressure on the system allocator and reducing memory fragmentation. Particularly beneficial in simulation loops with frequent allocations and deallocations. The pool is designed as a general-purpose utility available for integration throughout the codebase.

### Plugin System

Extensible trait-based plugin architecture for custom simulation extensions without modifying core code. Plugins can hook into simulation lifecycle events (start, step start/end, completion) to:
  - Collect custom metrics and statistics
  - Monitor and log simulation behavior
  - Modify simulation results with additional data
  - Implement custom algorithms and analysis
  
  The plugin system uses a feature-flags approach for compile-time plugin selection, ensuring zero runtime overhead for unused plugins. Plugins are type-safe, thread-safe (`Send + Sync`), and support downcasting for accessing plugin-specific methods. Perfect for research extensions, custom metrics collection, and experimental features without forking the codebase. See the `plugin` module documentation for implementation details and the `Plugin` trait for available lifecycle hooks.

