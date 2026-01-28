use crate::error::{Result, SimulationError};
use crate::{Entity, SkillId}; // Entity now wraps Person
use colored::Colorize;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::Command;

/// Metadata about a simulation run for reproducibility and audit trail.
///
/// This structure captures key information about when and how a simulation was executed,
/// enabling scientific reproducibility, comparison of runs, and audit trails.
///
/// # Example
///
/// ```
/// use simulation_framework::result::SimulationMetadata;
///
/// let metadata = SimulationMetadata::capture(42, 100, 500);
///
/// assert_eq!(metadata.seed, 42);
/// assert_eq!(metadata.entity_count, 100);
/// assert_eq!(metadata.max_steps, 500);
/// assert!(!metadata.timestamp.is_empty());
/// assert!(!metadata.rust_version.is_empty());
/// assert!(!metadata.framework_version.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// ISO 8601 timestamp when the simulation was created
    pub timestamp: String,

    /// Git commit hash of the simulation framework (if available)
    ///
    /// This field will be `None` if:
    /// - The code is not in a git repository
    /// - Git is not installed
    /// - The git command fails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit_hash: Option<String>,

    /// Random seed used for the simulation (for reproducibility)
    pub seed: u64,

    /// Number of entities (persons) in the simulation
    pub entity_count: usize,

    /// Maximum number of simulation steps
    pub max_steps: usize,

    /// Rust compiler version used to build the simulation
    pub rust_version: String,

    /// Version of the simulation framework
    pub framework_version: String,
}

impl SimulationMetadata {
    /// Capture metadata for the current simulation run.
    ///
    /// # Arguments
    ///
    /// * `seed` - Random seed for reproducibility
    /// * `entity_count` - Number of entities in the simulation
    /// * `max_steps` - Maximum number of simulation steps
    ///
    /// # Returns
    ///
    /// A `SimulationMetadata` struct with captured metadata
    pub fn capture(seed: u64, entity_count: usize, max_steps: usize) -> Self {
        // Get current timestamp in ISO 8601 format
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Try to get git commit hash
        let git_commit_hash = Self::get_git_commit_hash();

        // Get Rust version from RUSTC environment variable or fallback
        let rust_version = Self::get_rust_version();

        // Get framework version from Cargo.toml
        let framework_version = env!("CARGO_PKG_VERSION").to_string();

        Self {
            timestamp,
            git_commit_hash,
            seed,
            entity_count,
            max_steps,
            rust_version,
            framework_version,
        }
    }

    /// Attempt to get the current git commit hash.
    ///
    /// Returns `None` if git is not available, the directory is not a git repository,
    /// or the command fails.
    fn get_git_commit_hash() -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Get the Rust version string.
    ///
    /// Returns rustc version string by calling rustc at runtime if available,
    /// or a fallback message if rustc is not accessible.
    fn get_rust_version() -> String {
        Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Rust (version unknown)".to_string())
    }
}

/// Incremental statistics calculator using Welford's online algorithm.
///
/// This structure maintains running statistics (mean and variance) that can be updated
/// in O(1) time per value, avoiding the need for O(n) recalculation each step.
/// The algorithm is numerically stable and avoids catastrophic cancellation.
///
/// # Performance
///
/// - Update: O(1) per value
/// - Get statistics: O(1)
/// - Memory: O(1) - only stores running totals
///
/// # Algorithm
///
/// Uses Welford's online algorithm for variance calculation:
/// ```text
/// M_k = M_{k-1} + (x_k - M_{k-1}) / k
/// S_k = S_{k-1} + (x_k - M_{k-1}) * (x_k - M_k)
/// variance = S_k / (k - 1)
/// ```
///
/// # Example
///
/// ```
/// use simulation_framework::result::IncrementalStats;
///
/// let mut stats = IncrementalStats::new();
/// stats.update(10.0);
/// stats.update(20.0);
/// stats.update(30.0);
///
/// assert_eq!(stats.mean(), 20.0);
/// assert!((stats.variance() - 100.0).abs() < 1e-10);
/// assert!((stats.std_dev() - 10.0).abs() < 1e-10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalStats {
    /// Count of values processed
    count: usize,
    /// Running mean (M_k in Welford's algorithm)
    mean: f64,
    /// Running sum of squared differences (S_k in Welford's algorithm)
    m2: f64,
}

impl IncrementalStats {
    /// Create a new incremental statistics calculator.
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    /// Update statistics with a new value using Welford's algorithm.
    ///
    /// This is an O(1) operation that maintains numerically stable running statistics.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value to incorporate into the statistics
    pub fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    /// Get the current count of values.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get the current mean (average) value.
    ///
    /// Returns 0.0 if no values have been added.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Get the current sample variance.
    ///
    /// Returns 0.0 if fewer than 2 values have been added.
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }

    /// Get the current standard deviation.
    ///
    /// Returns 0.0 if fewer than 2 values have been added.
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Reset all statistics to initial state.
    pub fn reset(&mut self) {
        self.count = 0;
        self.mean = 0.0;
        self.m2 = 0.0;
    }
}

impl Default for IncrementalStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Data for a single simulation step, used for streaming output
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepData {
    /// Current step number
    pub step: usize,
    /// Number of trades in this step
    pub trades: usize,
    /// Total money exchanged in this step
    pub volume: f64,
    /// Average money across all persons at this step
    pub avg_money: f64,
    /// Gini coefficient at this step
    pub gini_coefficient: f64,
    /// Average reputation at this step
    pub avg_reputation: f64,
    /// Skill prices at this step (top 5 by price)
    pub top_skill_prices: Vec<SkillPriceInfo>,
}

/// Snapshot of wealth distribution statistics at a single simulation step.
///
/// This structure captures complete wealth inequality metrics at a specific
/// point in time, enabling time-series analysis of how wealth distribution
/// evolves during the simulation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WealthStatsSnapshot {
    /// The simulation step number for this snapshot
    pub step: usize,
    /// Average money across all persons at this step
    pub average: f64,
    /// Median money value at this step
    pub median: f64,
    /// Standard deviation of money distribution at this step
    pub std_dev: f64,
    /// Minimum money value at this step
    pub min_money: f64,
    /// Maximum money value at this step
    pub max_money: f64,
    /// Gini coefficient (wealth inequality) at this step
    /// 0 = perfect equality, 1 = perfect inequality
    pub gini_coefficient: f64,
    /// Herfindahl-Hirschman Index (market concentration) at this step
    /// Values < 1,500 indicate competitive distribution, 1,500-2,500 moderate, > 2,500 high
    pub herfindahl_index: f64,
    /// Share of total wealth held by top 10% at this step (0.0-1.0)
    pub top_10_percent_share: f64,
    /// Share of total wealth held by top 1% at this step (0.0-1.0)
    pub top_1_percent_share: f64,
    /// Share of total wealth held by bottom 50% at this step (0.0-1.0)
    pub bottom_50_percent_share: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyStats {
    pub average: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min_money: f64,
    pub max_money: f64,
    /// Gini coefficient: measure of wealth inequality (0 = perfect equality, 1 = perfect inequality)
    pub gini_coefficient: f64,
    /// Herfindahl-Hirschman Index: measure of market concentration for wealth (0 = perfect competition, 10000 = monopoly)
    /// Values < 1500 indicate competitive distribution, 1500-2500 moderate concentration, > 2500 high concentration
    pub herfindahl_index: f64,
    /// Share of total wealth held by the top 10% wealthiest persons (0.0-1.0)
    /// Values > 0.5 indicate high wealth concentration at the top
    pub top_10_percent_share: f64,
    /// Share of total wealth held by the top 1% wealthiest persons (0.0-1.0)
    /// Values > 0.2 indicate extreme wealth concentration
    pub top_1_percent_share: f64,
    /// Share of total wealth held by the bottom 50% of persons (0.0-1.0)
    /// Values < 0.1 indicate high inequality with poverty concentration
    pub bottom_50_percent_share: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReputationStats {
    pub average: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min_reputation: f64,
    pub max_reputation: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavingsStats {
    pub total_savings: f64,
    pub average_savings: f64,
    pub median_savings: f64,
    pub min_savings: f64,
    pub max_savings: f64,
}

/// Statistics about credit scores across all persons.
///
/// Credit scores range from 300 to 850 (FICO-like scale), with higher scores
/// indicating better creditworthiness and resulting in lower loan interest rates.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditScoreStats {
    /// Average credit score across all persons
    pub average_score: f64,
    /// Median credit score
    pub median_score: f64,
    /// Standard deviation of credit scores
    pub std_dev_score: f64,
    /// Minimum credit score
    pub min_score: u16,
    /// Maximum credit score
    pub max_score: u16,
    /// Number of persons with excellent credit (800-850)
    pub excellent_count: usize,
    /// Number of persons with very good credit (740-799)
    pub very_good_count: usize,
    /// Number of persons with good credit (670-739)
    pub good_count: usize,
    /// Number of persons with fair credit (580-669)
    pub fair_count: usize,
    /// Number of persons with poor credit (300-579)
    pub poor_count: usize,
    /// Total successful loan payments made
    pub total_successful_payments: usize,
    /// Total missed loan payments
    pub total_missed_payments: usize,
}

/// Statistics about skill quality ratings across all persons and skills.
///
/// Quality ratings range from 0.0 (minimum) to 5.0 (maximum), with 3.0 as average.
/// Higher quality skills command higher prices, creating quality competition in the market.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QualityStats {
    /// Average quality across all skills
    pub average_quality: f64,
    /// Median quality value
    pub median_quality: f64,
    /// Standard deviation of quality distribution
    pub std_dev_quality: f64,
    /// Minimum quality rating
    pub min_quality: f64,
    /// Maximum quality rating
    pub max_quality: f64,
    /// Number of skills at maximum quality (5.0)
    pub skills_at_max_quality: usize,
    /// Number of skills at minimum quality (0.0)
    pub skills_at_min_quality: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillPriceInfo {
    pub id: SkillId,
    pub price: f64,
}

/// Statistics for a single skill's trade activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillTradeStats {
    /// Unique identifier for the skill
    pub skill_id: SkillId,
    /// Total number of trades for this skill
    pub trade_count: usize,
    /// Total money volume exchanged for this skill
    pub total_volume: f64,
    /// Average price per trade for this skill
    pub avg_price: f64,
}

/// Market concentration metrics for a single skill
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillMarketConcentration {
    /// Unique identifier for the skill
    pub skill_id: SkillId,
    /// Herfindahl-Hirschman Index for this skill (0-10000)
    /// < 1500 = competitive, 1500-2500 = moderate concentration, > 2500 = high concentration
    pub herfindahl_index: f64,
    /// Concentration Ratio for top 4 sellers (CR4) as percentage (0.0-1.0)
    pub cr4: f64,
    /// Concentration Ratio for top 8 sellers (CR8) as percentage (0.0-1.0)
    pub cr8: f64,
    /// Market structure classification based on HHI
    pub market_structure: MarketStructure,
    /// Number of active sellers (providers) for this skill
    pub num_sellers: usize,
    /// Total trading volume for this skill
    pub total_volume: f64,
}

/// Classification of market structure based on concentration metrics
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MarketStructure {
    /// Competitive market (HHI < 1500)
    Competitive,
    /// Moderately concentrated market (HHI 1500-2500)
    ModerateConcentration,
    /// Highly concentrated market / oligopoly (HHI > 2500)
    HighConcentration,
}

/// Statistics about trade volume and economic activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeVolumeStats {
    /// Total number of successful trades across all steps
    pub total_trades: usize,
    /// Total money exchanged across all trades
    pub total_volume: f64,
    /// Average number of trades per step
    pub avg_trades_per_step: f64,
    /// Average money exchanged per step
    pub avg_volume_per_step: f64,
    /// Average transaction value (total volume / total trades)
    pub avg_transaction_value: f64,
    /// Minimum trades in a single step
    pub min_trades_per_step: usize,
    /// Maximum trades in a single step
    pub max_trades_per_step: usize,
    /// Velocity of money: how many times money changes hands on average
    ///
    /// Calculated as: Total Transaction Volume / Total Money Supply
    ///
    /// This metric indicates:
    /// - Economic activity intensity (higher velocity = more active economy)
    /// - How efficiently money circulates (hoarding reduces velocity)
    /// - Relationship between money supply and economic output
    ///
    /// A value of 5.0 means each unit of money was used in transactions 5 times on average during the simulation.
    /// Higher values indicate more dynamic trading, lower values suggest money is being hoarded.
    pub velocity_of_money: f64,
}

/// Represents a detected business cycle phase (expansion or contraction).
///
/// Business cycles are periods of economic growth (expansion) or decline (contraction)
/// identified by analyzing trade volume patterns. Each cycle is bounded by a peak
/// (local maximum in trade activity) and a trough (local minimum).
///
/// # Examples
///
/// ```
/// use simulation_framework::result::{BusinessCycle, CyclePhase};
///
/// // An expansion phase from step 10 to step 50
/// let expansion = BusinessCycle {
///     phase: CyclePhase::Expansion,
///     start_step: 10,
///     end_step: 50,
///     duration: 40,
///     avg_volume: 1500.0,
///     peak_volume: 2000.0,
///     trough_volume: 1000.0,
/// };
///
/// assert_eq!(expansion.duration, 40);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BusinessCycle {
    /// The phase of this cycle (expansion or contraction)
    pub phase: CyclePhase,
    /// Simulation step where this phase started
    pub start_step: usize,
    /// Simulation step where this phase ended
    pub end_step: usize,
    /// Duration of this phase in steps
    pub duration: usize,
    /// Average trade volume during this phase
    pub avg_volume: f64,
    /// Peak trade volume in this phase
    pub peak_volume: f64,
    /// Trough trade volume in this phase
    pub trough_volume: f64,
}

/// Represents the phase of a business cycle.
///
/// Business cycles alternate between expansion (increasing economic activity)
/// and contraction (decreasing economic activity).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CyclePhase {
    /// Expansion phase: Trade volume is increasing
    Expansion,
    /// Contraction phase: Trade volume is decreasing
    Contraction,
}

/// Statistics about detected business cycles in the simulation.
///
/// Analyzes trade volume patterns to identify economic cycles, including
/// periods of expansion (growth) and contraction (decline). This provides
/// insights into the natural cyclical behavior of the simulated economy.
///
/// # Algorithm
///
/// Uses a simple peak/trough detection algorithm:
/// 1. Identifies local maxima (peaks) and minima (troughs) in trade volume
/// 2. Groups consecutive steps between peaks and troughs into phases
/// 3. Classifies phases as expansion (increasing volume) or contraction (decreasing volume)
///
/// # Limitations
///
/// - Requires at least 10 simulation steps for meaningful analysis
/// - Does not use advanced filtering (e.g., Hodrick-Prescott filter)
/// - May detect noise as cycles in highly volatile simulations
///
/// # Examples
///
/// ```
/// use simulation_framework::result::BusinessCycleStats;
///
/// let stats = BusinessCycleStats {
///     total_cycles: 3,
///     avg_cycle_duration: 100.0,
///     avg_expansion_duration: 60.0,
///     avg_contraction_duration: 40.0,
///     detected_cycles: vec![],
/// };
///
/// assert_eq!(stats.total_cycles, 3);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BusinessCycleStats {
    /// Total number of complete cycles detected
    pub total_cycles: usize,
    /// Average duration of complete cycles (peak to peak)
    pub avg_cycle_duration: f64,
    /// Average duration of expansion phases
    pub avg_expansion_duration: f64,
    /// Average duration of contraction phases
    pub avg_contraction_duration: f64,
    /// List of all detected cycles with details
    pub detected_cycles: Vec<BusinessCycle>,
}

/// Statistics about failed trade attempts (trades that failed due to insufficient funds).
///
/// This provides insight into unmet demand and market accessibility issues.
/// High failure rates indicate economic stress where persons want to buy skills
/// but cannot afford them, revealing inefficiencies in wealth distribution.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailedTradeStats {
    /// Total number of trade attempts that failed due to insufficient funds
    pub total_failed_attempts: usize,
    /// Ratio of failed attempts to total attempts (failed / (successful + failed))
    /// Value ranges from 0.0 (no failures) to 1.0 (all attempts failed)
    pub failure_rate: f64,
    /// Average number of failed attempts per simulation step
    pub avg_failed_per_step: f64,
    /// Minimum failed attempts in a single step
    pub min_failed_per_step: usize,
    /// Maximum failed attempts in a single step
    pub max_failed_per_step: usize,
}

/// Statistics about black market activity (parallel informal market)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlackMarketStats {
    /// Total number of trades conducted on the black market
    pub total_black_market_trades: usize,
    /// Total money exchanged on the black market
    pub total_black_market_volume: f64,
    /// Average number of black market trades per step
    pub avg_black_market_trades_per_step: f64,
    /// Average money exchanged on black market per step
    pub avg_black_market_volume_per_step: f64,
    /// Percentage of all trades that used the black market
    pub black_market_trade_percentage: f64,
    /// Percentage of all trading volume that went through the black market
    pub black_market_volume_percentage: f64,
}

/// Statistics about the loan system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoanStats {
    /// Total number of loans issued during the simulation
    pub total_loans_issued: usize,
    /// Total number of loans fully repaid
    pub total_loans_repaid: usize,
    /// Number of active (not yet fully repaid) loans at simulation end
    pub active_loans: usize,
}

/// Statistics about the investment system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvestmentStats {
    /// Total number of investments created during the simulation
    pub total_investments_created: usize,
    /// Total number of investments completed (all returns paid)
    pub total_investments_completed: usize,
    /// Number of active investments at simulation end
    pub active_investments: usize,
    /// Total amount of money invested (principals)
    pub total_invested: f64,
    /// Total amount of returns paid to investors
    pub total_returns_paid: f64,
    /// Average ROI percentage across all completed investments
    pub avg_roi_percentage: f64,
}

/// Statistics about the contract system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractStats {
    /// Total number of contracts created during the simulation
    pub total_contracts_created: usize,
    /// Total number of contracts successfully completed
    pub total_contracts_completed: usize,
    /// Number of active contracts at simulation end
    pub active_contracts: usize,
    /// Average duration of completed contracts (in steps)
    pub avg_contract_duration: f64,
    /// Total value exchanged through contracts
    pub total_contract_value: f64,
}

