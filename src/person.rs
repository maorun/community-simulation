use crate::asset::AssetId;
use crate::credit_rating::CreditScore;
use crate::insurance::InsuranceId;
use crate::investment::InvestmentId;
use crate::loan::LoanId;
use crate::skill::{Skill, SkillId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type PersonId = usize;
pub type UrgencyLevel = u8; // Define UrgencyLevel (e.g., 1-3, higher is more urgent)

/// Represents the health status of a person in the simulation.
///
/// Health status affects a person's ability to trade and participate in the economy.
/// Sick persons have reduced trading capacity and can potentially transmit illness
/// to others during trade interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Person is healthy and can trade normally.
    Healthy,
    /// Person is sick and has reduced trading capacity.
    /// Contains the step when the person became sick for recovery tracking.
    Sick { infected_at_step: usize },
}

/// Represents a 2D location in the economic simulation.
///
/// Locations are used to model geographic distance between persons,
/// which can affect trade costs. Coordinates are in arbitrary units
/// (typically 0.0-100.0 range).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub x: f64,
    pub y: f64,
}

impl Location {
    /// Creates a new location at the given coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Location { x, y }
    }

    /// Calculates the Euclidean distance to another location.
    ///
    /// # Arguments
    /// * `other` - The other location to calculate distance to
    ///
    /// # Returns
    /// The Euclidean distance as a non-negative float
    pub fn distance_to(&self, other: &Location) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Represents the market segment a person belongs to based on their purchasing power.
///
/// Market segments categorize consumers by their price-quality preferences:
/// - **Budget**: Cost-conscious consumers, prioritize low prices (bottom 40% by wealth)
/// - **Mittelklasse**: Middle-market consumers, balance price and quality (40th-85th percentile)
/// - **Luxury**: Premium consumers, prioritize high quality (top 15% by wealth)
///
/// Market segmentation affects trade matching by creating preference for trading within
/// the same segment and influencing price/quality expectations.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash, Default,
)]
pub enum MarketSegment {
    /// Bottom 40% by wealth - prioritize affordability and low prices
    Budget,
    /// 40th-85th percentile - balance between price and quality
    #[default]
    Mittelklasse,
    /// Top 15% by wealth - prioritize quality and willing to pay premium prices
    Luxury,
}

impl MarketSegment {
    /// Determines market segment based on wealth percentile.
    ///
    /// # Arguments
    /// * `percentile` - Wealth percentile (0.0 to 1.0, where 1.0 = 100th percentile)
    ///
    /// # Returns
    /// The corresponding market segment
    ///
    /// # Examples
    /// ```
    /// use community_simulation::person::MarketSegment;
    ///
    /// assert_eq!(MarketSegment::from_percentile(0.20), MarketSegment::Budget);
    /// assert_eq!(MarketSegment::from_percentile(0.60), MarketSegment::Mittelklasse);
    /// assert_eq!(MarketSegment::from_percentile(0.90), MarketSegment::Luxury);
    /// ```
    pub fn from_percentile(percentile: f64) -> Self {
        if percentile >= 0.85 {
            MarketSegment::Luxury
        } else if percentile >= 0.40 {
            MarketSegment::Mittelklasse
        } else {
            MarketSegment::Budget
        }
    }

    /// Returns the quality expectation for this market segment (0.0-5.0 scale).
    ///
    /// Luxury consumers expect higher quality, while Budget consumers are more flexible.
    ///
    /// # Returns
    /// * `Budget`: 2.0 (accepts lower quality for better prices)
    /// * `Mittelklasse`: 3.0 (balanced quality expectations)
    /// * `Luxury`: 4.5 (demands premium quality)
    pub fn quality_expectation(&self) -> f64 {
        match self {
            MarketSegment::Budget => 2.0,
            MarketSegment::Mittelklasse => 3.0,
            MarketSegment::Luxury => 4.5,
        }
    }

    /// Returns the price acceptance range multiplier for this market segment.
    ///
    /// Determines how willing this segment is to pay above/below market price.
    /// Returns (min_multiplier, max_multiplier) tuple.
    ///
    /// # Returns
    /// * `Budget`: (0.5, 0.9) - seeks discounts, avoids full market price
    /// * `Mittelklasse`: (0.8, 1.2) - flexible within 20% of market price
    /// * `Luxury`: (1.0, 2.0) - willing to pay premium for quality
    pub fn price_acceptance_range(&self) -> (f64, f64) {
        match self {
            MarketSegment::Budget => (0.5, 0.9),
            MarketSegment::Mittelklasse => (0.8, 1.2),
            MarketSegment::Luxury => (1.0, 2.0),
        }
    }

    /// Returns all market segment variants.
    pub fn all_variants() -> [MarketSegment; 3] {
        [MarketSegment::Budget, MarketSegment::Mittelklasse, MarketSegment::Luxury]
    }

    /// Returns a descriptive string for this market segment.
    pub fn description(&self) -> &str {
        match self {
            MarketSegment::Budget => "Budget segment (below 40th percentile, price-conscious)",
            MarketSegment::Mittelklasse => "Middle-market segment (40th-85th percentile)",
            MarketSegment::Luxury => {
                "Luxury segment (at or above 85th percentile, quality-focused)"
            },
        }
    }
}

/// Represents the social class of a person based on their wealth relative to others.
///
/// Social classes are determined by wealth percentiles within the population:
/// - **Lower**: Bottom 25% (0-25th percentile)
/// - **Middle**: 25th-75th percentile
/// - **Upper**: 75th-95th percentile
/// - **Elite**: Top 5% (95th-100th percentile)
///
/// Social class affects access to resources, social networks, and economic opportunities.
/// The simulation tracks social mobility by recording class changes over time.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash, Default,
)]
pub enum SocialClass {
    /// Bottom 25% by wealth - limited resources and opportunities
    Lower,
    /// 25th-75th percentile - moderate resources and opportunities
    #[default]
    Middle,
    /// 75th-95th percentile - above average resources and opportunities
    Upper,
    /// Top 5% by wealth - extensive resources and opportunities
    Elite,
}

impl SocialClass {
    /// Determines social class based on wealth percentile.
    ///
    /// # Arguments
    /// * `percentile` - Wealth percentile (0.0 to 1.0, where 1.0 = 100th percentile)
    ///
    /// # Returns
    /// The corresponding social class
    ///
    /// # Examples
    /// ```
    /// use community_simulation::person::SocialClass;
    ///
    /// assert_eq!(SocialClass::from_percentile(0.10), SocialClass::Lower);
    /// assert_eq!(SocialClass::from_percentile(0.50), SocialClass::Middle);
    /// assert_eq!(SocialClass::from_percentile(0.85), SocialClass::Upper);
    /// assert_eq!(SocialClass::from_percentile(0.97), SocialClass::Elite);
    /// ```
    pub fn from_percentile(percentile: f64) -> Self {
        if percentile >= 0.95 {
            SocialClass::Elite
        } else if percentile >= 0.75 {
            SocialClass::Upper
        } else if percentile >= 0.25 {
            SocialClass::Middle
        } else {
            SocialClass::Lower
        }
    }

    /// Returns all social class variants.
    pub fn all_variants() -> [SocialClass; 4] {
        [SocialClass::Lower, SocialClass::Middle, SocialClass::Upper, SocialClass::Elite]
    }

    /// Returns a descriptive string for this social class.
    pub fn description(&self) -> &str {
        match self {
            SocialClass::Lower => "Lower class (bottom 25%)",
            SocialClass::Middle => "Middle class (25th-75th percentile)",
            SocialClass::Upper => "Upper class (75th-95th percentile)",
            SocialClass::Elite => "Elite class (top 5%)",
        }
    }
}

/// Records a change in social class at a specific simulation step.
///
/// Used to track social mobility over time by recording when persons
/// move between social classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassChange {
    /// The simulation step when the class change occurred
    pub step: usize,
    /// The previous social class
    pub from_class: SocialClass,
    /// The new social class
    pub to_class: SocialClass,
}

impl ClassChange {
    /// Creates a new class change record.
    pub fn new(step: usize, from_class: SocialClass, to_class: SocialClass) -> Self {
        ClassChange { step, from_class, to_class }
    }

    /// Returns true if this represents upward mobility.
    pub fn is_upward(&self) -> bool {
        self.to_class > self.from_class
    }

    /// Returns true if this represents downward mobility.
    pub fn is_downward(&self) -> bool {
        self.to_class < self.from_class
    }
}

/// Defines specialization strategy for skill development in the simulation.
/// Determines whether agents focus on mastering few skills (specialist) or learning many (generalist).
///
/// Specialists have higher quality in their few skills and can charge premium prices,
/// but face a narrower market. Generalists have broader market access but standard quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpecializationStrategy {
    /// Specialist: Focuses on few skills with higher quality and premium pricing.
    /// Higher quality (+1.0 bonus), higher prices, but narrower market focus.
    Specialist,
    /// Balanced: Standard approach with moderate quality and pricing.
    /// No quality or pricing adjustments, represents typical market participant.
    #[default]
    Balanced,
    /// Generalist: Learns many skills with standard quality but broader market access.
    /// No quality bonus, standard pricing, but more flexible in market participation.
    Generalist,
}

impl SpecializationStrategy {
    /// Returns the quality bonus for this specialization strategy.
    /// This bonus is added to the base quality rating for skills.
    ///
    /// # Returns
    /// * `Specialist`: +1.0 quality bonus
    /// * `Balanced`: 0.0 (no bonus)
    /// * `Generalist`: 0.0 (no bonus)
    pub fn quality_bonus(&self) -> f64 {
        match self {
            SpecializationStrategy::Specialist => 1.0,
            SpecializationStrategy::Balanced => 0.0,
            SpecializationStrategy::Generalist => 0.0,
        }
    }

