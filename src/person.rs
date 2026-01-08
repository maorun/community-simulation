use crate::loan::LoanId;
use crate::skill::{Skill, SkillId};
use serde::{Deserialize, Serialize};

pub type PersonId = usize;
pub type UrgencyLevel = u8; // Define UrgencyLevel (e.g., 1-3, higher is more urgent)

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
    /// Behavioral strategy that affects spending decisions.
    /// Determines how aggressively this person spends money to acquire needed skills.
    pub strategy: Strategy,
    /// Skills that this person has learned through education.
    /// These skills can also be provided to others in the market.
    pub learned_skills: Vec<Skill>,
}

impl Person {
    /// Creates a new person with multiple skills.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this person
    /// * `initial_money` - Starting money amount
    /// * `own_skills` - Vector of skills this person can provide
    /// * `strategy` - Behavioral strategy for spending decisions
    pub fn new(
        id: PersonId,
        initial_money: f64,
        own_skills: Vec<Skill>,
        strategy: Strategy,
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
            strategy,
            learned_skills: Vec::new(), // Start with no learned skills
        }
    }

    pub fn can_afford(&self, amount: f64) -> bool {
        self.money >= amount
    }

    /// Checks if the person can afford a purchase considering their behavioral strategy.
    /// Different strategies have different spending thresholds.
    ///
    /// # Arguments
    /// * `amount` - The cost of the skill to purchase
    ///
    /// # Returns
    /// `true` if the person's money multiplied by their strategy's spending multiplier
    /// is greater than or equal to the amount, `false` otherwise.
    ///
    /// # Examples
    /// - A Conservative person with $100 and a 0.7x multiplier can afford items up to $70
    /// - An Aggressive person with $100 and a 1.3x multiplier can afford items up to $130
    pub fn can_afford_with_strategy(&self, amount: f64) -> bool {
        let effective_money = self.money * self.strategy.spending_multiplier();
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
        let transaction = Transaction {
            step,
            skill_id,
            transaction_type,
            amount,
            counterparty_id,
        };
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

    /// Returns all skills this person can provide (both own_skills and learned_skills).
    ///
    /// # Returns
    /// A vector of references to all skills this person possesses
    pub fn all_skills(&self) -> Vec<&Skill> {
        self.own_skills
            .iter()
            .chain(self.learned_skills.iter())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::Skill;

    #[test]
    fn test_person_reputation_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::default());
        assert_eq!(person.reputation, 1.0, "Reputation should start at 1.0");
    }

    #[test]
    fn test_increase_reputation_as_seller() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

        person.increase_reputation_as_seller();
        assert_eq!(
            person.reputation, 1.01,
            "Reputation should increase by 0.01"
        );

        // Test multiple increases
        for _ in 0..99 {
            person.increase_reputation_as_seller();
        }
        assert_eq!(person.reputation, 2.0, "Reputation should be capped at 2.0");
    }

    #[test]
    fn test_increase_reputation_as_buyer() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

        person.increase_reputation_as_buyer();
        assert_eq!(
            person.reputation, 1.005,
            "Reputation should increase by 0.005"
        );

        // Test cap
        for _ in 0..200 {
            person.increase_reputation_as_buyer();
        }
        assert_eq!(person.reputation, 2.0, "Reputation should be capped at 2.0");
    }

    #[test]
    fn test_reputation_decay_above_neutral() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());
        person.reputation = 1.5;

        person.apply_reputation_decay();
        assert_eq!(
            person.reputation, 1.499,
            "Reputation should decay toward neutral"
        );

        // Test decay stops at neutral
        person.reputation = 1.0;
        person.apply_reputation_decay();
        assert_eq!(
            person.reputation, 1.0,
            "Reputation should not decay below neutral"
        );
    }

    #[test]
    fn test_reputation_decay_below_neutral() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());
        person.reputation = 0.5;

        person.apply_reputation_decay();
        assert_eq!(
            person.reputation, 0.501,
            "Reputation should increase toward neutral"
        );

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
        let person = Person::new(1, 100.0, vec![skill], Strategy::default());
        let multiplier = person.reputation_price_multiplier();
        assert_eq!(
            multiplier, 1.0,
            "Neutral reputation should have no price effect"
        );
    }

    #[test]
    fn test_reputation_price_multiplier_high() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());
        person.reputation = 2.0; // Maximum reputation

        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 0.9, "High reputation should give 10% discount");
    }

    #[test]
    fn test_reputation_price_multiplier_low() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());
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
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

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
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

        // Test 10% savings rate
        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 10.0, "Should save 10% of 100");
        assert_eq!(
            person.money, 90.0,
            "Money should be reduced by saved amount"
        );
        assert_eq!(person.savings, 10.0, "Savings should be 10");

        // Apply savings again - now on 90.0
        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 9.0, "Should save 10% of 90");
        assert_eq!(
            person.money, 81.0,
            "Money should be 81 after second savings"
        );
        assert_eq!(person.savings, 19.0, "Savings should accumulate to 19");
    }

    #[test]
    fn test_savings_zero_rate() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

        let saved = person.apply_savings(0.0);
        assert_eq!(saved, 0.0, "Should save nothing with 0% rate");
        assert_eq!(person.money, 100.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_negative_money() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, -10.0, vec![skill], Strategy::default());

        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 0.0, "Should not save with negative money");
        assert_eq!(person.money, -10.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_full_amount() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![skill], Strategy::default());

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
        let person = Person::new(1, 100.0, vec![skill], Strategy::Conservative);

        // Conservative has 0.7x multiplier, so with $100 can afford up to $70
        assert!(person.can_afford_with_strategy(70.0));
        assert!(person.can_afford_with_strategy(69.0));
        assert!(!person.can_afford_with_strategy(71.0));
        assert!(!person.can_afford_with_strategy(100.0));
    }

    #[test]
    fn test_can_afford_with_strategy_balanced() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Balanced);

        // Balanced has 1.0x multiplier, so with $100 can afford up to $100
        assert!(person.can_afford_with_strategy(100.0));
        assert!(person.can_afford_with_strategy(99.0));
        assert!(!person.can_afford_with_strategy(101.0));
    }

    #[test]
    fn test_can_afford_with_strategy_aggressive() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Aggressive);

        // Aggressive has 1.3x multiplier, so with $100 can afford up to $130
        assert!(person.can_afford_with_strategy(130.0));
        assert!(person.can_afford_with_strategy(129.0));
        assert!(!person.can_afford_with_strategy(131.0));
    }

    #[test]
    fn test_can_afford_with_strategy_frugal() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Frugal);

        // Frugal has 0.5x multiplier, so with $100 can afford up to $50
        assert!(person.can_afford_with_strategy(50.0));
        assert!(person.can_afford_with_strategy(49.0));
        assert!(!person.can_afford_with_strategy(51.0));
        assert!(!person.can_afford_with_strategy(100.0));
    }

    #[test]
    fn test_person_has_strategy_field() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![skill], Strategy::Aggressive);

        assert_eq!(person.strategy, Strategy::Aggressive);
    }

    #[test]
    fn test_learn_skill_success() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![own_skill], Strategy::default());

        let new_skill = Skill::new("NewSkill".to_string(), 15.0);
        let learning_cost = 30.0;

        let result = person.learn_skill(new_skill.clone(), learning_cost);

        assert!(result, "Should successfully learn the skill");
        assert_eq!(person.money, 70.0, "Money should be deducted");
        assert_eq!(
            person.learned_skills.len(),
            1,
            "Should have one learned skill"
        );
        assert_eq!(
            person.learned_skills[0].id, "NewSkill",
            "Learned skill should be NewSkill"
        );
    }

    #[test]
    fn test_learn_skill_cannot_afford() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 50.0, vec![own_skill], Strategy::default());

        let new_skill = Skill::new("ExpensiveSkill".to_string(), 15.0);
        let learning_cost = 100.0; // More than person has

        let result = person.learn_skill(new_skill, learning_cost);

        assert!(!result, "Should fail to learn due to insufficient money");
        assert_eq!(person.money, 50.0, "Money should not be deducted");
        assert_eq!(
            person.learned_skills.len(),
            0,
            "Should have no learned skills"
        );
    }

    #[test]
    fn test_learn_skill_already_has_as_own_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![own_skill.clone()], Strategy::default());

        let learning_cost = 30.0;
        let result = person.learn_skill(own_skill, learning_cost);

        assert!(
            !result,
            "Should fail to learn skill they already have as own_skill"
        );
        assert_eq!(person.money, 100.0, "Money should not be deducted");
        assert_eq!(
            person.learned_skills.len(),
            0,
            "Should have no learned skills"
        );
    }

    #[test]
    fn test_learn_skill_already_learned() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![own_skill], Strategy::default());

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
        assert_eq!(
            person.learned_skills.len(),
            1,
            "Should still have only one learned skill"
        );
    }

    #[test]
    fn test_has_skill_with_own_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, vec![own_skill], Strategy::default());

        assert!(
            person.has_skill(&"OwnSkill".to_string()),
            "Should have OwnSkill as own_skill"
        );
        assert!(
            !person.has_skill(&"OtherSkill".to_string()),
            "Should not have OtherSkill"
        );
    }

    #[test]
    fn test_has_skill_with_learned_skill() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![own_skill], Strategy::default());

        let learned_skill = Skill::new("LearnedSkill".to_string(), 15.0);
        person.learn_skill(learned_skill, 30.0);

        assert!(
            person.has_skill(&"LearnedSkill".to_string()),
            "Should have LearnedSkill as learned_skill"
        );
        assert!(
            person.has_skill(&"OwnSkill".to_string()),
            "Should still have OwnSkill"
        );
    }

    #[test]
    fn test_all_skills() {
        let own_skill = Skill::new("OwnSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, vec![own_skill], Strategy::default());

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
}