/// Statistics about the insurance system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsuranceStats {
    /// Total number of insurance policies issued during the simulation
    pub total_policies_issued: usize,
    /// Number of active (not expired, not claimed) policies at simulation end
    pub active_policies: usize,
    /// Total number of claims paid out
    pub total_claims_paid: usize,
    /// Total premiums collected from policyholders
    pub total_premiums_collected: f64,
    /// Total payouts made to claimants
    pub total_payouts_made: f64,
    /// Net profit/loss for the insurance system (premiums - payouts)
    pub net_result: f64,
    /// Loss ratio (payouts / premiums) - indicates profitability
    pub loss_ratio: f64,
}

/// Aggregated statistics across multiple Monte Carlo runs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonteCarloStats {
    /// Average value across all runs
    pub mean: f64,
    /// Standard deviation across all runs
    pub std_dev: f64,
    /// Minimum value across all runs
    pub min: f64,
    /// Maximum value across all runs
    pub max: f64,
    /// Median value across all runs
    pub median: f64,
}

/// Result from multiple Monte Carlo simulation runs
#[derive(Debug, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// Number of simulation runs performed
    pub num_runs: usize,
    /// Base seed used (each run uses seed + run_index)
    pub base_seed: u64,
    /// Individual results from each run
    pub runs: Vec<SimulationResult>,
    /// Aggregated statistics for final average money
    pub avg_money_stats: MonteCarloStats,
    /// Aggregated statistics for Gini coefficient
    pub gini_coefficient_stats: MonteCarloStats,
    /// Aggregated statistics for total trades
    pub total_trades_stats: MonteCarloStats,
    /// Aggregated statistics for average reputation
    pub avg_reputation_stats: MonteCarloStats,
}

/// Statistics about environmental resource consumption and sustainability
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentStats {
    /// Total resource consumption by resource type
    pub total_consumption: HashMap<String, f64>,
    /// Initial resource reserves by resource type
    pub initial_reserves: HashMap<String, f64>,
    /// Remaining reserves by resource type (can be negative if overconsumed)
    pub remaining_reserves: HashMap<String, f64>,
    /// Sustainability score per resource type (1.0 = sustainable, 0.0 = depleted, < 0 = overconsumed)
    pub sustainability_scores: HashMap<String, f64>,
    /// Overall sustainability score (average across all resources)
    pub overall_sustainability_score: f64,
    /// Whether the environment is sustainable (all resources >= 0)
    pub is_sustainable: bool,
}

/// Statistics about education system activity (skill learning)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EducationStats {
    /// Total number of skills learned across all persons
    pub total_skills_learned: usize,
    /// Average number of learned skills per person
    pub avg_learned_skills_per_person: f64,
    /// Person with the most learned skills
    pub max_learned_skills: usize,
    /// Total money spent on education across all persons
    pub total_education_spending: f64,
}

/// Statistics about mentorship system activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorshipStats {
    /// Total number of mentorships formed during the simulation
    pub total_mentorships: usize,
    /// Total number of successful mentored learning events
    pub successful_mentored_learnings: usize,
    /// Total cost savings from mentorship (compared to unmentored learning)
    pub total_cost_savings: f64,
    /// Number of unique persons who acted as mentors
    pub unique_mentors: usize,
    /// Number of unique persons who were mentored
    pub unique_mentees: usize,
}

/// Statistics about technology breakthrough events
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechnologyBreakthroughStats {
    /// Total number of breakthrough events during the simulation
    pub total_breakthroughs: usize,
    /// Number of unique skills that received breakthroughs
    pub unique_skills_affected: usize,
    /// Average efficiency boost across all breakthroughs (e.g., 1.3 = 30% average boost)
    pub average_efficiency_boost: f64,
    /// Minimum efficiency boost observed
    pub min_efficiency_boost: f64,
    /// Maximum efficiency boost observed
    pub max_efficiency_boost: f64,
    /// List of all breakthrough events with details
    pub breakthrough_events: Vec<BreakthroughEvent>,
}

/// Details of a single technology breakthrough event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreakthroughEvent {
    /// The skill that received the breakthrough
    pub skill_id: String,
    /// The efficiency multiplier applied (e.g., 1.3 = 30% boost)
    pub efficiency_boost: f64,
    /// The simulation step when the breakthrough occurred
    pub step: usize,
}

/// Statistics about certification system activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificationStats {
    /// Total number of certifications issued during the simulation
    pub total_issued: usize,
    /// Total number of certifications that expired during the simulation
    pub total_expired: usize,
    /// Number of certifications currently active (not expired) at simulation end
    pub active_certifications: usize,
    /// Total money spent on obtaining certifications across all persons
    pub total_cost: f64,
}

/// Statistics about the friendship system (social connections)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendshipStats {
    /// Total number of friendships formed during the simulation
    pub total_friendships: usize,
    /// Average number of friends per person
    pub avg_friends_per_person: f64,
    /// Maximum number of friends any person has
    pub max_friends: usize,
    /// Minimum number of friends any person has
    pub min_friends: usize,
    /// Friend network density (actual friendships / possible friendships)
    /// Range: 0.0 to 1.0, where 1.0 means everyone is friends with everyone
    pub network_density: f64,
}

/// Statistics for a single group/organization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleGroupStats {
    /// Group identifier
    pub group_id: usize,
    /// Number of persons in this group
    pub member_count: usize,
    /// Average money of group members
    pub avg_money: f64,
    /// Total money held by group members
    pub total_money: f64,
    /// Average reputation of group members
    pub avg_reputation: f64,
    /// Resource pool balance for this group (if resource pools enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_balance: Option<f64>,
    /// Total contributions to the pool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_contributions: Option<f64>,
    /// Total withdrawals from the pool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_withdrawals: Option<f64>,
}

/// Statistics about group/organization system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupStats {
    /// Total number of groups in the simulation
    pub total_groups: usize,
    /// Average group size (members per group)
    pub avg_group_size: f64,
    /// Minimum group size
    pub min_group_size: usize,
    /// Maximum group size
    pub max_group_size: usize,
    /// Statistics for each individual group
    pub groups: Vec<SingleGroupStats>,
    /// Total balance across all resource pools (if resource pools enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pool_balance: Option<f64>,
    /// Total contributions across all pools (if resource pools enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_contributions: Option<f64>,
    /// Total withdrawals across all pools (if resource pools enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_withdrawals: Option<f64>,
}

/// Information about a trading partner relationship
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartnerInfo {
    /// ID of the trading partner
    pub partner_id: usize,
    /// Number of trades with this partner
    pub trade_count: usize,
    /// Total value exchanged with this partner
    pub total_value: f64,
}

/// Trading statistics for an individual person
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonTradingStats {
    /// ID of the person
    pub person_id: usize,
    /// Number of unique trading partners
    pub unique_partners: usize,
    /// Total trades as a buyer
    pub total_trades_as_buyer: usize,
    /// Total trades as a seller
    pub total_trades_as_seller: usize,
    /// Top trading partners (sorted by trade count, descending)
    pub top_partners: Vec<PartnerInfo>,
}

/// Network-level trading statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMetrics {
    /// Average number of unique partners per person
    pub avg_unique_partners: f64,
    /// Network density (actual connections / possible connections)
    pub network_density: f64,
    /// Most active trading pair
    pub most_active_pair: Option<(usize, usize, usize)>, // (person1_id, person2_id, trade_count)
}

/// Complete trading partner statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingPartnerStats {
    /// Per-person trading statistics
    pub per_person: Vec<PersonTradingStats>,
    /// Network-level metrics
    pub network_metrics: NetworkMetrics,
}

/// Node (person) in the trading network graph
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkNode {
    /// Unique identifier for the person
    pub id: String,
    /// Current money balance
    pub money: f64,
    /// Reputation score
    pub reputation: f64,
    /// Total number of trades (as buyer + seller)
    pub trade_count: usize,
    /// Number of unique trading partners
    pub unique_partners: usize,
}

/// Edge (trading relationship) in the trading network graph
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkEdge {
    /// Source person ID
    pub source: String,
    /// Target person ID
    pub target: String,
    /// Number of trades between these persons
    pub weight: usize,
    /// Total money exchanged between these persons
    pub total_value: f64,
}

/// Complete trading network in graph format for visualization
/// Compatible with vis.js, D3.js, NetworkX, Gephi, Cytoscape
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingNetworkData {
    /// List of nodes (persons) in the network
    pub nodes: Vec<NetworkNode>,
    /// List of edges (trading relationships) in the network
    pub edges: Vec<NetworkEdge>,
}

/// Statistics about social mobility and wealth transitions between quintiles over time.
///
/// This structure captures how persons move between wealth quintiles (bottom 20%,
/// second 20%, etc.) throughout the simulation, providing insight into economic mobility
/// and the dynamics of wealth inequality.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MobilityStatistics {
    /// 5x5 transition matrix showing probability of moving between quintiles.
    /// Entry [i][j] represents the probability of moving from quintile i to quintile j.
    /// Quintiles: 0 = bottom 20%, 1 = second 20%, 2 = middle 20%, 3 = fourth 20%, 4 = top 20%
    pub transition_matrix: Vec<Vec<f64>>,
    /// Probability of moving to a higher quintile (upward mobility)
    pub upward_mobility_probability: f64,
    /// Probability of moving to a lower quintile (downward mobility)
    pub downward_mobility_probability: f64,
    /// Probability of remaining in the same quintile (persistence)
    pub quintile_persistence: f64,
    /// Average number of quintile changes per person across the simulation
    pub avg_quintile_changes: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Metadata about this simulation run for reproducibility
    pub metadata: SimulationMetadata,

    // Core simulation metrics
    pub total_steps: usize,
    pub total_duration: f64,
    pub step_times: Vec<f64>,  // Time taken for each step
    pub active_persons: usize, // Renamed from active_entities for clarity
    /// Number of steps that failed due to panics but were recovered gracefully
    pub failed_steps: usize,

    // Economic output
    pub final_money_distribution: Vec<f64>, // List of money amounts per person
    pub money_statistics: MoneyStats,

    // Reputation metrics
    pub final_reputation_distribution: Vec<f64>, // List of reputation scores per person
    pub reputation_statistics: ReputationStats,

    // Savings metrics
    pub final_savings_distribution: Vec<f64>, // List of savings amounts per person
    pub savings_statistics: SavingsStats,

    // Credit rating metrics (only populated when credit rating system is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_score_statistics: Option<CreditScoreStats>,

    pub final_skill_prices: Vec<SkillPriceInfo>, // Sorted by price
    pub most_valuable_skill: Option<SkillPriceInfo>,
    pub least_valuable_skill: Option<SkillPriceInfo>,

    // For graphical representation of price development over time
    // Key: SkillId, Value: Vec of prices, one entry per step
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub skill_price_history: HashMap<SkillId, Vec<f64>>,

    /// Time-series of wealth distribution statistics, one snapshot per simulation step.
    /// Enables analysis of how wealth inequality evolves over time.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub wealth_stats_history: Vec<WealthStatsSnapshot>,

    // Trade volume analysis
    pub trade_volume_statistics: TradeVolumeStats,
    /// Number of trades executed at each step
    pub trades_per_step: Vec<usize>,
    /// Total money volume exchanged at each step
    pub volume_per_step: Vec<f64>,
    /// Total transaction fees collected across all trades
    pub total_fees_collected: f64,
    /// Per-skill trade statistics (sorted by trade volume, highest first)
    pub per_skill_trade_stats: Vec<SkillTradeStats>,
    /// Market concentration analysis per skill (only present if sufficient trade data exists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_market_concentration: Option<Vec<SkillMarketConcentration>>,

    /// Business cycle detection statistics (only present if simulation ran for at least 10 steps).
    ///
    /// Analyzes trade volume patterns to identify economic cycles of expansion and contraction.
    /// Provides insights into the natural cyclical behavior of the simulated economy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_cycle_statistics: Option<BusinessCycleStats>,

    /// Failed trade attempt statistics (trade attempts that failed due to insufficient funds)
    pub failed_trade_statistics: FailedTradeStats,
    /// Number of failed trade attempts at each step
    pub failed_attempts_per_step: Vec<usize>,

    /// Black market statistics (only present if black market is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub black_market_statistics: Option<BlackMarketStats>,

    /// Total taxes collected from seller proceeds (only present if tax_rate > 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_taxes_collected: Option<f64>,

    /// Total taxes redistributed to persons (only present if tax redistribution is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_taxes_redistributed: Option<f64>,

    /// Loan system statistics (only present if loans are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loan_statistics: Option<LoanStats>,

    /// Investment system statistics (only present if investments are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub investment_statistics: Option<InvestmentStats>,

    /// Contract system statistics (only present if contracts are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_statistics: Option<ContractStats>,

    /// Education system statistics (only present if education is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_statistics: Option<EducationStats>,

    /// Mentorship system statistics (only present if mentorship is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentorship_statistics: Option<MentorshipStats>,

    /// Certification system statistics (only present if certification is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certification_statistics: Option<CertificationStats>,

    /// Environmental resource consumption and sustainability statistics (only present if environment tracking is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_statistics: Option<EnvironmentStats>,

    /// Friendship system statistics (only present if friendships are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub friendship_statistics: Option<FriendshipStats>,

    /// Trust network statistics (only present if trust networks are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_network_statistics: Option<crate::trust_network::TrustNetworkStats>,

    /// Trade agreement system statistics (only present if trade agreements are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_agreement_statistics: Option<crate::trade_agreement::TradeAgreementStatistics>,

    /// Insurance system statistics (only present if insurance is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insurance_statistics: Option<InsuranceStats>,

    /// Technology breakthrough statistics (only present if breakthroughs are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology_breakthrough_statistics: Option<TechnologyBreakthroughStats>,

    /// Group/organization statistics (only present if groups are configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_statistics: Option<GroupStats>,

    /// Trading partner statistics showing network relationships and trading patterns
    pub trading_partner_statistics: TradingPartnerStats,

    /// Network centrality analysis for the trading network.
    ///
    /// Provides comprehensive analysis of network structure including degree centrality,
    /// betweenness centrality, eigenvector centrality, PageRank, and identification
    /// of key market participants (hubs, brokers, influencers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub centrality_analysis: Option<crate::centrality::CentralityAnalysis>,

    /// Social mobility statistics tracking wealth transitions between quintiles over time.
    /// Only present if the simulation ran for at least 2 steps (need at least 2 time points).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobility_statistics: Option<MobilityStatistics>,

    /// Quality rating statistics for skills (only present if quality system is enabled).
    ///
    /// Tracks the distribution of skill quality ratings across all persons.
    /// Quality ratings range from 0.0-5.0 and affect skill prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_statistics: Option<QualityStats>,

    /// Event log (only present if event tracking is enabled).
    ///
    /// Contains timestamped events for trades, price updates, reputation changes,
    /// and step completions. Useful for detailed analysis and debugging.
    /// This field will be populated in future versions when event emission is fully integrated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<crate::event::SimulationEvent>>,

    // final_entities might be too verbose if Person struct grows large with transaction history.
    // Consider summarizing person data if needed, or providing it under a flag.
    // For now, let's keep it as it contains all person data including transaction history.
    pub final_persons_data: Vec<Entity>, // Renamed from final_entities
}

impl SimulationResult {
    /// Save simulation results to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the output file
    /// * `compress` - If true, compress the output using gzip and append .gz to the filename
    ///
    /// # Returns
    /// * `Result<()>` - Success or a SimulationError
    ///
    /// # Examples
    /// ```no_run
    /// # use simulation_framework::result::SimulationResult;
    /// # use simulation_framework::result::SimulationMetadata;
    /// # let result = SimulationResult {
    /// #     metadata: SimulationMetadata::capture(42, 10, 100),
    /// #     total_steps: 0,
    /// #     total_duration: 0.0,
    /// #     step_times: vec![],
    /// #     active_persons: 0,
    /// #     failed_steps: 0,
    /// #     final_money_distribution: vec![],
    /// #     money_statistics: simulation_framework::result::MoneyStats {
    /// #         average: 0.0, median: 0.0, std_dev: 0.0,
    /// #         min_money: 0.0, max_money: 0.0, gini_coefficient: 0.0, herfindahl_index: 0.0,
    /// #         top_10_percent_share: 0.0, top_1_percent_share: 0.0, bottom_50_percent_share: 0.0,
    /// #     },
    /// #     final_reputation_distribution: vec![],
    /// #     reputation_statistics: simulation_framework::result::ReputationStats {
    /// #         average: 0.0, median: 0.0, std_dev: 0.0,
    /// #         min_reputation: 0.0, max_reputation: 0.0,
    /// #     },
    /// #     final_savings_distribution: vec![],
    /// #     savings_statistics: simulation_framework::result::SavingsStats {
    /// #         total_savings: 0.0, average_savings: 0.0, median_savings: 0.0,
    /// #         min_savings: 0.0, max_savings: 0.0,
    /// #     },
    /// #     credit_score_statistics: None,
    /// #     final_skill_prices: vec![],
    /// #     most_valuable_skill: None,
    /// #     least_valuable_skill: None,
    /// #     skill_price_history: std::collections::HashMap::new(),
    /// #     wealth_stats_history: vec![],
    /// #     trade_volume_statistics: simulation_framework::result::TradeVolumeStats {
    /// #         total_trades: 0, total_volume: 0.0,
    /// #         avg_trades_per_step: 0.0, avg_volume_per_step: 0.0,
    /// #         avg_transaction_value: 0.0,
    /// #         min_trades_per_step: 0, max_trades_per_step: 0,
    /// #         velocity_of_money: 0.0,
    /// #     },
    /// #     trades_per_step: vec![],
    /// #     volume_per_step: vec![],
    /// #     total_fees_collected: 0.0,
    /// #     per_skill_trade_stats: vec![],
    /// #     skill_market_concentration: None,
    /// #     business_cycle_statistics: None,
    /// #     failed_trade_statistics: simulation_framework::result::FailedTradeStats {
    /// #         total_failed_attempts: 0, failure_rate: 0.0,
    /// #         avg_failed_per_step: 0.0,
    /// #         min_failed_per_step: 0, max_failed_per_step: 0,
    /// #     },
    /// #     failed_attempts_per_step: vec![],
    /// #     black_market_statistics: None,
    /// #     total_taxes_collected: None,
    /// #     total_taxes_redistributed: None,
    /// #     loan_statistics: None,
    /// #     investment_statistics: None,
    /// #     contract_statistics: None,
    /// #     education_statistics: None,
    /// #     mentorship_statistics: None,
    /// #     certification_statistics: None,
    /// #     environment_statistics: None,
    /// #     friendship_statistics: None,
    /// #     trust_network_statistics: None,
    /// #     trade_agreement_statistics: None,
    /// #     insurance_statistics: None,
    /// #     technology_breakthrough_statistics: None,
    /// #     group_statistics: None,
    /// #     trading_partner_statistics: simulation_framework::result::TradingPartnerStats {
    /// #         per_person: vec![],
    /// #         network_metrics: simulation_framework::result::NetworkMetrics {
    /// #             avg_unique_partners: 0.0,
    /// #             network_density: 0.0,
    /// #             most_active_pair: None,
    /// #         },
    /// #     },
    /// #     centrality_analysis: None,
    /// #     mobility_statistics: None,
    /// #     quality_statistics: None,
    /// #     events: None,
    /// #     final_persons_data: vec![],
    /// # };
    /// // Save uncompressed JSON
    /// result.save_to_file("results.json", false).unwrap();
    ///
    /// // Save compressed JSON
    /// result.save_to_file("results.json", true).unwrap(); // Creates results.json.gz
    /// ```
    pub fn save_to_file(&self, path: &str, compress: bool) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;

