use crate::error::{Result, SimulationError};
use crate::scenario::{DemandStrategy, Scenario};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Preset configuration names for typical simulation scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum PresetName {
    Default,
    SmallEconomy,
    LargeEconomy,
    CrisisScenario,
    HighInflation,
    TechGrowth,
    QuickTest,
}

impl PresetName {
    /// Get all available preset names
    pub fn all() -> Vec<PresetName> {
        vec![
            PresetName::Default,
            PresetName::SmallEconomy,
            PresetName::LargeEconomy,
            PresetName::CrisisScenario,
            PresetName::HighInflation,
            PresetName::TechGrowth,
            PresetName::QuickTest,
        ]
    }

    /// Get the string identifier for this preset
    pub fn as_str(&self) -> &str {
        match self {
            PresetName::Default => "default",
            PresetName::SmallEconomy => "small_economy",
            PresetName::LargeEconomy => "large_economy",
            PresetName::CrisisScenario => "crisis_scenario",
            PresetName::HighInflation => "high_inflation",
            PresetName::TechGrowth => "tech_growth",
            PresetName::QuickTest => "quick_test",
        }
    }

    /// Get a description of this preset
    pub fn description(&self) -> &str {
        match self {
            PresetName::Default => "Standard economy with 100 persons, 500 steps",
            PresetName::SmallEconomy => "Small economy with 20 persons for quick testing",
            PresetName::LargeEconomy => "Large economy with 500 persons for detailed analysis",
            PresetName::CrisisScenario => "Economic crisis with low initial money and high prices",
            PresetName::HighInflation => "High inflation scenario with dynamic pricing",
            PresetName::TechGrowth => "Technology growth scenario with high initial capital",
            PresetName::QuickTest => "Very small economy for rapid testing (10 persons, 50 steps)",
        }
    }
}

/// Implement FromStr trait for parsing preset names from strings
impl FromStr for PresetName {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(PresetName::Default),
            "small_economy" | "small" => Ok(PresetName::SmallEconomy),
            "large_economy" | "large" => Ok(PresetName::LargeEconomy),
            "crisis_scenario" | "crisis" => Ok(PresetName::CrisisScenario),
            "high_inflation" | "inflation" => Ok(PresetName::HighInflation),
            "tech_growth" | "tech" => Ok(PresetName::TechGrowth),
            "quick_test" | "quick" => Ok(PresetName::QuickTest),
            _ => Err(format!("Unknown preset: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    // General simulation parameters
    pub max_steps: usize,
    pub entity_count: usize, // This will be our number of persons
    pub seed: u64,

    // Economic simulation specific parameters
    pub initial_money_per_person: f64,
    pub base_skill_price: f64,
    // num_unique_skills will be equal to entity_count as each person has one unique skill
    /// Minimum price floor for skills.
    ///
    /// Prevents skill prices from dropping below this threshold, modeling real-world
    /// price floors like minimum wages or regulatory price controls.
    /// This ensures market stability and prevents skills from becoming worthless.
    /// Must be positive and less than or equal to base_skill_price.
    /// Default: 1.0
    #[serde(default = "default_min_skill_price")]
    pub min_skill_price: f64,

    // time_step might not be directly relevant for a turn-based economic sim,
    // but we can keep it or remove it later. For now, let's keep it.
    pub time_step: f64,
    pub scenario: Scenario,

    /// Demand generation strategy.
    ///
    /// Controls how the number of needed skills per person is determined each step.
    /// Different strategies create different market dynamics:
    /// - Uniform: Random 2-5 needs per person (balanced market, default)
    /// - Concentrated: Most persons have low demand, few have high (inequality)
    /// - Cyclical: Demand varies over time in cycles (business cycles)
    ///
    /// This enables experimentation with demand patterns to study their effects
    /// on market behavior, wealth distribution, and economic activity.
    /// Default: Uniform
    #[serde(default)]
    pub demand_strategy: DemandStrategy,

    /// Technology growth rate per simulation step.
    ///
    /// This rate determines how quickly skills become more efficient over time,
    /// simulating technological progress and productivity improvements.
    /// A rate of 0.001 means skills improve by 0.1% per step.
    /// Set to 0.0 to disable technological progress (default).
    #[serde(default)]
    pub tech_growth_rate: f64,

    /// Seasonal demand amplitude (0.0 = no seasonality, 0.0-1.0 = variation strength).
    ///
    /// Controls the strength of seasonal fluctuations in skill demand.
    /// A value of 0.5 means demand can vary ±50% from the base level.
    /// Set to 0.0 to disable seasonal effects (default).
    #[serde(default)]
    pub seasonal_amplitude: f64,

    /// Seasonal cycle period in simulation steps.
    ///
    /// Determines how many steps it takes for demand to complete one seasonal cycle.
    /// For example, a value of 100 means demand patterns repeat every 100 steps.
    /// Only used when seasonal_amplitude > 0.0.
    #[serde(default = "default_seasonal_period")]
    pub seasonal_period: usize,

    /// Transaction fee rate as a percentage of the transaction value.
    ///
    /// This represents the cost of conducting trade in the market.
    /// The fee is deducted from the seller's proceeds.
    /// A value of 0.05 means a 5% fee is charged on each transaction.
    /// Set to 0.0 to disable transaction fees (default).
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default)]
    pub transaction_fee: f64,