    /// Returns the price multiplier for this specialization strategy.
    /// This multiplier is applied to skill prices.
    ///
    /// # Returns
    /// * `Specialist`: 1.15 (15% premium)
    /// * `Balanced`: 1.0 (base price)
    /// * `Generalist`: 1.0 (base price)
    pub fn price_multiplier(&self) -> f64 {
        match self {
            SpecializationStrategy::Specialist => 1.15,
            SpecializationStrategy::Balanced => 1.0,
            SpecializationStrategy::Generalist => 1.0,
        }
    }

    /// Returns all strategy variants for random distribution.
    pub fn all_variants() -> [SpecializationStrategy; 3] {
        [
            SpecializationStrategy::Specialist,
            SpecializationStrategy::Balanced,
            SpecializationStrategy::Generalist,
        ]
    }
}

/// Defines different behavioral strategies for agents in the simulation.
/// Each strategy affects how aggressively a person spends money on needed skills.
///
/// # Debt Behavior
/// Some strategies (notably Aggressive) may allow agents to spend beyond their current money,
/// resulting in negative balances (debt). This is intentional and simulates risk-taking behavior.
/// The simulation already supports negative money as indicated by Gini coefficient calculations
/// that can exceed 1.0 when debt is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Strategy {
    /// Conservative strategy: Prefers saving, lower spending threshold (0.7x).
    /// These agents are risk-averse and only spend when they have ample reserves.
    Conservative,
    /// Balanced strategy: Normal behavior (1.0x), default strategy.
    /// These agents spend according to standard market conditions.
    #[default]
    Balanced,
    /// Aggressive strategy: Higher spending threshold (1.3x), prioritizes acquiring skills.
    /// These agents are willing to spend more of their money to acquire needed skills.
    Aggressive,
    /// Frugal strategy: Minimal spending (0.5x), maximum savings behavior.
    /// These agents spend only when absolutely necessary and hoard resources.
    Frugal,
}

impl Strategy {
    /// Returns the spending multiplier for this strategy.
    /// This multiplier is applied to determine how much of their money a person
    /// is willing to spend on acquiring needed skills.
    ///
    /// # Returns
    /// * `Conservative`: 0.7 (willing to spend up to 70% of money on a skill)
    /// * `Balanced`: 1.0 (willing to spend up to 100% of money on a skill)
    /// * `Aggressive`: 1.3 (willing to spend beyond current means, risk-taking)
    /// * `Frugal`: 0.5 (willing to spend up to 50% of money on a skill)
    pub fn spending_multiplier(&self) -> f64 {
        match self {
            Strategy::Conservative => 0.7,
            Strategy::Balanced => 1.0,
            Strategy::Aggressive => 1.3,
            Strategy::Frugal => 0.5,
        }
    }

    /// Returns all strategy variants for random distribution.
    pub fn all_variants() -> [Strategy; 4] {
        [
            Strategy::Conservative,
            Strategy::Balanced,
            Strategy::Aggressive,
            Strategy::Frugal,
        ]
    }
}

/// Parameters tracking strategy performance and adaptation for an agent.
///
/// Enables agents to learn from experience by tracking their success metrics
/// and adjusting their behavioral strategies accordingly. This creates adaptive
/// behavior where successful strategies are reinforced and unsuccessful ones
/// are modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    /// Money the person started with (for calculating growth rate).
    pub initial_money: f64,
    /// Money from the previous step (for tracking short-term changes).
    pub previous_money: f64,
    /// Total number of successful trades as buyer.
    pub successful_buys: usize,
    /// Total number of successful trades as seller.
    pub successful_sells: usize,
    /// Strategy adjustment factor (0.0-2.0, starts at 1.0).
    /// Multiplied with the base strategy spending multiplier.
    /// Higher values indicate more aggressive spending, lower values more conservative.
    pub adjustment_factor: f64,
    /// Number of times strategy has been adapted.
    pub adaptation_count: usize,
}

impl StrategyParameters {
    /// Creates a new StrategyParameters with initial values.
    pub fn new(initial_money: f64) -> Self {
        StrategyParameters {
            initial_money,
            previous_money: initial_money,
            successful_buys: 0,
            successful_sells: 0,
            adjustment_factor: 1.0, // Start with neutral adjustment
            adaptation_count: 0,
        }
    }

    /// Calculates the wealth growth rate since the previous step.
    /// Returns a value where 0.0 = no change, positive = growth, negative = decline.
    pub fn calculate_short_term_growth_rate(&self, current_money: f64) -> f64 {
        if self.previous_money == 0.0 {
            return 0.0;
        }
        (current_money - self.previous_money) / self.previous_money
    }

    /// Calculates the overall wealth growth rate since initialization.
    /// Returns a value where 0.0 = no change, positive = growth, negative = decline.
    pub fn calculate_long_term_growth_rate(&self, current_money: f64) -> f64 {
        if self.initial_money == 0.0 {
            return 0.0;
        }
        (current_money - self.initial_money) / self.initial_money
    }

    /// Updates the previous money for next step's comparison.
    pub fn update_previous_money(&mut self, current_money: f64) {
        self.previous_money = current_money;
    }

    /// Records a successful trade as buyer.
    pub fn record_successful_buy(&mut self) {
        self.successful_buys += 1;
    }

    /// Records a successful trade as seller.
    pub fn record_successful_sell(&mut self) {
        self.successful_sells += 1;
    }

    /// Total successful trades (both buying and selling).
    pub fn total_successful_trades(&self) -> usize {
        self.successful_buys + self.successful_sells
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub step: usize,
    pub skill_id: SkillId,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub counterparty_id: Option<PersonId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeededSkillItem {
    pub id: SkillId,
    pub urgency: UrgencyLevel,
}

/// Represents an active mentorship relationship where an experienced person
/// teaches a skill to another person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mentorship {
    /// ID of the person acting as mentor
    pub mentor_id: PersonId,
    /// ID of the person being mentored
    pub mentee_id: PersonId,
    /// The skill being taught
    pub skill_id: SkillId,
    /// Step when the mentorship began
    pub start_step: usize,
}

impl Mentorship {
    /// Creates a new mentorship relationship.
    pub fn new(
        mentor_id: PersonId,
        mentee_id: PersonId,
        skill_id: SkillId,
        start_step: usize,
    ) -> Self {
        Mentorship { mentor_id, mentee_id, skill_id, start_step }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub money: f64,
    /// Skills this person can provide to others in the market.
    /// Each person can have one or more skills they offer.
    pub own_skills: Vec<Skill>,
    // Now stores tuples of (SkillId, UrgencyLevel)
    pub needed_skills: Vec<NeededSkillItem>,
    pub transaction_history: Vec<Transaction>,
    // Stores SkillIds that have been satisfied in the current step
    pub satisfied_needs_current_step: Vec<SkillId>,
    /// Reputation score affecting trading conditions.
    /// Starts at 1.0 (neutral), increases with successful transactions,
    /// and can decay over time. Higher reputation may result in better prices.
    pub reputation: f64,
    /// Total amount saved from income.
    /// Accumulated over time based on the configured savings rate.
    pub savings: f64,
    /// IDs of loans where this person is the borrower
    pub borrowed_loans: Vec<LoanId>,
    /// IDs of loans where this person is the lender
    pub lent_loans: Vec<LoanId>,
    /// IDs of investments where this person is the investor
    pub active_investments: Vec<InvestmentId>,
    /// Behavioral strategy that affects spending decisions.
    /// Determines how aggressively this person spends money to acquire needed skills.
    pub strategy: Strategy,
    /// Specialization strategy affecting skill quality and pricing.
    /// Determines whether this person focuses on few skills (specialist) or many (generalist).
    pub specialization_strategy: SpecializationStrategy,
    /// Skills that this person has learned through education.
    /// These skills can also be provided to others in the market.
    pub learned_skills: Vec<Skill>,
    /// Set of person IDs who are friends with this person.
    /// Friends receive price discounts when trading with each other.
    pub friends: HashSet<PersonId>,
    /// IDs of trade agreements this person is part of.
    /// Agreement partners receive price discounts when trading with each other.
    pub trade_agreement_ids: Vec<usize>,
    /// Optional group/organization ID this person belongs to.
    /// Enables analysis of collective behavior and group-based statistics.
    /// None indicates no group membership.
    pub group_id: Option<usize>,
    /// Geographic location of this person in 2D space.
    /// Used for calculating distance-based trade costs when enabled.
    pub location: Location,
    /// Quality ratings for skills this person provides (0.0-5.0 scale).
    /// Maps SkillId to quality rating. Higher quality enables higher prices.
    /// Quality improves through successful trades and decays when not used.
    /// Only populated when quality system is enabled.
    pub skill_qualities: HashMap<SkillId, f64>,
    /// Credit score tracking creditworthiness (300-850 FICO-like scale).
    /// Affects interest rates on loans - higher scores get better rates.
    /// Calculated from payment history, debt level, credit history length, new credit, and credit mix.
    /// Only meaningful when credit rating system is enabled.
    pub credit_score: CreditScore,
    /// IDs of active insurance policies owned by this person.
    /// Insurance provides protection against various economic risks (credit defaults, low income, crises).
    pub insurance_policies: Vec<InsuranceId>,
    /// Parameters tracking strategy adaptation and learning.
    /// Enables agents to learn from experience and adjust their behavioral strategies.
    /// Only meaningful when adaptive strategies system is enabled.
    pub strategy_params: StrategyParameters,
    /// Current health status of the person.
    /// Sick persons have reduced trading capacity and may transmit illness during trades.
    /// Only meaningful when health system is enabled.
    pub health_status: HealthStatus,
    /// Influence score based on network position and friend count.
    /// Higher values indicate more influential persons in the social network.
    /// Calculated dynamically based on number of friends (network centrality).
    /// Only meaningful when influence system is enabled.
    pub influence_score: f64,
    /// Current social class based on wealth percentile within the population.
    /// Determined by comparing this person's wealth to all other persons.
    /// Updated periodically to reflect changes in relative wealth.
    /// Only meaningful when social class system is enabled.
    pub social_class: SocialClass,
    /// History of social class changes over the simulation.
    /// Records upward and downward mobility events with timestamps.
    /// Used to analyze social mobility patterns and class transitions.
    pub class_history: Vec<ClassChange>,
    /// Assets owned by this person (property, equipment, stocks).
    /// Assets represent long-term wealth that can appreciate or depreciate over time.
    /// Total asset value is included in wealth calculations.
    /// Only populated when asset system is enabled.
    pub owned_assets: Vec<AssetId>,
    /// Market segment based on wealth percentile and purchasing power.
    /// Determines price-quality preferences and trade matching patterns.
    /// Budget segment prioritizes low prices, Luxury segment prioritizes quality,
    /// Mittelklasse balances both. Updated periodically based on wealth changes.
    /// Only meaningful when market segmentation is enabled.
    pub market_segment: MarketSegment,
}

impl Person {
    /// Creates a new person with multiple skills.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this person
    /// * `initial_money` - Starting money amount
    /// * `own_skills` - Vector of skills this person can provide
    /// * `strategy` - Behavioral strategy for spending decisions
    /// * `location` - Geographic location of this person
    pub fn new(
        id: PersonId,
        initial_money: f64,
        own_skills: Vec<Skill>,
        strategy: Strategy,
        location: Location,
    ) -> Self {
        Person {
            id,
            money: initial_money,
            own_skills,
            needed_skills: Vec::new(),
            transaction_history: Vec::new(),
            satisfied_needs_current_step: Vec::new(),
            reputation: 1.0, // Start with neutral reputation
            savings: 0.0,    // Start with no savings
            borrowed_loans: Vec::new(),
            lent_loans: Vec::new(),
            active_investments: Vec::new(), // Start with no investments
            strategy,
            specialization_strategy: SpecializationStrategy::default(), // Start with balanced
            learned_skills: Vec::new(), // Start with no learned skills
            friends: HashSet::new(),    // Start with no friends
            trade_agreement_ids: Vec::new(), // Start with no trade agreements
            group_id: None,             // Start with no group assignment
            location,
            skill_qualities: HashMap::new(), // Start with empty quality map (populated when quality enabled)
            credit_score: CreditScore::new(), // Start with default credit score
            insurance_policies: Vec::new(),  // Start with no insurance policies
            strategy_params: StrategyParameters::new(initial_money), // Initialize strategy tracking
            health_status: HealthStatus::Healthy, // Start healthy
            influence_score: 1.0,            // Start with baseline influence
            social_class: SocialClass::default(), // Start with middle class (will be updated based on wealth)
            class_history: Vec::new(),            // Start with no class history
            owned_assets: Vec::new(),             // Start with no assets
            market_segment: MarketSegment::default(), // Start with Mittelklasse segment (will be updated based on wealth)
        }
    }