        if compress {
            // Add .gz extension if not already present
            let output_path = if path.ends_with(".gz") {
                path.to_string()
            } else {
                format!("{}.gz", path)
            };

            let file = File::create(&output_path)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(json.as_bytes())?;
            encoder.finish()?;
        } else {
            let mut file = File::create(path)?;
            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }

    /// Save simulation results to CSV files.
    /// Creates multiple CSV files with the given path as prefix:
    /// - {path}_summary.csv: Summary statistics
    /// - {path}_money.csv: Money distribution per person
    /// - {path}_reputation.csv: Reputation distribution per person
    /// - {path}_skill_prices.csv: Final skill prices
    /// - {path}_price_history.csv: Skill price history over time (if available)
    /// - {path}_wealth_stats_history.csv: Wealth distribution statistics over time (if available)
    /// - {path}_trade_volume.csv: Trade volume history over time
    /// - {path}_network_nodes.csv: Trading network nodes (persons)
    /// - {path}_network_edges.csv: Trading network edges (relationships)
    ///
    /// # Returns
    /// * `Result<()>` - Success or a SimulationError
    pub fn save_to_csv(&self, path_prefix: &str) -> Result<()> {
        // Save summary statistics
        self.save_summary_csv(&format!("{}_summary.csv", path_prefix))?;

        // Save money distribution
        self.save_money_csv(&format!("{}_money.csv", path_prefix))?;

        // Save reputation distribution
        self.save_reputation_csv(&format!("{}_reputation.csv", path_prefix))?;

        // Save skill prices
        self.save_skill_prices_csv(&format!("{}_skill_prices.csv", path_prefix))?;

        // Save price history if available
        if !self.skill_price_history.is_empty() {
            self.save_price_history_csv(&format!("{}_price_history.csv", path_prefix))?;
        }

        // Save wealth stats history if available
        if !self.wealth_stats_history.is_empty() {
            self.save_wealth_stats_history_csv(&format!(
                "{}_wealth_stats_history.csv",
                path_prefix
            ))?;
        }

        // Save trade volume history
        self.save_trade_volume_csv(&format!("{}_trade_volume.csv", path_prefix))?;

        // Save trading network
        self.save_trading_network_csv(path_prefix)?;

        Ok(())
    }

    fn save_summary_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Metric,Value")?;
        writeln!(file, "Total Steps,{}", self.total_steps)?;
        writeln!(file, "Total Duration (s),{:.4}", self.total_duration)?;
        writeln!(file, "Active Persons,{}", self.active_persons)?;
        writeln!(file, "Failed Steps,{}", self.failed_steps)?;

        if !self.step_times.is_empty() {
            let avg_step_time = self.step_times.iter().sum::<f64>() / self.step_times.len() as f64;
            writeln!(file, "Average Step Time (s),{:.6}", avg_step_time)?;
        }

        writeln!(file)?;
        writeln!(file, "Money Statistics")?;
        writeln!(file, "Average Money,{:.4}", self.money_statistics.average)?;
        writeln!(file, "Median Money,{:.4}", self.money_statistics.median)?;
        writeln!(file, "Std Dev Money,{:.4}", self.money_statistics.std_dev)?;
        writeln!(file, "Min Money,{:.4}", self.money_statistics.min_money)?;
        writeln!(file, "Max Money,{:.4}", self.money_statistics.max_money)?;
        writeln!(
            file,
            "Gini Coefficient,{:.6}",
            self.money_statistics.gini_coefficient
        )?;
        writeln!(
            file,
            "Top 10%% Wealth Share (%%),{:.4}",
            self.money_statistics.top_10_percent_share * 100.0
        )?;
        writeln!(
            file,
            "Top 1%% Wealth Share (%%),{:.4}",
            self.money_statistics.top_1_percent_share * 100.0
        )?;
        writeln!(
            file,
            "Bottom 50%% Wealth Share (%%),{:.4}",
            self.money_statistics.bottom_50_percent_share * 100.0
        )?;
        writeln!(
            file,
            "Herfindahl Index,{:.2}",
            self.money_statistics.herfindahl_index
        )?;

        writeln!(file)?;
        writeln!(file, "Reputation Statistics")?;
        writeln!(
            file,
            "Average Reputation,{:.6}",
            self.reputation_statistics.average
        )?;
        writeln!(
            file,
            "Median Reputation,{:.6}",
            self.reputation_statistics.median
        )?;
        writeln!(
            file,
            "Std Dev Reputation,{:.6}",
            self.reputation_statistics.std_dev
        )?;
        writeln!(
            file,
            "Min Reputation,{:.6}",
            self.reputation_statistics.min_reputation
        )?;
        writeln!(
            file,
            "Max Reputation,{:.6}",
            self.reputation_statistics.max_reputation
        )?;

        writeln!(file)?;
        writeln!(file, "Trade Volume Statistics")?;
        writeln!(
            file,
            "Total Trades,{}",
            self.trade_volume_statistics.total_trades
        )?;
        writeln!(
            file,
            "Total Volume,{:.4}",
            self.trade_volume_statistics.total_volume
        )?;
        writeln!(
            file,
            "Avg Trades Per Step,{:.4}",
            self.trade_volume_statistics.avg_trades_per_step
        )?;
        writeln!(
            file,
            "Avg Volume Per Step,{:.4}",
            self.trade_volume_statistics.avg_volume_per_step
        )?;
        writeln!(
            file,
            "Avg Transaction Value,{:.4}",
            self.trade_volume_statistics.avg_transaction_value
        )?;
        writeln!(
            file,
            "Min Trades Per Step,{}",
            self.trade_volume_statistics.min_trades_per_step
        )?;
        writeln!(
            file,
            "Max Trades Per Step,{}",
            self.trade_volume_statistics.max_trades_per_step
        )?;
        writeln!(
            file,
            "Velocity of Money,{:.4}",
            self.trade_volume_statistics.velocity_of_money
        )?;

        writeln!(file)?;
        writeln!(file, "Failed Trade Attempt Statistics")?;
        writeln!(
            file,
            "Total Failed Attempts,{}",
            self.failed_trade_statistics.total_failed_attempts
        )?;
        writeln!(
            file,
            "Failure Rate,{:.6}",
            self.failed_trade_statistics.failure_rate
        )?;
        writeln!(
            file,
            "Avg Failed Per Step,{:.4}",
            self.failed_trade_statistics.avg_failed_per_step
        )?;
        writeln!(
            file,
            "Min Failed Per Step,{}",
            self.failed_trade_statistics.min_failed_per_step
        )?;
        writeln!(
            file,
            "Max Failed Per Step,{}",
            self.failed_trade_statistics.max_failed_per_step
        )?;

        Ok(())
    }

    fn save_money_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Person_ID,Money")?;
        for (id, money) in self.final_money_distribution.iter().enumerate() {
            writeln!(file, "{},{:.4}", id, money)?;
        }

        Ok(())
    }

    fn save_reputation_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Person_ID,Reputation")?;
        for (id, reputation) in self.final_reputation_distribution.iter().enumerate() {
            writeln!(file, "{},{:.6}", id, reputation)?;
        }

        Ok(())
    }

    fn save_skill_prices_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Skill_ID,Final_Price")?;
        for skill_info in &self.final_skill_prices {
            writeln!(file, "{},{:.4}", skill_info.id, skill_info.price)?;
        }

        Ok(())
    }

    fn save_price_history_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        // Collect all skill IDs and sort them for consistent output
        let mut skill_ids: Vec<_> = self.skill_price_history.keys().collect();
        skill_ids.sort();

        // Write header
        write!(file, "Step")?;
        for skill_id in &skill_ids {
            write!(file, ",Skill_{}", skill_id)?;
        }
        writeln!(file)?;

        // Determine max number of steps (should be the same for all skills)
        let max_steps = self
            .skill_price_history
            .values()
            .map(|prices| prices.len())
            .max()
            .unwrap_or(0);

        // Write data rows
        for step in 0..max_steps {
            write!(file, "{}", step)?;
            for skill_id in &skill_ids {
                let price = self
                    .skill_price_history
                    .get(*skill_id)
                    .and_then(|prices| prices.get(step));

                if let Some(&price) = price {
                    write!(file, ",{:.4}", price)?;
                } else {
                    write!(file, ",")?;
                }
            }
            writeln!(file)?;
        }

        Ok(())
    }

    fn save_trade_volume_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Step,Trades_Count,Volume_Exchanged")?;
        for (step, (&trades, &volume)) in self
            .trades_per_step
            .iter()
            .zip(self.volume_per_step.iter())
            .enumerate()
        {
            writeln!(file, "{},{},{:.4}", step, trades, volume)?;
        }

        Ok(())
    }

    /// Save wealth stats history to CSV file.
    /// Each row represents wealth distribution statistics at a specific simulation step.
    fn save_wealth_stats_history_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        // Write CSV header
        writeln!(
            file,
            "Step,Average,Median,Std_Dev,Min,Max,Gini_Coefficient,Herfindahl_Index,Top_10_Percent_Share,Top_1_Percent_Share,Bottom_50_Percent_Share"
        )?;

        // Write data rows
        for snapshot in &self.wealth_stats_history {
            writeln!(
                file,
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                snapshot.step,
                snapshot.average,
                snapshot.median,
                snapshot.std_dev,
                snapshot.min_money,
                snapshot.max_money,
                snapshot.gini_coefficient,
                snapshot.herfindahl_index,
                snapshot.top_10_percent_share,
                snapshot.top_1_percent_share,
                snapshot.bottom_50_percent_share
            )?;
        }

        Ok(())
    }

    /// Render an ASCII histogram of wealth distribution to the terminal.
    ///
    /// Groups persons into 10 percentile buckets and displays a bar chart
    /// showing the distribution of wealth across the population.
    fn print_wealth_histogram(&self) {
        if self.final_money_distribution.is_empty() {
            return;
        }

        // Sort money values to create percentile buckets
        let mut sorted_money = self.final_money_distribution.clone();
        sorted_money.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Create 10 buckets (deciles)
        let num_buckets = 10;
        let bucket_size = sorted_money.len() / num_buckets;

        // Count persons in each bucket
        let mut buckets: Vec<(String, usize)> = Vec::new();
        for i in 0..num_buckets {
            let start_idx = i * bucket_size;
            let end_idx = if i == num_buckets - 1 {
                sorted_money.len()
            } else {
                (i + 1) * bucket_size
            };

            let count = end_idx - start_idx;
            let label = format!("{:>3}-{:<3}%", i * 10, (i + 1) * 10);
            buckets.push((label, count));
        }

        // Find max count for scaling
        let max_count = buckets.iter().map(|(_, count)| *count).max().unwrap_or(1);

        // Calculate bar width (max 50 characters)
        let max_bar_width = 50;

        // Print histogram
        for (label, count) in &buckets {
            let bar_length = if max_count > 0 {
                ((*count as f64 / max_count as f64) * max_bar_width as f64).round() as usize
            } else {
                0
            };

            // Create bar with color gradient (green for lower percentiles, red for upper)
            let bar = if bar_length > 0 {
                let bar_str = "█".repeat(bar_length);
                // Color based on percentile: lower = green, higher = yellow/red
                let percentile_start = label
                    .split('-')
                    .next()
                    .unwrap_or("0")
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0);
                if percentile_start < 30 {
                    bar_str.bright_green()
                } else if percentile_start < 70 {
                    bar_str.bright_yellow()
                } else {
                    bar_str.bright_red()
                }
            } else {
                "".normal()
            };

            println!(
                "  {} {} {}",
                label.dimmed(),
                bar,
                format!("({} persons)", count).dimmed()
            );
        }
    }

    /// Print a human-readable summary of the simulation results to stdout.
    ///
    /// # Arguments
    /// * `show_histogram` - Whether to display the ASCII wealth distribution histogram (default: true)
    pub fn print_summary(&self, show_histogram: bool) {
        println!(
            "\n{}",
            "=== Economic Simulation Summary ===".bright_cyan().bold()
        );
        println!("{} {}", "Total steps:".bold(), self.total_steps);
        println!("{} {:.2}s", "Total duration:".bold(), self.total_duration);
        if !self.step_times.is_empty() {
            let avg_step_time_ms =
                self.step_times.iter().sum::<f64>() / self.step_times.len() as f64 * 1000.0;
            println!("{} {:.4}ms", "Average step time:".bold(), avg_step_time_ms);
        }
        println!(
            "{} {}",
            "Active persons remaining:".bold(),
            self.active_persons
        );

        // Display failed steps if any occurred
        if self.failed_steps > 0 {
            println!(
                "{} {} {}",
                "Failed steps (recovered):".bold(),
                format!("{}", self.failed_steps).bright_red(),
                format!(
                    "({:.1}% of total)",
                    self.failed_steps as f64 / self.total_steps as f64 * 100.0
                )
                .dimmed()
            );
        }

        let performance = if self.total_duration > 0.0 {
            self.total_steps as f64 / self.total_duration
        } else {
            0.0
        };
        println!(
            "{} {}",
            "Performance:".bold(),
            format!("{:.0} steps/second", performance).bright_yellow()
        );

        println!("\n{}", "--- Money Distribution ---".bright_green().bold());
        println!(
            "{} {:.2}",
            "Average Money:".bold(),
            self.money_statistics.average
        );
        println!(
            "{} {:.2}",
            "Median Money:".bold(),
            self.money_statistics.median
        );
        println!(
            "{} {:.2}",
            "Std Dev Money:".bold(),
            self.money_statistics.std_dev
        );
        println!(
            "{} {:.2} / {:.2}",
            "Min/Max Money:".bold(),
            self.money_statistics.min_money,
            self.money_statistics.max_money
        );

        // Color code Gini coefficient based on inequality level
        let gini_str = format!("{:.4}", self.money_statistics.gini_coefficient);
        let gini_colored = if self.money_statistics.gini_coefficient < 0.3 {
            gini_str.bright_green()
        } else if self.money_statistics.gini_coefficient < 0.5 {
            gini_str.bright_yellow()
        } else {
            gini_str.bright_red()
        };
        println!(
            "  {} {} {}",
            "Gini Coefficient:".bold(),
            gini_colored,
            "(0 = perfect equality, 1 = perfect inequality)".dimmed()
        );

        // Print wealth concentration ratios
        println!(
            "  {} {:.2}% {}",
            "Top 10% Wealth Share:".bold(),
            self.money_statistics.top_10_percent_share * 100.0,
            "(of total wealth)".dimmed()
        );
        println!(
            "  {} {:.2}% {}",
            "Top 1% Wealth Share:".bold(),
            self.money_statistics.top_1_percent_share * 100.0,
            "(of total wealth)".dimmed()
        );
        println!(
            "  {} {:.2}% {}",
            "Bottom 50% Wealth Share:".bold(),
            self.money_statistics.bottom_50_percent_share * 100.0,
            "(of total wealth)".dimmed()
        );

        // Color code HHI based on concentration level
        let hhi = self.money_statistics.herfindahl_index;
        let hhi_str = format!("{:.2}", hhi);
        let hhi_colored = if hhi < 1500.0 {
            hhi_str.bright_green()
        } else if hhi < 2500.0 {
            hhi_str.bright_yellow()
        } else {
            hhi_str.bright_red()
        };
        println!(
            "  {} {} {}",
            "Herfindahl Index:".bold(),
            hhi_colored,
            "(< 1500 = competitive, 1500-2500 = moderate, > 2500 = high concentration)".dimmed()
        );

        // Display ASCII histogram if requested
        if show_histogram && !self.final_money_distribution.is_empty() {
            println!(
                "\n{}",
                "--- Wealth Distribution Histogram ---"
                    .bright_green()
                    .bold()
            );
            self.print_wealth_histogram();
        }

        println!(
            "\n{}",
            "--- Reputation Distribution ---".bright_magenta().bold()
        );
        println!(
            "{} {:.4}",
            "Average Reputation:".bold(),
            self.reputation_statistics.average
        );
        println!(
            "{} {:.4}",
            "Median Reputation:".bold(),
            self.reputation_statistics.median
        );
        println!(
            "{} {:.4}",
            "Std Dev Reputation:".bold(),
            self.reputation_statistics.std_dev
        );
        println!(
            "{} {:.4} / {:.4}",
            "Min/Max Reputation:".bold(),
            self.reputation_statistics.min_reputation,
            self.reputation_statistics.max_reputation
        );

        // Print quality statistics if quality system was enabled
        if let Some(ref quality_stats) = self.quality_statistics {
            println!(
                "\n{}",
                "--- Quality Rating Distribution ---".bright_cyan().bold()
            );
            println!(
                "{} {:.2}",
                "Average Quality:".bold(),
                quality_stats.average_quality
            );
            println!(
                "{} {:.2}",
                "Median Quality:".bold(),
                quality_stats.median_quality
            );
            println!(
                "{} {:.2}",
                "Std Dev Quality:".bold(),
                quality_stats.std_dev_quality
            );
            println!(
                "{} {:.2} / {:.2} {}",
                "Min/Max Quality:".bold(),
                quality_stats.min_quality,
                quality_stats.max_quality,
                "(scale: 0.0-5.0)".dimmed()
            );
            println!(
                "{} {}",
                "Skills at Max Quality (5.0):".bold(),
                quality_stats.skills_at_max_quality
            );
            println!(
                "{} {}",
                "Skills at Min Quality (0.0):".bold(),
                quality_stats.skills_at_min_quality
            );
        }

        println!("\n{}", "--- Skill Valuations ---".bright_blue().bold());
        if let Some(skill) = &self.most_valuable_skill {
            println!(
                "{} {} {}",
                "Most Valuable Skill:".bold(),
                skill.id.to_string().bright_cyan(),
                format!("(Price: {:.2})", skill.price).bright_green()
            );
        }
        if let Some(skill) = &self.least_valuable_skill {
            println!(
                "{} {} {}",
                "Least Valuable Skill:".bold(),
                skill.id.to_string().bright_cyan(),
                format!("(Price: {:.2})", skill.price).bright_red()
            );
        }

        println!(
            "\n{}",
            "--- Trade Volume Statistics ---".bright_yellow().bold()
        );
        println!(
            "{} {}",
            "Total Trades:".bold(),
            self.trade_volume_statistics.total_trades
        );
        println!(
            "{} {:.2}",
            "Total Volume Exchanged:".bold(),
            self.trade_volume_statistics.total_volume
        );
        println!(
            "{} {:.2}",
            "Avg Trades Per Step:".bold(),
            self.trade_volume_statistics.avg_trades_per_step
        );
        println!(
            "{} {:.2}",
            "Avg Volume Per Step:".bold(),
            self.trade_volume_statistics.avg_volume_per_step
        );
        println!(
            "{} {:.2}",
            "Avg Transaction Value:".bold(),
            self.trade_volume_statistics.avg_transaction_value
        );
        println!(
            "{} {} / {}",
            "Min/Max Trades Per Step:".bold(),
            self.trade_volume_statistics.min_trades_per_step,
            self.trade_volume_statistics.max_trades_per_step
        );

        // Display velocity of money
        println!(
            "{} {:.2} {}",
            "Velocity of Money:".bold(),
            self.trade_volume_statistics.velocity_of_money,
            "(times money changed hands)".dimmed()
        );

        // Display failed trade attempt statistics
        println!(
            "\n{}",
            "--- Failed Trade Attempts ---".bright_magenta().bold()
        );
        println!(
            "{} {}",
            "Total Failed Attempts:".bold(),
            self.failed_trade_statistics.total_failed_attempts
        );

        // Color code failure rate based on severity
        let failure_rate_pct = self.failed_trade_statistics.failure_rate * 100.0;
        let failure_rate_str = format!("{:.2}%", failure_rate_pct);
        let failure_rate_colored = if failure_rate_pct < 10.0 {
            failure_rate_str.bright_green()
        } else if failure_rate_pct < 30.0 {
            failure_rate_str.bright_yellow()
        } else {
            failure_rate_str.bright_red()
        };
        println!(
            "{} {} {}",
            "Failure Rate:".bold(),
            failure_rate_colored,
            "(failed / total attempts)".dimmed()
        );

        println!(
            "{} {:.2}",
            "Avg Failed Per Step:".bold(),
            self.failed_trade_statistics.avg_failed_per_step
        );
        println!(
            "{} {} / {}",
            "Min/Max Failed Per Step:".bold(),
            self.failed_trade_statistics.min_failed_per_step,
            self.failed_trade_statistics.max_failed_per_step
        );

        println!("\n{}", "Top 5 Most Valuable Skills:".bright_cyan().bold());
        for skill_info in self.final_skill_prices.iter().take(5) {
            println!(
                "  {} {} {:.2}",
                "-".dimmed(),
                format!("{}:", skill_info.id).bright_white(),
                skill_info.price
            );
        }

        println!(
            "\n{}",
            "Top 5 Least Valuable Skills (excluding those at min price if many):"
                .bright_cyan()
                .bold()
        );
        // Iterate in reverse, but skip if all are min_price
        let mut count = 0;
        for skill_info in self.final_skill_prices.iter().rev().take(10) {
            // Check more than 5 to find some not at min
            if count < 5 {
                // Basic heuristic: if it's significantly above absolute min, show it.
                // This needs better logic if many skills bottom out at min_skill_price.
                // For now, just show them.
                println!(
                    "  {} {} {:.2}",
                    "-".dimmed(),
                    format!("{}:", skill_info.id).bright_white(),
                    skill_info.price
                );
                count += 1;
            }
        }
        if self.skill_price_history.keys().len() > 0 {
            println!(
                "\n{} {} {}",
                "Skill price history for".dimmed(),
                format!("{}", self.skill_price_history.keys().len()).bright_white(),
                "skills available in JSON output.".dimmed()
            );
        }
    }

    /// Export the trading network as a graph structure suitable for visualization.
    ///
    /// Converts trading partner statistics into a graph format compatible with
    /// popular visualization tools like vis.js, D3.js, NetworkX, Gephi, and Cytoscape.
    ///
    /// # Returns
    /// * `TradingNetworkData` - Graph structure with nodes (persons) and edges (trading relationships)
    ///
    /// # Example
    /// ```no_run
    /// # use simulation_framework::{SimulationEngine, SimulationConfig};
    /// # let config = SimulationConfig::default();
    /// # let mut engine = SimulationEngine::new(config);
    /// let result = engine.run();
    /// let network = result.export_trading_network();
    /// // Network can now be serialized to JSON for visualization
    /// let json = serde_json::to_string_pretty(&network).unwrap();
    /// ```
    pub fn export_trading_network(&self) -> TradingNetworkData {
        // Build nodes from persons
        let nodes: Vec<NetworkNode> = self
            .trading_partner_statistics
            .per_person
            .iter()
            .map(|person_stats| {
                let person_id = person_stats.person_id;
                // Use person_id to look up data in distributions (arrays are indexed by person_id)
                let money = self
                    .final_money_distribution
                    .get(person_id)
                    .copied()
                    .unwrap_or(0.0);
                let reputation = self
                    .final_reputation_distribution
                    .get(person_id)
                    .copied()
                    .unwrap_or(1.0);
                let trade_count =
                    person_stats.total_trades_as_buyer + person_stats.total_trades_as_seller;

                NetworkNode {
                    id: format!("Person{}", person_id),
                    money,
                    reputation,
                    trade_count,
                    unique_partners: person_stats.unique_partners,
                }
            })
            .collect();

        // Build edges from trading relationships
        // Use a map to aggregate bidirectional trades into undirected edges
        let mut edge_map: std::collections::HashMap<(usize, usize), (usize, f64)> =
            std::collections::HashMap::new();

        for person_stats in &self.trading_partner_statistics.per_person {
            let person_id = person_stats.person_id;
            for partner in &person_stats.top_partners {
                let partner_id = partner.partner_id;
                // Create normalized pair (smaller ID first) for undirected edge
                let edge_key = if person_id < partner_id {
                    (person_id, partner_id)
                } else {
                    (partner_id, person_id)
                };

                // Aggregate trade count and value
                let entry = edge_map.entry(edge_key).or_insert((0, 0.0));
                entry.0 += partner.trade_count;
                entry.1 += partner.total_value;
            }
        }

        // Convert edge map to edge list
        let edges: Vec<NetworkEdge> = edge_map
            .into_iter()
            .map(
                |((source_id, target_id), (weight, total_value))| NetworkEdge {
                    source: format!("Person{}", source_id),
                    target: format!("Person{}", target_id),
                    weight,
                    total_value,
                },
            )
            .collect();

        TradingNetworkData { nodes, edges }
    }

    /// Save trading network to a JSON file in graph format.
    ///
    /// The exported JSON is compatible with visualization libraries like:
    /// - vis.js (JavaScript network visualization)
    /// - D3.js (force-directed graphs)
    /// - NetworkX (Python network analysis) - use json.load()
    /// - Gephi (import as JSON)
    /// - Cytoscape (with appropriate plugins)
    ///
    /// # Arguments
    /// * `path` - File path for the JSON output
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```no_run
    /// # use simulation_framework::{SimulationEngine, SimulationConfig};
    /// # let config = SimulationConfig::default();
    /// # let mut engine = SimulationEngine::new(config);
    /// let result = engine.run();
    /// result.save_trading_network_json("network.json").unwrap();
    /// ```
    pub fn save_trading_network_json(&self, path: &str) -> Result<()> {
        let network = self.export_trading_network();
        let json = serde_json::to_string_pretty(&network)
            .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Save trading network to CSV files (nodes and edges).
    ///
    /// Creates two CSV files:
    /// - {prefix}_network_nodes.csv: Person attributes (id, money, reputation, trade_count, unique_partners)
    /// - {prefix}_network_edges.csv: Trading relationships (source, target, weight, total_value)
    ///
    /// These files are compatible with:
    /// - Gephi (import nodes and edges separately)
    /// - Cytoscape (import as network)
    /// - NetworkX (read with nx.read_edgelist())
    /// - igraph (read with read_graph())
    /// - pandas/R (data frame analysis)
    ///
    /// # Arguments
    /// * `path_prefix` - File path prefix (e.g., "output/network" creates "output/network_network_nodes.csv")
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```no_run
    /// # use simulation_framework::{SimulationEngine, SimulationConfig};
    /// # let config = SimulationConfig::default();
    /// # let mut engine = SimulationEngine::new(config);
    /// let result = engine.run();
    /// result.save_trading_network_csv("results").unwrap();
    /// // Creates: results_network_nodes.csv and results_network_edges.csv
    /// ```
    pub fn save_trading_network_csv(&self, path_prefix: &str) -> Result<()> {
        let network = self.export_trading_network();

        // Save nodes
        let nodes_path = format!("{}_network_nodes.csv", path_prefix);
        let mut nodes_file = File::create(&nodes_path)?;
        writeln!(
            nodes_file,
            "id,money,reputation,trade_count,unique_partners"
        )?;
        for node in &network.nodes {
            writeln!(
                nodes_file,
                "{},{:.4},{:.6},{},{}",
                node.id, node.money, node.reputation, node.trade_count, node.unique_partners
            )?;
        }

        // Save edges
        let edges_path = format!("{}_network_edges.csv", path_prefix);
        let mut edges_file = File::create(&edges_path)?;
        writeln!(edges_file, "source,target,weight,total_value")?;
        for edge in &network.edges {
            writeln!(
                edges_file,
                "{},{},{},{:.4}",
                edge.source, edge.target, edge.weight, edge.total_value
            )?;
        }

        Ok(())
    }
}