    /// Savings rate as a percentage of current money to save each step.
    ///
    /// Each simulation step, persons will save this percentage of their current money.
    /// The saved money is moved from available cash to savings, affecting spending capacity.
    /// A value of 0.05 means 5% of current money is saved each step.
    /// Set to 0.0 to disable savings (default).
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default)]
    pub savings_rate: f64,

    /// Enable loan system where persons can borrow and lend money.
    ///
    /// When enabled, persons can request loans from others when they lack money for purchases.
    /// Loans have interest rates and repayment schedules.
    /// Set to false to disable loans (default).
    #[serde(default)]
    pub enable_loans: bool,

    /// Interest rate per step for loans (e.g., 0.01 = 1% per step).
    ///
    /// This is the interest charged on loan principal per simulation step.
    /// Only used when enable_loans is true.
    /// A value of 0.01 means the borrower pays 1% interest per step.
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default = "default_loan_interest_rate")]
    pub loan_interest_rate: f64,

    /// Default repayment period for loans in simulation steps.
    ///
    /// Determines how many steps a borrower has to repay a loan.
    /// Only used when enable_loans is true.
    /// For example, a value of 20 means loans are repaid over 20 steps.
    #[serde(default = "default_loan_repayment_period")]
    pub loan_repayment_period: usize,

    /// Minimum money threshold for a person to be eligible to lend.
    ///
    /// Persons must have at least this much money to provide loans to others.
    /// Only used when enable_loans is true.
    /// This prevents persons from becoming too poor by lending all their money.
    #[serde(default = "default_min_money_to_lend")]
    pub min_money_to_lend: f64,

    /// Interval (in steps) between automatic checkpoint saves.
    ///
    /// When set to a value > 0, the simulation will automatically save its state
    /// every N steps to the checkpoint file. Set to 0 to disable auto-checkpointing (default).
    /// For example, a value of 100 means a checkpoint is saved every 100 steps.
    #[serde(default)]
    pub checkpoint_interval: usize,

    /// Path to the checkpoint file for saving/loading simulation state.
    ///
    /// When resume_from_checkpoint is true, the simulation loads its initial state from this file.
    /// When checkpoint_interval > 0, the simulation saves its state to this file.
    /// If not specified, defaults to "checkpoint.json" when needed.
    #[serde(default)]
    pub checkpoint_file: Option<String>,

    /// Resume the simulation from a previously saved checkpoint.
    ///
    /// When true, the simulation will load its state from the checkpoint file
    /// instead of initializing from scratch. The checkpoint file must exist.
    /// Set to false to start a new simulation (default).
    #[serde(default)]
    pub resume_from_checkpoint: bool,

    /// Tax rate as a percentage of trade income (0.0-1.0, e.g., 0.10 = 10% tax).
    ///
    /// This represents an income tax collected on seller's proceeds from trades.
    /// The tax is deducted from the seller's proceeds after the transaction fee.
    /// A value of 0.10 means a 10% tax is collected on each sale.
    /// Set to 0.0 to disable taxation (default).
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default)]
    pub tax_rate: f64,

    /// Enable redistribution of collected taxes to all persons.
    ///
    /// When enabled, taxes collected during each step are distributed equally
    /// among all persons at the end of each step. This simulates basic income
    /// or wealth redistribution policies.
    /// Set to false to collect taxes without redistribution (default).
    #[serde(default)]
    pub enable_tax_redistribution: bool,

    /// Number of skills each person can provide.
    ///
    /// Determines how many different skills each person possesses and can offer to others.
    /// A value of 1 means each person specializes in a single skill (default).
    /// Higher values create more versatile persons who can participate in multiple markets.
    /// Valid range: 1 to entity_count
    ///
    /// **Note:** The total number of unique skills in the market remains entity_count,
    /// but with skills_per_person > 1, skills will be distributed across multiple persons,
    /// increasing market redundancy and competition.
    #[serde(default = "default_skills_per_person")]
    pub skills_per_person: usize,

    /// Path to stream step-by-step simulation data in JSONL format.
    ///
    /// When enabled, the simulation appends one JSON object per line to this file after each step.
    /// This allows real-time monitoring of long-running simulations and reduces memory usage
    /// by not storing all step data in memory.
    /// Set to None to disable streaming output (default).
    #[serde(default)]
    pub stream_output_path: Option<String>,

    /// Weight for urgency in priority-based buying decisions (0.0-1.0).
    ///
    /// Controls how much the urgency level influences purchase priority.
    /// Higher values make buyers prioritize urgent needs more strongly.
    /// Default: 0.5 (balanced with other factors)
    #[serde(default = "default_priority_urgency_weight")]
    pub priority_urgency_weight: f64,

    /// Weight for affordability in priority-based buying decisions (0.0-1.0).
    ///
    /// Controls how much the cost relative to available money influences purchase priority.
    /// Higher values make buyers prioritize cheaper items more strongly.
    /// Default: 0.3 (moderate consideration of affordability)
    #[serde(default = "default_priority_affordability_weight")]
    pub priority_affordability_weight: f64,

    /// Weight for efficiency in priority-based buying decisions (0.0-1.0).
    ///
    /// Controls how much skill efficiency (from technological progress) influences purchase priority.
    /// Higher values make buyers prioritize more efficient skills more strongly.
    /// Default: 0.1 (minor consideration of efficiency)
    #[serde(default = "default_priority_efficiency_weight")]
    pub priority_efficiency_weight: f64,

    /// Weight for seller reputation in priority-based buying decisions (0.0-1.0).
    ///
    /// Controls how much the seller's reputation influences purchase priority.
    /// Higher values make buyers strongly prefer reputable sellers.
    /// Default: 0.1 (minor consideration of reputation)
    #[serde(default = "default_priority_reputation_weight")]
    pub priority_reputation_weight: f64,

    /// Enable a parallel black market with different pricing rules.
    ///
    /// When enabled, a percentage of trades are routed to an alternative market
    /// that operates with different prices and rules, simulating informal economy.
    /// Set to false to disable black market (default).
    #[serde(default)]
    pub enable_black_market: bool,

    /// Price multiplier for the black market (0.0-2.0).
    ///
    /// Skills on the black market are priced at this multiple of the regular market price.
    /// Values < 1.0 make black market cheaper (typical), values > 1.0 make it more expensive.
    /// For example, 0.8 means black market prices are 20% lower than regular market.
    /// Only used when enable_black_market is true.
    /// Default: 0.8 (20% discount)
    #[serde(default = "default_black_market_price_multiplier")]
    pub black_market_price_multiplier: f64,

    /// Percentage of trades routed to black market (0.0-1.0).
    ///
    /// Determines what fraction of eligible trades occur on the black market.
    /// For example, 0.2 means 20% of trades use the black market.
    /// Only used when enable_black_market is true.
    /// Default: 0.2 (20% of trades)
    #[serde(default = "default_black_market_participation_rate")]
    pub black_market_participation_rate: f64,

    /// Enable contract system for long-term agreements between persons.
    ///
    /// When enabled, persons can form long-term contracts that lock in prices
    /// for regular transactions over multiple steps, providing stability and predictability.
    /// Set to false to disable contracts (default).
    #[serde(default)]
    pub enable_contracts: bool,

    /// Maximum duration for contracts in simulation steps.
    ///
    /// Determines the maximum length of time a contract can remain active.
    /// Contracts will execute automatically for this many steps at a fixed price.
    /// Only used when enable_contracts is true.
    /// Default: 50 steps
    #[serde(default = "default_max_contract_duration")]
    pub max_contract_duration: usize,

    /// Minimum duration for contracts in simulation steps.
    ///
    /// Determines the minimum length of time a contract must last.
    /// This prevents very short contracts that wouldn't provide stability benefits.
    /// Only used when enable_contracts is true.
    /// Default: 10 steps
    #[serde(default = "default_min_contract_duration")]
    pub min_contract_duration: usize,

    /// Percentage discount on market price for contract trades (0.0-1.0).
    ///
    /// Contracts offer price stability, and this discount incentivizes their formation.
    /// For example, 0.1 means contract prices are 10% lower than current market price.
    /// Only used when enable_contracts is true.
    /// Default: 0.05 (5% discount)
    #[serde(default = "default_contract_price_discount")]
    pub contract_price_discount: f64,

    /// Enable education system where persons can learn new skills.
    ///
    /// When enabled, persons can invest money to learn new skills over time,
    /// simulating human capital formation and skill development.
    /// Set to false to disable skill learning (default).
    #[serde(default)]
    pub enable_education: bool,

    /// Cost multiplier for learning a new skill based on its market price.
    ///
    /// The cost to learn a skill is calculated as: skill_price * learning_cost_multiplier.
    /// For example, a value of 3.0 means learning costs 3x the current market price.
    /// This represents time, effort, and resources needed for education.
    /// Only used when enable_education is true.
    /// Default: 3.0 (learning costs 3x market price)
    #[serde(default = "default_learning_cost_multiplier")]
    pub learning_cost_multiplier: f64,

    /// Probability per step that a person will attempt to learn a new skill (0.0-1.0).
    ///
    /// Each step, persons have this probability of trying to learn a skill they don't have.
    /// Higher values lead to faster skill accumulation across the population.
    /// Only used when enable_education is true.
    /// Default: 0.1 (10% chance per step)
    #[serde(default = "default_learning_probability")]
    pub learning_probability: f64,

    /// Enable crisis events that create economic shocks during the simulation.
    ///
    /// When enabled, random crisis events (market crashes, demand shocks, supply shocks,
    /// currency devaluations) can occur, testing the resilience of the simulated economy.
    /// Set to false to disable crisis events (default).
    #[serde(default)]
    pub enable_crisis_events: bool,

    /// Probability per step that a crisis event will occur (0.0-1.0).
    ///
    /// Each simulation step has this probability of triggering a random crisis event.
    /// Lower values create rare but impactful crises, higher values create more frequent disruptions.
    /// Only used when enable_crisis_events is true.
    /// Default: 0.02 (2% chance per step, roughly one crisis every 50 steps)
    #[serde(default = "default_crisis_probability")]
    pub crisis_probability: f64,

    /// Crisis severity level (0.0-1.0).
    ///
    /// Controls how severe crisis effects are when they occur.
    /// 0.0 = minimal impact, 1.0 = maximum impact.
    /// For example, with severity 0.5, a market crash might reduce prices by ~30%,
    /// while severity 1.0 would cause a ~40% reduction.
    /// Only used when enable_crisis_events is true.
    /// Default: 0.5 (moderate severity)
    #[serde(default = "default_crisis_severity")]
    pub crisis_severity: f64,

    /// Enable friendship system where persons can form social bonds.
    ///
    /// When enabled, persons who successfully trade together have a chance to become friends.
    /// Friends receive price discounts when trading with each other, simulating trust and
    /// social capital in economic transactions.
    /// Set to false to disable friendships (default).
    #[serde(default)]
    pub enable_friendships: bool,

    /// Probability that a successful trade leads to friendship formation (0.0-1.0).
    ///
    /// After each successful trade between two persons, they have this probability of
    /// becoming friends (if they aren't already). Higher values lead to faster friend
    /// network formation.
    /// Only used when enable_friendships is true.
    /// Default: 0.1 (10% chance per successful trade)
    #[serde(default = "default_friendship_probability")]
    pub friendship_probability: f64,

    /// Price discount for trades between friends as a percentage (0.0-1.0).
    ///
    /// When two friends trade, the price is reduced by this percentage.
    /// For example, 0.1 means a 10% discount for friend-to-friend trades.
    /// This simulates trust, loyalty, and social capital reducing transaction costs.
    /// Only used when enable_friendships is true.
    /// Default: 0.1 (10% discount for friends)
    #[serde(default = "default_friendship_discount")]
    pub friendship_discount: f64,

    /// Number of groups/organizations to create in the simulation.
    ///
    /// When set, persons are assigned to groups in a round-robin fashion at initialization.
    /// Groups enable analysis of collective behavior and group-based economic dynamics.
    /// Set to None to disable group assignment (default).
    /// Valid range: 1 to entity_count
    #[serde(default)]
    pub num_groups: Option<usize>,

    /// Distance cost multiplier for geographic trade costs (0.0 = disabled).
    ///
    /// Controls the impact of geographic distance on trade costs.
    /// When set to a positive value, trade costs increase based on the Euclidean distance
    /// between buyer and seller locations: final_cost = base_cost * (1 + distance * distance_cost_factor)
    /// For example, with distance_cost_factor = 0.01 and distance = 50 units,
    /// the cost increases by 50% (1 + 50 * 0.01 = 1.5).
    /// Set to 0.0 to disable distance-based costs (default).
    /// Valid range: 0.0 to 1.0 (0% to 100% per distance unit)
    #[serde(default)]
    pub distance_cost_factor: f64,

    /// Price elasticity factor controlling sensitivity to supply/demand imbalances.
    ///
    /// This factor determines how dramatically prices change when supply doesn't match demand.
    /// Higher values mean prices are more responsive to market forces, potentially leading to
    /// more volatile markets. Lower values create more price stability but slower market adjustment.
    ///
    /// Typical range: 0.05-0.2
    /// - 0.05: Very inelastic, stable prices (like utilities, healthcare)
    /// - 0.1: Moderate elasticity (default, balanced markets)
    /// - 0.2: High elasticity, volatile prices (like fashion, tech)
    ///
    /// Default: 0.1 (10% price adjustment per unit supply/demand imbalance)
    /// Valid range: 0.0-1.0
    #[serde(default = "default_price_elasticity_factor")]
    pub price_elasticity_factor: f64,

    /// Volatility percentage for random price fluctuations.
    ///
    /// Adds random noise to prices each simulation step to model unpredictable market forces,
    /// news events, sentiment changes, and other real-world uncertainties. The value represents
    /// the range of random variation as a percentage of the current price.
    ///
    /// For example, 0.02 means prices can randomly vary by ±2% each step.
    ///
    /// Typical range: 0.0-0.1
    /// - 0.0: No volatility, deterministic price evolution
    /// - 0.02: Low volatility (default, stable markets)
    /// - 0.05: Moderate volatility (normal commodities)
    /// - 0.1: High volatility (cryptocurrency, speculative assets)
    ///
    /// Default: 0.02 (±2% random variation)
    /// Valid range: 0.0-0.5
    #[serde(default = "default_volatility_percentage")]
    pub volatility_percentage: f64,

    /// Enable event tracking during simulation.
    ///
    /// When enabled, the simulation collects events for key occurrences:
    /// - Trade executions (buyer, seller, skill, price)
    /// - Price updates (skill, old price, new price)
    /// - Reputation changes (person, old reputation, new reputation)
    /// - Step completions (step number, trades count, volume)
    ///
    /// Events are collected in memory and can be accessed in the simulation results
    /// for detailed analysis, debugging, or exporting timelines.
    ///
    /// **Performance**: When disabled (default), event emission is a no-op with zero overhead.
    /// When enabled, there's a small memory cost proportional to the number of events.
    ///
    /// Set to false to disable event tracking (default).
    #[serde(default)]
    pub enable_events: bool,

    /// Enable production system where persons can combine skills to create new skills.
    ///
    /// When enabled, persons can use recipes to combine two skills they possess into
    /// a new, more valuable skill. This simulates supply chains, skill composition,
    /// and economic specialization.
    ///
    /// Production requires:
    /// - Person must have both input skills required by a recipe
    /// - Person must have enough money to cover production costs
    /// - Production costs are based on input skill prices and recipe multipliers
    ///
    /// Set to false to disable production system (default).
    #[serde(default)]
    pub enable_production: bool,

    /// Probability per step that a person will attempt production (0.0-1.0).
    ///
    /// Each simulation step, persons have this probability of attempting to produce
    /// a new skill if they have the required inputs and money.
    /// Higher values lead to more active production and faster skill evolution.
    /// Only used when enable_production is true.
    /// Default: 0.05 (5% chance per step)
    #[serde(default = "default_production_probability")]
    pub production_probability: f64,

    /// Enable environmental resource tracking and sustainability metrics.
    ///
    /// When enabled, the simulation tracks resource consumption (Energy, Water, Materials, Land)
    /// for each transaction and calculates sustainability scores. Resources have finite reserves,
    /// and overconsumption can be detected and analyzed.
    ///
    /// This enables modeling ecological economics and studying the environmental impact
    /// of different economic behaviors and policies.
    ///
    /// Set to false to disable environmental tracking (default).
    #[serde(default)]
    pub enable_environment: bool,

    /// Resource cost per transaction as a multiplier of the transaction value.
    ///
    /// Each transaction consumes resources proportional to its value.
    /// A value of 1.0 means a $10 transaction consumes 10 units of resources.
    /// Resources are distributed evenly across all resource types (Energy, Water, Materials, Land).
    ///
    /// Only used when enable_environment is true.
    /// Default: 1.0 (resource consumption matches transaction value)
    /// Valid range: 0.0 to 10.0
    #[serde(default = "default_resource_cost_per_transaction")]
    pub resource_cost_per_transaction: f64,

    /// Initial resource reserves for environmental tracking.
    ///
    /// When enable_environment is true and this is None, default reserves are used:
    /// - Energy: 100,000 units
    /// - Water: 100,000 units
    /// - Materials: 100,000 units
    /// - Land: 10,000 units
    ///
    /// Custom reserves can be specified as a map from resource name to amount.
    /// Only used when enable_environment is true.
    #[serde(default)]
    pub custom_resource_reserves: Option<std::collections::HashMap<String, f64>>,

    /// Enable voting system for governance and collective decision-making.
    ///
    /// When enabled, persons can create proposals and vote on them using the specified voting method.
    /// Proposals can affect simulation parameters like tax rates, base prices, or transaction fees.
    /// Voting enables studying democratic governance mechanisms and their effects on economic outcomes.
    ///
    /// Set to false to disable voting system (default).
    #[serde(default)]
    pub enable_voting: bool,

    /// Voting method to use for all proposals.
    ///
    /// Determines how voting power is calculated for each person:
    /// - SimpleMajority: One person, one vote (pure democracy)
    /// - WeightedByWealth: Voting power proportional to wealth (plutocracy)
    /// - QuadraticVoting: Square root of wealth for balanced influence
    ///
    /// Only used when enable_voting is true.
    /// Default: SimpleMajority
    #[serde(default)]
    pub voting_method: crate::voting::VotingMethod,

    /// Default proposal duration in simulation steps.
    ///
    /// New proposals expire after this many steps, at which point they are tallied.
    /// This represents the voting period for each proposal.
    /// Only used when enable_voting is true.
    /// Default: 20 steps (voting period)
    #[serde(default = "default_proposal_duration")]
    pub proposal_duration: usize,

    /// Probability per step that a random proposal will be created (0.0-1.0).
    ///
    /// Each simulation step, there is this probability of creating a new random proposal
    /// (tax rate change, price change, fee change, or generic) for persons to vote on.
    /// Higher values lead to more frequent voting activity.
    /// Only used when enable_voting is true.
    /// Default: 0.05 (5% chance per step, approximately one proposal every 20 steps)
    #[serde(default = "default_proposal_probability")]
    pub proposal_probability: f64,

    /// Probability per step that each person will vote on active proposals (0.0-1.0).
    ///
    /// Each simulation step, each person has this probability of casting a vote on
    /// one of the currently active proposals. This represents voter participation/turnout.
    /// Higher values lead to higher voter participation and faster decision-making.
    /// Only used when enable_voting is true.
    /// Default: 0.3 (30% chance per person per step)
    #[serde(default = "default_voting_participation_rate")]
    pub voting_participation_rate: f64,
}

