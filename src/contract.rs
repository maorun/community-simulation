use crate::person::PersonId;
use crate::skill::SkillId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a contract
pub type ContractId = usize;

/// Represents a long-term agreement between two persons for regular skill exchanges
///
/// A contract is an agreement where one person (buyer) commits to purchasing a skill
/// from another person (seller) at a fixed price over multiple simulation steps.
/// This provides price stability and predictable income/expenses for both parties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique identifier for this contract
    pub id: ContractId,
    /// The person buying the skill (consumer)
    pub buyer_id: PersonId,
    /// The person selling the skill (provider)
    pub seller_id: PersonId,
    /// The skill being traded
    pub skill_id: SkillId,
    /// The agreed-upon fixed price per transaction
    pub price: f64,
    /// The total number of steps this contract is valid for
    pub duration: usize,
    /// The number of steps remaining until the contract expires
    pub remaining_steps: usize,
    /// The simulation step when the contract was created
    pub created_at_step: usize,
    /// The total number of transactions executed under this contract
    pub transactions_executed: usize,
    /// Whether the contract is still active (not expired or terminated)
    pub is_active: bool,
}

impl Contract {
    /// Creates a new contract with the specified parameters
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this contract
    /// * `buyer_id` - The person who will buy the skill
    /// * `seller_id` - The person who will sell the skill
    /// * `skill_id` - The skill being contracted
    /// * `price` - The fixed price per transaction
    /// * `duration` - The number of steps this contract lasts
    /// * `created_at_step` - The simulation step when the contract is created
    ///
    /// # Returns
    ///
    /// A new `Contract` instance ready for execution
    pub fn new(
        id: ContractId,
        buyer_id: PersonId,
        seller_id: PersonId,
        skill_id: SkillId,
        price: f64,
        duration: usize,
        created_at_step: usize,
    ) -> Self {
        Contract {
            id,
            buyer_id,
            seller_id,
            skill_id,
            price,
            duration,
            remaining_steps: duration,
            created_at_step,
            transactions_executed: 0,
            is_active: true,
        }
    }

    /// Checks if the contract is still active
    ///
    /// A contract is active if it hasn't expired and hasn't been terminated
    pub fn is_active(&self) -> bool {
        self.is_active && self.remaining_steps > 0
    }

    /// Executes one step of the contract, decrementing remaining steps
    ///
    /// This should be called when a transaction is successfully completed under this contract
    pub fn execute_step(&mut self) {
        if self.is_active() {
            self.remaining_steps = self.remaining_steps.saturating_sub(1);
            self.transactions_executed += 1;

            if self.remaining_steps == 0 {
                self.is_active = false;
            }
        }
    }

    /// Terminates the contract early (before expiration)
    ///
    /// This might be called if one party can no longer fulfill the contract
    pub fn terminate(&mut self) {
        self.is_active = false;
    }

    /// Returns the total value of all transactions that have been executed
    pub fn total_value_exchanged(&self) -> f64 {
        self.transactions_executed as f64 * self.price
    }

    /// Returns the expected total value when the contract completes
    pub fn expected_total_value(&self) -> f64 {
        self.duration as f64 * self.price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper constant for skill ID following the "SkillN" pattern used in the codebase
    const TEST_SKILL_ID: &str = "Skill5";

    #[test]
    fn test_contract_creation() {
        let contract = Contract::new(1, 10, 20, TEST_SKILL_ID.to_string(), 15.0, 10, 100);

        assert_eq!(contract.id, 1);
        assert_eq!(contract.buyer_id, 10);
        assert_eq!(contract.seller_id, 20);
        assert_eq!(contract.skill_id, TEST_SKILL_ID);
        assert_eq!(contract.price, 15.0);
        assert_eq!(contract.duration, 10);
        assert_eq!(contract.remaining_steps, 10);
        assert_eq!(contract.created_at_step, 100);
        assert_eq!(contract.transactions_executed, 0);
        assert!(contract.is_active());
    }

    #[test]
    fn test_contract_execution() {
        let mut contract = Contract::new(1, 10, 20, TEST_SKILL_ID.to_string(), 15.0, 3, 100);

        assert!(contract.is_active());
        assert_eq!(contract.remaining_steps, 3);
        assert_eq!(contract.transactions_executed, 0);

        contract.execute_step();
        assert!(contract.is_active());
        assert_eq!(contract.remaining_steps, 2);
        assert_eq!(contract.transactions_executed, 1);

        contract.execute_step();
        assert!(contract.is_active());
        assert_eq!(contract.remaining_steps, 1);
        assert_eq!(contract.transactions_executed, 2);

        contract.execute_step();
        assert!(!contract.is_active()); // Contract should expire after last step
        assert_eq!(contract.remaining_steps, 0);
        assert_eq!(contract.transactions_executed, 3);
    }

    #[test]
    fn test_contract_termination() {
        let mut contract = Contract::new(1, 10, 20, TEST_SKILL_ID.to_string(), 15.0, 10, 100);

        assert!(contract.is_active());

        contract.terminate();

        assert!(!contract.is_active());
        // Remaining steps should still reflect original value
        assert_eq!(contract.remaining_steps, 10);
    }

    #[test]
    fn test_contract_value_calculations() {
        let mut contract = Contract::new(1, 10, 20, TEST_SKILL_ID.to_string(), 15.0, 5, 100);

        assert_eq!(contract.expected_total_value(), 75.0); // 5 * 15.0
        assert_eq!(contract.total_value_exchanged(), 0.0);

        contract.execute_step();
        assert_eq!(contract.total_value_exchanged(), 15.0);

        contract.execute_step();
        assert_eq!(contract.total_value_exchanged(), 30.0);

        assert_eq!(contract.expected_total_value(), 75.0); // Unchanged
    }

    #[test]
    fn test_execute_after_expiration() {
        let mut contract = Contract::new(1, 10, 20, TEST_SKILL_ID.to_string(), 15.0, 1, 100);

        contract.execute_step();
        assert!(!contract.is_active());

        // Executing after expiration should not change state
        contract.execute_step();
        assert_eq!(contract.transactions_executed, 1);
        assert_eq!(contract.remaining_steps, 0);
    }
}