/// Calculate the Gini coefficient for a given distribution of values.
///
/// The Gini coefficient is a measure of inequality ranging from 0 (perfect equality)
/// to 1 (perfect inequality). Values above 1 can occur when negative values exist.
///
/// # Arguments
/// * `sorted_values` - A slice of values sorted in ascending order
/// * `sum` - The sum of all values
///
/// # Formula
/// G = (2 * sum(i * x_i)) / (n * sum(x_i)) - (n + 1) / n
/// where x_i are sorted values and i is the rank (1-indexed)
///
/// # Returns
/// The Gini coefficient as f64
pub fn calculate_gini_coefficient(sorted_values: &[f64], sum: f64) -> f64 {
    if sorted_values.is_empty() || sum == 0.0 {
        return 0.0;
    }

    let n = sorted_values.len();

    // Parallelize the weighted sum calculation for large datasets
    // Use parallel iterator when we have enough data to benefit from parallelization
    // Note: Code duplication between parallel/sequential branches is intentional for performance.
    // Extracting to a helper would require dynamic dispatch or trait objects, adding overhead.
    // This pattern is idiomatic for conditional parallelization with Rayon.
    let weighted_sum: f64 = if n > 1000 {
        sorted_values
            .par_iter()
            .enumerate()
            .map(|(i, &value)| (i + 1) as f64 * value)
            .sum()
    } else {
        sorted_values
            .iter()
            .enumerate()
            .map(|(i, &value)| (i + 1) as f64 * value)
            .sum()
    };

    (2.0 * weighted_sum) / (n as f64 * sum) - (n as f64 + 1.0) / n as f64
}

/// Calculate the Herfindahl-Hirschman Index (HHI) for a given distribution of values.
///
/// The HHI measures market concentration by summing the squared market shares.
/// It ranges from near 0 (perfect competition with many equal participants) to 10,000 (monopoly).
///
/// # Interpretation
/// * HHI < 1,500: Competitive market (low concentration)
/// * HHI 1,500-2,500: Moderate concentration
/// * HHI > 2,500: High concentration (potential oligopoly/monopoly concerns)
///
/// # Arguments
/// * `values` - A slice of values representing shares (e.g., money, market share)
///
/// # Formula
/// HHI = sum((share_i * 100)^2) for all i
/// where share_i = value_i / total_value
///
/// # Returns
/// The HHI as f64, scaled to 0-10,000 range
///
/// # Examples
/// ```
/// use simulation_framework::result::calculate_herfindahl_index;
///
/// // Perfect equality (4 participants with 25% each): HHI = 2,500
/// let equal_shares = vec![25.0, 25.0, 25.0, 25.0];
/// let hhi = calculate_herfindahl_index(&equal_shares);
/// assert!((hhi - 2500.0).abs() < 0.1);
///
/// // Monopoly (1 participant with 100%): HHI = 10,000
/// let monopoly = vec![100.0];
/// let hhi = calculate_herfindahl_index(&monopoly);
/// assert!((hhi - 10000.0).abs() < 0.1);
/// ```
pub fn calculate_herfindahl_index(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let total: f64 = values.iter().sum();
    if total == 0.0 {
        return 0.0;
    }

    // Calculate HHI: sum of squared market shares (as percentages)
    // Parallelize for large datasets to improve performance
    if values.len() > 1000 {
        values
            .par_iter()
            .map(|&value| {
                let share_percentage = (value / total) * 100.0;
                share_percentage * share_percentage
            })
            .sum()
    } else {
        values
            .iter()
            .map(|&value| {
                let share_percentage = (value / total) * 100.0;
                share_percentage * share_percentage
            })
            .sum()
    }
}

/// Calculate market concentration metrics for a single skill.
///
/// This function computes comprehensive market power indicators for a specific skill:
/// - **HHI (Herfindahl-Hirschman Index)**: Sum of squared market shares (0-10000 scale)
/// - **CR4 (Concentration Ratio 4)**: Market share of top 4 sellers (0.0-1.0)
/// - **CR8 (Concentration Ratio 8)**: Market share of top 8 sellers (0.0-1.0)  
/// - **Market Structure**: Classification based on HHI thresholds
///
/// # Arguments
/// * `skill_id` - The skill to analyze
/// * `seller_volumes` - HashMap mapping seller IDs to their total trading volume for this skill
///
/// # Returns
/// `SkillMarketConcentration` with all concentration metrics, or `None` if insufficient data
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use simulation_framework::result::calculate_skill_market_concentration;
///
/// let mut volumes = HashMap::new();
/// volumes.insert(0, 100.0);
/// volumes.insert(1, 50.0);
/// volumes.insert(2, 50.0);
/// let concentration = calculate_skill_market_concentration("Programming".to_string(), &volumes).unwrap();
/// assert!(concentration.herfindahl_index > 3000.0); // High concentration
/// ```
pub fn calculate_skill_market_concentration(
    skill_id: SkillId,
    seller_volumes: &HashMap<usize, f64>,
) -> Option<SkillMarketConcentration> {
    if seller_volumes.is_empty() {
        return None;
    }

    // Collect seller volumes into a vector and sort descending
    let mut volumes: Vec<f64> = seller_volumes.values().copied().collect();
    volumes.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let total_volume: f64 = volumes.iter().sum();
    if total_volume == 0.0 {
        return None;
    }

    // Calculate HHI using existing function (operates on raw values)
    let herfindahl_index = calculate_herfindahl_index(&volumes);

    // Calculate CR4 (top 4 sellers' market share)
    let cr4 = if volumes.len() >= 4 {
        volumes.iter().take(4).sum::<f64>() / total_volume
    } else {
        // All sellers included when fewer than 4 exist
        1.0
    };

    // Calculate CR8 (top 8 sellers' market share)
    let cr8 = if volumes.len() >= 8 {
        volumes.iter().take(8).sum::<f64>() / total_volume
    } else {
        // All sellers included when fewer than 8 exist
        1.0
    };

    // Classify market structure based on HHI
    let market_structure = if herfindahl_index < 1500.0 {
        MarketStructure::Competitive
    } else if herfindahl_index <= 2500.0 {
        MarketStructure::ModerateConcentration
    } else {
        MarketStructure::HighConcentration
    };

    Some(SkillMarketConcentration {
        skill_id,
        herfindahl_index,
        cr4,
        cr8,
        market_structure,
        num_sellers: volumes.len(),
        total_volume,
    })
}

/// Calculate wealth concentration ratios for different percentile groups.
///
/// This function computes what share of total wealth is held by different groups:
/// - Top 10% wealthiest persons
/// - Top 1% wealthiest persons  
/// - Bottom 50% of persons
///
/// These metrics provide intuitive measures of wealth inequality that complement
/// the Gini coefficient. High values for top groups and low values for bottom groups
/// indicate high inequality.
///
/// # Arguments
/// * `sorted_values` - A slice of values sorted in ascending order (poorest to richest)
/// * `sum` - The sum of all values
///
/// # Returns
/// A tuple of (top_10_pct_share, top_1_pct_share, bottom_50_pct_share)
///
/// # Examples
/// ```
/// use simulation_framework::result::calculate_wealth_concentration;
///
/// // Perfect equality: each group holds wealth proportional to size
/// let equal = vec![100.0; 100];
/// let sum: f64 = equal.iter().sum();
/// let (top10, top1, bottom50) = calculate_wealth_concentration(&equal, sum);
/// assert!((top10 - 0.1).abs() < 0.01); // Top 10% holds 10%
/// assert!((top1 - 0.01).abs() < 0.01); // Top 1% holds 1%
/// assert!((bottom50 - 0.5).abs() < 0.01); // Bottom 50% holds 50%
/// ```
pub fn calculate_wealth_concentration(sorted_values: &[f64], sum: f64) -> (f64, f64, f64) {
    if sorted_values.is_empty() || sum == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let n = sorted_values.len();

    // Calculate index boundaries for each group
    // Note: sorted_values is ascending (poorest to richest)
    // Use max(1, ...) to ensure at least one person in top groups for small populations
    let top_10_pct_count = (n as f64 * 0.1).ceil() as usize;
    let top_10_pct_start_idx = n.saturating_sub(top_10_pct_count);

    let top_1_pct_count = ((n as f64 * 0.01).ceil() as usize).max(1);
    let top_1_pct_start_idx = n.saturating_sub(top_1_pct_count);

    let bottom_50_pct_count = (n as f64 * 0.5).ceil() as usize;
    let bottom_50_pct_end_idx = bottom_50_pct_count.min(n);

    // Sum wealth for each group
    // Parallelize summing for large datasets
    let (top_10_pct_wealth, top_1_pct_wealth, bottom_50_pct_wealth): (f64, f64, f64) = if n > 1000 {
        (
            sorted_values[top_10_pct_start_idx..].par_iter().sum(),
            sorted_values[top_1_pct_start_idx..].par_iter().sum(),
            sorted_values[..bottom_50_pct_end_idx].par_iter().sum(),
        )
    } else {
        (
            sorted_values[top_10_pct_start_idx..].iter().sum(),
            sorted_values[top_1_pct_start_idx..].iter().sum(),
            sorted_values[..bottom_50_pct_end_idx].iter().sum(),
        )
    };

    // Calculate shares as fractions of total wealth
    let top_10_pct_share = top_10_pct_wealth / sum;
    let top_1_pct_share = top_1_pct_wealth / sum;
    let bottom_50_pct_share = bottom_50_pct_wealth / sum;

    (top_10_pct_share, top_1_pct_share, bottom_50_pct_share)
}