    pub fn can_afford(&self, amount: f64) -> bool {
        self.money >= amount
    }

    /// Checks if the person can afford a purchase considering their behavioral strategy.
    /// Different strategies have different spending thresholds.
    ///
    /// When adaptive strategies are enabled, the adjustment factor is also applied.
    ///
    /// # Arguments
    /// * `amount` - The cost of the skill to purchase
    ///
    /// # Returns
    /// `true` if the person's money multiplied by their strategy's effective spending multiplier
    /// is greater than or equal to the amount, `false` otherwise.
    ///
    /// # Examples
    /// - A Conservative person with $100 and a 0.7x multiplier can afford items up to $70
    /// - An Aggressive person with $100 and a 1.3x multiplier can afford items up to $130
    /// - With adaptation, the multiplier is further adjusted by the adjustment_factor
    pub fn can_afford_with_strategy(&self, amount: f64) -> bool {
        let base_multiplier = self.strategy.spending_multiplier();
        let effective_multiplier = base_multiplier * self.strategy_params.adjustment_factor;
        let effective_money = self.money * effective_multiplier;
        effective_money >= amount
    }

    pub fn record_transaction(
        &mut self,
        step: usize,
        skill_id: SkillId,
        transaction_type: TransactionType,
        amount: f64,
        counterparty_id: Option<PersonId>,
    ) {
        let transaction = Transaction { step, skill_id, transaction_type, amount, counterparty_id };
        self.transaction_history.push(transaction);
    }

    /// Increases reputation after a successful sale transaction.
    /// Sellers gain more reputation than buyers to incentivize quality service.
    pub fn increase_reputation_as_seller(&mut self) {
        self.reputation += 0.01; // Gain 1% reputation per successful sale
        self.reputation = self.reputation.min(2.0); // Cap at 2.0 (excellent reputation)
    }

    /// Increases reputation after a successful purchase transaction.
    /// Buyers gain less reputation than sellers.
    pub fn increase_reputation_as_buyer(&mut self) {
        self.reputation += 0.005; // Gain 0.5% reputation per successful purchase
        self.reputation = self.reputation.min(2.0); // Cap at 2.0
    }

    /// Applies reputation decay to simulate the need for ongoing positive behavior.
    /// Called periodically (e.g., every simulation step).
    pub fn apply_reputation_decay(&mut self) {
        // Reputation slowly moves toward neutral (1.0)
        if self.reputation > 1.0 {
            self.reputation -= 0.001; // Slow decay for high reputation
            self.reputation = self.reputation.max(1.0); // Don't go below neutral
        } else if self.reputation < 1.0 {
            self.reputation += 0.001; // Slow recovery for low reputation
            self.reputation = self.reputation.min(1.0); // Don't go above neutral
        }
    }

    /// Returns true if the person is currently healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
    }

    /// Returns true if the person is currently sick.
    pub fn is_sick(&self) -> bool {
        matches!(self.health_status, HealthStatus::Sick { .. })
    }

    /// Infects the person with illness at the given step.
    pub fn infect(&mut self, current_step: usize) {
        self.health_status = HealthStatus::Sick { infected_at_step: current_step };
    }

    /// Attempts to recover the person if they have been sick long enough.
    /// Returns true if recovery occurred, false otherwise.
    ///
    /// # Arguments
    /// * `current_step` - The current simulation step
    /// * `recovery_duration` - Number of steps required to recover
    pub fn try_recover(&mut self, current_step: usize, recovery_duration: usize) -> bool {
        if let HealthStatus::Sick { infected_at_step } = self.health_status {
            if current_step >= infected_at_step + recovery_duration {
                self.health_status = HealthStatus::Healthy;
                return true;
            }
        }
        false
    }

    /// Returns the productivity multiplier based on health status.
    /// Sick persons have reduced productivity (e.g., 0.5 = 50% productivity).
    pub fn health_productivity_multiplier(&self) -> f64 {
        match self.health_status {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Sick { .. } => 0.5, // 50% productivity when sick
        }
    }

    /// Calculates a price multiplier based on reputation.
    /// Higher reputation = lower prices (discount for trusted sellers).
    /// Returns a multiplier in range [0.9, 1.1]
    pub fn reputation_price_multiplier(&self) -> f64 {
        // Reputation typically ranges from 0.0 to 2.0 (capped at both ends)
        // At reputation 1.0 (neutral): multiplier = 1.0 (no change)
        // At reputation 2.0 (excellent): multiplier = 0.9 (10% discount)
        // At reputation 0.0 (worst): multiplier = 1.1 (10% premium)
        // Formula: multiplier = 1.0 - (reputation - 1.0) * 0.1
        let multiplier = 1.0 - (self.reputation - 1.0) * 0.1;
        multiplier.clamp(0.9, 1.1)
    }

    /// Saves a portion of current money based on the savings rate.
    /// The money is transferred from available cash to savings.
    ///
    /// # Arguments
    /// * `savings_rate` - Rate between 0.0 and 1.0 representing percentage to save
    ///
    /// # Returns
    /// The amount that was saved in this operation
    pub fn apply_savings(&mut self, savings_rate: f64) -> f64 {
        if savings_rate <= 0.0 || self.money <= 0.0 {
            return 0.0;
        }

        let amount_to_save = self.money * savings_rate;
        self.money -= amount_to_save;
        self.savings += amount_to_save;
        amount_to_save
    }

    /// Adds a friend to this person's friend list.
    /// Friendship is not automatically bidirectional - both persons must add each other.
    pub fn add_friend(&mut self, friend_id: PersonId) {
        self.friends.insert(friend_id);
    }

    /// Checks if this person is friends with another person.
    pub fn is_friend_with(&self, other_id: PersonId) -> bool {
        self.friends.contains(&other_id)
    }

    /// Returns the number of friends this person has.
    pub fn friend_count(&self) -> usize {
        self.friends.len()
    }

