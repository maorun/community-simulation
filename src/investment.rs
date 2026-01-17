use crate::person::PersonId;
use crate::skill::SkillId;
use serde::{Deserialize, Serialize};

/// Unique identifier for an investment
pub type InvestmentId = usize;

/// Types of investments that can be made in the simulation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentType {
    /// Investment in another person's education/skill development
    Education {
        /// The skill being learned by the target person
        skill_id: SkillId,
    },
    /// Investment in production capacity
    Production {
        /// The production recipe being enhanced
        recipe_name: String,
    },
}

/// Represents an investment made by one person in the simulation
///
/// An investment is a commitment of money with the expectation of future returns.
/// Returns are paid periodically over the investment duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    /// Unique identifier for this investment
    pub id: InvestmentId,
    /// The person making the investment
    pub investor_id: PersonId,
    /// The target of the investment (if applicable, e.g., for education investments)
    pub target_id: Option<PersonId>,
    /// The type of investment
    pub investment_type: InvestmentType,
    /// The initial amount invested
    pub principal: f64,
    /// The expected return rate per step (e.g., 0.02 = 2% per step)
    pub return_rate: f64,
    /// The number of steps over which returns are paid
    pub duration: usize,
    /// The simulation step when the investment was created
    pub created_at_step: usize,
    /// The return payment per step
    pub return_per_step: f64,
    /// The number of returns already paid
    pub returns_paid: usize,
    /// Total returns paid so far
    pub total_returns_paid: f64,
    /// Whether the investment has completed its duration
    pub is_completed: bool,
}

impl Investment {
    /// Creates a new investment with the specified parameters
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this investment
    /// * `investor_id` - The person making the investment
    /// * `target_id` - Optional target person (for education investments)
    /// * `investment_type` - The type of investment
    /// * `principal` - The amount being invested
    /// * `return_rate` - The expected return rate per step (e.g., 0.02 = 2% per step)
    /// * `duration` - The number of steps over which returns are paid
    /// * `created_at_step` - The simulation step when the investment is created
    ///
    /// # Returns
    ///
    /// A new `Investment` instance with calculated return amounts
    pub fn new(
        id: InvestmentId,
        investor_id: PersonId,
        target_id: Option<PersonId>,
        investment_type: InvestmentType,
        principal: f64,
        return_rate: f64,
        duration: usize,
        created_at_step: usize,
    ) -> Self {
        // Calculate return per step based on principal and return rate
        // Model: The investor receives their principal back PLUS returns
        // Total profit = principal * return_rate * duration
        // Total amount returned = principal + profit
        // return_per_step = total_amount_returned / duration
        let total_profit = principal * return_rate * duration as f64;
        let total_amount_returned = principal + total_profit;
        let return_per_step = total_amount_returned / duration as f64;

        Investment {
            id,
            investor_id,
            target_id,
            investment_type,
            principal,
            return_rate,
            duration,
            created_at_step,
            return_per_step,
            returns_paid: 0,
            total_returns_paid: 0.0,
            is_completed: false,
        }
    }

    /// Processes a single return payment from the investment
    ///
    /// Increments the payment counter and tracks total returns.
    /// Marks the investment as completed if all returns have been paid.
    ///
    /// # Returns
    ///
    /// The amount of the return payment
    pub fn collect_return(&mut self) -> f64 {
        if self.is_completed {
            return 0.0;
        }

        self.returns_paid += 1;
        self.total_returns_paid += self.return_per_step;

        // Mark as completed if we've paid all returns
        if self.returns_paid >= self.duration {
            self.is_completed = true;
        }

        self.return_per_step
    }

    /// Returns the total amount that will be returned over the investment lifetime
    pub fn total_expected_return(&self) -> f64 {
        self.return_per_step * self.duration as f64
    }

    /// Returns the ROI (return on investment) as a percentage
    pub fn roi_percentage(&self) -> f64 {
        let total_return = self.total_expected_return();
        ((total_return - self.principal) / self.principal) * 100.0
    }

