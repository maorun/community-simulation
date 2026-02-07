//! # Credit Rating System Module
//!
//! Implements a credit scoring system similar to FICO (300-850 scale) that evaluates
//! creditworthiness of persons based on their financial behavior.
//!
//! ## Features
//!
//! - **FICO-like Scoring**: 0-850 scale mimicking real-world credit scores
//! - **Multi-Factor Calculation**: Combines 5 factors with industry-standard weights:
//!   - Payment History (35%): Track record of loan repayments
//!   - Debt Level (30%): Current debt-to-money ratio
//!   - Credit History Length (15%): Duration of credit activity
//!   - New Credit (10%): Recent loan activity
//!   - Credit Mix (10%): Variety of credit types
//! - **Dynamic Interest Rates**: Credit scores affect loan interest rates
//! - **Default Tracking**: Monitors missed payments and defaults
//!
//! ## Credit Score Interpretation
//!
//! - 800-850: Excellent - Best interest rates
//! - 740-799: Very Good - Above average rates
//! - 670-739: Good - Near average rates
//! - 580-669: Fair - Below average rates, higher interest
//! - 300-579: Poor - Highest interest rates, limited credit access
//! - < 300: No credit history - Default medium interest rate

use serde::{Deserialize, Serialize};

/// Default credit score for persons with no credit history (fair credit).
pub const DEFAULT_CREDIT_SCORE: u16 = 650;

/// Credit score data structure tracking a person's creditworthiness.
///
/// Scores range from 300 to 850, following the FICO credit score model.
/// Higher scores indicate better credit quality and result in lower interest rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScore {
    /// Current credit score (300-850 scale).
    /// Starts at 650 (fair credit) for persons with no credit history.
    pub score: u16,

    /// Total number of loan payments successfully made on time.
    /// Used to calculate payment history factor (35% of score).
    pub successful_payments: usize,

    /// Total number of loan payments that were missed or late.
    /// Severely damages credit score as payment history is most important factor.
    pub missed_payments: usize,

    /// Number of simulation steps since first loan.
    /// Longer credit history improves score (15% weight).
    /// Zero indicates no credit history.
    pub credit_history_steps: usize,

    /// Number of new loans taken in recent period (last 50 steps).
    /// Too many new loans can temporarily lower score (10% weight).
    pub recent_loans_count: usize,

    /// Step number when recent loan count was last reset.
    /// Used to track the "recent" period for new credit factor.
    pub recent_loans_reset_step: usize,

    /// Total number of different loan types (currently always 1: simple loans).
    /// Credit mix affects score (10% weight). In future versions could track
    /// different loan types (short-term, long-term, collateralized, etc.).
    pub credit_mix: usize,
}

impl Default for CreditScore {
    /// Creates a default credit score for a person with no credit history.
    /// Starts at 650 (fair credit) which is the median for new borrowers.
    fn default() -> Self {
        CreditScore {
            score: DEFAULT_CREDIT_SCORE, // Fair credit - neutral starting point
            successful_payments: 0,
            missed_payments: 0,
            credit_history_steps: 0,
            recent_loans_count: 0,
            recent_loans_reset_step: 0,
            credit_mix: 0,
        }
    }
}

impl CreditScore {
    /// Creates a new credit score with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the credit score based on multiple factors.
    ///
    /// Uses the standard FICO-like weighting:
    /// - Payment History: 35%
    /// - Debt Level: 30%
    /// - Credit History Length: 15%
    /// - New Credit: 10%
    /// - Credit Mix: 10%
    ///
    /// # Arguments
    ///
    /// * `current_debt` - Total outstanding debt (remaining principal on all loans)
    /// * `current_money` - Current money available (for debt-to-money ratio)
    /// * `current_step` - Current simulation step (for history length calculation)
    ///
    /// # Returns
    ///
    /// Updated credit score between 300 and 850
    pub fn calculate_score(&mut self, current_debt: f64, current_money: f64, current_step: usize) {
        // If no credit history, maintain default score
        if self.credit_history_steps == 0
            && self.successful_payments == 0
            && self.missed_payments == 0
        {
            self.score = 650; // Fair credit for no history
            return;
        }

        // Factor 1: Payment History (35% - most important)
        let payment_history_score = self.calculate_payment_history_factor();

        // Factor 2: Debt Level (30% - second most important)
        let debt_score = self.calculate_debt_factor(current_debt, current_money);

        // Factor 3: Credit History Length (15%)
        let history_score = self.calculate_credit_history_factor();

        // Factor 4: New Credit (10%)
        let new_credit_score = self.calculate_new_credit_factor(current_step);

        // Factor 5: Credit Mix (10%)
        let credit_mix_score = self.calculate_credit_mix_factor();

        // Weighted combination
        let raw_score = payment_history_score * 0.35
            + debt_score * 0.30
            + history_score * 0.15
            + new_credit_score * 0.10
            + credit_mix_score * 0.10;

        // Scale to 300-850 range and round
        self.score = ((raw_score * 550.0) + 300.0).round().clamp(300.0, 850.0) as u16;
    }