fn default_production_probability() -> f64 {
    0.05 // 5% chance per step to attempt production
}

fn default_resource_cost_per_transaction() -> f64 {
    1.0 // Resource consumption matches transaction value
}

fn default_seasonal_period() -> usize {
    100
}

fn default_min_skill_price() -> f64 {
    1.0 // Minimum price floor to prevent market crashes
}

fn default_loan_interest_rate() -> f64 {
    0.01 // 1% per step
}

fn default_loan_repayment_period() -> usize {
    20 // 20 steps to repay
}

fn default_min_money_to_lend() -> f64 {
    50.0 // Must have at least 50 money to lend
}

fn default_skills_per_person() -> usize {
    1 // Each person specializes in one skill by default
}

fn default_priority_urgency_weight() -> f64 {
    0.5 // Balanced consideration of urgency
}

fn default_priority_affordability_weight() -> f64 {
    0.3 // Moderate consideration of affordability
}

fn default_priority_efficiency_weight() -> f64 {
    0.1 // Minor consideration of efficiency
}

fn default_priority_reputation_weight() -> f64 {
    0.1 // Minor consideration of reputation
}

fn default_black_market_price_multiplier() -> f64 {
    0.8 // Black market is 20% cheaper
}

fn default_black_market_participation_rate() -> f64 {
    0.2 // 20% of trades use black market
}