/// SIMD-optimized sum calculation using chunked processing.
///
/// This function processes the array in chunks of 4 f64 elements (256 bits total),
/// enabling the compiler to auto-vectorize the loop using SIMD instructions:
/// - x86-64: SSE (128-bit) or AVX (256-bit) instructions
/// - ARM: NEON (128-bit) instructions
///
/// The chunked approach improves cache locality and instruction-level parallelism.
///
/// # Arguments
/// * `values` - Slice of f64 values to sum
///
/// # Returns
/// * Sum of all values as f64
///
/// # Performance
/// On modern CPUs with SIMD support, this achieves ~2-4x speedup over naive iteration
/// for arrays larger than 100 elements due to vectorization and reduced loop overhead.
#[inline]
fn simd_optimized_sum(values: &[f64]) -> f64 {
    // Use 4-way accumulation to enable SIMD auto-vectorization
    // 4 f64 elements = 256 bits, suitable for AVX or two SSE operations
    let (chunks, remainder) = values.as_chunks::<4>();

    // Process 4 elements at a time - enables AVX/SSE vectorization
    let chunk_sum: f64 = chunks.iter().map(|&[a, b, c, d]| a + b + c + d).sum();

    // Handle remaining elements
    let remainder_sum: f64 = remainder.iter().sum();

    chunk_sum + remainder_sum
}

/// SIMD-optimized variance calculation using chunked processing.
///
/// Similar to `simd_optimized_sum`, this function uses 4-way chunking to enable
/// compiler auto-vectorization of the variance calculation loop.
///
/// # Arguments
/// * `values` - Slice of f64 values
/// * `mean` - Pre-calculated mean value
///
/// # Returns
/// * Sum of squared differences from the mean
#[inline]
fn simd_optimized_variance_sum(values: &[f64], mean: f64) -> f64 {
    let (chunks, remainder) = values.as_chunks::<4>();

    // Vectorized variance calculation - 4 elements at a time
    let chunk_var: f64 = chunks
        .iter()
        .map(|&[a, b, c, d]| {
            let da = a - mean;
            let db = b - mean;
            let dc = c - mean;
            let dd = d - mean;
            da * da + db * db + dc * dc + dd * dd
        })
        .sum();

    let remainder_var: f64 = remainder.iter().map(|v| (v - mean).powi(2)).sum();

    chunk_var + remainder_var
}

/// Optimal chunk size for parallel processing with SIMD.
/// This value balances SIMD efficiency (4-element chunks) with parallelization overhead.
/// Each chunk of 256 elements provides good work distribution across threads.
const PARALLEL_SIMD_CHUNK_SIZE: usize = 256;

/// Calculate basic statistics (mean, std_dev, min, max, median) for Monte Carlo
/// simulation runs (Monte Carlo or parameter sweeps).
///
/// This function uses SIMD-optimized algorithms for sum and variance calculations,
/// enabling 2-4x performance improvement on modern CPUs with vectorization support.
/// The implementation uses chunked processing that enables compiler auto-vectorization
/// without requiring nightly Rust or explicit SIMD intrinsics.
///
/// # Arguments
/// * `values` - Slice of f64 values to analyze
///
/// # Returns
/// * `MonteCarloStats` - Statistics including mean, std_dev, min, max, median
///
/// # Performance
/// - For small arrays (< 100 elements): Similar performance to scalar code
/// - For medium arrays (100-1000 elements): ~2x speedup via SIMD auto-vectorization
/// - For large arrays (> 1000 elements): Uses parallel computation with Rayon + SIMD
///
/// # Examples
/// ```
/// use simulation_framework::result::calculate_statistics;
///
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let stats = calculate_statistics(&values);
/// assert_eq!(stats.mean, 3.0);
/// assert_eq!(stats.median, 3.0);
/// ```
pub fn calculate_statistics(values: &[f64]) -> MonteCarloStats {
    if values.is_empty() {
        return MonteCarloStats {
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
        };
    }

    // SIMD-optimized mean calculation
    let mean = simd_optimized_sum(values) / values.len() as f64;

    // Use sample standard deviation (N-1) for finite samples in Monte Carlo analysis
    let variance = if values.len() > 1 {
        let variance_sum = if values.len() > 1000 {
            // For large datasets: combine Rayon parallelization with SIMD
            // Each parallel chunk uses SIMD-optimized variance calculation
            values
                .par_chunks(PARALLEL_SIMD_CHUNK_SIZE)
                .map(|chunk| simd_optimized_variance_sum(chunk, mean))
                .sum::<f64>()
        } else {
            // For smaller datasets: pure SIMD optimization without parallelization overhead
            simd_optimized_variance_sum(values, mean)
        };
        variance_sum / (values.len() - 1) as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    };

    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);

    MonteCarloStats {
        mean,
        std_dev,
        min,
        max,
        median,
    }
}

