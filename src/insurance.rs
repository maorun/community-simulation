use crate::person::PersonId;
use serde::{Deserialize, Serialize};

/// Reputation adjustment factor for insurance premiums (±20%)
///
/// Higher reputation leads to lower premiums (better risk), lower reputation
/// leads to higher premiums (worse risk). At reputation 2.0 (excellent):
/// 20% discount. At reputation 0.0 (worst): 20% premium increase.
pub const REPUTATION_PREMIUM_ADJUSTMENT: f64 = 0.2;

/// Unique identifier for an insurance policy
pub type InsuranceId = usize;

/// Types of insurance coverage available in the simulation
///
/// Insurance policies protect against different types of economic risks,
/// providing payouts when certain adverse events occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsuranceType {
    /// Covers credit default risk - pays out when loans cannot be repaid
    ///
    /// Protects lenders against borrower default. When a borrower cannot
    /// make loan payments, this insurance provides compensation to the lender.
    Credit,

    /// Covers income risk - pays out when income falls below threshold
    ///
    /// Provides support when a person's income (trade proceeds) falls too low,
    /// helping maintain minimum living standards during economic hardship.
    Income,

    /// Covers crisis event risk - pays out during economic crises
    ///
    /// Protects against catastrophic economic events like market crashes,
    /// currency devaluations, or technology shocks that affect wealth.
    Crisis,
}

impl InsuranceType {
    /// Returns all insurance types for iteration
    pub fn all_types() -> [InsuranceType; 3] {
        [
            InsuranceType::Credit,
            InsuranceType::Income,
            InsuranceType::Crisis,
        ]
    }

    /// Returns a human-readable name for this insurance type
    pub fn name(&self) -> &str {
        match self {
            InsuranceType::Credit => "Credit Insurance",
            InsuranceType::Income => "Income Insurance",
            InsuranceType::Crisis => "Crisis Insurance",
        }
    }
}

/// Represents an insurance policy in the simulation
///
/// Insurance policies are purchased by persons to protect against various
/// economic risks. Premiums are paid upfront, and payouts occur when
/// specified trigger conditions are met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insurance {
    /// Unique identifier for this insurance policy
    pub id: InsuranceId,

    /// The person who owns this insurance policy (the insured)
    pub owner_id: PersonId,

    /// The type of coverage provided by this policy
    pub insurance_type: InsuranceType,

    /// The premium amount paid for this policy (one-time upfront payment)
    pub premium: f64,

    /// The maximum coverage amount (payout limit)
    pub coverage: f64,

    /// The simulation step when the policy was purchased
    pub purchased_at_step: usize,

    /// The policy duration in simulation steps
    ///
    /// After this many steps, the policy expires and must be renewed.
    /// Set to 0 for indefinite coverage.
    pub duration: usize,

    /// Whether a claim has been filed and paid out for this policy
    pub has_claimed: bool,

    /// The amount paid out if a claim was made
    pub payout_amount: f64,

    /// Whether this policy is still active (not expired and not fully paid out)
    pub is_active: bool,
}

impl Insurance {
    /// Creates a new insurance policy
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this policy
    /// * `owner_id` - The person purchasing the insurance
    /// * `insurance_type` - The type of coverage
    /// * `premium` - The upfront premium cost
    /// * `coverage` - The maximum payout amount
    /// * `duration` - How many steps the policy remains active (0 = indefinite)
    /// * `purchased_at_step` - The simulation step when purchased
    ///
    /// # Returns
    ///
    /// A new `Insurance` policy instance
    pub fn new(
        id: InsuranceId,
        owner_id: PersonId,
        insurance_type: InsuranceType,
        premium: f64,
        coverage: f64,
        duration: usize,
        purchased_at_step: usize,
    ) -> Self {
        Insurance {
            id,
            owner_id,
            insurance_type,
            premium,
            coverage,
            purchased_at_step,
            duration,
            has_claimed: false,
            payout_amount: 0.0,
            is_active: true,
        }
    }

    /// Checks if this policy has expired
    ///
    /// # Arguments
    ///
    /// * `current_step` - The current simulation step
    ///
    /// # Returns
    ///
    /// `true` if the policy has expired (duration > 0 and elapsed time >= duration)
    pub fn is_expired(&self, current_step: usize) -> bool {
        if self.duration == 0 {
            return false; // Indefinite coverage never expires
        }
        current_step >= self.purchased_at_step + self.duration
    }

    /// Processes an insurance claim
    ///
    /// # Arguments
    ///
    /// * `claim_amount` - The amount being claimed (limited by coverage)
    /// * `current_step` - The current simulation step
    ///
    /// # Returns
    ///
    /// The actual payout amount (min of claim_amount and remaining coverage)
    pub fn file_claim(&mut self, claim_amount: f64, current_step: usize) -> f64 {
        // Check if policy is still active
        if !self.is_active || self.has_claimed || self.is_expired(current_step) {
            return 0.0;
        }

        // Calculate payout (limited by coverage)
        let payout = claim_amount.min(self.coverage);

        // Update policy state
        self.has_claimed = true;
        self.payout_amount = payout;
        self.is_active = false; // Policy expires after claim

        payout
    }

    /// Deactivates the policy (e.g., when it expires without a claim)
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Calculates the base premium for an insurance policy based on coverage amount
    ///
    /// # Arguments
    ///
    /// * `coverage` - The desired coverage amount
    /// * `base_multiplier` - Base premium rate (e.g., 0.05 = 5% of coverage)
    ///
    /// # Returns
    ///
    /// The base premium amount before reputation adjustments
    pub fn calculate_base_premium(coverage: f64, base_multiplier: f64) -> f64 {
        coverage * base_multiplier
    }