fn default_price_elasticity_factor() -> f64 {
    0.1 // 10% price adjustment per unit supply/demand imbalance
}

fn default_volatility_percentage() -> f64 {
    0.02 // ±2% random price variation per step
}

fn default_max_contract_duration() -> usize {
    50 // Maximum 50 steps per contract
}

fn default_min_contract_duration() -> usize {
    10 // Minimum 10 steps per contract
}

fn default_contract_price_discount() -> f64 {
    0.05 // 5% discount for contract stability
}

fn default_learning_cost_multiplier() -> f64 {
    3.0 // Learning costs 3x the market price
}

fn default_learning_probability() -> f64 {
    0.1 // 10% chance per step to attempt learning
}

fn default_crisis_probability() -> f64 {
    0.02 // 2% chance per step (approximately once every 50 steps)
}

fn default_crisis_severity() -> f64 {
    0.5 // Moderate severity (50% of maximum impact)
}

fn default_friendship_probability() -> f64 {
    0.1 // 10% chance per successful trade
}

fn default_friendship_discount() -> f64 {
    0.1 // 10% discount for friend trades
}

fn default_proposal_duration() -> usize {
    20 // 20 steps voting period
}

fn default_proposal_probability() -> f64 {
    0.05 // 5% chance per step to create a proposal
}