    /// Calculates influence score based on friend count (network centrality).
    /// Influence grows with number of friends but with diminishing returns (logarithmic scaling).
    /// Formula: influence = 1.0 + log(1 + friend_count)
    ///
    /// # Arguments
    /// * `friend_count` - Number of friends the person has
    ///
    /// # Returns
    /// Influence score (minimum 1.0, grows logarithmically with friend count)
    pub fn calculate_influence_from_friends(friend_count: usize) -> f64 {
        let count = friend_count as f64;
        // Logarithmic scaling: influence grows with friends but with diminishing returns
        // Base influence is 1.0, grows to ~2.6 with 10 friends, ~3.4 with 30 friends
        1.0 + (1.0 + count).ln()
    }

    /// Calculates and updates the influence score based on friend count (network centrality).
    /// Influence grows with number of friends but with diminishing returns (logarithmic scaling).
    /// Formula: influence = 1.0 + log(1 + friend_count)
    /// This gives baseline influence of 1.0 with logarithmic growth as network expands.
    pub fn update_influence_score(&mut self) {
        self.influence_score = Self::calculate_influence_from_friends(self.friends.len());
    }

    /// Updates this person's social class based on their wealth percentile.
    ///
    /// If the social class changes, records the change in class_history.
    ///
    /// # Arguments
    /// * `wealth_percentile` - This person's wealth percentile (0.0 to 1.0)
    /// * `current_step` - The current simulation step number
    ///
    /// # Examples
    /// ```
    /// use community_simulation::person::{Person, Strategy, Location, SocialClass};
    /// use community_simulation::skill::Skill;
    ///
    /// let mut person = Person::new(1, 100.0, vec![Skill::new("Test".to_string(), 10.0)], Strategy::Balanced, Location::new(0.0, 0.0));
    /// person.update_social_class(0.5, 100); // Middle class at 50th percentile
    /// assert_eq!(person.social_class, SocialClass::Middle);
    /// assert_eq!(person.class_history.len(), 0); // No change recorded (started as Middle)
    ///
    /// person.update_social_class(0.97, 200); // Move to Elite class
    /// assert_eq!(person.social_class, SocialClass::Elite);
    /// assert_eq!(person.class_history.len(), 1); // Change recorded
    /// assert!(person.class_history[0].is_upward());
    /// ```
    pub fn update_social_class(&mut self, wealth_percentile: f64, current_step: usize) {
        let new_class = SocialClass::from_percentile(wealth_percentile);

        if new_class != self.social_class {
            let change = ClassChange::new(current_step, self.social_class, new_class);
            self.class_history.push(change);
            self.social_class = new_class;
        }
    }

    /// Attempts to learn a new skill if the person can afford it.
    ///
    /// # Arguments
    /// * `skill` - The skill to learn (will be cloned and added to learned_skills)
    /// * `cost` - The cost to learn this skill
    ///
    /// # Returns
    /// `true` if the skill was successfully learned, `false` if the person couldn't afford it
    /// or already knows the skill
    pub fn learn_skill(&mut self, skill: Skill, cost: f64) -> bool {
        // Check if person already has this skill (either as own_skill or learned)
        if self.has_skill(&skill.id) {
            return false;
        }

        // Check if person can afford the learning cost
        if !self.can_afford(cost) {
            return false;
        }

        // Deduct the cost and add the skill
        self.money -= cost;
        self.learned_skills.push(skill);
        true
    }

    /// Checks if this person has a specific skill (either as own_skill or learned).
    ///
    /// # Arguments
    /// * `skill_id` - The ID of the skill to check
    ///
    /// # Returns
    /// `true` if the person has this skill, `false` otherwise
    pub fn has_skill(&self, skill_id: &SkillId) -> bool {
        self.own_skills.iter().any(|s| &s.id == skill_id)
            || self.learned_skills.iter().any(|s| &s.id == skill_id)
    }

    /// Adapts the person's strategy based on recent performance.
    ///
    /// Uses a simple reinforcement learning approach:
    /// - If wealth is growing, increase aggression (higher adjustment factor)
    /// - If wealth is declining, become more conservative (lower adjustment factor)
    ///
    /// # Arguments
    /// * `adaptation_rate` - How quickly to adapt (0.0-1.0, typically 0.05-0.2)
    /// * `rng` - Random number generator for exploration
    /// * `exploration_rate` - Probability of random exploration (0.0-1.0)
    ///
    /// # Returns
    /// `true` if the strategy was adapted, `false` otherwise
    pub fn adapt_strategy<R: rand::Rng>(
        &mut self,
        adaptation_rate: f64,
        rng: &mut R,
        exploration_rate: f64,
    ) -> bool {
        // With exploration_rate probability, make a random adjustment
        if rng.random_range(0.0..1.0) < exploration_rate {
            // Random exploration: adjust factor randomly
            let random_adjustment = rng.random_range(-0.1..0.1);
            self.strategy_params.adjustment_factor += random_adjustment;
            self.strategy_params.adjustment_factor =
                self.strategy_params.adjustment_factor.clamp(0.5, 2.0);
            self.strategy_params.adaptation_count += 1;
            return true;
        }

        // Calculate wealth growth rate
        let growth_rate = self.strategy_params.calculate_short_term_growth_rate(self.money);

        // Only adapt if growth is significant (avoid noise)
        if growth_rate.abs() < 0.01 {
            // Less than 1% change, don't adapt
            self.strategy_params.update_previous_money(self.money);
            return false;
        }

        // Positive growth -> increase aggression slightly
        // Negative growth -> decrease aggression (become more conservative)
        let adjustment = growth_rate * adaptation_rate;
        self.strategy_params.adjustment_factor += adjustment;

        // Clamp to reasonable bounds (0.5 to 2.0)
        // 0.5 = very conservative (half normal spending)
        // 2.0 = very aggressive (double normal spending)
        self.strategy_params.adjustment_factor =
            self.strategy_params.adjustment_factor.clamp(0.5, 2.0);

        self.strategy_params.adaptation_count += 1;
        self.strategy_params.update_previous_money(self.money);

        true
    }

    /// Returns the effective spending multiplier including adaptation.
    pub fn get_effective_spending_multiplier(&self) -> f64 {
        self.strategy.spending_multiplier() * self.strategy_params.adjustment_factor
    }

