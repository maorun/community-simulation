use crate::loan::LoanId;
use crate::skill::{Skill, SkillId};
use serde::{Deserialize, Serialize};

pub type PersonId = usize;
pub type UrgencyLevel = u8; // Define UrgencyLevel (e.g., 1-3, higher is more urgent)

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
}

impl Person {
    /// Creates a new person with multiple skills.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this person
    /// * `initial_money` - Starting money amount
    /// * `own_skills` - Vector of skills this person can provide
    pub fn new(id: PersonId, initial_money: f64, own_skills: Vec<Skill>) -> Self {
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
        }
    }

    pub fn can_afford(&self, amount: f64) -> bool {
        self.money >= amount
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::Skill;

    #[test]
    fn test_person_reputation_initialization() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let person = Person::new(1, 100.0, skill);
        assert_eq!(person.reputation, 1.0, "Reputation should start at 1.0");
    }

    #[test]
    fn test_increase_reputation_as_seller() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, skill);

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
        let mut person = Person::new(1, 100.0, skill);

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
        let mut person = Person::new(1, 100.0, skill);
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
        let mut person = Person::new(1, 100.0, skill);
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
        let person = Person::new(1, 100.0, skill);
        let multiplier = person.reputation_price_multiplier();
        assert_eq!(
            multiplier, 1.0,
            "Neutral reputation should have no price effect"
        );
    }

    #[test]
    fn test_reputation_price_multiplier_high() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, skill);
        person.reputation = 2.0; // Maximum reputation

        let multiplier = person.reputation_price_multiplier();
        assert_eq!(multiplier, 0.9, "High reputation should give 10% discount");
    }

    #[test]
    fn test_reputation_price_multiplier_low() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, skill);
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
        let mut person = Person::new(1, 100.0, skill);

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
        let mut person = Person::new(1, 100.0, skill);

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
        let mut person = Person::new(1, 100.0, skill);

        let saved = person.apply_savings(0.0);
        assert_eq!(saved, 0.0, "Should save nothing with 0% rate");
        assert_eq!(person.money, 100.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_negative_money() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, -10.0, skill);

        let saved = person.apply_savings(0.1);
        assert_eq!(saved, 0.0, "Should not save with negative money");
        assert_eq!(person.money, -10.0, "Money should not change");
        assert_eq!(person.savings, 0.0, "Savings should remain 0");
    }

    #[test]
    fn test_savings_full_amount() {
        let skill = Skill::new("TestSkill".to_string(), 10.0);
        let mut person = Person::new(1, 100.0, skill);

        let saved = person.apply_savings(1.0);
        assert_eq!(saved, 100.0, "Should save 100% (all money)");
        assert_eq!(person.money, 0.0, "Money should be 0");
        assert_eq!(person.savings, 100.0, "Savings should be 100");
    }
}