fn default_voting_participation_rate() -> f64 {
    0.3 // 30% chance per person per step to vote
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_steps: 500,    // Default to 500 steps for market convergence
            entity_count: 100, // 100 persons
            seed: 42,
            initial_money_per_person: 100.0, // 100 Euros
            base_skill_price: 10.0,          // 10 Euros base price for skills
            min_skill_price: 1.0,            // Minimum price floor
            time_step: 1.0,                  // Represents one discrete step or turn
            scenario: Scenario::Original,
            demand_strategy: DemandStrategy::default(),
            tech_growth_rate: 0.0,   // Disabled by default
            seasonal_amplitude: 0.0, // Disabled by default
            seasonal_period: 100,    // Default cycle length
            transaction_fee: 0.0,    // Disabled by default
            savings_rate: 0.0,       // Disabled by default
            enable_loans: false,     // Disabled by default
            loan_interest_rate: 0.01,
            loan_repayment_period: 20,
            min_money_to_lend: 50.0,
            checkpoint_interval: 0,               // Disabled by default
            checkpoint_file: None,                // No default checkpoint file
            resume_from_checkpoint: false,        // Don't resume by default
            tax_rate: 0.0,                        // Disabled by default
            enable_tax_redistribution: false,     // Disabled by default
            skills_per_person: 1,                 // One skill per person by default
            stream_output_path: None,             // Disabled by default
            priority_urgency_weight: 0.5,         // Balanced urgency consideration
            priority_affordability_weight: 0.3,   // Moderate affordability consideration
            priority_efficiency_weight: 0.1,      // Minor efficiency consideration
            priority_reputation_weight: 0.1,      // Minor reputation consideration
            enable_black_market: false,           // Disabled by default
            black_market_price_multiplier: 0.8,   // 20% cheaper
            black_market_participation_rate: 0.2, // 20% of trades
            enable_contracts: false,              // Disabled by default
            max_contract_duration: 50,            // Maximum 50 steps
            min_contract_duration: 10,            // Minimum 10 steps
            contract_price_discount: 0.05,        // 5% discount
            enable_education: false,              // Disabled by default
            learning_cost_multiplier: 3.0,        // Learning costs 3x market price
            learning_probability: 0.1,            // 10% chance per step
            enable_crisis_events: false,          // Disabled by default
            crisis_probability: 0.02,             // 2% chance per step
            crisis_severity: 0.5,                 // Moderate severity
            enable_friendships: false,            // Disabled by default
            friendship_probability: 0.1,          // 10% chance per trade
            friendship_discount: 0.1,             // 10% discount for friends
            num_groups: None,                     // No groups by default
            distance_cost_factor: 0.0,            // Disabled by default
            price_elasticity_factor: 0.1,         // 10% price adjustment per unit imbalance
            volatility_percentage: 0.02,          // ±2% random price variation
            enable_events: false,                 // Disabled by default
            enable_production: false,             // Disabled by default
            production_probability: 0.05,         // 5% chance per step
            enable_environment: false,            // Disabled by default
            resource_cost_per_transaction: 1.0,   // Resource consumption matches transaction value
            custom_resource_reserves: None,       // Use default reserves
            enable_voting: false,                 // Disabled by default
            voting_method: crate::voting::VotingMethod::SimpleMajority, // One person, one vote
            proposal_duration: 20,                // 20 steps voting period
            proposal_probability: 0.05,           // 5% chance per step to create proposal
            voting_participation_rate: 0.3,       // 30% chance per person per step to vote
        }
    }
}

impl SimulationConfig {
    /// Validates the configuration parameters to ensure they are within acceptable ranges.
    ///
    /// # Returns
    /// * `Ok(())` if all parameters are valid
    /// * `Err(SimulationError::ValidationError)` with a descriptive error message if validation fails
    ///
    /// # Validation Rules
    /// - `max_steps` must be greater than 0
    /// - `entity_count` must be greater than 0
    /// - `initial_money_per_person` must be non-negative
    /// - `base_skill_price` must be greater than 0
    /// - `time_step` must be greater than 0
    /// - `tech_growth_rate` must be non-negative
    /// - `seasonal_amplitude` must be between 0.0 and 1.0 (inclusive)
    /// - `seasonal_period` must be greater than 0
    ///
    /// # Examples
    /// ```
    /// use simulation_framework::SimulationConfig;
    ///
    /// let mut config = SimulationConfig::default();
    /// assert!(config.validate().is_ok());
    ///
    /// config.max_steps = 0;
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.max_steps == 0 {
            return Err(SimulationError::ValidationError(
                "max_steps must be greater than 0".to_string(),
            ));
        }

        if self.entity_count == 0 {
            return Err(SimulationError::ValidationError(
                "entity_count (number of persons) must be greater than 0".to_string(),
            ));
        }