    /// Returns all skills this person can provide (both own_skills and learned_skills).
    ///
    /// # Returns
    /// A vector of references to all skills this person possesses
    pub fn all_skills(&self) -> Vec<&Skill> {
        self.own_skills.iter().chain(self.learned_skills.iter()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::Skill;
    use rand::SeedableRng;

    // Helper function for tests - creates a default test location
    fn test_location() -> Location {
        Location::new(50.0, 50.0)
    }

    #[test]
    fn test_location_creation() {
        let loc = Location::new(10.0, 20.0);
        assert_eq!(loc.x, 10.0);
        assert_eq!(loc.y, 20.0);
    }

    #[test]
    fn test_location_distance_same_location() {
        let loc1 = Location::new(50.0, 50.0);
        let loc2 = Location::new(50.0, 50.0);
        assert_eq!(loc1.distance_to(&loc2), 0.0);
    }

    #[test]
    fn test_location_distance_horizontal() {
        let loc1 = Location::new(0.0, 0.0);
        let loc2 = Location::new(3.0, 0.0);
        assert_eq!(loc1.distance_to(&loc2), 3.0);
    }

    #[test]
    fn test_location_distance_vertical() {
        let loc1 = Location::new(0.0, 0.0);
        let loc2 = Location::new(0.0, 4.0);
        assert_eq!(loc1.distance_to(&loc2), 4.0);
    }

    #[test]
    fn test_location_distance_diagonal() {
        let loc1 = Location::new(0.0, 0.0);
        let loc2 = Location::new(3.0, 4.0);
        assert_eq!(loc1.distance_to(&loc2), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_location_distance_symmetric() {
        let loc1 = Location::new(10.0, 20.0);
        let loc2 = Location::new(30.0, 40.0);
        assert_eq!(loc1.distance_to(&loc2), loc2.distance_to(&loc1));
    }

    #[test]
    fn test_social_class_ordering() {
        // Verify that the derived Ord implementation produces the expected ordering
        // Lower < Middle < Upper < Elite
        assert!(SocialClass::Lower < SocialClass::Middle);
        assert!(SocialClass::Middle < SocialClass::Upper);
        assert!(SocialClass::Upper < SocialClass::Elite);

        // Verify transitivity
        assert!(SocialClass::Lower < SocialClass::Upper);
        assert!(SocialClass::Lower < SocialClass::Elite);
        assert!(SocialClass::Middle < SocialClass::Elite);
    }

    #[test]
    fn test_class_change_is_upward() {
        let change = ClassChange::new(100, SocialClass::Lower, SocialClass::Middle);
        assert!(change.is_upward());
        assert!(!change.is_downward());

        let change2 = ClassChange::new(100, SocialClass::Middle, SocialClass::Elite);
        assert!(change2.is_upward());
    }

    #[test]
    fn test_class_change_is_downward() {
        let change = ClassChange::new(100, SocialClass::Upper, SocialClass::Middle);
        assert!(change.is_downward());
        assert!(!change.is_upward());

        let change2 = ClassChange::new(100, SocialClass::Elite, SocialClass::Lower);
        assert!(change2.is_downward());
    }

    #[test]
    fn test_market_segment_from_percentile_budget() {
        // Test Budget segment (bottom 40%)
        assert_eq!(MarketSegment::from_percentile(0.0), MarketSegment::Budget);
        assert_eq!(MarketSegment::from_percentile(0.20), MarketSegment::Budget);
        assert_eq!(MarketSegment::from_percentile(0.39), MarketSegment::Budget);
    }

    #[test]
    fn test_market_segment_from_percentile_mittelklasse() {
        // Test Mittelklasse segment (40th-85th percentile)
        assert_eq!(MarketSegment::from_percentile(0.40), MarketSegment::Mittelklasse);
        assert_eq!(MarketSegment::from_percentile(0.60), MarketSegment::Mittelklasse);
        assert_eq!(MarketSegment::from_percentile(0.84), MarketSegment::Mittelklasse);
    }

    #[test]
    fn test_market_segment_from_percentile_luxury() {
        // Test Luxury segment (top 15%)
        assert_eq!(MarketSegment::from_percentile(0.85), MarketSegment::Luxury);
        assert_eq!(MarketSegment::from_percentile(0.90), MarketSegment::Luxury);
        assert_eq!(MarketSegment::from_percentile(1.0), MarketSegment::Luxury);
    }

    #[test]
    fn test_market_segment_quality_expectations() {
        assert_eq!(MarketSegment::Budget.quality_expectation(), 2.0);
        assert_eq!(MarketSegment::Mittelklasse.quality_expectation(), 3.0);
        assert_eq!(MarketSegment::Luxury.quality_expectation(), 4.5);
    }

    #[test]
    fn test_market_segment_price_acceptance_ranges() {
        // Budget segment: tight range, discounts only
        let (min, max) = MarketSegment::Budget.price_acceptance_range();
        assert_eq!(min, 0.5);
        assert_eq!(max, 0.9);

        // Mittelklasse segment: moderate flexibility
        let (min, max) = MarketSegment::Mittelklasse.price_acceptance_range();
        assert_eq!(min, 0.8);
        assert_eq!(max, 1.2);

        // Luxury segment: willing to pay premiums
        let (min, max) = MarketSegment::Luxury.price_acceptance_range();
        assert_eq!(min, 1.0);
        assert_eq!(max, 2.0);
    }

    #[test]
    fn test_market_segment_ordering() {
        // Verify that the derived Ord implementation produces the expected ordering
        assert!(MarketSegment::Budget < MarketSegment::Mittelklasse);
        assert!(MarketSegment::Mittelklasse < MarketSegment::Luxury);
        assert!(MarketSegment::Budget < MarketSegment::Luxury);
    }

    #[test]
    fn test_market_segment_descriptions() {
        assert!(MarketSegment::Budget.description().contains("below 40th"));
        assert!(MarketSegment::Mittelklasse.description().contains("40th-85th"));
        assert!(MarketSegment::Luxury.description().contains("at or above 85th"));
    }

    #[test]
    fn test_person_market_segment_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        assert_eq!(
            person.market_segment,
            MarketSegment::default(),
            "Market segment should start as default (Mittelklasse)"
        );
    }

    #[test]
    fn test_person_reputation_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        assert_eq!(person.reputation, 1.0, "Reputation should start at 1.0");
    }

    #[test]
    fn test_increase_reputation_as_seller() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.increase_reputation_as_seller();
        assert_eq!(person.reputation, 1.01, "Reputation should increase by 0.01");

        // Test multiple increases
        for _ in 0..99 {
            person.increase_reputation_as_seller();
        }
        assert_eq!(person.reputation, 2.0, "Reputation should be capped at 2.0");
    }

    #[test]
    fn test_increase_reputation_as_buyer() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.increase_reputation_as_buyer();
        assert_eq!(person.reputation, 1.005, "Reputation should increase by 0.005");

        // Test cap
        for _ in 0..200 {
            person.increase_reputation_as_buyer();
        }
        assert_eq!(person.reputation, 2.0, "Reputation should be capped at 2.0");
    }

    #[test]
    fn test_reputation_decay_above_neutral() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        person.reputation = 1.5;

        person.apply_reputation_decay();
        assert_eq!(person.reputation, 1.499, "Reputation should decay toward neutral");

        // Test decay stops at neutral
        person.reputation = 1.0;
        person.apply_reputation_decay();
        assert_eq!(person.reputation, 1.0, "Reputation should not decay below neutral");
    }

    #[test]
    fn test_reputation_decay_below_neutral() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        person.reputation = 0.5;

        person.apply_reputation_decay();
        assert_eq!(person.reputation, 0.501, "Reputation should increase toward neutral");