    /// Calculates payment history factor (0.0 to 1.0).
    /// Perfect payment history = 1.0, any missed payments reduce score significantly.
    fn calculate_payment_history_factor(&self) -> f64 {
        let total_payments = self.successful_payments + self.missed_payments;
        if total_payments == 0 {
            return 0.7; // Neutral for no history
        }

        let success_rate = self.successful_payments as f64 / total_payments as f64;
        // Each missed payment has severe impact
        // 100% success = 1.0, 90% success = 0.7, 80% success = 0.5
        success_rate.powf(2.0).clamp(0.0, 1.0)
    }

    /// Calculates debt level factor (0.0 to 1.0).
    /// Low debt-to-money ratio = 1.0, high ratio = lower score.
    fn calculate_debt_factor(&self, current_debt: f64, current_money: f64) -> f64 {
        if current_debt <= 0.0 {
            return 1.0; // No debt = excellent
        }

        // Calculate debt-to-money ratio
        // For credit scoring, lower is better
        let total_assets = current_money + current_debt; // Total resources
        if total_assets <= 0.0 {
            return 0.3; // Heavily penalize negative net worth
        }

        let debt_ratio = current_debt / total_assets;
        // 0% debt = 1.0, 30% debt = 0.7, 60% debt = 0.4, 90%+ debt = 0.1
        (1.0 - debt_ratio * 1.5).clamp(0.1, 1.0)
    }

    /// Calculates credit history length factor (0.0 to 1.0).
    /// Longer history = better score. Maxes out at ~200 steps.
    fn calculate_credit_history_factor(&self) -> f64 {
        if self.credit_history_steps == 0 {
            return 0.5; // Neutral for no history
        }

        // Asymptotic growth: longer history is better but diminishing returns
        // 0 steps = 0.5, 50 steps = 0.7, 100 steps = 0.85, 200+ steps = 0.95+
        let normalized = (self.credit_history_steps as f64 / 200.0).min(1.0);
        0.5 + (normalized * 0.5)
    }

    /// Calculates new credit factor (0.0 to 1.0).
    /// Too many recent loans = lower score (suggests desperation).
    fn calculate_new_credit_factor(&self, current_step: usize) -> f64 {
        // Reset recent loans count if enough time has passed (50 steps = "recent period")
        let steps_since_reset = current_step.saturating_sub(self.recent_loans_reset_step);
        let active_recent_loans = if steps_since_reset >= 50 {
            0 // Reset counter after 50 steps
        } else {
            self.recent_loans_count
        };

        // 0 recent loans = 1.0, 1 loan = 0.9, 2 loans = 0.7, 3+ loans = 0.5
        match active_recent_loans {
            0 => 1.0,
            1 => 0.9,
            2 => 0.7,
            3 => 0.5,
            _ => 0.3,
        }
    }

    /// Calculates credit mix factor (0.0 to 1.0).
    /// More types of credit = better score. Currently simple (only one loan type).
    fn calculate_credit_mix_factor(&self) -> f64 {
        if self.credit_mix == 0 {
            return 0.5; // Neutral for no credit
        }

        // For now, we only have one type of loan
        // In future: 1 type = 0.7, 2 types = 0.85, 3+ types = 1.0
        0.7 // Single credit type
    }

    /// Records a successful loan payment.
    pub fn record_successful_payment(&mut self) {
        self.successful_payments += 1;
    }

    /// Records a missed loan payment. Significantly damages credit score.
    pub fn record_missed_payment(&mut self) {
        self.missed_payments += 1;
    }