/// Calculate trading partner statistics from entities' transaction history
///
/// This function analyzes the transaction history of all persons to identify
/// trading relationships, partner counts, and network metrics.
///
/// # Arguments
/// * `entities` - Slice of all entities (persons) in the simulation
///
/// # Returns
/// * `TradingPartnerStats` - Complete trading partner statistics
pub fn calculate_trading_partner_statistics(entities: &[Entity]) -> TradingPartnerStats {
    use std::collections::{HashMap, HashSet};

    // Type alias to simplify complex nested HashMap
    // Maps person_id to (buyer_count, seller_count, partner_map)
    // where partner_map is partner_id -> (trade_count, total_value)
    type PersonTradeData = (usize, usize, HashMap<usize, (usize, f64)>);

    let active_entities: Vec<_> = entities.iter().filter(|e| e.active).collect();
    let num_persons = active_entities.len();

    // Track partners and trade details for each person
    let mut person_stats: HashMap<usize, PersonTradeData> = HashMap::new();

    // Track all unique pairs for network density calculation
    let mut unique_pairs: HashSet<(usize, usize)> = HashSet::new();

    // Process each entity's transaction history
    for entity in &active_entities {
        let person_id = entity.person_data.id;
        let entry = person_stats
            .entry(person_id)
            .or_insert((0, 0, HashMap::new()));

        for transaction in &entity.person_data.transaction_history {
            if let Some(partner_id) = transaction.counterparty_id {
                // Record trade based on type
                match transaction.transaction_type {
                    crate::person::TransactionType::Buy => {
                        entry.0 += 1; // Increment buyer count
                    }
                    crate::person::TransactionType::Sell => {
                        entry.1 += 1; // Increment seller count
                    }
                }

                // Track partner relationship
                let partner_entry = entry.2.entry(partner_id).or_insert((0, 0.0));
                partner_entry.0 += 1; // Increment trade count with this partner
                partner_entry.1 += transaction.amount; // Add trade value

                // Add to unique pairs (normalize pair ordering)
                let pair = if person_id < partner_id {
                    (person_id, partner_id)
                } else {
                    (partner_id, person_id)
                };
                unique_pairs.insert(pair);
            }
        }
    }

    // Build per-person statistics
    // Parallelize computation when we have many entities (>100) for better performance
    // Note: person_stats HashMap is read-only here (no writes), so .get().cloned() is thread-safe
    let per_person: Vec<PersonTradingStats> = if active_entities.len() > 100 {
        let mut stats: Vec<PersonTradingStats> = active_entities
            .par_iter()
            .map(|entity| {
                let person_id = entity.person_data.id;
                // Thread-safe read-only access to HashMap via .get().cloned()
                let (buyer_count, seller_count, partners) =
                    person_stats.get(&person_id).cloned().unwrap_or_default();

                // Sort partners by trade count (descending) and take top 5
                let mut partner_list: Vec<(usize, usize, f64)> = partners
                    .iter()
                    .map(|(&id, &(count, value))| (id, count, value))
                    .collect();
                partner_list.sort_by(|a, b| b.1.cmp(&a.1));

                let top_partners: Vec<PartnerInfo> = partner_list
                    .iter()
                    .take(5)
                    .map(|&(id, count, value)| PartnerInfo {
                        partner_id: id,
                        trade_count: count,
                        total_value: value,
                    })
                    .collect();

                PersonTradingStats {
                    person_id,
                    unique_partners: partners.len(),
                    total_trades_as_buyer: buyer_count,
                    total_trades_as_seller: seller_count,
                    top_partners,
                }
            })
            .collect();
        // Sort by person_id for consistent, deterministic output
        stats.sort_by_key(|s| s.person_id);
        stats
    } else {
        let mut stats = Vec::new();
        for entity in &active_entities {
            let person_id = entity.person_data.id;
            let (buyer_count, seller_count, partners) =
                person_stats.get(&person_id).cloned().unwrap_or_default();

            // Sort partners by trade count (descending) and take top 5
            let mut partner_list: Vec<(usize, usize, f64)> = partners
                .iter()
                .map(|(&id, &(count, value))| (id, count, value))
                .collect();
            partner_list.sort_by(|a, b| b.1.cmp(&a.1));

            let top_partners: Vec<PartnerInfo> = partner_list
                .iter()
                .take(5)
                .map(|&(id, count, value)| PartnerInfo {
                    partner_id: id,
                    trade_count: count,
                    total_value: value,
                })
                .collect();

            stats.push(PersonTradingStats {
                person_id,
                unique_partners: partners.len(),
                total_trades_as_buyer: buyer_count,
                total_trades_as_seller: seller_count,
                top_partners,
            });
        }
        // Sort by person_id for consistent output
        stats.sort_by_key(|s| s.person_id);
        stats
    };

    // Calculate network metrics
    let avg_unique_partners = if !per_person.is_empty() {
        per_person.iter().map(|s| s.unique_partners).sum::<usize>() as f64 / per_person.len() as f64
    } else {
        0.0
    };

    // Network density = (actual connections) / (possible connections)
    // Possible connections = n * (n - 1) / 2 for undirected network
    let possible_connections = if num_persons > 1 {
        (num_persons * (num_persons - 1)) / 2
    } else {
        1
    };
    let network_density = if possible_connections > 0 {
        unique_pairs.len() as f64 / possible_connections as f64
    } else {
        0.0
    };

    // Find most active trading pair
    let most_active_pair = if !unique_pairs.is_empty() {
        // Count trades for each pair
        let mut pair_trade_counts: HashMap<(usize, usize), usize> = HashMap::new();

        for entity in &active_entities {
            for transaction in &entity.person_data.transaction_history {
                if let Some(partner_id) = transaction.counterparty_id {
                    let person_id = entity.person_data.id;
                    let pair = if person_id < partner_id {
                        (person_id, partner_id)
                    } else {
                        (partner_id, person_id)
                    };
                    *pair_trade_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        // Find pair with maximum trades
        pair_trade_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|((p1, p2), count)| (p1, p2, count))
    } else {
        None
    };

    TradingPartnerStats {
        per_person,
        network_metrics: NetworkMetrics {
            avg_unique_partners,
            network_density,
            most_active_pair,
        },
    }
}

/// Write step data to a JSONL (JSON Lines) stream file
///
/// This function appends a single line of JSON data to the streaming output file.
/// Each line represents one simulation step and contains key metrics for monitoring.
///
/// # Arguments
/// * `writer` - A mutable reference to the BufWriter for the output file
/// * `step_data` - The step data to write
///
/// # Returns
/// * `Result<()>` - Ok if successful, or an error if writing failed
pub fn write_step_to_stream(writer: &mut BufWriter<File>, step_data: &StepData) -> Result<()> {
    let json = serde_json::to_string(step_data)
        .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;
    writeln!(writer, "{}", json).map_err(SimulationError::IoError)?;
    writer.flush().map_err(SimulationError::IoError)?;
    Ok(())
}

impl MonteCarloResult {
    /// Create a new MonteCarloResult from a collection of simulation results
    pub fn from_runs(runs: Vec<SimulationResult>, base_seed: u64) -> Self {
        let num_runs = runs.len();

        // Collect metrics from each run
        let avg_moneys: Vec<f64> = runs.iter().map(|r| r.money_statistics.average).collect();
        let gini_coefficients: Vec<f64> = runs
            .iter()
            .map(|r| r.money_statistics.gini_coefficient)
            .collect();
        let total_trades: Vec<f64> = runs
            .iter()
            .map(|r| r.trade_volume_statistics.total_trades as f64)
            .collect();
        let avg_reputations: Vec<f64> = runs
            .iter()
            .map(|r| r.reputation_statistics.average)
            .collect();

        Self {
            num_runs,
            base_seed,
            runs,
            avg_money_stats: calculate_statistics(&avg_moneys),
            gini_coefficient_stats: calculate_statistics(&gini_coefficients),
            total_trades_stats: calculate_statistics(&total_trades),
            avg_reputation_stats: calculate_statistics(&avg_reputations),
        }
    }

    /// Save Monte Carlo results to a JSON file
    pub fn save_to_file(&self, path: &str, compress: bool) -> Result<()> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;

        if compress {
            let file = File::create(format!("{path}.gz"))?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(json_str.as_bytes())?;
            encoder.finish()?;
        } else {
            let mut file = File::create(path)?;
            file.write_all(json_str.as_bytes())?;
        }

        Ok(())
    }

    /// Print a summary of the Monte Carlo results to the console
    pub fn print_summary(&self) {
        println!(
            "\n{}",
            "=== Monte Carlo Simulation Results ==="
                .bright_cyan()
                .bold()
        );
        println!("Number of runs: {}", self.num_runs);
        println!("Base seed: {}\n", self.base_seed);

        println!("{}", "Average Money Across Runs:".bright_yellow());
        Self::print_stat_summary(&self.avg_money_stats);

        println!("\n{}", "Gini Coefficient Across Runs:".bright_yellow());
        Self::print_stat_summary(&self.gini_coefficient_stats);

        println!("\n{}", "Total Trades Across Runs:".bright_yellow());
        Self::print_stat_summary(&self.total_trades_stats);

        println!("\n{}", "Average Reputation Across Runs:".bright_yellow());
        Self::print_stat_summary(&self.avg_reputation_stats);
    }

    fn print_stat_summary(stats: &MonteCarloStats) {
        println!("  Mean:   {:.2}", stats.mean);
        println!("  Median: {:.2}", stats.median);
        println!("  StdDev: {:.2}", stats.std_dev);
        println!("  Min:    {:.2}", stats.min);
        println!("  Max:    {:.2}", stats.max);
    }
}

/// Calculate social mobility statistics from quintile tracking data.
///
/// Analyzes how persons move between wealth quintiles (bottom 20%, second 20%, etc.)
/// over the simulation, calculating transition probabilities and mobility metrics.
///
/// # Arguments
/// * `mobility_quintiles` - Map of person_id to vector of quintile assignments (0-4) at each step
///
/// # Returns
/// * `Option<MobilityStatistics>` - Mobility statistics if sufficient data exists (at least 2 steps)
pub fn calculate_mobility_statistics(
    mobility_quintiles: &HashMap<usize, Vec<usize>>,
) -> Option<MobilityStatistics> {
    if mobility_quintiles.is_empty() {
        return None;
    }

    // Check if we have at least 2 time points (need transitions)
    let min_length = mobility_quintiles.values().map(|v| v.len()).min()?;
    if min_length < 2 {
        return None;
    }

    // Initialize 5x5 transition count matrix
    let mut transition_counts: Vec<Vec<usize>> = vec![vec![0; 5]; 5];
    let mut total_transitions = 0;
    let mut upward_moves = 0;
    let mut downward_moves = 0;
    let mut same_quintile = 0;
    let mut total_quintile_changes_count = 0;

    // Count transitions for each person
    for quintiles in mobility_quintiles.values() {
        let mut person_changes = 0;
        for i in 0..quintiles.len() - 1 {
            let from_quintile = quintiles[i];
            let to_quintile = quintiles[i + 1];

            transition_counts[from_quintile][to_quintile] += 1;
            total_transitions += 1;

            if to_quintile > from_quintile {
                upward_moves += 1;
                person_changes += 1;
            } else if to_quintile < from_quintile {
                downward_moves += 1;
                person_changes += 1;
            } else {
                same_quintile += 1;
            }
        }
        total_quintile_changes_count += person_changes;
    }

    // Convert counts to probabilities
    let transition_matrix: Vec<Vec<f64>> = transition_counts
        .iter()
        .map(|row| {
            let row_sum: usize = row.iter().sum();
            if row_sum > 0 {
                row.iter()
                    .map(|&count| count as f64 / row_sum as f64)
                    .collect()
            } else {
                vec![0.0; 5]
            }
        })
        .collect();

    let upward_mobility_probability = if total_transitions > 0 {
        upward_moves as f64 / total_transitions as f64
    } else {
        0.0
    };

    let downward_mobility_probability = if total_transitions > 0 {
        downward_moves as f64 / total_transitions as f64
    } else {
        0.0
    };

    let quintile_persistence = if total_transitions > 0 {
        same_quintile as f64 / total_transitions as f64
    } else {
        0.0
    };

    let avg_quintile_changes = if !mobility_quintiles.is_empty() {
        total_quintile_changes_count as f64 / mobility_quintiles.len() as f64
    } else {
        0.0
    };

    Some(MobilityStatistics {
        transition_matrix,
        upward_mobility_probability,
        downward_mobility_probability,
        quintile_persistence,
        avg_quintile_changes,
    })
}

/// Detect business cycles (expansions and contractions) from trade volume data.
///
/// Uses a simple peak/trough detection algorithm to identify periods of economic
/// expansion (increasing trade volume) and contraction (decreasing trade volume).
///
/// # Algorithm
///
/// 1. **Smoothing**: Apply a simple moving average to reduce noise (window size = 3)
/// 2. **Peak/Trough Detection**: Identify local maxima and minima in smoothed data
///    - A peak is a point higher than both neighbors
///    - A trough is a point lower than both neighbors
/// 3. **Cycle Classification**:
///    - Expansion: From trough to peak (volume increasing)
///    - Contraction: From peak to trough (volume decreasing)
/// 4. **Statistics**: Calculate duration and volume metrics for each phase
///
/// # Arguments
///
/// * `volume_per_step` - Vector of trade volumes for each simulation step
///
/// # Returns
///
/// `Some(BusinessCycleStats)` if at least one complete cycle is detected,
/// `None` if the simulation is too short (< 10 steps) or no cycles found.
///
/// # Limitations
///
/// - Requires at least 10 simulation steps for meaningful analysis
/// - Simple algorithm may detect noise as cycles in highly volatile data
/// - Does not use advanced filtering (e.g., Hodrick-Prescott filter)
/// - Best suited for medium to long simulations (100+ steps)
///
/// # Examples
///
/// ```
/// use simulation_framework::result::detect_business_cycles;
///
/// // Simulate a simple cycle: low -> high -> low
/// let volumes = vec![100.0, 120.0, 150.0, 180.0, 170.0, 140.0, 110.0, 90.0, 100.0, 120.0];
/// let cycles = detect_business_cycles(&volumes);
///
/// // Should detect at least one cycle
/// assert!(cycles.is_some());
/// ```
pub fn detect_business_cycles(volume_per_step: &[f64]) -> Option<BusinessCycleStats> {
    // Need at least 10 steps for meaningful cycle detection
    if volume_per_step.len() < 10 {
        return None;
    }

    // Step 1: Apply simple moving average smoothing (window = 3)
    let smoothed = smooth_data(volume_per_step, 3);

    // Step 2: Detect peaks and troughs
    let mut turning_points = Vec::new();

    for i in 1..smoothed.len() - 1 {
        let prev = smoothed[i - 1];
        let curr = smoothed[i];
        let next = smoothed[i + 1];

        // Detect peak (local maximum)
        if curr > prev && curr > next {
            turning_points.push((i, curr, CyclePhase::Contraction)); // Peak starts contraction
        }
        // Detect trough (local minimum)
        else if curr < prev && curr < next {
            turning_points.push((i, curr, CyclePhase::Expansion)); // Trough starts expansion
        }
    }

    // Need at least 2 turning points to form a cycle
    if turning_points.len() < 2 {
        return None;
    }

    // Step 3: Build cycles from turning points
    let mut detected_cycles = Vec::new();

    for i in 0..turning_points.len() - 1 {
        let (start_step, _start_vol, phase) = turning_points[i];
        let (end_step, _end_vol, _) = turning_points[i + 1];

        let duration = end_step - start_step;

        // Calculate average, peak, and trough volume for this phase in a single pass
        let phase_slice = &volume_per_step[start_step..=end_step];
        let (sum, peak, trough) = phase_slice.iter().fold(
            (0.0, f64::NEG_INFINITY, f64::INFINITY),
            |(sum, peak, trough), &val| (sum + val, peak.max(val), trough.min(val)),
        );
        let avg_volume = sum / phase_slice.len() as f64;
        let peak_volume = peak;
        let trough_volume = trough;

        detected_cycles.push(BusinessCycle {
            phase,
            start_step,
            end_step,
            duration,
            avg_volume,
            peak_volume,
            trough_volume,
        });
    }

    // Step 4: Calculate cycle statistics
    if detected_cycles.is_empty() {
        return None;
    }

    let expansions: Vec<&BusinessCycle> = detected_cycles
        .iter()
        .filter(|c| c.phase == CyclePhase::Expansion)
        .collect();

    let contractions: Vec<&BusinessCycle> = detected_cycles
        .iter()
        .filter(|c| c.phase == CyclePhase::Contraction)
        .collect();

    let avg_expansion_duration = if expansions.is_empty() {
        0.0
    } else {
        expansions.iter().map(|c| c.duration as f64).sum::<f64>() / expansions.len() as f64
    };

    let avg_contraction_duration = if contractions.is_empty() {
        0.0
    } else {
        contractions.iter().map(|c| c.duration as f64).sum::<f64>() / contractions.len() as f64
    };

    // Complete cycles are expansion + contraction pairs
    let total_cycles = std::cmp::min(expansions.len(), contractions.len());

    let avg_cycle_duration = if total_cycles > 0 {
        avg_expansion_duration + avg_contraction_duration
    } else {
        0.0
    };

    Some(BusinessCycleStats {
        total_cycles,
        avg_cycle_duration,
        avg_expansion_duration,
        avg_contraction_duration,
        detected_cycles,
    })
}

/// Apply simple moving average smoothing to a time series.
///
/// Reduces noise in the data by averaging each point with its neighbors.
/// The window parameter controls how many neighbors to include on each side.
///
/// # Arguments
///
/// * `data` - The time series data to smooth
/// * `window` - The smoothing window size (should be odd number, typically 3 or 5)
///
/// # Returns
///
/// A new vector with smoothed values. Edge cases use smaller windows.
fn smooth_data(data: &[f64], window: usize) -> Vec<f64> {
    if window < 3 {
        return data.to_vec();
    }

    let half_window = window / 2;
    let mut smoothed = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        let start = i.saturating_sub(half_window);
        let end = (i + half_window + 1).min(data.len());
        let sum: f64 = data[start..end].iter().sum();
        let count = (end - start) as f64;
        smoothed.push(sum / count);
    }

    smoothed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_incremental_stats_empty() {
        let stats = IncrementalStats::new();
        assert_eq!(stats.count(), 0);
        assert_eq!(stats.mean(), 0.0);
        assert_eq!(stats.variance(), 0.0);
        assert_eq!(stats.std_dev(), 0.0);
    }

    #[test]
    fn test_incremental_stats_single_value() {
        let mut stats = IncrementalStats::new();
        stats.update(42.0);
        assert_eq!(stats.count(), 1);
        assert_eq!(stats.mean(), 42.0);
        assert_eq!(stats.variance(), 0.0); // Single value has no variance
        assert_eq!(stats.std_dev(), 0.0);
    }

    #[test]
    fn test_incremental_stats_multiple_values() {
        let mut stats = IncrementalStats::new();
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        for &v in &values {
            stats.update(v);
        }

        // Expected: mean = 30.0, variance = 250.0, std_dev = sqrt(250.0)
        let expected_mean = 30.0;
        let expected_variance = 250.0;
        let expected_std_dev = (250.0_f64).sqrt(); // sqrt(250.0) ≈ 15.811388

        assert_eq!(stats.count(), 5);
        assert_eq!(stats.mean(), expected_mean);
        assert!((stats.variance() - expected_variance).abs() < 1e-10);
        assert!((stats.std_dev() - expected_std_dev).abs() < 1e-10);
    }

    #[test]
    fn test_incremental_stats_vs_batch_calculation() {
        // Test that incremental stats match batch calculation
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        // Incremental
        let mut incremental = IncrementalStats::new();
        for &v in &values {
            incremental.update(v);
        }

        // Batch calculation
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        let variance = values
            .iter()
            .map(|&v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / (count - 1.0);
        let std_dev = variance.sqrt();

        assert_eq!(incremental.mean(), mean);
        assert!((incremental.variance() - variance).abs() < 1e-10);
        assert!((incremental.std_dev() - std_dev).abs() < 1e-10);
    }

    #[test]
    fn test_incremental_stats_reset() {
        let mut stats = IncrementalStats::new();
        stats.update(10.0);
        stats.update(20.0);
        stats.update(30.0);

        assert_eq!(stats.count(), 3);
        assert_eq!(stats.mean(), 20.0);

        stats.reset();

        assert_eq!(stats.count(), 0);
        assert_eq!(stats.mean(), 0.0);
        assert_eq!(stats.variance(), 0.0);
    }

    #[test]
    fn test_incremental_stats_numerical_stability() {
        // Test with values that could cause numerical issues with naive algorithms
        let mut stats = IncrementalStats::new();
        let base = 1e9;
        let values = vec![base + 1.0, base + 2.0, base + 3.0, base + 4.0, base + 5.0];

        for &v in &values {
            stats.update(v);
        }

        // Mean should be base + 3.0
        assert!((stats.mean() - (base + 3.0)).abs() < 1e-6);
        // Variance should be 2.5 (same as [1,2,3,4,5])
        assert!((stats.variance() - 2.5).abs() < 1e-6);
    }

    fn get_test_result() -> SimulationResult {
        SimulationResult {
            metadata: SimulationMetadata::capture(42, 10, 100),
            total_steps: 10,
            total_duration: 1.23,
            step_times: vec![0.1, 0.12, 0.1, 0.13, 0.1, 0.11, 0.1, 0.14, 0.1, 0.13],
            active_persons: 5,
            failed_steps: 0,
            final_money_distribution: vec![50.0, 80.0, 100.0, 120.0, 150.0],
            money_statistics: MoneyStats {
                average: 100.0,
                median: 100.0,
                std_dev: 31.62,
                min_money: 50.0,
                max_money: 150.0,
                gini_coefficient: 0.2,
                herfindahl_index: 2200.0,
                top_10_percent_share: 0.3,
                top_1_percent_share: 0.15,
                bottom_50_percent_share: 0.25,
            },
            final_reputation_distribution: vec![0.95, 1.0, 1.0, 1.05, 1.1],
            reputation_statistics: ReputationStats {
                average: 1.02,
                median: 1.0,
                std_dev: 0.05,
                min_reputation: 0.95,
                max_reputation: 1.1,
            },
            final_savings_distribution: vec![0.0, 5.0, 10.0, 15.0, 20.0],
            savings_statistics: SavingsStats {
                total_savings: 50.0,
                average_savings: 10.0,
                median_savings: 10.0,
                min_savings: 0.0,
                max_savings: 20.0,
            },
            credit_score_statistics: None,
            final_skill_prices: vec![],
            most_valuable_skill: None,
            least_valuable_skill: None,
            skill_price_history: HashMap::new(),
            wealth_stats_history: Vec::new(),
            trade_volume_statistics: TradeVolumeStats {
                total_trades: 100,
                total_volume: 1000.0,
                avg_trades_per_step: 10.0,
                avg_volume_per_step: 100.0,
                avg_transaction_value: 10.0,
                min_trades_per_step: 5,
                max_trades_per_step: 15,
                velocity_of_money: 2.0,
            },
            trades_per_step: vec![10, 12, 8, 10, 15, 9, 11, 10, 5, 10],
            volume_per_step: vec![
                100.0, 120.0, 80.0, 100.0, 150.0, 90.0, 110.0, 100.0, 50.0, 100.0,
            ],
            total_fees_collected: 0.0,
            per_skill_trade_stats: vec![],
            skill_market_concentration: None,
            business_cycle_statistics: None,
            failed_trade_statistics: FailedTradeStats {
                total_failed_attempts: 20,
                failure_rate: 0.1667, // 20 / (100 + 20) = 0.1667
                avg_failed_per_step: 2.0,
                min_failed_per_step: 0,
                max_failed_per_step: 5,
            },
            failed_attempts_per_step: vec![2, 3, 1, 2, 5, 1, 2, 2, 0, 2],
            black_market_statistics: None,
            total_taxes_collected: None,
            total_taxes_redistributed: None,
            loan_statistics: None,
            investment_statistics: None,
            contract_statistics: None,
            education_statistics: None,
            mentorship_statistics: None,
            certification_statistics: None,
            environment_statistics: None,
            friendship_statistics: None,
            trust_network_statistics: None,
            trade_agreement_statistics: None,
            insurance_statistics: None,
            technology_breakthrough_statistics: None,
            group_statistics: None,
            trading_partner_statistics: TradingPartnerStats {
                per_person: vec![],
                network_metrics: NetworkMetrics {
                    avg_unique_partners: 0.0,
                    network_density: 0.0,
                    most_active_pair: None,
                },
            },
            centrality_analysis: None,
            mobility_statistics: None,
            quality_statistics: None,
            events: None,
            final_persons_data: vec![],
        }
    }

    /// Helper function to reduce code duplication in CSV export tests.
    /// Creates a temp directory, saves CSV files, and reads a specific CSV file.
    ///
    /// # Arguments
    /// * `result` - The SimulationResult to save
    /// * `file_suffix` - The suffix for the CSV file to read (e.g., "summary", "money")
    ///
    /// # Returns
    /// The contents of the specified CSV file as a String
    fn read_csv_file_from_test(result: &SimulationResult, file_suffix: &str) -> String {
        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        let file_path = format!("{}_{}.csv", path_prefix, file_suffix);
        let mut contents = String::new();
        File::open(&file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        contents
    }

    #[test]
    fn test_print_summary() {
        let result = get_test_result();
        // This test just checks that print_summary doesn't panic.
        // Test with histogram enabled
        result.print_summary(true);
        // Test with histogram disabled
        result.print_summary(false);
    }

    #[test]
    fn test_print_summary_with_nan_values() {
        // Test that NaN values don't cause panics when sorting in print_wealth_histogram
        let mut result = get_test_result();

        // Add some NaN values to the money distribution
        result.final_money_distribution = vec![100.0, 200.0, f64::NAN, 300.0, f64::NAN, 400.0];

        // This should not panic - NaN values should be handled gracefully
        result.print_summary(true);
    }

    #[test]
    fn test_print_summary_with_infinity_values() {
        // Test that Infinity values don't cause panics when sorting
        let mut result = get_test_result();

        // Add some Infinity values to the money distribution
        result.final_money_distribution =
            vec![100.0, f64::INFINITY, 200.0, f64::NEG_INFINITY, 300.0];

        // This should not panic - Infinity values should be handled gracefully
        result.print_summary(true);
    }

    #[test]
    fn test_save_to_file() {
        let result = get_test_result();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        result.save_to_file(path, false).unwrap();

        let mut contents = String::new();
        file.reopen()
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
    }

    #[test]
    fn test_save_to_file_compressed() {
        use flate2::read::GzDecoder;

        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_output.json");
        let path_str = path.to_str().unwrap();

        // Save compressed file
        result.save_to_file(path_str, true).unwrap();

        // Verify .gz file was created
        let gz_path = format!("{}.gz", path_str);
        assert!(std::path::Path::new(&gz_path).exists());

        // Decompress and verify contents
        let file = File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
        assert!(contents.contains("\"active_persons\": 5"));
    }

    #[test]
    fn test_save_to_file_compressed_with_gz_extension() {
        use flate2::read::GzDecoder;

        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_output.json.gz");
        let path_str = path.to_str().unwrap();

        // Save compressed file with .gz already in the path
        result.save_to_file(path_str, true).unwrap();

        // Verify file was created without double .gz extension
        assert!(std::path::Path::new(path_str).exists());
        let double_gz_path = format!("{}.gz", path_str);
        assert!(!std::path::Path::new(&double_gz_path).exists());

        // Decompress and verify contents
        let file = File::open(path_str).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
    }

    #[test]
    fn test_compressed_file_is_smaller() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();

        // Save uncompressed
        let uncompressed_path = temp_dir.path().join("uncompressed.json");
        result
            .save_to_file(uncompressed_path.to_str().unwrap(), false)
            .unwrap();

        // Save compressed
        let compressed_path = temp_dir.path().join("compressed.json");
        result
            .save_to_file(compressed_path.to_str().unwrap(), true)
            .unwrap();
        let compressed_gz_path = format!("{}.gz", compressed_path.to_str().unwrap());

        // Compare file sizes
        let uncompressed_size = std::fs::metadata(&uncompressed_path).unwrap().len();
        let compressed_size = std::fs::metadata(&compressed_gz_path).unwrap().len();

        // Compressed should be smaller (for this test data)
        assert!(
            compressed_size < uncompressed_size,
            "Compressed size {} should be less than uncompressed size {}",
            compressed_size,
            uncompressed_size
        );
    }

    fn calculate_money_stats(money_values: &[f64]) -> MoneyStats {
        if money_values.is_empty() {
            return MoneyStats {
                average: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min_money: 0.0,
                max_money: 0.0,
                gini_coefficient: 0.0,
                herfindahl_index: 0.0,
                top_10_percent_share: 0.0,
                top_1_percent_share: 0.0,
                bottom_50_percent_share: 0.0,
            };
        }

        let mut sorted_money = money_values.to_vec();
        sorted_money.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = sorted_money.iter().sum();
        let count = sorted_money.len() as f64;
        let average = sum / count;

        let median = if count > 0.0 {
            if count as usize % 2 == 1 {
                sorted_money[count as usize / 2]
            } else {
                (sorted_money[count as usize / 2 - 1] + sorted_money[count as usize / 2]) / 2.0
            }
        } else {
            0.0
        };

        let variance = sorted_money
            .iter()
            .map(|value| {
                let diff = average - value;
                diff * diff
            })
            .sum::<f64>()
            / count;
        let std_dev = variance.sqrt();

        // Calculate Gini coefficient using the shared utility function
        let gini_coefficient = calculate_gini_coefficient(&sorted_money, sum);

        // Calculate Herfindahl Index using the shared utility function
        let herfindahl_index = calculate_herfindahl_index(&sorted_money);

        // Calculate wealth concentration ratios
        let (top_10_percent_share, top_1_percent_share, bottom_50_percent_share) =
            calculate_wealth_concentration(&sorted_money, sum);

        MoneyStats {
            average,
            median,
            std_dev,
            min_money: *sorted_money.first().unwrap_or(&0.0),
            max_money: *sorted_money.last().unwrap_or(&0.0),
            gini_coefficient,
            herfindahl_index,
            top_10_percent_share,
            top_1_percent_share,
            bottom_50_percent_share,
        }
    }

    #[test]
    fn test_money_stats_empty() {
        let stats = calculate_money_stats(&[]);
        assert_eq!(stats.average, 0.0);
        assert_eq!(stats.median, 0.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min_money, 0.0);
        assert_eq!(stats.max_money, 0.0);
        assert_eq!(stats.gini_coefficient, 0.0);
        assert_eq!(stats.herfindahl_index, 0.0);
        assert_eq!(stats.top_10_percent_share, 0.0);
        assert_eq!(stats.top_1_percent_share, 0.0);
        assert_eq!(stats.bottom_50_percent_share, 0.0);
    }

    #[test]
    fn test_money_stats_single_value() {
        let stats = calculate_money_stats(&[100.0]);
        assert_eq!(stats.average, 100.0);
        assert_eq!(stats.median, 100.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min_money, 100.0);
        assert_eq!(stats.max_money, 100.0);
        assert_eq!(stats.gini_coefficient, 0.0);
        // Single person = monopoly = HHI of 10,000
        assert!((stats.herfindahl_index - 10000.0).abs() < 0.1);
        // Single person holds 100% of wealth
        assert!((stats.top_10_percent_share - 1.0).abs() < 1e-10);
        assert!((stats.top_1_percent_share - 1.0).abs() < 1e-10);
        assert!((stats.bottom_50_percent_share - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_money_stats_multiple_values_odd() {
        let money = [10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.average, 30.0);
        assert_eq!(stats.median, 30.0);
        assert_eq!(stats.min_money, 10.0);
        assert_eq!(stats.max_money, 50.0);
        // Std dev for [10,20,30,40,50] is sqrt(((20^2 + 10^2 + 0^2 + 10^2 + 20^2)/5)) = sqrt((400+100+0+100+400)/5) = sqrt(1000/5) = sqrt(200) = 14.1421356
        assert!((stats.std_dev - 14.1421356).abs() < 1e-6);
    }

    #[test]
    fn test_money_stats_multiple_values_even() {
        let money = [10.0, 20.0, 30.0, 60.0]; // Avg = 30, Median = 25
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.average, 30.0);
        assert_eq!(stats.median, 25.0); // (20+30)/2
        assert_eq!(stats.min_money, 10.0);
        assert_eq!(stats.max_money, 60.0);
        // Std dev for [10,20,30,60] (avg 30) is sqrt(((20^2 + 10^2 + 0^2 + 30^2)/4)) = sqrt((400+100+0+900)/4) = sqrt(1400/4) = sqrt(350) = 18.7082869
        assert!((stats.std_dev - 18.7082869).abs() < 1e-6);
    }

    #[test]
    fn test_gini_coefficient_perfect_equality() {
        // All persons have equal money - should be 0 (perfect equality)
        let money = [100.0, 100.0, 100.0, 100.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_perfect_inequality() {
        // One person has all money, others have nothing - should be close to 1
        let money = [0.0, 0.0, 0.0, 100.0];
        let stats = calculate_money_stats(&money);
        // For n=4, perfect inequality Gini = (n-1)/n = 3/4 = 0.75
        assert!((stats.gini_coefficient - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_gini_coefficient_moderate_inequality() {
        // Some inequality but not extreme
        let money = [10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = calculate_money_stats(&money);
        // For linearly increasing values, Gini should be around 0.2667
        // G = (2 * sum(i * x_i)) / (n * sum(x_i)) - (n + 1) / n
        // sum(i * x_i) = 1*10 + 2*20 + 3*30 + 4*40 + 5*50 = 10 + 40 + 90 + 160 + 250 = 550
        // sum(x_i) = 150, n = 5
        // G = (2 * 550) / (5 * 150) - 6 / 5 = 1100 / 750 - 1.2 = 1.4667 - 1.2 = 0.2667
        assert!((stats.gini_coefficient - 0.26666667).abs() < 1e-6);
    }

    #[test]
    fn test_gini_coefficient_empty_distribution() {
        let money: Vec<f64> = vec![];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_single_person() {
        let money = [100.0];
        let stats = calculate_money_stats(&money);
        // Single person: Gini should be 0 (no inequality possible)
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_two_persons_equal() {
        let money = [50.0, 50.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_two_persons_unequal() {
        let money = [25.0, 75.0];
        let stats = calculate_money_stats(&money);
        // For n=2: G = (2 * (1*25 + 2*75)) / (2 * 100) - 3/2
        // = (2 * (25 + 150)) / 200 - 1.5 = 350 / 200 - 1.5 = 1.75 - 1.5 = 0.25
        assert!((stats.gini_coefficient - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_wealth_concentration_perfect_equality() {
        // 100 persons with equal wealth
        let equal = vec![100.0; 100];
        let sum: f64 = equal.iter().sum();
        let (top10, top1, bottom50) = calculate_wealth_concentration(&equal, sum);

        // With perfect equality, each group holds wealth proportional to its size
        assert!((top10 - 0.1).abs() < 0.01); // Top 10% holds ~10%
        assert!((top1 - 0.01).abs() < 0.01); // Top 1% holds ~1%
        assert!((bottom50 - 0.5).abs() < 0.01); // Bottom 50% holds ~50%
    }

    #[test]
    fn test_wealth_concentration_high_inequality() {
        // High inequality: top person has much more
        let mut values = vec![10.0; 99]; // 99 persons with 10 each
        values.push(1000.0); // 1 person with 1000
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f64 = values.iter().sum();

        let (top10, top1, bottom50) = calculate_wealth_concentration(&values, sum);

        // Top 1% (1 person) should hold significant wealth
        // Total wealth = 99*10 + 1000 = 1990
        // Top 1% wealth = 1000, share = 1000/1990 ≈ 0.503
        assert!((top1 - 0.5025).abs() < 0.01);

        // Top 10% (10 persons including the richest)
        // Top 10% wealth = 9*10 + 1000 = 1090, share = 1090/1990 ≈ 0.548
        assert!((top10 - 0.5477).abs() < 0.01);

        // Bottom 50% (50 persons with 10 each)
        // Bottom 50% wealth = 50*10 = 500, share = 500/1990 ≈ 0.251
        assert!((bottom50 - 0.2513).abs() < 0.01);
    }

    #[test]
    fn test_wealth_concentration_empty() {
        let (top10, top1, bottom50) = calculate_wealth_concentration(&[], 0.0);
        assert_eq!(top10, 0.0);
        assert_eq!(top1, 0.0);
        assert_eq!(bottom50, 0.0);
    }

    #[test]
    fn test_wealth_concentration_single_person() {
        let values = vec![100.0];
        let sum: f64 = values.iter().sum();
        let (top10, top1, bottom50) = calculate_wealth_concentration(&values, sum);

        // Single person is in all groups
        assert!((top10 - 1.0).abs() < 1e-10);
        assert!((top1 - 1.0).abs() < 1e-10);
        assert!((bottom50 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_wealth_concentration_small_population() {
        // 10 persons with ascending wealth
        let values: Vec<f64> = (1..=10).map(|x| x as f64 * 10.0).collect();
        let sum: f64 = values.iter().sum(); // 550

        let (top10, top1, bottom50) = calculate_wealth_concentration(&values, sum);

        // Top 10% = top 1 person = 100, share = 100/550 ≈ 0.182
        assert!((top10 - 0.1818).abs() < 0.01);

        // Top 1% = top 1 person (ceil(10 * 0.01) = 1) = 100, share = 100/550 ≈ 0.182
        assert!((top1 - 0.1818).abs() < 0.01);

        // Bottom 50% = bottom 5 persons = 10+20+30+40+50 = 150, share = 150/550 ≈ 0.273
        assert!((bottom50 - 0.2727).abs() < 0.01);
    }

    #[test]
    fn test_save_to_csv_summary() {
        let result = get_test_result();
        let contents = read_csv_file_from_test(&result, "summary");

        // Check that summary file was created and contains expected content
        assert!(contents.contains("Metric,Value"));
        assert!(contents.contains("Total Steps,10"));
        assert!(contents.contains("Active Persons,5"));
        assert!(contents.contains("Average Money,100"));
        assert!(contents.contains("Gini Coefficient,0.2"));
    }

    #[test]
    fn test_save_to_csv_money_distribution() {
        let result = get_test_result();
        let contents = read_csv_file_from_test(&result, "money");

        // Check money distribution file
        assert!(contents.contains("Person_ID,Money"));
        assert!(contents.contains("0,50."));
        assert!(contents.contains("1,80."));
        assert!(contents.contains("2,100."));
        assert!(contents.contains("3,120."));
        assert!(contents.contains("4,150."));
    }

    #[test]
    fn test_save_to_csv_reputation_distribution() {
        let result = get_test_result();
        let contents = read_csv_file_from_test(&result, "reputation");

        // Check reputation distribution file
        assert!(contents.contains("Person_ID,Reputation"));
        assert!(contents.contains("0,0.95"));
        assert!(contents.contains("2,1.0"));
        assert!(contents.contains("4,1.1"));
    }

    #[test]
    fn test_save_to_csv_price_history() {
        let mut result = get_test_result();

        // Add price history data (SkillId is String type)
        let mut price_history = HashMap::new();
        price_history.insert("Skill_0".to_string(), vec![10.0, 11.0, 12.0]);
        price_history.insert("Skill_1".to_string(), vec![15.0, 14.5, 14.0]);
        result.skill_price_history = price_history;

        let contents = read_csv_file_from_test(&result, "price_history");

        // Check price history file
        assert!(contents.contains("Step,Skill_"));
        assert!(contents.contains("0,10."));
        assert!(contents.contains("1,11."));
        assert!(contents.contains("2,12."));
    }

    #[test]
    fn test_trade_volume_statistics() {
        let result = get_test_result();

        // Verify trade volume statistics are calculated correctly
        assert_eq!(result.trade_volume_statistics.total_trades, 100);
        assert_eq!(result.trade_volume_statistics.total_volume, 1000.0);
        assert_eq!(result.trade_volume_statistics.avg_trades_per_step, 10.0);
        assert_eq!(result.trade_volume_statistics.avg_volume_per_step, 100.0);
        assert_eq!(result.trade_volume_statistics.avg_transaction_value, 10.0);
        assert_eq!(result.trade_volume_statistics.min_trades_per_step, 5);
        assert_eq!(result.trade_volume_statistics.max_trades_per_step, 15);
    }

    #[test]
    fn test_save_to_csv_trade_volume() {
        let result = get_test_result();
        let contents = read_csv_file_from_test(&result, "trade_volume");

        // Check trade volume file
        assert!(contents.contains("Step,Trades_Count,Volume_Exchanged"));
        assert!(contents.contains("0,10,100."));
        assert!(contents.contains("4,15,150."));
        assert!(contents.contains("8,5,50."));
    }

    #[test]
    fn test_herfindahl_index_empty() {
        let values: Vec<f64> = vec![];
        let hhi = calculate_herfindahl_index(&values);
        assert_eq!(hhi, 0.0);
    }

    #[test]
    fn test_herfindahl_index_monopoly() {
        // One participant with everything = HHI of 10,000
        let values = vec![100.0];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 10000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_perfect_equality() {
        // 4 participants with 25% each = HHI of 2,500
        let values = vec![25.0, 25.0, 25.0, 25.0];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 2500.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_ten_equal() {
        // 10 participants with 10% each = HHI of 1,000
        let values = vec![10.0; 10];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_high_concentration() {
        // One large player (60%) and 4 small players (10% each) = HHI of 4,000
        let values = vec![60.0, 10.0, 10.0, 10.0, 10.0];
        let hhi = calculate_herfindahl_index(&values);
        // HHI = 60^2 + 10^2 + 10^2 + 10^2 + 10^2 = 3600 + 100 + 100 + 100 + 100 = 4000
        assert!((hhi - 4000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_moderate_concentration() {
        // Moderate concentration
        let values = vec![30.0, 25.0, 20.0, 15.0, 10.0];
        let hhi = calculate_herfindahl_index(&values);
        // HHI = 30^2 + 25^2 + 20^2 + 15^2 + 10^2 = 900 + 625 + 400 + 225 + 100 = 2250
        assert!((hhi - 2250.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_low_concentration() {
        // Many small players = low HHI
        let values = vec![5.0; 20];
        let hhi = calculate_herfindahl_index(&values);
        // 20 participants with 5% each = HHI of 500
        assert!((hhi - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_skill_market_concentration_competitive() {
        use std::collections::HashMap;

        // 10 sellers with equal market shares - should be competitive
        let mut volumes = HashMap::new();
        for i in 0..10 {
            volumes.insert(i, 10.0);
        }

        let concentration =
            calculate_skill_market_concentration("TestSkill".to_string(), &volumes).unwrap();

        assert_eq!(concentration.num_sellers, 10);
        assert_eq!(concentration.total_volume, 100.0);
        assert!(concentration.herfindahl_index < 1500.0); // Competitive
        assert_eq!(concentration.market_structure, MarketStructure::Competitive);
        assert!((concentration.cr4 - 0.4).abs() < 0.01); // Top 4 = 40%
        assert!((concentration.cr8 - 0.8).abs() < 0.01); // Top 8 = 80%
    }

    #[test]
    fn test_skill_market_concentration_moderate() {
        use std::collections::HashMap;

        // 5 sellers with moderately concentrated market
        let mut volumes = HashMap::new();
        volumes.insert(0, 25.0);
        volumes.insert(1, 25.0);
        volumes.insert(2, 20.0);
        volumes.insert(3, 15.0);
        volumes.insert(4, 15.0);

        let concentration =
            calculate_skill_market_concentration("TestSkill".to_string(), &volumes).unwrap();

        assert_eq!(concentration.num_sellers, 5);
        assert_eq!(concentration.total_volume, 100.0);
        assert!(
            concentration.herfindahl_index >= 1500.0 && concentration.herfindahl_index <= 2500.0
        );
        assert_eq!(
            concentration.market_structure,
            MarketStructure::ModerateConcentration
        );
        assert!((concentration.cr4 - 0.85).abs() < 0.01); // Top 4 = 85%
    }

    #[test]
    fn test_skill_market_concentration_high() {
        use std::collections::HashMap;

        // High concentration - 2 dominant sellers
        let mut volumes = HashMap::new();
        volumes.insert(0, 50.0);
        volumes.insert(1, 40.0);
        volumes.insert(2, 5.0);
        volumes.insert(3, 5.0);

        let concentration =
            calculate_skill_market_concentration("TestSkill".to_string(), &volumes).unwrap();

        assert_eq!(concentration.num_sellers, 4);
        assert_eq!(concentration.total_volume, 100.0);
        assert!(concentration.herfindahl_index > 2500.0); // Highly concentrated
        assert_eq!(
            concentration.market_structure,
            MarketStructure::HighConcentration
        );
        assert!((concentration.cr4 - 1.0).abs() < 0.01); // All 4 = 100%
    }

    #[test]
    fn test_skill_market_concentration_monopoly() {
        use std::collections::HashMap;

        // Pure monopoly - single seller
        let mut volumes = HashMap::new();
        volumes.insert(0, 100.0);

        let concentration =
            calculate_skill_market_concentration("TestSkill".to_string(), &volumes).unwrap();

        assert_eq!(concentration.num_sellers, 1);
        assert_eq!(concentration.total_volume, 100.0);
        assert!((concentration.herfindahl_index - 10000.0).abs() < 1.0); // Perfect monopoly
        assert_eq!(
            concentration.market_structure,
            MarketStructure::HighConcentration
        );
        assert!((concentration.cr4 - 1.0).abs() < 0.01); // Only seller = 100%
        assert!((concentration.cr8 - 1.0).abs() < 0.01); // Only seller = 100%
    }

    #[test]
    fn test_skill_market_concentration_empty() {
        use std::collections::HashMap;

        // No sellers - should return None
        let volumes = HashMap::new();
        let concentration = calculate_skill_market_concentration("TestSkill".to_string(), &volumes);

        assert!(concentration.is_none());
    }

    #[test]
    fn test_skill_market_concentration_few_sellers() {
        use std::collections::HashMap;

        // Only 2 sellers - CR4 and CR8 should both be 100%
        let mut volumes = HashMap::new();
        volumes.insert(0, 60.0);
        volumes.insert(1, 40.0);

        let concentration =
            calculate_skill_market_concentration("TestSkill".to_string(), &volumes).unwrap();

        assert_eq!(concentration.num_sellers, 2);
        assert!((concentration.cr4 - 1.0).abs() < 0.01); // Both sellers = 100%
        assert!((concentration.cr8 - 1.0).abs() < 0.01); // Both sellers = 100%
    }

    #[test]
    fn test_herfindahl_index_zero_sum() {
        // Zero total should return 0
        let values = vec![0.0, 0.0, 0.0];
        let hhi = calculate_herfindahl_index(&values);
        assert_eq!(hhi, 0.0);
    }

    #[test]
    fn test_money_stats_includes_hhi() {
        // Test that calculate_money_stats includes HHI
        let money = vec![25.0, 25.0, 25.0, 25.0];
        let stats = calculate_money_stats(&money);
        // Perfect equality: HHI should be 2500
        assert!((stats.herfindahl_index - 2500.0).abs() < 0.1);
    }

    #[test]
    fn test_money_stats_hhi_monopoly() {
        // One person has all money
        let money = vec![0.0, 0.0, 0.0, 100.0];
        let stats = calculate_money_stats(&money);
        // Monopoly: HHI should be 10000
        assert!((stats.herfindahl_index - 10000.0).abs() < 0.1);
    }

    #[test]
    fn test_trading_partner_statistics() {
        use crate::person::{Strategy, Transaction, TransactionType};
        use crate::skill::Skill;
        use crate::Entity;

        // Create test entities with transaction histories
        let mut entities = vec![];

        // Create persons with skills
        for i in 0..5 {
            let skill = Skill::new(format!("Skill{}", i), 10.0);
            let location = crate::person::Location::new(50.0, 50.0);
            entities.push(Entity::new(
                i,
                100.0,
                vec![skill],
                Strategy::Balanced,
                location,
            ));
        }

        // Add transactions to simulate trades
        // Person 0 buys from Person 1 three times
        for _ in 0..3 {
            entities[0]
                .person_data
                .transaction_history
                .push(Transaction {
                    step: 1,
                    skill_id: "Skill1".to_string(),
                    transaction_type: TransactionType::Buy,
                    amount: 10.0,
                    counterparty_id: Some(1),
                });
        }

        // Person 1 sells to Person 0 three times
        for _ in 0..3 {
            entities[1]
                .person_data
                .transaction_history
                .push(Transaction {
                    step: 1,
                    skill_id: "Skill0".to_string(),
                    transaction_type: TransactionType::Sell,
                    amount: 10.0,
                    counterparty_id: Some(0),
                });
        }

        // Person 2 buys from Person 3 once
        entities[2]
            .person_data
            .transaction_history
            .push(Transaction {
                step: 1,
                skill_id: "Skill3".to_string(),
                transaction_type: TransactionType::Buy,
                amount: 15.0,
                counterparty_id: Some(3),
            });

        // Person 3 sells to Person 2 once
        entities[3]
            .person_data
            .transaction_history
            .push(Transaction {
                step: 1,
                skill_id: "Skill2".to_string(),
                transaction_type: TransactionType::Sell,
                amount: 15.0,
                counterparty_id: Some(2),
            });

        // Calculate trading partner statistics
        let stats = calculate_trading_partner_statistics(&entities);

        // Verify per-person statistics
        assert_eq!(stats.per_person.len(), 5);

        // Person 0: 1 unique partner, 3 trades as buyer, 0 as seller
        let person0_stats = stats.per_person.iter().find(|s| s.person_id == 0).unwrap();
        assert_eq!(person0_stats.unique_partners, 1);
        assert_eq!(person0_stats.total_trades_as_buyer, 3);
        assert_eq!(person0_stats.total_trades_as_seller, 0);
        assert_eq!(person0_stats.top_partners.len(), 1);
        assert_eq!(person0_stats.top_partners[0].partner_id, 1);
        assert_eq!(person0_stats.top_partners[0].trade_count, 3);
        assert!((person0_stats.top_partners[0].total_value - 30.0).abs() < 0.01);

        // Person 1: 1 unique partner, 0 trades as buyer, 3 as seller
        let person1_stats = stats.per_person.iter().find(|s| s.person_id == 1).unwrap();
        assert_eq!(person1_stats.unique_partners, 1);
        assert_eq!(person1_stats.total_trades_as_buyer, 0);
        assert_eq!(person1_stats.total_trades_as_seller, 3);

        // Person 4: no trades
        let person4_stats = stats.per_person.iter().find(|s| s.person_id == 4).unwrap();
        assert_eq!(person4_stats.unique_partners, 0);
        assert_eq!(person4_stats.total_trades_as_buyer, 0);
        assert_eq!(person4_stats.total_trades_as_seller, 0);

        // Verify network metrics
        // Average unique partners = (1 + 1 + 1 + 1 + 0) / 5 = 0.8
        assert!((stats.network_metrics.avg_unique_partners - 0.8).abs() < 0.01);

        // Network density = 2 unique pairs / 10 possible pairs = 0.2
        // Possible pairs for 5 persons = 5 * 4 / 2 = 10
        // Unique pairs: (0,1) and (2,3)
        assert!((stats.network_metrics.network_density - 0.2).abs() < 0.01);

        // Most active pair should be (0, 1) with 6 trades (3 buy + 3 sell)
        let most_active = stats.network_metrics.most_active_pair.unwrap();
        assert_eq!(most_active.0, 0);
        assert_eq!(most_active.1, 1);
        assert_eq!(most_active.2, 6); // 3 from person 0 + 3 from person 1
    }

    #[test]
    fn test_export_trading_network() {
        // Create a test result with trading partner statistics
        let mut result = get_test_result();

        // Add some trading partner statistics
        result.trading_partner_statistics = TradingPartnerStats {
            per_person: vec![
                PersonTradingStats {
                    person_id: 0,
                    unique_partners: 2,
                    total_trades_as_buyer: 3,
                    total_trades_as_seller: 1,
                    top_partners: vec![
                        PartnerInfo {
                            partner_id: 1,
                            trade_count: 2,
                            total_value: 50.0,
                        },
                        PartnerInfo {
                            partner_id: 2,
                            trade_count: 2,
                            total_value: 30.0,
                        },
                    ],
                },
                PersonTradingStats {
                    person_id: 1,
                    unique_partners: 1,
                    total_trades_as_buyer: 1,
                    total_trades_as_seller: 2,
                    top_partners: vec![PartnerInfo {
                        partner_id: 0,
                        trade_count: 2,
                        total_value: 50.0,
                    }],
                },
                PersonTradingStats {
                    person_id: 2,
                    unique_partners: 1,
                    total_trades_as_buyer: 2,
                    total_trades_as_seller: 0,
                    top_partners: vec![PartnerInfo {
                        partner_id: 0,
                        trade_count: 2,
                        total_value: 30.0,
                    }],
                },
            ],
            network_metrics: NetworkMetrics {
                avg_unique_partners: 1.33,
                network_density: 0.3,
                most_active_pair: Some((0, 1, 4)),
            },
        };

        // Export the trading network
        let network = result.export_trading_network();

        // Verify nodes
        assert_eq!(network.nodes.len(), 3);
        assert_eq!(network.nodes[0].id, "Person0");
        assert_eq!(network.nodes[0].money, 50.0);
        assert_eq!(network.nodes[0].reputation, 0.95);
        assert_eq!(network.nodes[0].trade_count, 4); // 3 as buyer + 1 as seller
        assert_eq!(network.nodes[0].unique_partners, 2);

        // Verify edges
        assert_eq!(network.edges.len(), 2);

        // Find edge between Person0 and Person1
        let edge_0_1 = network
            .edges
            .iter()
            .find(|e| {
                (e.source == "Person0" && e.target == "Person1")
                    || (e.source == "Person1" && e.target == "Person0")
            })
            .unwrap();
        assert_eq!(edge_0_1.weight, 4); // 2 from each side
        assert!((edge_0_1.total_value - 100.0).abs() < 0.01); // 50 + 50

        // Find edge between Person0 and Person2
        let edge_0_2 = network
            .edges
            .iter()
            .find(|e| {
                (e.source == "Person0" && e.target == "Person2")
                    || (e.source == "Person2" && e.target == "Person0")
            })
            .unwrap();
        assert_eq!(edge_0_2.weight, 4); // 2 from each side
        assert!((edge_0_2.total_value - 60.0).abs() < 0.01); // 30 + 30
    }

    #[test]
    fn test_save_trading_network_json() {
        let mut result = get_test_result();
        result.trading_partner_statistics = TradingPartnerStats {
            per_person: vec![PersonTradingStats {
                person_id: 0,
                unique_partners: 1,
                total_trades_as_buyer: 2,
                total_trades_as_seller: 1,
                top_partners: vec![PartnerInfo {
                    partner_id: 1,
                    trade_count: 3,
                    total_value: 45.0,
                }],
            }],
            network_metrics: NetworkMetrics {
                avg_unique_partners: 1.0,
                network_density: 0.5,
                most_active_pair: Some((0, 1, 3)),
            },
        };

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        result.save_trading_network_json(path).unwrap();

        let mut contents = String::new();
        file.reopen()
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("\"nodes\""));
        assert!(contents.contains("\"edges\""));
        assert!(contents.contains("Person0"));
    }

    #[test]
    fn test_save_trading_network_csv() {
        let mut result = get_test_result();
        result.trading_partner_statistics = TradingPartnerStats {
            per_person: vec![PersonTradingStats {
                person_id: 0,
                unique_partners: 1,
                total_trades_as_buyer: 2,
                total_trades_as_seller: 1,
                top_partners: vec![PartnerInfo {
                    partner_id: 1,
                    trade_count: 3,
                    total_value: 45.0,
                }],
            }],
            network_metrics: NetworkMetrics {
                avg_unique_partners: 1.0,
                network_density: 0.5,
                most_active_pair: Some((0, 1, 3)),
            },
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let prefix = temp_dir.path().join("test_network");
        let prefix_str = prefix.to_str().unwrap();

        result.save_trading_network_csv(prefix_str).unwrap();

        // Verify nodes CSV
        let nodes_path = format!("{}_network_nodes.csv", prefix_str);
        let mut nodes_contents = String::new();
        File::open(&nodes_path)
            .unwrap()
            .read_to_string(&mut nodes_contents)
            .unwrap();
        assert!(nodes_contents.contains("id,money,reputation,trade_count,unique_partners"));
        assert!(nodes_contents.contains("Person0"));

        // Verify edges CSV
        let edges_path = format!("{}_network_edges.csv", prefix_str);
        let mut edges_contents = String::new();
        File::open(&edges_path)
            .unwrap()
            .read_to_string(&mut edges_contents)
            .unwrap();
        assert!(edges_contents.contains("source,target,weight,total_value"));
    }

    // SIMD Optimization Tests

    #[test]
    fn test_simd_sum_correctness() {
        // Test with various sizes to ensure chunking works correctly

        // Empty array
        let empty: Vec<f64> = vec![];
        assert_eq!(simd_optimized_sum(&empty), 0.0);

        // Single element
        let single = vec![42.0];
        assert_eq!(simd_optimized_sum(&single), 42.0);

        // Exact multiple of 4 (perfect chunks)
        let perfect_chunks = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert_eq!(simd_optimized_sum(&perfect_chunks), 36.0);

        // Not a multiple of 4 (has remainder)
        let with_remainder = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(simd_optimized_sum(&with_remainder), 15.0);

        // Large array
        let large: Vec<f64> = (1..=1000).map(|x| x as f64).collect();
        let expected_sum = (1000.0 * 1001.0) / 2.0; // Sum of 1 to 1000
        assert_eq!(simd_optimized_sum(&large), expected_sum);

        // Test with negative numbers
        let mixed = vec![-5.0, 10.0, -3.0, 7.0, 2.0];
        assert_eq!(simd_optimized_sum(&mixed), 11.0);
    }

    #[test]
    fn test_simd_variance_correctness() {
        // Test variance calculation with known values

        // Perfect variance (mean = 3.0)
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = 3.0;
        let var_sum = simd_optimized_variance_sum(&values, mean);
        // Variance sum = (2^2 + 1^2 + 0^2 + 1^2 + 2^2) = 10
        assert_eq!(var_sum, 10.0);

        // All same values (variance should be 0)
        let uniform = vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0];
        let var_sum_uniform = simd_optimized_variance_sum(&uniform, 5.0);
        assert_eq!(var_sum_uniform, 0.0);

        // With remainder (not multiple of 4)
        let with_remainder = vec![1.0, 3.0, 5.0];
        let mean_rem = 3.0;
        let var_sum_rem = simd_optimized_variance_sum(&with_remainder, mean_rem);
        // Variance sum = (2^2 + 0^2 + 2^2) = 8
        assert_eq!(var_sum_rem, 8.0);
    }

    #[test]
    fn test_calculate_statistics_with_simd() {
        // Ensure SIMD-optimized calculate_statistics produces correct results

        // Basic test
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = calculate_statistics(&values);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        // Sample std dev = sqrt(10/4) = 1.5811...
        assert!((stats.std_dev - 1.5811).abs() < 0.001);

        // Large dataset to trigger parallel SIMD path
        let large: Vec<f64> = (1..=2000).map(|x| x as f64).collect();
        let stats_large = calculate_statistics(&large);
        assert_eq!(stats_large.mean, 1000.5);
        assert_eq!(stats_large.median, 1000.5);
        assert_eq!(stats_large.min, 1.0);
        assert_eq!(stats_large.max, 2000.0);

        // Test with non-multiples of 4 to ensure remainder handling
        let odd_size = vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0];
        let stats_odd = calculate_statistics(&odd_size);
        assert_eq!(stats_odd.mean, 8.0);
        assert_eq!(stats_odd.median, 8.0);

        // Empty array
        let empty: Vec<f64> = vec![];
        let stats_empty = calculate_statistics(&empty);
        assert_eq!(stats_empty.mean, 0.0);
        assert_eq!(stats_empty.median, 0.0);
    }

    #[test]
    fn test_simd_optimization_equivalence() {
        // Verify that SIMD results match scalar computation
        // This is crucial for correctness

        use rand::rngs::StdRng;
        use rand::Rng;
        use rand::SeedableRng;

        let mut rng = StdRng::seed_from_u64(42);

        // Generate random test data
        for size in [0, 1, 3, 4, 5, 7, 8, 15, 16, 17, 100, 257, 1000, 2048] {
            let values: Vec<f64> = (0..size).map(|_| rng.gen_range(-100.0..100.0)).collect();

            if values.is_empty() {
                continue;
            }

            // Compute using SIMD
            let simd_sum = simd_optimized_sum(&values);

            // Compute using standard iteration (scalar)
            let scalar_sum: f64 = values.iter().sum();

            // Should be identical
            assert!(
                (simd_sum - scalar_sum).abs() < 1e-10,
                "SIMD sum mismatch at size {}: {} vs {}",
                size,
                simd_sum,
                scalar_sum
            );

            // Test variance as well
            let mean = simd_sum / values.len() as f64;
            let simd_var = simd_optimized_variance_sum(&values, mean);
            let scalar_var: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();

            assert!(
                (simd_var - scalar_var).abs() < 1e-6,
                "SIMD variance mismatch at size {}: {} vs {}",
                size,
                simd_var,
                scalar_var
            );
        }
    }

    #[test]
    fn test_calculate_mobility_statistics_empty() {
        let empty_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let result = calculate_mobility_statistics(&empty_map);
        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_mobility_statistics_single_step() {
        let mut mobility_quintiles = HashMap::new();
        mobility_quintiles.insert(0, vec![2]); // Only one time point
        mobility_quintiles.insert(1, vec![1]);

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_none()); // Need at least 2 time points for transitions
    }

    #[test]
    fn test_calculate_mobility_statistics_no_mobility() {
        // All persons stay in their initial quintile
        let mut mobility_quintiles = HashMap::new();
        mobility_quintiles.insert(0, vec![0, 0, 0]); // Bottom quintile, stays
        mobility_quintiles.insert(1, vec![2, 2, 2]); // Middle quintile, stays
        mobility_quintiles.insert(2, vec![4, 4, 4]); // Top quintile, stays

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_some());

        let stats = result.unwrap();

        // All transitions should be diagonal (same quintile)
        assert_eq!(stats.upward_mobility_probability, 0.0);
        assert_eq!(stats.downward_mobility_probability, 0.0);
        assert_eq!(stats.quintile_persistence, 1.0); // 100% persistence
        assert_eq!(stats.avg_quintile_changes, 0.0);
    }

    #[test]
    fn test_calculate_mobility_statistics_upward_only() {
        // All persons move upward
        let mut mobility_quintiles = HashMap::new();
        mobility_quintiles.insert(0, vec![0, 1, 2]); // Moves up two quintiles
        mobility_quintiles.insert(1, vec![1, 2, 3]); // Moves up two quintiles

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_some());

        let stats = result.unwrap();

        assert_eq!(stats.upward_mobility_probability, 1.0); // 100% upward
        assert_eq!(stats.downward_mobility_probability, 0.0);
        assert_eq!(stats.quintile_persistence, 0.0);
        assert_eq!(stats.avg_quintile_changes, 2.0); // 2 changes per person on average
    }

    #[test]
    fn test_calculate_mobility_statistics_downward_only() {
        // All persons move downward
        let mut mobility_quintiles = HashMap::new();
        mobility_quintiles.insert(0, vec![4, 3, 2]); // Moves down two quintiles
        mobility_quintiles.insert(1, vec![3, 2, 1]); // Moves down two quintiles

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_some());

        let stats = result.unwrap();

        assert_eq!(stats.upward_mobility_probability, 0.0);
        assert_eq!(stats.downward_mobility_probability, 1.0); // 100% downward
        assert_eq!(stats.quintile_persistence, 0.0);
        assert_eq!(stats.avg_quintile_changes, 2.0);
    }

    #[test]
    fn test_calculate_mobility_statistics_mixed() {
        // Mixed mobility patterns
        let mut mobility_quintiles = HashMap::new();
        mobility_quintiles.insert(0, vec![0, 1, 1, 2]); // Upward: 0->1, stays 1->1, 1->2 upward
        mobility_quintiles.insert(1, vec![4, 3, 3, 2]); // Downward: 4->3, stays 3->3, 3->2 downward
        mobility_quintiles.insert(2, vec![2, 2, 2, 2]); // No change: all stays

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_some());

        let stats = result.unwrap();

        // Person 0: 0->1 (up), 1->1 (stay), 1->2 (up) = 2 up, 1 stay
        // Person 1: 4->3 (down), 3->3 (stay), 3->2 (down) = 2 down, 1 stay
        // Person 2: 2->2 (stay), 2->2 (stay), 2->2 (stay) = 3 stay
        // Total: 2 up, 2 down, 5 stay = 9 total transitions
        assert!((stats.upward_mobility_probability - 2.0 / 9.0).abs() < 1e-10);
        assert!((stats.downward_mobility_probability - 2.0 / 9.0).abs() < 1e-10);
        assert!((stats.quintile_persistence - 5.0 / 9.0).abs() < 1e-10);

        // Person 0: 2 changes, Person 1: 2 changes, Person 2: 0 changes
        // Average: 4/3 = 1.333...
        assert!((stats.avg_quintile_changes - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_mobility_statistics_transition_matrix() {
        // Simple case to verify transition matrix calculation
        let mut mobility_quintiles = HashMap::new();
        // From quintile 0: goes to 0 (stay) and 1 (up)
        mobility_quintiles.insert(0, vec![0, 0, 1]);
        // From quintile 1: goes to 1 (stay) and 0 (down)
        mobility_quintiles.insert(1, vec![1, 1, 0]);

        let result = calculate_mobility_statistics(&mobility_quintiles);
        assert!(result.is_some());

        let stats = result.unwrap();
        let matrix = stats.transition_matrix;

        // Check matrix is 5x5
        assert_eq!(matrix.len(), 5);
        assert_eq!(matrix[0].len(), 5);

        // From quintile 0: 2 transitions (0->0, 0->0, 0->1)
        // So 2/2 = 1.0 to stay at 0, and 0/2 = 0.0 to go to 1
        // Wait, we have 0->0 (step 0 to 1), 0->1 (step 1 to 2)
        // So from quintile 0: 1 transition to 0, 1 transition to 1
        // Matrix[0][0] = 0.5, Matrix[0][1] = 0.5
        assert!((matrix[0][0] - 0.5).abs() < 1e-10);
        assert!((matrix[0][1] - 0.5).abs() < 1e-10);

        // From quintile 1: 1->1 (step 0 to 1), 1->0 (step 1 to 2)
        // So from quintile 1: 1 transition to 1, 1 transition to 0
        // Matrix[1][1] = 0.5, Matrix[1][0] = 0.5
        assert!((matrix[1][0] - 0.5).abs() < 1e-10);
        assert!((matrix[1][1] - 0.5).abs() < 1e-10);
    }
}