        // Test recovery stops at neutral
        person.reputation = 1.0;
        person.apply_reputation_decay();
        assert_eq!(
            person.reputation, 1.0,
            "Reputation should not increase above neutral during decay"
        );
    }

    #[test]
    fn test_reputation_price_multiplier_neutral() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 1.0, "Neutral reputation should have no price effect");
    }

    #[test]
    fn test_reputation_price_multiplier_high() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        person.reputation = 2.0; // Maximum reputation

        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 0.9, "High reputation should give 10% discount");
    }

    #[test]
    fn test_reputation_price_multiplier_low() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        person.reputation = 0.5;

        let multiplier = person.reputation_price_multiplier();
        // At reputation 0.5: multiplier = 1.0 - (0.5 - 1.0) * 0.1 = 1.0 + 0.05 = 1.05
        // This represents a 5% price premium for moderate low reputation
        assert!(
            (multiplier - 1.05).abs() < 0.001,
            "Low reputation (0.5) should give 5% premium, got {}",
            multiplier
        );
    }

    #[test]
    fn test_reputation_price_multiplier_clamping() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Test extreme high reputation
        person.reputation = 10.0;
        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 0.9, "Price multiplier should be clamped at 0.9");

        // Test extreme low reputation (needs to be at 0.0 to get clamped at 1.1)
        person.reputation = 0.0;
        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 1.1, "Price multiplier should be clamped at 1.1");
    }

    #[test]
    fn test_savings_basic() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Test 10% savings rate
        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 10.0, "Should save 10% of 100");
        assert_eq!(person.money, 90.0, "Money should be reduced by saved amount");
        assert_eq!(person.savings, 10.0, "Savings should be 10");

        // Apply savings again - now on 90.0
        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 9.0, "Should save 10% of 90");
        assert_eq!(person.money, 81.0, "Money should be 81 after second savings");
        assert_eq!(person.savings, 19.0, "Savings should accumulate to 19");
    }

    #[test]
    fn test_savings_zero_rate() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        let saved = person.apply_savings(0.0);
        assert_eq!(saved, 0.0, "Should save nothing with 0% rate");
        assert_eq!(person.money, 100.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_negative_money() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, -10.0, vec![skill], Strategy::default(), test_location());

        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 0.0, "Should not save with negative money");
        assert_eq!(person.money, -10.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_full_amount() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        let saved = person.apply_savings(1.0);
        assert_eq!(saved, 100.0, "Should save 100% (all money)");
        assert_eq!(person.money, 0.0, "Money should be 0");
        assert_eq!(person.savings, 100.0, "Savings should be 100");
    }

    #[test]
    fn test_strategy_spending_multipliers() {
        assert_eq!(Strategy::Conservative.spending_multiplier(), 0.7);
        assert_eq!(Strategy::Balanced.spending_multiplier(), 1.0);
        assert_eq!(Strategy::Aggressive.spending_multiplier(), 1.3);
        assert_eq!(Strategy::Frugal.spending_multiplier(), 0.5);
    }

    #[test]
    fn test_strategy_default() {
        assert_eq!(Strategy::default(), Strategy::Balanced);
    }

    #[test]
    fn test_can_afford_with_strategy_conservative() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Conservative, test_location());

        // Conservative has 0.7x multiplier, so with $100 can afford up to $70
        assert!(person.can_afford_with_strategy(70.0));
        assert!(person.can_afford_with_strategy(69.0));
        assert!(!person.can_afford_with_strategy(71.0));
        assert!(!person.can_afford_with_strategy(100.0));
    }

    #[test]
    fn test_can_afford_with_strategy_balanced() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());

        // Balanced has 1.0x multiplier, so with $100 can afford up to $100
        assert!(person.can_afford_with_strategy(100.0));
        assert!(person.can_afford_with_strategy(99.0));
        assert!(!person.can_afford_with_strategy(101.0));
    }

    #[test]
    fn test_can_afford_with_strategy_aggressive() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Aggressive, test_location());

        // Aggressive has 1.3x multiplier, so with $100 can afford up to $130
        assert!(person.can_afford_with_strategy(130.0));
        assert!(person.can_afford_with_strategy(129.0));
        assert!(!person.can_afford_with_strategy(131.0));
    }

    #[test]
    fn test_can_afford_with_strategy_frugal() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Frugal, test_location());

        // Frugal has 0.5x multiplier, so with $100 can afford up to $50
        assert!(person.can_afford_with_strategy(50.0));
        assert!(person.can_afford_with_strategy(49.0));
        assert!(!person.can_afford_with_strategy(51.0));
        assert!(!person.can_afford_with_strategy(100.0));
    }

    #[test]
    fn test_person_has_strategy_field() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Aggressive, test_location());

        assert_eq!(person.strategy, Strategy::Aggressive);
    }

    #[test]
    fn test_learn_skill_success() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 100.0, vec![own_skill], Strategy::default(), test_location());

        let new_skill = Skill::new("NewSkill".to_string(), 15.0);
        let learning_cost = 30.0;

        let result = person.learn_skill(new_skill.clone(), learning_cost);

        assert!(result, "Should successfully learn the skill");
        assert_eq!(person.money, 70.0, "Money should be deducted");
        assert_eq!(person.learned_skills.len(), 1, "Should have one learned skill");
        assert_eq!(person.learned_skills[0].id, "NewSkill", "Learned skill should be NewSkill");
    }

    #[test]
    fn test_learn_skill_cannot_afford() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 50.0, vec![own_skill], Strategy::default(), test_location());

        let new_skill = Skill::new("ExpensiveSkill".to_string(), 15.0);
        let learning_cost = 100.0; // More than person has

        let result = person.learn_skill(new_skill, learning_cost);

        assert!(!result, "Should fail to learn due to insufficient money");
        assert_eq!(person.money, 50.0, "Money should not be deducted");
        assert_eq!(person.learned_skills.len(), 0, "Should have no learned skills");
    }

    #[test]
    fn test_learn_skill_already_has_as_own_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 100.0, vec![own_skill.clone()], Strategy::default(), test_location());

        let learning_cost = 30.0;
        let result = person.learn_skill(own_skill, learning_cost);

        assert!(!result, "Should fail to learn skill they already have as own_skill");
        assert_eq!(person.money, 100.0, "Money should not be deducted");
        assert_eq!(person.learned_skills.len(), 0, "Should have no learned skills");
    }

    #[test]
    fn test_learn_skill_already_learned() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 100.0, vec![own_skill], Strategy::default(), test_location());

        let new_skill = Skill::new("NewSkill".to_string(), 15.0);
        let learning_cost = 20.0;

        // Learn the skill once
        person.learn_skill(new_skill.clone(), learning_cost);
        let money_after_first_learning = person.money;

        // Try to learn it again
        let result = person.learn_skill(new_skill, learning_cost);

        assert!(!result, "Should fail to learn skill they already learned");
        assert_eq!(
            person.money, money_after_first_learning,
            "Money should not be deducted on second attempt"
        );
        assert_eq!(person.learned_skills.len(), 1, "Should still have only one learned skill");
    }

    #[test]
    fn test_has_skill_with_own_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![own_skill], Strategy::default(), test_location());

        assert!(person.has_skill(&"OwnSkill".to_string()), "Should have OwnSkill as own_skill");
        assert!(!person.has_skill(&"OtherSkill".to_string()), "Should not have OtherSkill");
    }

    #[test]
    fn test_has_skill_with_learned_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 100.0, vec![own_skill], Strategy::default(), test_location());

        let learned_skill = Skill::new("LearnedSkill".to_string(), 15.0);
        person.learn_skill(learned_skill, 30.0);

        assert!(
            person.has_skill(&"LearnedSkill".to_string()),
            "Should have LearnedSkill as learned_skill"
        );
        assert!(person.has_skill(&"OwnSkill".to_string()), "Should still have OwnSkill");
    }

    #[test]
    fn test_all_skills() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person =
            Person::new(1, 100.0, vec![own_skill], Strategy::default(), test_location());

        let learned_skill1 = Skill::new("LearnedSkill1".to_string(), 15.0);
        let learned_skill2 = Skill::new("LearnedSkill2".to_string(), 20.0);
        person.learn_skill(learned_skill1, 30.0);
        person.learn_skill(learned_skill2, 40.0);

        let all_skills = person.all_skills();
        assert_eq!(all_skills.len(), 3, "Should have 3 total skills");

        let skill_ids: Vec<String> = all_skills.iter().map(|s| s.id.clone()).collect();
        assert!(skill_ids.contains(&"OwnSkill".to_string()));
        assert!(skill_ids.contains(&"LearnedSkill1".to_string()));
        assert!(skill_ids.contains(&"LearnedSkill2".to_string()));
    }

    #[test]
    fn test_strategy_parameters_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        assert_eq!(person.strategy_params.initial_money, 100.0);
        assert_eq!(person.strategy_params.previous_money, 100.0);
        assert_eq!(person.strategy_params.successful_buys, 0);
        assert_eq!(person.strategy_params.successful_sells, 0);
        assert_eq!(person.strategy_params.adjustment_factor, 1.0);
        assert_eq!(person.strategy_params.adaptation_count, 0);
    }

    #[test]
    fn test_strategy_params_growth_rate_calculation() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Simulate wealth growth
        person.money = 110.0; // 10% growth
        let growth_rate = person.strategy_params.calculate_short_term_growth_rate(person.money);
        assert!(
            (growth_rate - 0.1).abs() < 0.01,
            "Growth rate should be approximately 0.1 (10%)"
        );

        // Test long-term growth
        let long_term_growth = person.strategy_params.calculate_long_term_growth_rate(person.money);
        assert!((long_term_growth - 0.1).abs() < 0.01, "Long-term growth should be 10%");
    }

    #[test]
    fn test_strategy_adaptation_positive_growth() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());
        let mut rng = rand::rngs::StdRng::seed_from_u64(0); // Deterministic RNG that avoids exploration

        // Simulate wealth growth
        person.money = 110.0; // 10% growth

        // Adapt strategy with low exploration rate
        let adapted = person.adapt_strategy(0.1, &mut rng, 0.0);

        assert!(adapted, "Strategy should adapt with significant growth");
        assert!(
            person.strategy_params.adjustment_factor > 1.0,
            "Adjustment factor should increase with positive growth"
        );
        assert_eq!(person.strategy_params.adaptation_count, 1);
    }

    #[test]
    fn test_strategy_adaptation_negative_growth() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());
        let mut rng = rand::rngs::StdRng::seed_from_u64(0); // Deterministic RNG

        // Simulate wealth decline
        person.money = 90.0; // -10% decline

        // Adapt strategy with low exploration rate
        let adapted = person.adapt_strategy(0.1, &mut rng, 0.0);

        assert!(adapted, "Strategy should adapt with significant decline");
        assert!(
            person.strategy_params.adjustment_factor < 1.0,
            "Adjustment factor should decrease with negative growth"
        );
        assert_eq!(person.strategy_params.adaptation_count, 1);
    }

    #[test]
    fn test_strategy_no_adaptation_small_change() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        // Simulate very small wealth change (less than 1%)
        person.money = 100.5; // 0.5% growth

        // Try to adapt
        let adapted = person.adapt_strategy(0.1, &mut rng, 0.0);

        assert!(!adapted, "Strategy should not adapt with insignificant change");
        assert_eq!(
            person.strategy_params.adjustment_factor, 1.0,
            "Adjustment factor should remain unchanged"
        );
    }

    #[test]
    fn test_strategy_adjustment_bounds() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        // Simulate extreme wealth growth multiple times
        for _ in 0..20 {
            person.money *= 1.5; // 50% growth each iteration
            person.adapt_strategy(0.5, &mut rng, 0.0); // High adaptation rate
            person.strategy_params.update_previous_money(person.money);
        }

        // Check that adjustment factor is bounded
        assert!(
            person.strategy_params.adjustment_factor <= 2.0,
            "Adjustment factor should be capped at 2.0"
        );
        assert!(
            person.strategy_params.adjustment_factor >= 0.5,
            "Adjustment factor should not go below 0.5"
        );
    }

    #[test]
    fn test_can_afford_with_adjusted_strategy() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());

        // Initially with adjustment_factor = 1.0, Balanced strategy (1.0x multiplier)
        // Can afford up to 100.0 * 1.0 * 1.0 = 100.0
        assert!(person.can_afford_with_strategy(100.0));
        assert!(!person.can_afford_with_strategy(101.0));

        // Increase adjustment factor to simulate successful adaptation
        person.strategy_params.adjustment_factor = 1.5;

        // Now can afford up to 100.0 * 1.0 * 1.5 = 150.0
        assert!(person.can_afford_with_strategy(150.0));
        assert!(!person.can_afford_with_strategy(151.0));
    }

    #[test]
    fn test_record_successful_trades() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Initially no trades
        assert_eq!(person.strategy_params.total_successful_trades(), 0);

        // Record some trades
        person.strategy_params.record_successful_buy();
        person.strategy_params.record_successful_buy();
        person.strategy_params.record_successful_sell();

        assert_eq!(person.strategy_params.successful_buys, 2);
        assert_eq!(person.strategy_params.successful_sells, 1);
        assert_eq!(person.strategy_params.total_successful_trades(), 3);
    }

    #[test]
    fn test_specialization_strategy_quality_bonus_specialist() {
        let strategy = SpecializationStrategy::Specialist;
        assert_eq!(strategy.quality_bonus(), 1.0, "Specialist should have +1.0 quality bonus");
    }

    #[test]
    fn test_specialization_strategy_quality_bonus_balanced() {
        let strategy = SpecializationStrategy::Balanced;
        assert_eq!(strategy.quality_bonus(), 0.0, "Balanced should have no quality bonus");
    }

    #[test]
    fn test_specialization_strategy_quality_bonus_generalist() {
        let strategy = SpecializationStrategy::Generalist;
        assert_eq!(strategy.quality_bonus(), 0.0, "Generalist should have no quality bonus");
    }

    #[test]
    fn test_specialization_strategy_price_multiplier_specialist() {
        let strategy = SpecializationStrategy::Specialist;
        assert!(
            (strategy.price_multiplier() - 1.15).abs() < 0.001,
            "Specialist should have 1.15x price multiplier (15% premium)"
        );
    }

    #[test]
    fn test_specialization_strategy_price_multiplier_balanced() {
        let strategy = SpecializationStrategy::Balanced;
        assert_eq!(strategy.price_multiplier(), 1.0, "Balanced should have base price multiplier");
    }

    #[test]
    fn test_specialization_strategy_price_multiplier_generalist() {
        let strategy = SpecializationStrategy::Generalist;
        assert_eq!(
            strategy.price_multiplier(),
            1.0,
            "Generalist should have base price multiplier"
        );
    }

    #[test]
    fn test_specialization_strategy_all_variants() {
        let variants = SpecializationStrategy::all_variants();
        assert_eq!(variants.len(), 3, "Should have exactly 3 specialization strategy variants");
        assert_eq!(variants[0], SpecializationStrategy::Specialist);
        assert_eq!(variants[1], SpecializationStrategy::Balanced);
        assert_eq!(variants[2], SpecializationStrategy::Generalist);
    }

    #[test]
    fn test_person_specialization_strategy_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());
        assert_eq!(
            person.specialization_strategy,
            SpecializationStrategy::default(),
            "Specialization strategy should be default (Balanced)"
        );
    }

    #[test]
    fn test_can_afford_basic() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        assert!(person.can_afford(100.0));
        assert!(person.can_afford(50.0));
        assert!(person.can_afford(0.0));
        assert!(!person.can_afford(100.01));
        assert!(!person.can_afford(150.0));
    }

    #[test]
    fn test_can_afford_negative_money() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, -50.0, vec![skill], Strategy::default(), test_location());

        assert!(!person.can_afford(0.0));
        assert!(!person.can_afford(10.0));
    }

    #[test]
    fn test_record_transaction_buy() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.record_transaction(10, "SomeSkill".to_string(), TransactionType::Buy, 25.0, Some(2));

        assert_eq!(person.transaction_history.len(), 1);
        assert_eq!(person.transaction_history[0].step, 10);
        assert_eq!(person.transaction_history[0].skill_id, "SomeSkill");
        assert_eq!(person.transaction_history[0].amount, 25.0);
        assert_eq!(person.transaction_history[0].counterparty_id, Some(2));
    }

    #[test]
    fn test_record_transaction_sell() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.record_transaction(
            15,
            "AnotherSkill".to_string(),
            TransactionType::Sell,
            50.0,
            None,
        );

        assert_eq!(person.transaction_history.len(), 1);
        assert_eq!(person.transaction_history[0].step, 15);
        assert_eq!(person.transaction_history[0].skill_id, "AnotherSkill");
        assert_eq!(person.transaction_history[0].amount, 50.0);
        assert_eq!(person.transaction_history[0].counterparty_id, None);
    }

    #[test]
    fn test_record_multiple_transactions() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.record_transaction(1, "Skill1".to_string(), TransactionType::Buy, 10.0, Some(2));
        person.record_transaction(2, "Skill2".to_string(), TransactionType::Sell, 20.0, Some(3));
        person.record_transaction(3, "Skill3".to_string(), TransactionType::Buy, 15.0, None);

        assert_eq!(person.transaction_history.len(), 3);
    }

    #[test]
    fn test_is_healthy_default() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        assert!(person.is_healthy());
        assert!(!person.is_sick());
    }

    #[test]
    fn test_infect_person() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.infect(50);

        assert!(!person.is_healthy());
        assert!(person.is_sick());
        match person.health_status {
            HealthStatus::Sick { infected_at_step } => assert_eq!(infected_at_step, 50),
            _ => panic!("Expected Sick health status"),
        }
    }

    #[test]
    fn test_try_recover_success() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.infect(10);
        assert!(person.is_sick());

        // Try to recover before recovery duration - should fail
        let recovered = person.try_recover(14, 5);
        assert!(!recovered);
        assert!(person.is_sick());

        // Try to recover at exactly recovery duration - should succeed
        let recovered = person.try_recover(15, 5);
        assert!(recovered);
        assert!(person.is_healthy());
        assert!(!person.is_sick());
    }

    #[test]
    fn test_try_recover_after_duration() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.infect(10);

        // Try to recover well after recovery duration - should succeed
        let recovered = person.try_recover(50, 5);
        assert!(recovered);
        assert!(person.is_healthy());
    }

    #[test]
    fn test_try_recover_when_healthy() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Person is healthy, try_recover should return false
        let recovered = person.try_recover(100, 5);
        assert!(!recovered);
        assert!(person.is_healthy());
    }

    #[test]
    fn test_health_productivity_multiplier_healthy() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        assert_eq!(person.health_productivity_multiplier(), 1.0);
    }

    #[test]
    fn test_health_productivity_multiplier_sick() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.infect(10);
        assert_eq!(person.health_productivity_multiplier(), 0.5);
    }

    #[test]
    fn test_add_friend() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        assert_eq!(person.friend_count(), 0);
        assert!(!person.is_friend_with(2));

        person.add_friend(2);

        assert_eq!(person.friend_count(), 1);
        assert!(person.is_friend_with(2));
        assert!(!person.is_friend_with(3));
    }

    #[test]
    fn test_add_multiple_friends() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.add_friend(2);
        person.add_friend(3);
        person.add_friend(4);

        assert_eq!(person.friend_count(), 3);
        assert!(person.is_friend_with(2));
        assert!(person.is_friend_with(3));
        assert!(person.is_friend_with(4));
        assert!(!person.is_friend_with(5));
    }

    #[test]
    fn test_add_friend_duplicate() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.add_friend(2);
        person.add_friend(2); // Add same friend again

        // HashSet should deduplicate
        assert_eq!(person.friend_count(), 1);
    }

    #[test]
    fn test_calculate_influence_from_friends_zero() {
        let influence = Person::calculate_influence_from_friends(0);
        assert!(
            (influence - 1.0).abs() < 0.001,
            "0 friends should give baseline influence of 1.0"
        );
    }

    #[test]
    fn test_calculate_influence_from_friends_increases() {
        let influence_0 = Person::calculate_influence_from_friends(0);
        let influence_10 = Person::calculate_influence_from_friends(10);
        let influence_30 = Person::calculate_influence_from_friends(30);

        assert!(influence_10 > influence_0, "More friends should increase influence");
        assert!(
            influence_30 > influence_10,
            "Even more friends should further increase influence"
        );
    }

    #[test]
    fn test_calculate_influence_from_friends_logarithmic() {
        // Verify logarithmic scaling (diminishing returns)
        let influence_10 = Person::calculate_influence_from_friends(10);
        let influence_20 = Person::calculate_influence_from_friends(20);
        let influence_30 = Person::calculate_influence_from_friends(30);

        let growth_10_to_20 = influence_20 - influence_10;
        let growth_20_to_30 = influence_30 - influence_20;

        // Growth should diminish (logarithmic scaling)
        assert!(
            growth_10_to_20 > growth_20_to_30,
            "Influence growth should have diminishing returns"
        );
    }

    #[test]
    fn test_update_influence_score() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Initially 0 friends
        person.update_influence_score();
        let initial_influence = person.influence_score;
        assert!((initial_influence - 1.0).abs() < 0.001);

        // Add friends and update
        person.add_friend(2);
        person.add_friend(3);
        person.add_friend(4);
        person.update_influence_score();

        assert!(
            person.influence_score > initial_influence,
            "Influence should increase with friends"
        );
        let expected_influence = Person::calculate_influence_from_friends(3);
        assert!((person.influence_score - expected_influence).abs() < 0.001);
    }

    #[test]
    fn test_update_social_class_no_change() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Person starts as Middle class (default)
        assert_eq!(person.social_class, SocialClass::Middle);

        // Update with percentile that keeps them in Middle
        person.update_social_class(0.5, 10);

        assert_eq!(person.social_class, SocialClass::Middle);
        assert_eq!(person.class_history.len(), 0, "No change should not be recorded");
    }

    #[test]
    fn test_update_social_class_upward() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Person starts as Middle class
        assert_eq!(person.social_class, SocialClass::Middle);

        // Update to Upper class
        person.update_social_class(0.85, 100);

        assert_eq!(person.social_class, SocialClass::Upper);
        assert_eq!(person.class_history.len(), 1);
        assert_eq!(person.class_history[0].step, 100);
        assert_eq!(person.class_history[0].from_class, SocialClass::Middle);
        assert_eq!(person.class_history[0].to_class, SocialClass::Upper);
        assert!(person.class_history[0].is_upward());
        assert!(!person.class_history[0].is_downward());
    }

    #[test]
    fn test_update_social_class_downward() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        // Start as Middle class, update to Lower
        person.update_social_class(0.15, 50);

        assert_eq!(person.social_class, SocialClass::Lower);
        assert_eq!(person.class_history.len(), 1);
        assert!(person.class_history[0].is_downward());
        assert!(!person.class_history[0].is_upward());
    }

    #[test]
    fn test_update_social_class_multiple_changes() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        person.update_social_class(0.85, 10); // Move to Upper
        person.update_social_class(0.97, 20); // Move to Elite
        person.update_social_class(0.80, 30); // Move back to Upper

        assert_eq!(person.class_history.len(), 3);
        assert_eq!(person.social_class, SocialClass::Upper);
    }

    #[test]
    fn test_get_effective_spending_multiplier() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Aggressive, test_location());

        // Initial: Aggressive (1.3) * adjustment_factor (1.0) = 1.3
        let multiplier = person.get_effective_spending_multiplier();
        assert!((multiplier - 1.3).abs() < 0.001);

        // Change adjustment factor
        person.strategy_params.adjustment_factor = 1.5;
        let multiplier = person.get_effective_spending_multiplier();
        assert!((multiplier - 1.95).abs() < 0.001); // 1.3 * 1.5 = 1.95
    }

    #[test]
    fn test_social_class_from_percentile_lower() {
        assert_eq!(SocialClass::from_percentile(0.0), SocialClass::Lower);
        assert_eq!(SocialClass::from_percentile(0.10), SocialClass::Lower);
        assert_eq!(SocialClass::from_percentile(0.24), SocialClass::Lower);
    }

    #[test]
    fn test_social_class_from_percentile_middle() {
        assert_eq!(SocialClass::from_percentile(0.25), SocialClass::Middle);
        assert_eq!(SocialClass::from_percentile(0.50), SocialClass::Middle);
        assert_eq!(SocialClass::from_percentile(0.74), SocialClass::Middle);
    }

    #[test]
    fn test_social_class_from_percentile_upper() {
        assert_eq!(SocialClass::from_percentile(0.75), SocialClass::Upper);
        assert_eq!(SocialClass::from_percentile(0.85), SocialClass::Upper);
        assert_eq!(SocialClass::from_percentile(0.94), SocialClass::Upper);
    }

    #[test]
    fn test_social_class_from_percentile_elite() {
        assert_eq!(SocialClass::from_percentile(0.95), SocialClass::Elite);
        assert_eq!(SocialClass::from_percentile(0.99), SocialClass::Elite);
        assert_eq!(SocialClass::from_percentile(1.0), SocialClass::Elite);
    }

    #[test]
    fn test_social_class_all_variants() {
        let variants = SocialClass::all_variants();
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0], SocialClass::Lower);
        assert_eq!(variants[1], SocialClass::Middle);
        assert_eq!(variants[2], SocialClass::Upper);
        assert_eq!(variants[3], SocialClass::Elite);
    }

    #[test]
    fn test_social_class_description() {
        assert_eq!(SocialClass::Lower.description(), "Lower class (bottom 25%)");
        assert_eq!(SocialClass::Middle.description(), "Middle class (25th-75th percentile)");
        assert_eq!(SocialClass::Upper.description(), "Upper class (75th-95th percentile)");
        assert_eq!(SocialClass::Elite.description(), "Elite class (top 5%)");
    }

    #[test]
    fn test_social_class_default() {
        assert_eq!(SocialClass::default(), SocialClass::Middle);
    }

    #[test]
    fn test_strategy_all_variants() {
        let variants = Strategy::all_variants();
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0], Strategy::Conservative);
        assert_eq!(variants[1], Strategy::Balanced);
        assert_eq!(variants[2], Strategy::Aggressive);
        assert_eq!(variants[3], Strategy::Frugal);
    }

    #[test]
    fn test_specialization_strategy_default() {
        assert_eq!(SpecializationStrategy::default(), SpecializationStrategy::Balanced);
    }

    #[test]
    fn test_class_change_new() {
        let change = ClassChange::new(42, SocialClass::Lower, SocialClass::Upper);
        assert_eq!(change.step, 42);
        assert_eq!(change.from_class, SocialClass::Lower);
        assert_eq!(change.to_class, SocialClass::Upper);
    }

    #[test]
    fn test_class_change_same_class() {
        let change = ClassChange::new(100, SocialClass::Middle, SocialClass::Middle);
        assert!(!change.is_upward(), "Same class should not be upward");
        assert!(!change.is_downward(), "Same class should not be downward");
    }

    #[test]
    fn test_mentorship_new() {
        let mentorship = Mentorship::new(1, 2, "Programming".to_string(), 50);
        assert_eq!(mentorship.mentor_id, 1);
        assert_eq!(mentorship.mentee_id, 2);
        assert_eq!(mentorship.skill_id, "Programming");
        assert_eq!(mentorship.start_step, 50);
    }

    #[test]
    fn test_strategy_params_new() {
        let params = StrategyParameters::new(150.0);
        assert_eq!(params.initial_money, 150.0);
        assert_eq!(params.previous_money, 150.0);
        assert_eq!(params.successful_buys, 0);
        assert_eq!(params.successful_sells, 0);
        assert_eq!(params.adjustment_factor, 1.0);
        assert_eq!(params.adaptation_count, 0);
    }

    #[test]
    fn test_strategy_params_update_previous_money() {
        let mut params = StrategyParameters::new(100.0);
        assert_eq!(params.previous_money, 100.0);

        params.update_previous_money(120.0);
        assert_eq!(params.previous_money, 120.0);

        params.update_previous_money(95.0);
        assert_eq!(params.previous_money, 95.0);
    }

    #[test]
    fn test_strategy_params_growth_rate_zero_previous_money() {
        let params = StrategyParameters::new(0.0);
        let growth_rate = params.calculate_short_term_growth_rate(100.0);
        assert_eq!(growth_rate, 0.0, "Should return 0 when previous_money is 0");
    }

    #[test]
    fn test_strategy_params_growth_rate_zero_initial_money() {
        let mut params = StrategyParameters::new(0.0);
        params.previous_money = 50.0;
        let long_term_growth = params.calculate_long_term_growth_rate(100.0);
        assert_eq!(long_term_growth, 0.0, "Should return 0 when initial_money is 0");
    }

    #[test]
    fn test_strategy_params_negative_growth() {
        let mut params = StrategyParameters::new(100.0);
        params.previous_money = 100.0;
        let growth_rate = params.calculate_short_term_growth_rate(80.0);
        assert!((growth_rate - (-0.2)).abs() < 0.01, "Should be -0.2 (20% decline)");
    }

    #[test]
    fn test_strategy_params_total_successful_trades_zero() {
        let params = StrategyParameters::new(100.0);
        assert_eq!(params.total_successful_trades(), 0);
    }

    #[test]
    fn test_needed_skill_item_creation() {
        let item = NeededSkillItem { id: "TestSkill".to_string(), urgency: 2 };
        assert_eq!(item.id, "TestSkill");
        assert_eq!(item.urgency, 2);
    }

    #[test]
    fn test_health_status_healthy_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_sick_equality() {
        let sick1 = HealthStatus::Sick { infected_at_step: 10 };
        let sick2 = HealthStatus::Sick { infected_at_step: 10 };
        let sick3 = HealthStatus::Sick { infected_at_step: 20 };

        assert_eq!(sick1, sick2);
        assert_ne!(sick1, sick3);
        assert_ne!(HealthStatus::Healthy, sick1);
    }

    #[test]
    fn test_location_negative_coordinates() {
        let loc = Location::new(-10.0, -20.0);
        assert_eq!(loc.x, -10.0);
        assert_eq!(loc.y, -20.0);
    }

    #[test]
    fn test_location_distance_with_negatives() {
        let loc1 = Location::new(-3.0, -4.0);
        let loc2 = Location::new(0.0, 0.0);
        assert_eq!(loc1.distance_to(&loc2), 5.0);
    }

    #[test]
    fn test_person_initialization_all_fields() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let location = Location::new(25.0, 75.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Conservative, location);

        assert_eq!(person.id, 1);
        assert_eq!(person.money, 100.0);
        assert_eq!(person.own_skills.len(), 1);
        assert_eq!(person.needed_skills.len(), 0);
        assert_eq!(person.transaction_history.len(), 0);
        assert_eq!(person.satisfied_needs_current_step.len(), 0);
        assert_eq!(person.reputation, 1.0);
        assert_eq!(person.savings, 0.0);
        assert_eq!(person.borrowed_loans.len(), 0);
        assert_eq!(person.lent_loans.len(), 0);
        assert_eq!(person.active_investments.len(), 0);
        assert_eq!(person.strategy, Strategy::Conservative);
        assert_eq!(person.specialization_strategy, SpecializationStrategy::Balanced);
        assert_eq!(person.learned_skills.len(), 0);
        assert_eq!(person.friends.len(), 0);
        assert_eq!(person.trade_agreement_ids.len(), 0);
        assert_eq!(person.group_id, None);
        assert_eq!(person.location.x, 25.0);
        assert_eq!(person.location.y, 75.0);
        assert_eq!(person.skill_qualities.len(), 0);
        assert_eq!(person.insurance_policies.len(), 0);
        assert_eq!(person.health_status, HealthStatus::Healthy);
        assert_eq!(person.influence_score, 1.0);
        assert_eq!(person.social_class, SocialClass::Middle);
        assert_eq!(person.class_history.len(), 0);
        assert_eq!(person.owned_assets.len(), 0);
    }

    #[test]
    fn test_person_with_multiple_skills() {
        let skill1 = Skill::new("Skill1".to_string(), 10.0);
        let skill2 = Skill::new("Skill2".to_string(), 20.0);
        let skill3 = Skill::new("Skill3".to_string(), 30.0);
        let person = Person::new(
            1,
            100.0,
            vec![skill1, skill2, skill3],
            Strategy::default(),
            test_location(),
        );

        assert_eq!(person.own_skills.len(), 3);
        assert_eq!(person.all_skills().len(), 3);
    }

    #[test]
    fn test_strategy_adaptation_exploration() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::Balanced, test_location());
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let _initial_factor = person.strategy_params.adjustment_factor;

        // With 100% exploration rate, should always adapt randomly
        let adapted = person.adapt_strategy(0.1, &mut rng, 1.0);

        assert!(adapted, "Should adapt with exploration");
        // Factor may change due to random exploration
        assert_eq!(person.strategy_params.adaptation_count, 1);
    }

    #[test]
    fn test_savings_negative_rate() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default(), test_location());

        let saved = person.apply_savings(-0.1);
        assert_eq!(saved, 0.0, "Should not save with negative rate");
        assert_eq!(person.money, 100.0);
        assert_eq!(person.savings, 0.0);
    }
}