    /// Returns the net profit/loss from the investment
    pub fn net_profit(&self) -> f64 {
        self.total_returns_paid - self.principal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_investment_creation() {
        let investment = Investment::new(
            0,
            1,
            Some(2),
            InvestmentType::Education {
                skill_id: "Programming".to_string(),
            },
            100.0,
            0.02,
            10,
            0,
        );

        assert_eq!(investment.id, 0);
        assert_eq!(investment.investor_id, 1);
        assert_eq!(investment.target_id, Some(2));
        assert_eq!(investment.principal, 100.0);
        assert_eq!(investment.return_rate, 0.02);
        assert_eq!(investment.duration, 10);
        assert_eq!(investment.returns_paid, 0);
        assert_eq!(investment.total_returns_paid, 0.0);
        assert!(!investment.is_completed);
    }

    #[test]
    fn test_investment_return_calculation() {
        let investment = Investment::new(
            0,
            1,
            Some(2),
            InvestmentType::Education {
                skill_id: "Programming".to_string(),
            },
            100.0,
            0.02,
            10,
            0,
        );

        // Total profit = 100 * 0.02 * 10 = 20
        // Total amount returned = 100 + 20 = 120
        // Return per step = 120 / 10 = 12
        assert_eq!(investment.return_per_step, 12.0);
        assert_eq!(investment.total_expected_return(), 120.0);
        assert_eq!(investment.roi_percentage(), 20.0); // 20% ROI
    }

    #[test]
    fn test_investment_collect_returns() {
        let mut investment = Investment::new(
            0,
            1,
            Some(2),
            InvestmentType::Education {
                skill_id: "Programming".to_string(),
            },
            100.0,
            0.02,
            10,
            0,
        );

        // Collect first return (should be 12.0)
        let return_amount = investment.collect_return();
        assert_eq!(return_amount, 12.0);
        assert_eq!(investment.returns_paid, 1);
        assert_eq!(investment.total_returns_paid, 12.0);
        assert!(!investment.is_completed);

        // Collect remaining returns
        for _ in 1..10 {
            investment.collect_return();
        }

        assert_eq!(investment.returns_paid, 10);
        assert_eq!(investment.total_returns_paid, 120.0);
        assert!(investment.is_completed);

        // Collecting after completion returns 0
        assert_eq!(investment.collect_return(), 0.0);
    }

    #[test]
    fn test_investment_net_profit() {
        let mut investment = Investment::new(
            0,
            1,
            None,
            InvestmentType::Production {
                recipe_name: "AdvancedProduction".to_string(),
            },
            100.0,
            0.02,
            10,
            0,
        );

        // Initially, no returns collected, so net profit is negative (lost the principal)
        assert_eq!(investment.net_profit(), -100.0);

        // After collecting all returns
        for _ in 0..10 {
            investment.collect_return();
        }

        // Total profit = 100 * 0.02 * 10 = 20
        // Total amount returned = principal + profit = 100 + 20 = 120
        // Net profit = 120 - 100 = 20
        assert_eq!(investment.total_returns_paid, 120.0);
        assert_eq!(investment.net_profit(), 20.0); // Profit!
    }

    #[test]
    fn test_profitable_investment() {
        // Create an investment with high return rate to ensure profitability
        let mut investment = Investment::new(
            0,
            1,
            None,
            InvestmentType::Production {
                recipe_name: "HighYield".to_string(),
            },
            100.0,
            0.15, // 15% per step
            10,
            0,
        );

        // Total profit = 100 * 0.15 * 10 = 150
        // Total amount returned = 100 + 150 = 250
        // This should be profitable
        for _ in 0..10 {
            investment.collect_return();
        }

        assert_eq!(investment.total_returns_paid, 250.0);
        assert_eq!(investment.net_profit(), 150.0); // Profit!
    }
}