    /// Initializes credit history when a person takes their first loan.
    ///
    /// # Arguments
    ///
    /// * `current_step` - The simulation step when the first loan is taken
    pub fn start_credit_history(&mut self, current_step: usize) {
        if self.credit_history_steps == 0 {
            self.credit_history_steps = 1;
            self.recent_loans_reset_step = current_step;
        }
        self.credit_mix = 1; // We have one type of credit (simple loans)
    }

    /// Records a new loan and updates recent loan count.
    ///
    /// # Arguments
    ///
    /// * `current_step` - The simulation step when the loan is taken
    pub fn record_new_loan(&mut self, current_step: usize) {
        // Reset recent loans if 50+ steps have passed
        let steps_since_reset = current_step.saturating_sub(self.recent_loans_reset_step);
        if steps_since_reset >= 50 {
            self.recent_loans_count = 0;
            self.recent_loans_reset_step = current_step;
        }

        self.recent_loans_count += 1;
    }

    /// Increments credit history length. Should be called each simulation step
    /// for persons with active or past credit history.
    pub fn increment_credit_history(&mut self) {
        if self.credit_history_steps > 0 {
            self.credit_history_steps += 1;
        }
    }

    /// Calculates interest rate based on credit score.
    ///
    /// Maps credit score to interest rate with better scores getting lower rates.
    /// Uses typical credit score tiers from lending industry.
    ///
    /// # Arguments
    ///
    /// * `base_interest_rate` - The base interest rate for average credit (score 670)
    ///
    /// # Returns
    ///
    /// Interest rate multiplier (e.g., 0.01 for 1% per step)
    ///
    /// # Credit Score Tiers
    ///
    /// - 800-850 (Excellent): 0.5x base rate (50% discount)
    /// - 740-799 (Very Good): 0.7x base rate (30% discount)
    /// - 670-739 (Good): 1.0x base rate (standard)
    /// - 580-669 (Fair): 1.5x base rate (50% premium)
    /// - 300-579 (Poor): 2.5x base rate (150% premium)
    pub fn calculate_interest_rate(&self, base_interest_rate: f64) -> f64 {
        let multiplier = match self.score {
            800..=850 => 0.5, // Excellent: half the base rate
            740..=799 => 0.7, // Very Good: 30% discount
            670..=739 => 1.0, // Good: base rate
            580..=669 => 1.5, // Fair: 50% premium
            300..=579 => 2.5, // Poor: 150% premium
            _ => 1.0,         // Fallback to base rate
        };

        base_interest_rate * multiplier
    }

    /// Returns a human-readable rating category based on the credit score.
    pub fn rating_category(&self) -> &str {
        match self.score {
            800..=850 => "Excellent",
            740..=799 => "Very Good",
            670..=739 => "Good",
            580..=669 => "Fair",
            300..=579 => "Poor",
            _ => "No Rating",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_credit_score() {
        let score = CreditScore::default();
        assert_eq!(score.score, 650); // Fair credit
        assert_eq!(score.successful_payments, 0);
        assert_eq!(score.missed_payments, 0);
        assert_eq!(score.credit_history_steps, 0);
    }

    #[test]
    fn test_perfect_payment_history() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);

        // Record 10 successful payments, no missed payments
        for _ in 0..10 {
            score.record_successful_payment();
        }

        score.calculate_score(0.0, 100.0, 100);
        assert!(score.score >= 750, "Perfect payment history should yield high score");
    }