        if self.initial_money_per_person.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "initial_money_per_person must be non-negative, got: {}",
                self.initial_money_per_person
            )));
        }

        if self.base_skill_price <= 0.0 {
            return Err(SimulationError::ValidationError(format!(
                "base_skill_price must be greater than 0, got: {}",
                self.base_skill_price
            )));
        }

        if self.min_skill_price <= 0.0 {
            return Err(SimulationError::ValidationError(format!(
                "min_skill_price must be greater than 0, got: {}",
                self.min_skill_price
            )));
        }

        if self.min_skill_price > self.base_skill_price {
            return Err(SimulationError::ValidationError(format!(
                "min_skill_price ({}) cannot exceed base_skill_price ({})",
                self.min_skill_price, self.base_skill_price
            )));
        }

        if self.time_step <= 0.0 {
            return Err(SimulationError::ValidationError(format!(
                "time_step must be greater than 0, got: {}",
                self.time_step
            )));
        }

        if self.tech_growth_rate.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "tech_growth_rate must be non-negative, got: {}",
                self.tech_growth_rate
            )));
        }

        if !(0.0..=1.0).contains(&self.seasonal_amplitude) {
            return Err(SimulationError::ValidationError(format!(
                "seasonal_amplitude must be between 0.0 and 1.0, got: {}",
                self.seasonal_amplitude
            )));
        }

        if self.seasonal_period == 0 {
            return Err(SimulationError::ValidationError(
                "seasonal_period must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.transaction_fee) {
            return Err(SimulationError::ValidationError(format!(
                "transaction_fee must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.transaction_fee
            )));
        }

        if !(0.0..=1.0).contains(&self.savings_rate) {
            return Err(SimulationError::ValidationError(format!(
                "savings_rate must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.savings_rate
            )));
        }

        if !(0.0..=1.0).contains(&self.loan_interest_rate) {
            return Err(SimulationError::ValidationError(format!(
                "loan_interest_rate must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.loan_interest_rate
            )));
        }

        if self.loan_repayment_period == 0 {
            return Err(SimulationError::ValidationError(
                "loan_repayment_period must be greater than 0".to_string(),
            ));
        }

        if self.min_money_to_lend.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "min_money_to_lend must be non-negative, got: {}",
                self.min_money_to_lend
            )));
        }

        // Additional sanity checks for extreme values
        if self.max_steps > 1_000_000 {
            return Err(SimulationError::ValidationError(format!(
                "max_steps is too large ({}), maximum recommended value is 1,000,000",
                self.max_steps
            )));
        }

        if self.entity_count > 100_000 {
            return Err(SimulationError::ValidationError(format!(
                "entity_count is too large ({}), maximum recommended value is 100,000",
                self.entity_count
            )));
        }

        if self.tech_growth_rate > 1.0 {
            return Err(SimulationError::ValidationError(format!(
                "tech_growth_rate is too large ({}), values above 1.0 (100% per step) are unrealistic",
                self.tech_growth_rate
            )));
        }

        if !(0.0..=1.0).contains(&self.tax_rate) {
            return Err(SimulationError::ValidationError(format!(
                "tax_rate must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.tax_rate
            )));
        }

        if self.black_market_price_multiplier < 0.0 || self.black_market_price_multiplier > 2.0 {
            return Err(SimulationError::ValidationError(format!(
                "black_market_price_multiplier must be between 0.0 (exclusive) and 2.0, got: {}",
                self.black_market_price_multiplier
            )));
        }

        if !(0.0..=1.0).contains(&self.black_market_participation_rate) {
            return Err(SimulationError::ValidationError(format!(
                "black_market_participation_rate must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.black_market_participation_rate
            )));
        }

        if self.skills_per_person == 0 {
            return Err(SimulationError::ValidationError(
                "skills_per_person must be at least 1".to_string(),
            ));
        }

        if self.skills_per_person > self.entity_count {
            return Err(SimulationError::ValidationError(format!(
                "skills_per_person ({}) cannot exceed entity_count ({})",
                self.skills_per_person, self.entity_count
            )));
        }

        if !(0.0..=1.0).contains(&self.priority_urgency_weight) {
            return Err(SimulationError::ValidationError(format!(
                "priority_urgency_weight must be between 0.0 and 1.0, got: {}",
                self.priority_urgency_weight
            )));
        }

        if !(0.0..=1.0).contains(&self.priority_affordability_weight) {
            return Err(SimulationError::ValidationError(format!(
                "priority_affordability_weight must be between 0.0 and 1.0, got: {}",
                self.priority_affordability_weight
            )));
        }

        if !(0.0..=1.0).contains(&self.priority_efficiency_weight) {
            return Err(SimulationError::ValidationError(format!(
                "priority_efficiency_weight must be between 0.0 and 1.0, got: {}",
                self.priority_efficiency_weight
            )));
        }

        if !(0.0..=1.0).contains(&self.priority_reputation_weight) {
            return Err(SimulationError::ValidationError(format!(
                "priority_reputation_weight must be between 0.0 and 1.0, got: {}",
                self.priority_reputation_weight
            )));
        }

        if self.enable_contracts {
            if self.min_contract_duration == 0 {
                return Err(SimulationError::ValidationError(
                    "min_contract_duration must be greater than 0 when contracts are enabled"
                        .to_string(),
                ));
            }

            if self.max_contract_duration == 0 {
                return Err(SimulationError::ValidationError(
                    "max_contract_duration must be greater than 0 when contracts are enabled"
                        .to_string(),
                ));
            }

            if self.min_contract_duration > self.max_contract_duration {
                return Err(SimulationError::ValidationError(format!(
                    "min_contract_duration ({}) cannot exceed max_contract_duration ({})",
                    self.min_contract_duration, self.max_contract_duration
                )));
            }

            if !(0.0..=1.0).contains(&self.contract_price_discount) {
                return Err(SimulationError::ValidationError(format!(
                    "contract_price_discount must be between 0.0 and 1.0 (0% to 100%), got: {}",
                    self.contract_price_discount
                )));
            }
        }

        if self.enable_education {
            if self.learning_cost_multiplier < 0.0 {
                return Err(SimulationError::ValidationError(format!(
                    "learning_cost_multiplier must be non-negative, got: {}",
                    self.learning_cost_multiplier
                )));
            }

            if !(0.0..=1.0).contains(&self.learning_probability) {
                return Err(SimulationError::ValidationError(format!(
                    "learning_probability must be between 0.0 and 1.0 (0% to 100%), got: {}",
                    self.learning_probability
                )));
            }
        }

        if self.enable_crisis_events {
            if !(0.0..=1.0).contains(&self.crisis_probability) {
                return Err(SimulationError::ValidationError(format!(
                    "crisis_probability must be between 0.0 and 1.0 (0% to 100%), got: {}",
                    self.crisis_probability
                )));
            }

            if !(0.0..=1.0).contains(&self.crisis_severity) {
                return Err(SimulationError::ValidationError(format!(
                    "crisis_severity must be between 0.0 and 1.0, got: {}",
                    self.crisis_severity
                )));
            }
        }

        // Friendship system validation
        if self.enable_friendships {
            if !(0.0..=1.0).contains(&self.friendship_probability) {
                return Err(SimulationError::ValidationError(format!(
                    "friendship_probability must be between 0.0 and 1.0, got: {}",
                    self.friendship_probability
                )));
            }

            if !(0.0..=1.0).contains(&self.friendship_discount) {
                return Err(SimulationError::ValidationError(format!(
                    "friendship_discount must be between 0.0 and 1.0, got: {}",
                    self.friendship_discount
                )));
            }
        }

        // Group system validation
        if let Some(num_groups) = self.num_groups {
            if num_groups == 0 {
                return Err(SimulationError::ValidationError(
                    "num_groups must be at least 1 when specified".to_string(),
                ));
            }

            if num_groups > self.entity_count {
                return Err(SimulationError::ValidationError(format!(
                    "num_groups ({}) cannot exceed entity_count ({})",
                    num_groups, self.entity_count
                )));
            }
        }

        // Distance cost factor validation
        if self.distance_cost_factor.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "distance_cost_factor must be non-negative, got: {}",
                self.distance_cost_factor
            )));
        }

        if self.distance_cost_factor >= 1.0 {
            return Err(SimulationError::ValidationError(format!(
                "distance_cost_factor must be less than 1.0 ({}), values >= 1.0 (100%+ cost increase per distance unit) are unrealistic. Recommended range: 0.0-0.1",
                self.distance_cost_factor
            )));
        }

        // Market dynamics parameter validation
        if self.price_elasticity_factor.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "price_elasticity_factor must be non-negative, got: {}",
                self.price_elasticity_factor
            )));
        }

        if self.price_elasticity_factor > 1.0 {
            return Err(SimulationError::ValidationError(format!(
                "price_elasticity_factor should not exceed 1.0 (100% adjustment), got: {}",
                self.price_elasticity_factor
            )));
        }

        if self.volatility_percentage.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "volatility_percentage must be non-negative, got: {}",
                self.volatility_percentage
            )));
        }

        if self.volatility_percentage > 0.5 {
            return Err(SimulationError::ValidationError(format!(
                "volatility_percentage should not exceed 0.5 (50% variation), got: {}",
                self.volatility_percentage
            )));
        }

        // Production system validation
        // Validate production_probability range unconditionally to prevent configuration issues
        // when toggling production on/off
        if !(0.0..=1.0).contains(&self.production_probability) {
            return Err(SimulationError::ValidationError(format!(
                "production_probability must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.production_probability
            )));
        }

        // Environment system validation
        // Validate resource_cost_per_transaction range unconditionally
        if !(0.0..=10.0).contains(&self.resource_cost_per_transaction) {
            return Err(SimulationError::ValidationError(format!(
                "resource_cost_per_transaction must be between 0.0 and 10.0, got: {}",
                self.resource_cost_per_transaction
            )));
        }

        // Validate custom resource reserves if provided
        if let Some(ref reserves) = self.custom_resource_reserves {
            for (resource_name, &amount) in reserves {
                if amount < 0.0 {
                    return Err(SimulationError::ValidationError(format!(
                        "custom_resource_reserves for '{}' must be non-negative, got: {}",
                        resource_name, amount
                    )));
                }
            }
        }

        Ok(())
    }

    /// Create a configuration from a preset.
    ///
    /// # Arguments
    /// * `preset` - The preset name to use
    ///
    /// # Returns
    /// * `SimulationConfig` - A configuration with preset values
    ///
    /// # Examples
    /// ```
    /// use simulation_framework::{SimulationConfig, PresetName};
    ///
    /// let config = SimulationConfig::from_preset(PresetName::SmallEconomy);
    /// assert_eq!(config.entity_count, 20);
    /// ```
    pub fn from_preset(preset: PresetName) -> Self {
        match preset {
            PresetName::Default => Self::default(),
            PresetName::SmallEconomy => Self {
                max_steps: 100,
                entity_count: 20,
                ..Self::default()
            },
            PresetName::LargeEconomy => Self {
                max_steps: 2000,
                entity_count: 500,
                initial_money_per_person: 200.0,
                ..Self::default()
            },
            PresetName::CrisisScenario => Self {
                max_steps: 1000,
                entity_count: 100,
                initial_money_per_person: 50.0,
                base_skill_price: 25.0,
                min_skill_price: 2.0,       // Higher floor for crisis scenario
                enable_crisis_events: true, // Enable crisis events for crisis scenario!
                crisis_probability: 0.05,   // Higher probability (5% per step)
                crisis_severity: 0.7,       // Higher severity for crisis scenario
                price_elasticity_factor: 0.15, // Higher volatility for crisis scenario
                volatility_percentage: 0.05, // More chaotic market
                ..Self::default()
            },
            PresetName::HighInflation => Self {
                max_steps: 1000,
                entity_count: 100,
                base_skill_price: 15.0,
                scenario: Scenario::DynamicPricing,
                price_elasticity_factor: 0.15, // More responsive for inflation
                volatility_percentage: 0.04,   // Higher volatility for inflation
                ..Self::default()
            },
            PresetName::TechGrowth => Self {
                max_steps: 1500,
                entity_count: 150,
                initial_money_per_person: 250.0,
                base_skill_price: 8.0,
                min_skill_price: 0.5,    // Lower floor for tech growth scenario
                tech_growth_rate: 0.001, // 0.1% growth per step - significant over 1500 steps
                price_elasticity_factor: 0.08, // Lower elasticity for stable tech growth
                volatility_percentage: 0.01, // Lower volatility for stable growth
                ..Self::default()
            },
            PresetName::QuickTest => Self {
                max_steps: 50,
                entity_count: 10,
                ..Self::default()
            },
        }
    }

    /// Load configuration from a YAML or TOML file.
    /// File format is auto-detected based on file extension.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file (.yaml, .yml, or .toml)
    ///
    /// # Returns
    /// * `Result<SimulationConfig>` - The loaded config or a SimulationError
    ///
    /// # Examples
    /// ```no_run
    /// use simulation_framework::SimulationConfig;
    ///
    /// let config = SimulationConfig::from_file("config.yaml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(SimulationError::ConfigFileRead)?;

        // Detect format based on file extension
        let extension = path.extension().and_then(|s| s.to_str()).ok_or_else(|| {
            SimulationError::UnsupportedConfigFormat("(no extension)".to_string())
        })?;

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                let config: SimulationConfig = serde_yaml::from_str(&contents)
                    .map_err(|e| SimulationError::YamlParse(e.to_string()))?;
                Ok(config)
            }
            "toml" => {
                let config: SimulationConfig = toml::from_str(&contents)
                    .map_err(|e| SimulationError::TomlParse(e.to_string()))?;
                Ok(config)
            }
            _ => Err(SimulationError::UnsupportedConfigFormat(
                extension.to_string(),
            )),
        }
    }

    /// Merge configuration from a file with CLI overrides.
    /// Values from CLI (if present) take precedence over file values.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    /// * `cli_overrides` - Function that applies CLI overrides to the config
    ///
    /// # Returns
    /// * `Result<SimulationConfig>` - The merged config or a SimulationError
    pub fn from_file_with_overrides<P: AsRef<Path>, F>(path: P, cli_overrides: F) -> Result<Self>
    where
        F: FnOnce(&mut SimulationConfig),
    {
        let mut config = Self::from_file(path)?;
        cli_overrides(&mut config);
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn test_load_yaml_config() {
        let yaml_content = r#"
max_steps: 1000
entity_count: 50
seed: 123
initial_money_per_person: 200.0
base_skill_price: 15.0
time_step: 1.0
scenario: Original
"#;
        let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.max_steps, 1000);
        assert_eq!(config.entity_count, 50);
        assert_eq!(config.seed, 123);
        assert_eq!(config.initial_money_per_person, 200.0);
        assert_eq!(config.base_skill_price, 15.0);
    }

    #[test]
    fn test_load_toml_config() {
        let toml_content = r#"
max_steps = 2000
entity_count = 75
seed = 456
initial_money_per_person = 300.0
base_skill_price = 20.0
time_step = 1.0
scenario = "DynamicPricing"
"#;
        let mut temp_file = Builder::new().suffix(".toml").tempfile().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.max_steps, 2000);
        assert_eq!(config.entity_count, 75);
        assert_eq!(config.seed, 456);
        assert_eq!(config.initial_money_per_person, 300.0);
        assert_eq!(config.base_skill_price, 20.0);
    }

    #[test]
    fn test_invalid_file_extension() {
        let mut temp_file = Builder::new().suffix(".txt").tempfile().unwrap();
        temp_file.write_all(b"invalid content").unwrap();
        temp_file.flush().unwrap();

        let result = SimulationConfig::from_file(temp_file.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Unsupported configuration file format"));
    }

    #[test]
    fn test_missing_file() {
        let result = SimulationConfig::from_file("/nonexistent/config.yaml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Failed to read configuration file"));
    }

    #[test]
    fn test_config_with_overrides() {
        let yaml_content = r#"
max_steps: 1000
entity_count: 50
seed: 123
initial_money_per_person: 200.0
base_skill_price: 15.0
time_step: 1.0
scenario: Original
"#;
        let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file_with_overrides(temp_file.path(), |cfg| {
            cfg.max_steps = 5000; // CLI override
            cfg.seed = 999; // CLI override
        })
        .unwrap();

        assert_eq!(config.max_steps, 5000); // Overridden
        assert_eq!(config.entity_count, 50); // From file
        assert_eq!(config.seed, 999); // Overridden
        assert_eq!(config.initial_money_per_person, 200.0); // From file
    }

    #[test]
    fn test_preset_default() {
        let config = SimulationConfig::from_preset(PresetName::Default);
        let default_config = SimulationConfig::default();

        assert_eq!(config.max_steps, default_config.max_steps);
        assert_eq!(config.entity_count, default_config.entity_count);
        assert_eq!(
            config.initial_money_per_person,
            default_config.initial_money_per_person
        );
    }

    #[test]
    fn test_preset_small_economy() {
        let config = SimulationConfig::from_preset(PresetName::SmallEconomy);

        assert_eq!(config.entity_count, 20);
        assert_eq!(config.max_steps, 100);
        assert_eq!(config.initial_money_per_person, 100.0);
    }

    #[test]
    fn test_preset_large_economy() {
        let config = SimulationConfig::from_preset(PresetName::LargeEconomy);

        assert_eq!(config.entity_count, 500);
        assert_eq!(config.max_steps, 2000);
        assert_eq!(config.initial_money_per_person, 200.0);
    }

    #[test]
    fn test_preset_crisis_scenario() {
        let config = SimulationConfig::from_preset(PresetName::CrisisScenario);

        assert_eq!(config.entity_count, 100);
        assert_eq!(config.max_steps, 1000);
        assert_eq!(config.initial_money_per_person, 50.0);
        assert_eq!(config.base_skill_price, 25.0);
    }

    #[test]
    fn test_preset_high_inflation() {
        let config = SimulationConfig::from_preset(PresetName::HighInflation);

        assert_eq!(config.scenario, Scenario::DynamicPricing);
        assert_eq!(config.entity_count, 100);
        assert_eq!(config.base_skill_price, 15.0);
    }

    #[test]
    fn test_preset_quick_test() {
        let config = SimulationConfig::from_preset(PresetName::QuickTest);

        assert_eq!(config.entity_count, 10);
        assert_eq!(config.max_steps, 50);
    }

    #[test]
    fn test_preset_name_from_str() {
        assert_eq!(
            PresetName::from_str("default").unwrap(),
            PresetName::Default
        );
        assert_eq!(
            PresetName::from_str("small_economy").unwrap(),
            PresetName::SmallEconomy
        );
        assert_eq!(
            PresetName::from_str("small").unwrap(),
            PresetName::SmallEconomy
        );
        assert_eq!(
            PresetName::from_str("crisis").unwrap(),
            PresetName::CrisisScenario
        );
        assert!(PresetName::from_str("nonexistent").is_err());
    }

    #[test]
    fn test_preset_name_as_str() {
        assert_eq!(PresetName::Default.as_str(), "default");
        assert_eq!(PresetName::SmallEconomy.as_str(), "small_economy");
        assert_eq!(PresetName::QuickTest.as_str(), "quick_test");
    }

    #[test]
    fn test_all_presets_are_valid() {
        // Ensure all presets can be created without panicking
        for preset in PresetName::all() {
            let config = SimulationConfig::from_preset(preset.clone());
            assert!(config.entity_count > 0);
            assert!(config.max_steps > 0);
            assert!(config.initial_money_per_person > 0.0);
            assert!(config.base_skill_price > 0.0);
        }
    }

    #[test]
    fn test_validate_default_config() {
        let config = SimulationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_all_presets() {
        // Ensure all preset configurations pass validation
        for preset in PresetName::all() {
            let config = SimulationConfig::from_preset(preset.clone());
            assert!(
                config.validate().is_ok(),
                "Preset {:?} should pass validation",
                preset
            );
        }
    }

    #[test]
    fn test_validate_zero_max_steps() {
        let config = SimulationConfig {
            max_steps: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("max_steps must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_entity_count() {
        let config = SimulationConfig {
            entity_count: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("entity_count (number of persons) must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_initial_money() {
        let config = SimulationConfig {
            initial_money_per_person: -10.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("initial_money_per_person must be non-negative"));
    }

    #[test]
    fn test_validate_zero_base_skill_price() {
        let config = SimulationConfig {
            base_skill_price: 0.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("base_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_base_skill_price() {
        let config = SimulationConfig {
            base_skill_price: -5.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("base_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_time_step() {
        let config = SimulationConfig {
            time_step: 0.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("time_step must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_tech_growth_rate() {
        let config = SimulationConfig {
            tech_growth_rate: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("tech_growth_rate must be non-negative"));
    }

    #[test]
    fn test_validate_excessive_tech_growth_rate() {
        let config = SimulationConfig {
            tech_growth_rate: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("tech_growth_rate is too large"));
    }

    #[test]
    fn test_validate_seasonal_amplitude_out_of_range() {
        let config = SimulationConfig {
            seasonal_amplitude: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("seasonal_amplitude must be between 0.0 and 1.0"));

        let config2 = SimulationConfig {
            seasonal_amplitude: -0.1,
            ..Default::default()
        };
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_validate_zero_seasonal_period() {
        let config = SimulationConfig {
            seasonal_period: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("seasonal_period must be greater than 0"));
    }

    #[test]
    fn test_validate_extreme_max_steps() {
        let config = SimulationConfig {
            max_steps: 2_000_000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("max_steps is too large"));
    }

    #[test]
    fn test_validate_extreme_entity_count() {
        let config = SimulationConfig {
            entity_count: 200_000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("entity_count is too large"));
    }

    #[test]
    fn test_validate_valid_edge_cases() {
        // Test that boundary values are accepted

        // Max valid tech growth rate
        let config1 = SimulationConfig {
            tech_growth_rate: 1.0,
            ..Default::default()
        };
        assert!(config1.validate().is_ok());

        // Max valid seasonal amplitude
        let config2 = SimulationConfig {
            seasonal_amplitude: 1.0,
            ..Default::default()
        };
        assert!(config2.validate().is_ok());

        // Min valid seasonal amplitude
        let config3 = SimulationConfig {
            seasonal_amplitude: 0.0,
            ..Default::default()
        };
        assert!(config3.validate().is_ok());

        // Zero initial money (allowed - represents starting with no money)
        let config4 = SimulationConfig {
            initial_money_per_person: 0.0,
            ..Default::default()
        };
        assert!(config4.validate().is_ok());

        // Single person
        let config5 = SimulationConfig {
            entity_count: 1,
            ..Default::default()
        };
        assert!(config5.validate().is_ok());

        // Single step
        let config6 = SimulationConfig {
            max_steps: 1,
            ..Default::default()
        };
        assert!(config6.validate().is_ok());
    }

    #[test]
    fn test_validate_min_skill_price_zero() {
        let config = SimulationConfig {
            min_skill_price: 0.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("min_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_min_skill_price_negative() {
        let config = SimulationConfig {
            min_skill_price: -5.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("min_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_min_skill_price_exceeds_base() {
        let config = SimulationConfig {
            base_skill_price: 10.0,
            min_skill_price: 15.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("min_skill_price (15) cannot exceed base_skill_price (10)"));
    }

    #[test]
    fn test_validate_min_skill_price_equals_base() {
        let config = SimulationConfig {
            base_skill_price: 10.0,
            min_skill_price: 10.0,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_min_skill_price_valid() {
        let config = SimulationConfig {
            base_skill_price: 10.0,
            min_skill_price: 5.0,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_price_elasticity_negative() {
        let config = SimulationConfig {
            price_elasticity_factor: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("price_elasticity_factor must be non-negative"));
    }

    #[test]
    fn test_validate_price_elasticity_too_high() {
        let config = SimulationConfig {
            price_elasticity_factor: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("price_elasticity_factor should not exceed 1.0"));
    }

    #[test]
    fn test_validate_price_elasticity_valid_range() {
        // Test various valid values
        let test_values = vec![0.0, 0.05, 0.1, 0.2, 0.5, 1.0];
        for value in test_values {
            let config = SimulationConfig {
                price_elasticity_factor: value,
                ..Default::default()
            };
            assert!(
                config.validate().is_ok(),
                "Failed for elasticity value: {}",
                value
            );
        }
    }

    #[test]
    fn test_validate_volatility_negative() {
        let config = SimulationConfig {
            volatility_percentage: -0.01,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("volatility_percentage must be non-negative"));
    }

    #[test]
    fn test_validate_volatility_too_high() {
        let config = SimulationConfig {
            volatility_percentage: 0.6,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("volatility_percentage should not exceed 0.5"));
    }

    #[test]
    fn test_validate_volatility_valid_range() {
        // Test various valid values
        let test_values = vec![0.0, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5];
        for value in test_values {
            let config = SimulationConfig {
                volatility_percentage: value,
                ..Default::default()
            };
            assert!(
                config.validate().is_ok(),
                "Failed for volatility value: {}",
                value
            );
        }
    }

    #[test]
    fn test_market_dynamics_defaults() {
        let config = SimulationConfig::default();
        assert_eq!(config.price_elasticity_factor, 0.1);
        assert_eq!(config.volatility_percentage, 0.02);
    }

    #[test]
    fn test_market_dynamics_in_presets() {
        // Test that crisis scenario has higher volatility
        let crisis_config = SimulationConfig::from_preset(PresetName::CrisisScenario);
        assert_eq!(crisis_config.price_elasticity_factor, 0.15);
        assert_eq!(crisis_config.volatility_percentage, 0.05);

        // Test that tech growth has lower volatility
        let tech_config = SimulationConfig::from_preset(PresetName::TechGrowth);
        assert_eq!(tech_config.price_elasticity_factor, 0.08);
        assert_eq!(tech_config.volatility_percentage, 0.01);
    }
}
