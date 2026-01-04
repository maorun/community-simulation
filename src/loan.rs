use crate::person::PersonId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a loan
pub type LoanId = usize;

/// Represents a loan between two persons in the simulation
///
/// A loan is an agreement where one person (lender) provides money to another person (borrower)
/// with the expectation of repayment over time with interest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    /// Unique identifier for this loan
    pub id: LoanId,
    /// The person providing the money
    pub lender_id: PersonId,
    /// The person receiving the money
    pub borrower_id: PersonId,
    /// The principal amount borrowed
    pub principal: f64,
    /// The interest rate per step (e.g., 0.01 = 1% per step)
    pub interest_rate: f64,
    /// The remaining principal amount to be repaid
    pub remaining_principal: f64,
    /// The number of steps over which the loan should be repaid
    pub repayment_period: usize,
    /// The simulation step when the loan was created
    pub created_at_step: usize,
    /// The amount to be repaid per step (principal + interest)
    pub payment_per_step: f64,
    /// The number of payments already made
    pub payments_made: usize,
    /// Whether the loan has been fully repaid
    pub is_repaid: bool,
}

impl Loan {
    /// Creates a new loan with the specified parameters
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this loan
    /// * `lender_id` - The person providing the money
    /// * `borrower_id` - The person receiving the money
    /// * `principal` - The amount being borrowed
    /// * `interest_rate` - The interest rate per step (e.g., 0.01 = 1% per step)
    /// * `repayment_period` - The number of steps over which to repay the loan
    /// * `created_at_step` - The simulation step when the loan is created
    ///
    /// # Returns
    ///
    /// A new `Loan` instance with calculated payment amounts
    pub fn new(
        id: LoanId,
        lender_id: PersonId,
        borrower_id: PersonId,
        principal: f64,
        interest_rate: f64,
        repayment_period: usize,
        created_at_step: usize,
    ) -> Self {
        // Calculate total amount to repay (principal + total interest)
        let total_interest = principal * interest_rate * repayment_period as f64;
        let total_to_repay = principal + total_interest;
        let payment_per_step = total_to_repay / repayment_period as f64;

        Loan {
            id,
            lender_id,
            borrower_id,
            principal,
            interest_rate,
            remaining_principal: principal,
            repayment_period,
            created_at_step,
            payment_per_step,
            payments_made: 0,
            is_repaid: false,
        }
    }

    /// Processes a single loan payment
    ///
    /// Decreases the remaining principal and increments the payment counter.
    /// Marks the loan as repaid if all payments have been made.
    ///
    /// Uses simple interest: the payment amount is fixed and calculated at loan creation.
    /// Each payment includes a portion of principal plus interest.
    ///
    /// # Returns
    ///
    /// The amount of the payment
    pub fn make_payment(&mut self) -> f64 {
        if self.is_repaid {
            return 0.0;
        }

        self.payments_made += 1;

        // With simple interest, each payment is equal and reduces the remaining principal
        // by a fixed amount: original_principal / repayment_period
        let principal_per_payment = self.principal / self.repayment_period as f64;
        self.remaining_principal = (self.remaining_principal - principal_per_payment).max(0.0);

        // Mark as repaid if we've made all payments or remaining principal is effectively zero
        if self.payments_made >= self.repayment_period || self.remaining_principal < 0.01 {
            self.is_repaid = true;
            self.remaining_principal = 0.0;
        }

        self.payment_per_step
    }

    /// Returns the total amount that will be repaid over the life of the loan
    pub fn total_repayment_amount(&self) -> f64 {
        self.payment_per_step * self.repayment_period as f64
    }

    /// Returns the total interest that will be paid over the life of the loan
    pub fn total_interest(&self) -> f64 {
        self.total_repayment_amount() - self.principal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loan_creation() {
        let loan = Loan::new(0, 1, 2, 100.0, 0.01, 10, 0);

        assert_eq!(loan.id, 0);
        assert_eq!(loan.lender_id, 1);
        assert_eq!(loan.borrower_id, 2);
        assert_eq!(loan.principal, 100.0);
        assert_eq!(loan.interest_rate, 0.01);
        assert_eq!(loan.remaining_principal, 100.0);
        assert_eq!(loan.repayment_period, 10);
        assert_eq!(loan.payments_made, 0);
        assert!(!loan.is_repaid);
    }

    #[test]
    fn test_loan_payment_calculation() {
        let loan = Loan::new(0, 1, 2, 100.0, 0.01, 10, 0);

        // Total interest = 100 * 0.01 * 10 = 10
        // Total to repay = 100 + 10 = 110
        // Payment per step = 110 / 10 = 11
        assert_eq!(loan.payment_per_step, 11.0);
        assert_eq!(loan.total_repayment_amount(), 110.0);
        assert_eq!(loan.total_interest(), 10.0);
    }

    #[test]
    fn test_loan_make_payment() {
        let mut loan = Loan::new(0, 1, 2, 100.0, 0.01, 10, 0);

        let payment = loan.make_payment();
        assert_eq!(payment, 11.0);
        assert_eq!(loan.payments_made, 1);
        assert!(!loan.is_repaid);

        // Make all remaining payments
        for _ in 1..10 {
            loan.make_payment();
        }

        assert_eq!(loan.payments_made, 10);
        assert!(loan.is_repaid);
        assert_eq!(loan.remaining_principal, 0.0);
    }

    #[test]
    fn test_loan_no_payment_when_repaid() {
        let mut loan = Loan::new(0, 1, 2, 100.0, 0.01, 1, 0);

        loan.make_payment();
        assert!(loan.is_repaid);

        // Trying to make another payment should return 0
        let payment = loan.make_payment();
        assert_eq!(payment, 0.0);
    }

    #[test]
    fn test_loan_zero_interest() {
        let loan = Loan::new(0, 1, 2, 100.0, 0.0, 10, 0);

        // With zero interest, payment per step should equal principal / period
        assert_eq!(loan.payment_per_step, 10.0);
        assert_eq!(loan.total_interest(), 0.0);
    }
}