    #[test]
    fn test_missed_payments_damage_score() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);

        // Mix of successful and missed payments
        for _ in 0..7 {
            score.record_successful_payment();
        }
        for _ in 0..3 {
            score.record_missed_payment();
        }

        score.calculate_score(0.0, 100.0, 100);
        assert!(score.score < 700, "Missed payments should significantly lower score");
    }

    #[test]
    fn test_high_debt_lowers_score() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);
        score.record_successful_payment();

        // High debt relative to money
        score.calculate_score(500.0, 100.0, 10); // 500 debt, 100 money
        let high_debt_score = score.score;

        // Low debt relative to money
        score.calculate_score(10.0, 100.0, 10); // 10 debt, 100 money
        let low_debt_score = score.score;

        assert!(low_debt_score > high_debt_score, "Low debt should yield higher score");
    }

    #[test]
    fn test_credit_history_length_improves_score() {
        let mut score1 = CreditScore::new();
        score1.start_credit_history(0);
        score1.credit_history_steps = 10;
        score1.record_successful_payment();
        score1.calculate_score(0.0, 100.0, 10);
        let short_history_score = score1.score;

        let mut score2 = CreditScore::new();
        score2.start_credit_history(0);
        score2.credit_history_steps = 200;
        score2.record_successful_payment();
        score2.calculate_score(0.0, 100.0, 200);
        let long_history_score = score2.score;

        assert!(
            long_history_score > short_history_score,
            "Longer credit history should yield higher score"
        );
    }

    #[test]
    fn test_interest_rate_calculation() {
        let base_rate = 0.02; // 2% base interest

        let excellent = CreditScore { score: 820, ..Default::default() };
        let good = CreditScore { score: 700, ..Default::default() };
        let poor = CreditScore { score: 500, ..Default::default() };

        let excellent_rate = excellent.calculate_interest_rate(base_rate);
        let good_rate = good.calculate_interest_rate(base_rate);
        let poor_rate = poor.calculate_interest_rate(base_rate);

        assert!(excellent_rate < good_rate, "Excellent credit should get better rate than good");
        assert!(good_rate < poor_rate, "Good credit should get better rate than poor");
        assert_eq!(excellent_rate, base_rate * 0.5); // 50% of base
        assert_eq!(good_rate, base_rate * 1.0); // 100% of base
        assert_eq!(poor_rate, base_rate * 2.5); // 250% of base
    }

    #[test]
    fn test_rating_category() {
        assert_eq!(CreditScore { score: 820, ..Default::default() }.rating_category(), "Excellent");
        assert_eq!(CreditScore { score: 750, ..Default::default() }.rating_category(), "Very Good");
        assert_eq!(CreditScore { score: 700, ..Default::default() }.rating_category(), "Good");
        assert_eq!(CreditScore { score: 620, ..Default::default() }.rating_category(), "Fair");
        assert_eq!(CreditScore { score: 500, ..Default::default() }.rating_category(), "Poor");
    }

    #[test]
    fn test_recent_loans_reset() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);

        // Take 3 loans in quick succession
        score.record_new_loan(0);
        score.record_new_loan(5);
        score.record_new_loan(10);
        assert_eq!(score.recent_loans_count, 3);

        // After 50 steps, recent loans should reset
        score.record_new_loan(60);
        assert_eq!(score.recent_loans_count, 1, "Recent loans should reset after 50 steps");
    }

    #[test]
    fn test_calculate_score_with_no_history() {
        let mut score = CreditScore::new();
        // With no credit history, score should remain at default
        score.calculate_score(0.0, 100.0, 10);
        assert_eq!(score.score, 650);
    }

    #[test]
    fn test_debt_factor_with_zero_debt() {
        let score = CreditScore::new();
        let factor = score.calculate_debt_factor(0.0, 100.0);
        assert_eq!(factor, 1.0, "Zero debt should give perfect debt factor");
    }

    #[test]
    fn test_debt_factor_with_negative_assets() {
        let score = CreditScore::new();
        // total_assets = current_money + current_debt
        // For negative net worth: -150 + 100 = -50 (negative total assets)
        let factor = score.calculate_debt_factor(100.0, -150.0);
        assert_eq!(factor, 0.3, "Negative total assets should heavily penalize");
    }

    #[test]
    fn test_payment_history_factor_no_payments() {
        let score = CreditScore::new();
        let factor = score.calculate_payment_history_factor();
        assert_eq!(factor, 0.7, "No payment history should be neutral");
    }

    #[test]
    fn test_credit_history_factor_no_history() {
        let score = CreditScore::new();
        let factor = score.calculate_credit_history_factor();
        assert_eq!(factor, 0.5, "No credit history should be neutral");
    }

    #[test]
    fn test_new_credit_factor_multiple_loans() {
        let score =
            CreditScore { recent_loans_count: 4, recent_loans_reset_step: 0, ..Default::default() };
        let factor = score.calculate_new_credit_factor(10);
        assert_eq!(factor, 0.3, "4+ recent loans should give lowest factor");
    }

    #[test]
    fn test_new_credit_factor_reset_after_50_steps() {
        let score =
            CreditScore { recent_loans_count: 3, recent_loans_reset_step: 0, ..Default::default() };
        let factor = score.calculate_new_credit_factor(60);
        assert_eq!(factor, 1.0, "Recent loans should reset after 50 steps");
    }

    #[test]
    fn test_credit_mix_factor_no_credit() {
        let score = CreditScore::new();
        let factor = score.calculate_credit_mix_factor();
        assert_eq!(factor, 0.5, "No credit should be neutral");
    }

    #[test]
    fn test_credit_mix_factor_with_credit() {
        let mut score = CreditScore::new();
        score.credit_mix = 1;
        let factor = score.calculate_credit_mix_factor();
        assert_eq!(factor, 0.7, "Single credit type should give 0.7 factor");
    }

    #[test]
    fn test_start_credit_history_only_once() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);
        assert_eq!(score.credit_history_steps, 1);
        assert_eq!(score.credit_mix, 1);

        // Starting again should not reset
        score.credit_history_steps = 50;
        score.start_credit_history(100);
        assert_eq!(score.credit_history_steps, 50, "Should not reset existing history");
    }

    #[test]
    fn test_increment_credit_history() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);
        assert_eq!(score.credit_history_steps, 1);

        score.increment_credit_history();
        assert_eq!(score.credit_history_steps, 2);

        score.increment_credit_history();
        assert_eq!(score.credit_history_steps, 3);
    }

    #[test]
    fn test_increment_credit_history_no_history() {
        let mut score = CreditScore::new();
        // Without starting history, increment should do nothing
        score.increment_credit_history();
        assert_eq!(score.credit_history_steps, 0);
    }

    #[test]
    fn test_interest_rate_edge_cases() {
        let base_rate = 0.02;

        // Test boundary values
        let score_800 = CreditScore { score: 800, ..Default::default() };
        assert_eq!(score_800.calculate_interest_rate(base_rate), base_rate * 0.5);

        let score_740 = CreditScore { score: 740, ..Default::default() };
        assert_eq!(score_740.calculate_interest_rate(base_rate), base_rate * 0.7);

        let score_670 = CreditScore { score: 670, ..Default::default() };
        assert_eq!(score_670.calculate_interest_rate(base_rate), base_rate * 1.0);

        let score_580 = CreditScore { score: 580, ..Default::default() };
        assert_eq!(score_580.calculate_interest_rate(base_rate), base_rate * 1.5);

        let score_300 = CreditScore { score: 300, ..Default::default() };
        assert_eq!(score_300.calculate_interest_rate(base_rate), base_rate * 2.5);
    }

    #[test]
    fn test_rating_category_edge_cases() {
        // Test boundary values
        assert_eq!(CreditScore { score: 800, ..Default::default() }.rating_category(), "Excellent");
        assert_eq!(CreditScore { score: 850, ..Default::default() }.rating_category(), "Excellent");
        assert_eq!(CreditScore { score: 739, ..Default::default() }.rating_category(), "Good");
        assert_eq!(CreditScore { score: 669, ..Default::default() }.rating_category(), "Fair");
        assert_eq!(CreditScore { score: 579, ..Default::default() }.rating_category(), "Poor");
        assert_eq!(CreditScore { score: 299, ..Default::default() }.rating_category(), "No Rating");
    }

    #[test]
    fn test_record_new_loan_resets_counter() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);
        score.recent_loans_reset_step = 0;
        score.recent_loans_count = 5;

        // Recording new loan after 50+ steps should reset
        score.record_new_loan(60);
        assert_eq!(score.recent_loans_count, 1);
        assert_eq!(score.recent_loans_reset_step, 60);
    }

    #[test]
    fn test_calculate_score_comprehensive() {
        let mut score = CreditScore::new();
        score.start_credit_history(0);

        // Build up a credit profile
        for _ in 0..10 {
            score.record_successful_payment();
        }
        score.record_missed_payment();

        for _ in 0..50 {
            score.increment_credit_history();
        }

        score.record_new_loan(10);
        score.record_new_loan(20);

        score.calculate_score(50.0, 200.0, 60);

        // Score should be calculated based on all factors
        assert!(score.score >= 300 && score.score <= 850, "Score should be in valid range");
    }
}