    /// Adjusts premium based on person's reputation
    ///
    /// Higher reputation leads to lower premiums (less risk for insurer).
    ///
    /// # Arguments
    ///
    /// * `base_premium` - The base premium before reputation adjustment
    /// * `reputation` - The person's reputation score (typically 0.0-2.0)
    ///
    /// # Returns
    ///
    /// The adjusted premium amount
    pub fn apply_reputation_discount(base_premium: f64, reputation: f64) -> f64 {
        // Reputation typically ranges from 0.0 to 2.0 (capped at both ends)
        // At reputation 1.0 (neutral): no adjustment
        // At reputation 2.0 (excellent): REPUTATION_PREMIUM_ADJUSTMENT discount (20%)
        // At reputation 0.0 (worst): REPUTATION_PREMIUM_ADJUSTMENT increase (20%)
        // Formula: multiplier = 1.0 - (reputation - 1.0) * REPUTATION_PREMIUM_ADJUSTMENT
        let multiplier = 1.0 - (reputation - 1.0) * REPUTATION_PREMIUM_ADJUSTMENT;
        let min_multiplier = 1.0 - REPUTATION_PREMIUM_ADJUSTMENT; // 0.8 (20% discount)
        let max_multiplier = 1.0 + REPUTATION_PREMIUM_ADJUSTMENT; // 1.2 (20% increase)
        let multiplier = multiplier.clamp(min_multiplier, max_multiplier);
        base_premium * multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insurance_creation() {
        let insurance = Insurance::new(1, 42, InsuranceType::Crisis, 10.0, 100.0, 50, 0);

        assert_eq!(insurance.id, 1);
        assert_eq!(insurance.owner_id, 42);
        assert_eq!(insurance.insurance_type, InsuranceType::Crisis);
        assert_eq!(insurance.premium, 10.0);
        assert_eq!(insurance.coverage, 100.0);
        assert_eq!(insurance.duration, 50);
        assert!(insurance.is_active);
        assert!(!insurance.has_claimed);
        assert_eq!(insurance.payout_amount, 0.0);
    }

    #[test]
    fn test_insurance_expiration() {
        let insurance = Insurance::new(1, 42, InsuranceType::Crisis, 10.0, 100.0, 50, 0);

        assert!(!insurance.is_expired(0));
        assert!(!insurance.is_expired(49));
        assert!(insurance.is_expired(50));
        assert!(insurance.is_expired(100));
    }

    #[test]
    fn test_indefinite_insurance_never_expires() {
        let insurance = Insurance::new(
            1,
            42,
            InsuranceType::Income,
            10.0,
            100.0,
            0, // indefinite
            0,
        );

        assert!(!insurance.is_expired(0));
        assert!(!insurance.is_expired(1000));
        assert!(!insurance.is_expired(10000));
    }

    #[test]
    fn test_insurance_claim() {
        let mut insurance = Insurance::new(1, 42, InsuranceType::Crisis, 10.0, 100.0, 50, 0);

        // Claim within coverage
        let payout = insurance.file_claim(50.0, 10);
        assert_eq!(payout, 50.0);
        assert!(insurance.has_claimed);
        assert!(!insurance.is_active);
        assert_eq!(insurance.payout_amount, 50.0);

        // Second claim should fail (already claimed)
        let payout2 = insurance.file_claim(50.0, 11);
        assert_eq!(payout2, 0.0);
    }

    #[test]
    fn test_insurance_claim_exceeds_coverage() {
        let mut insurance = Insurance::new(1, 42, InsuranceType::Income, 10.0, 100.0, 50, 0);

        // Claim exceeds coverage - should be capped
        let payout = insurance.file_claim(150.0, 10);
        assert_eq!(payout, 100.0); // Limited to coverage amount
        assert_eq!(insurance.payout_amount, 100.0);
    }

    #[test]
    fn test_expired_policy_claim_fails() {
        let mut insurance = Insurance::new(1, 42, InsuranceType::Crisis, 10.0, 100.0, 50, 0);

        // Try to claim after expiration
        let payout = insurance.file_claim(50.0, 60);
        assert_eq!(payout, 0.0);
        assert!(!insurance.has_claimed);
    }

    #[test]
    fn test_base_premium_calculation() {
        let premium = Insurance::calculate_base_premium(100.0, 0.05);
        assert_eq!(premium, 5.0); // 5% of 100

        let premium2 = Insurance::calculate_base_premium(200.0, 0.1);
        assert_eq!(premium2, 20.0); // 10% of 200
    }

    #[test]
    fn test_reputation_discount() {
        // Excellent reputation (2.0) -> 20% discount
        let adjusted = Insurance::apply_reputation_discount(100.0, 2.0);
        assert_eq!(adjusted, 80.0);

        // Neutral reputation (1.0) -> no change
        let adjusted = Insurance::apply_reputation_discount(100.0, 1.0);
        assert_eq!(adjusted, 100.0);

        // Poor reputation (0.0) -> 20% increase
        let adjusted = Insurance::apply_reputation_discount(100.0, 0.0);
        assert_eq!(adjusted, 120.0);
    }

    #[test]
    fn test_insurance_types() {
        let types = InsuranceType::all_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&InsuranceType::Credit));
        assert!(types.contains(&InsuranceType::Income));
        assert!(types.contains(&InsuranceType::Crisis));
    }
}
