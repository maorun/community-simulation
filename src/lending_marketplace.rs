//! # Peer-to-Peer Lending Marketplace Module
//!
//! Implements a decentralized credit marketplace where persons can directly lend to each other
//! without a central institution. Lenders create offers with their desired terms, and borrowers
//! are matched based on credit ratings and risk preferences.
//!
//! ## Features
//!
//! - **Decentralized Lending**: Direct person-to-person loans without intermediaries
//! - **Risk-Based Pricing**: Interest rates adjusted based on credit scores
//! - **Automatic Matching**: Algorithm matches borrowers with suitable lenders
//! - **Platform Fees**: Optional transaction fees for marketplace operation

use crate::person::PersonId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a lending offer
pub type LendingOfferId = usize;

/// Represents a lending offer posted by a person willing to lend money.
///
/// Lenders specify the amount they're willing to lend, their desired interest rate,
/// and optionally a minimum credit score they'll accept from borrowers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingOffer {
    /// Unique identifier for this offer
    pub id: LendingOfferId,
    /// The person offering to lend money
    pub lender_id: PersonId,
    /// Maximum amount willing to lend
    pub max_amount: f64,
    /// Desired interest rate per step (e.g., 0.01 = 1% per step)
    pub interest_rate: f64,
    /// Minimum credit score required for borrowers (300-850)
    /// None means no minimum requirement
    pub min_credit_score: Option<u16>,
    /// The simulation step when this offer was created
    pub created_at_step: usize,
}

impl LendingOffer {
    /// Creates a new lending offer.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this offer
    /// * `lender_id` - The person offering to lend
    /// * `max_amount` - Maximum amount willing to lend
    /// * `interest_rate` - Desired interest rate per step
    /// * `min_credit_score` - Optional minimum credit score requirement
    /// * `created_at_step` - The simulation step when created
    pub fn new(
        id: LendingOfferId,
        lender_id: PersonId,
        max_amount: f64,
        interest_rate: f64,
        min_credit_score: Option<u16>,
        created_at_step: usize,
    ) -> Self {
        LendingOffer { id, lender_id, max_amount, interest_rate, min_credit_score, created_at_step }
    }

    /// Checks if a borrower meets the credit requirements for this offer.
    ///
    /// # Arguments
    ///
    /// * `credit_score` - The credit score to check
    ///
    /// # Returns
    ///
    /// `true` if the credit score meets the requirements, `false` otherwise
    pub fn accepts_credit_score(&self, credit_score: u16) -> bool {
        match self.min_credit_score {
            Some(min_score) => credit_score >= min_score,
            None => true,
        }
    }
}

/// Represents the peer-to-peer lending marketplace.
///
/// Manages lending offers and matches lenders with borrowers based on
/// credit ratings, risk preferences, and availability of funds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingMarketplace {
    /// All active lending offers
    pub offers: Vec<LendingOffer>,
    /// Counter for generating unique offer IDs
    pub offer_counter: LendingOfferId,
    /// Platform fee rate charged on successful loans (e.g., 0.01 = 1%)
    pub platform_fee_rate: f64,
    /// Total fees collected by the platform
    pub total_fees_collected: f64,
}

impl LendingMarketplace {
    /// Creates a new lending marketplace.
    ///
    /// # Arguments
    ///
    /// * `platform_fee_rate` - Fee rate charged on loans (e.g., 0.01 = 1%)
    ///
    /// # Returns
    ///
    /// A new `LendingMarketplace` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::lending_marketplace::LendingMarketplace;
    ///
    /// let marketplace = LendingMarketplace::new(0.01);
    /// assert_eq!(marketplace.platform_fee_rate, 0.01);
    /// assert_eq!(marketplace.offers.len(), 0);
    /// ```
    pub fn new(platform_fee_rate: f64) -> Self {
        LendingMarketplace {
            offers: Vec::new(),
            offer_counter: 0,
            platform_fee_rate,
            total_fees_collected: 0.0,
        }
    }

    /// Adds a new lending offer to the marketplace.
    ///
    /// # Arguments
    ///
    /// * `lender_id` - The person offering to lend
    /// * `max_amount` - Maximum amount willing to lend
    /// * `interest_rate` - Desired interest rate per step
    /// * `min_credit_score` - Optional minimum credit score requirement
    /// * `current_step` - Current simulation step
    ///
    /// # Returns
    ///
    /// The ID of the newly created offer
    pub fn add_offer(
        &mut self,
        lender_id: PersonId,
        max_amount: f64,
        interest_rate: f64,
        min_credit_score: Option<u16>,
        current_step: usize,
    ) -> LendingOfferId {
        let offer_id = self.offer_counter;
        self.offer_counter += 1;

        let offer = LendingOffer::new(
            offer_id,
            lender_id,
            max_amount,
            interest_rate,
            min_credit_score,
            current_step,
        );

        self.offers.push(offer);
        offer_id
    }

    /// Finds the best matching lending offer for a borrower.
    ///
    /// Selects offers that:
    /// 1. Accept the borrower's credit score
    /// 2. Have sufficient funds available
    /// 3. Offer the lowest interest rate
    ///
    /// # Arguments
    ///
    /// * `borrower_credit_score` - Credit score of the borrower
    /// * `requested_amount` - Amount the borrower wants to borrow
    ///
    /// # Returns
    ///
    /// The best matching `LendingOffer` if one exists, `None` otherwise
    pub fn find_best_offer(
        &self,
        borrower_credit_score: u16,
        requested_amount: f64,
    ) -> Option<&LendingOffer> {
        self.offers
            .iter()
            .filter(|offer| {
                offer.accepts_credit_score(borrower_credit_score)
                    && offer.max_amount >= requested_amount
            })
            .min_by(|a, b| {
                a.interest_rate
                    .partial_cmp(&b.interest_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Finds the best matching lending offer for a borrower, optionally incorporating nepotism / favoritism.
    ///
    /// When nepotism is enabled:
    /// - Lenders who are friends or in the trust network of the borrower may accept a lower credit score
    ///   (credit leniency / tolerance proportional to nepotism_bias).
    /// - If `inner_circle_only` is true, offers from inner-circle lenders are strictly prioritized
    ///   if any exist with sufficient funds.
    /// - The borrower discounts interest rates of friends when comparing offers:
    ///   effective interest = rate / (1.0 + nepotism_bias * social_closeness * 2.0).
    ///
    /// Returns `(best_offer, is_nepotistic)` where `is_nepotistic` is true if the selected offer
    /// was chosen due to social connections despite not being the optimal economic offer under pure merit.
    pub fn find_best_offer_with_nepotism<F>(
        &self,
        borrower_credit_score: u16,
        requested_amount: f64,
        nepotism_enabled: bool,
        nepotism_bias: f64,
        inner_circle_only: bool,
        mut social_closeness_fn: F,
    ) -> Option<(&LendingOffer, bool)>
    where
        F: FnMut(PersonId) -> f64,
    {
        if !nepotism_enabled {
            return self
                .find_best_offer(borrower_credit_score, requested_amount)
                .map(|o| (o, false));
        }

        // What would standard meritocratic matching choose?
        let standard_best = self.find_best_offer(borrower_credit_score, requested_amount);
        let safe_bias = nepotism_bias.clamp(0.0, 1.0);

        // Filter offers by amount and credit score (with friendship credit leniency)
        let mut candidates: Vec<(&LendingOffer, f64)> = Vec::new(); // (offer, social_closeness)

        for offer in &self.offers {
            if offer.max_amount < requested_amount {
                continue;
            }

            let social_closeness = social_closeness_fn(offer.lender_id).clamp(0.0, 1.0);

            // Credit score leniency for friends:
            // A friend lender is willing to forgive up to (safe_bias * 150) credit points
            let effective_min_credit = match offer.min_credit_score {
                Some(min_score) => {
                    let forgiveness =
                        (safe_bias * social_closeness * 150.0).round().max(0.0) as u16;
                    min_score.saturating_sub(forgiveness)
                },
                None => 0,
            };

            if borrower_credit_score >= effective_min_credit {
                candidates.push((offer, social_closeness));
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // If inner_circle_only, filter to offers from connections if any exist
        let filtered_candidates: Vec<(&LendingOffer, f64)> = if inner_circle_only {
            let inner_circle: Vec<(&LendingOffer, f64)> =
                candidates.iter().filter(|(_, closeness)| *closeness > 0.0).copied().collect();
            if !inner_circle.is_empty() {
                inner_circle
            } else {
                candidates
            }
        } else {
            candidates
        };

        // Select the offer with the lowest perceived interest rate (discounted by social closeness)
        let selected = filtered_candidates
            .into_iter()
            .min_by(|(offer_a, closeness_a), (offer_b, closeness_b)| {
                let perceived_rate_a =
                    offer_a.interest_rate / (1.0 + safe_bias * closeness_a * 2.0);
                let perceived_rate_b =
                    offer_b.interest_rate / (1.0 + safe_bias * closeness_b * 2.0);
                perceived_rate_a
                    .partial_cmp(&perceived_rate_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(offer, _)| offer)?;

        let is_nepotistic = match standard_best {
            Some(best_merit) => selected.id != best_merit.id,
            None => true, // Approved only because of friendship leniency
        };

        Some((selected, is_nepotistic))
    }

    /// Removes a lending offer from the marketplace.
    ///
    /// # Arguments
    ///
    /// * `offer_id` - The ID of the offer to remove
    ///
    /// # Returns
    ///
    /// The removed offer if it existed, `None` otherwise
    pub fn remove_offer(&mut self, offer_id: LendingOfferId) -> Option<LendingOffer> {
        if let Some(pos) = self.offers.iter().position(|o| o.id == offer_id) {
            Some(self.offers.remove(pos))
        } else {
            None
        }
    }

    /// Records a platform fee for a successful loan.
    ///
    /// # Arguments
    ///
    /// * `loan_amount` - The principal amount of the loan
    ///
    /// # Returns
    ///
    /// The amount of the platform fee charged
    pub fn charge_platform_fee(&mut self, loan_amount: f64) -> f64 {
        let fee = loan_amount * self.platform_fee_rate;
        self.total_fees_collected += fee;
        fee
    }

    /// Clears all offers (useful for testing or marketplace resets).
    pub fn clear_offers(&mut self) {
        self.offers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lending_offer_creation() {
        let offer = LendingOffer::new(0, 1, 1000.0, 0.02, Some(650), 0);

        assert_eq!(offer.id, 0);
        assert_eq!(offer.lender_id, 1);
        assert_eq!(offer.max_amount, 1000.0);
        assert_eq!(offer.interest_rate, 0.02);
        assert_eq!(offer.min_credit_score, Some(650));
        assert_eq!(offer.created_at_step, 0);
    }

    #[test]
    fn test_lending_offer_accepts_credit_score() {
        let offer_with_min = LendingOffer::new(0, 1, 1000.0, 0.02, Some(650), 0);
        let offer_without_min = LendingOffer::new(1, 2, 1000.0, 0.02, None, 0);

        // Offer with minimum score requirement
        assert!(offer_with_min.accepts_credit_score(650));
        assert!(offer_with_min.accepts_credit_score(700));
        assert!(!offer_with_min.accepts_credit_score(600));

        // Offer without minimum score requirement accepts any score
        assert!(offer_without_min.accepts_credit_score(300));
        assert!(offer_without_min.accepts_credit_score(850));
    }

    #[test]
    fn test_marketplace_creation() {
        let marketplace = LendingMarketplace::new(0.01);

        assert_eq!(marketplace.platform_fee_rate, 0.01);
        assert_eq!(marketplace.offers.len(), 0);
        assert_eq!(marketplace.offer_counter, 0);
        assert_eq!(marketplace.total_fees_collected, 0.0);
    }

    #[test]
    fn test_marketplace_add_offer() {
        let mut marketplace = LendingMarketplace::new(0.01);

        let offer_id = marketplace.add_offer(1, 1000.0, 0.02, Some(650), 0);

        assert_eq!(offer_id, 0);
        assert_eq!(marketplace.offers.len(), 1);
        assert_eq!(marketplace.offer_counter, 1);

        let offer = &marketplace.offers[0];
        assert_eq!(offer.lender_id, 1);
        assert_eq!(offer.max_amount, 1000.0);
    }

    #[test]
    fn test_marketplace_find_best_offer() {
        let mut marketplace = LendingMarketplace::new(0.01);

        // Add offers with different interest rates and requirements
        marketplace.add_offer(1, 1000.0, 0.03, Some(650), 0); // Higher rate, stricter
        marketplace.add_offer(2, 1000.0, 0.01, Some(700), 0); // Lower rate, very strict
        marketplace.add_offer(3, 1000.0, 0.02, Some(600), 0); // Medium rate, lenient

        // Borrower with excellent credit should get the lowest rate (0.01)
        let best_for_excellent = marketplace.find_best_offer(750, 500.0);
        assert!(best_for_excellent.is_some());
        assert_eq!(best_for_excellent.unwrap().interest_rate, 0.01);
        assert_eq!(best_for_excellent.unwrap().lender_id, 2);

        // Borrower with good credit (650-699) should get 0.02 rate
        let best_for_good = marketplace.find_best_offer(650, 500.0);
        assert!(best_for_good.is_some());
        assert_eq!(best_for_good.unwrap().interest_rate, 0.02);
        assert_eq!(best_for_good.unwrap().lender_id, 3);

        // Borrower with poor credit (<600) should get 0.03 rate (only option)
        // But they can't get any offer because all have higher minimums
        let best_for_poor = marketplace.find_best_offer(550, 500.0);
        assert!(best_for_poor.is_none());
    }

    #[test]
    fn test_marketplace_find_best_offer_insufficient_amount() {
        let mut marketplace = LendingMarketplace::new(0.01);

        marketplace.add_offer(1, 500.0, 0.02, None, 0);

        // Borrower requesting more than available should get no match
        let result = marketplace.find_best_offer(650, 1000.0);
        assert!(result.is_none());

        // Borrower requesting less should get a match
        let result = marketplace.find_best_offer(650, 300.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_marketplace_remove_offer() {
        let mut marketplace = LendingMarketplace::new(0.01);

        let offer_id = marketplace.add_offer(1, 1000.0, 0.02, Some(650), 0);
        assert_eq!(marketplace.offers.len(), 1);

        let removed = marketplace.remove_offer(offer_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, offer_id);
        assert_eq!(marketplace.offers.len(), 0);

        // Trying to remove again should return None
        let removed_again = marketplace.remove_offer(offer_id);
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_marketplace_charge_platform_fee() {
        let mut marketplace = LendingMarketplace::new(0.01);

        let fee = marketplace.charge_platform_fee(1000.0);
        assert_eq!(fee, 10.0);
        assert_eq!(marketplace.total_fees_collected, 10.0);

        // Charge another fee
        let fee2 = marketplace.charge_platform_fee(500.0);
        assert_eq!(fee2, 5.0);
        assert_eq!(marketplace.total_fees_collected, 15.0);
    }

    #[test]
    fn test_marketplace_clear_offers() {
        let mut marketplace = LendingMarketplace::new(0.01);

        marketplace.add_offer(1, 1000.0, 0.02, Some(650), 0);
        marketplace.add_offer(2, 1500.0, 0.03, Some(700), 0);
        assert_eq!(marketplace.offers.len(), 2);

        marketplace.clear_offers();
        assert_eq!(marketplace.offers.len(), 0);
    }

    #[test]
    fn test_marketplace_find_best_offer_with_nepotism_credit_leniency() {
        let mut marketplace = LendingMarketplace::new(0.01);
        marketplace.add_offer(10, 1000.0, 0.03, Some(700), 0); // Friend lender requires 700

        // Borrower has 660 credit score (below 700)
        // Without nepotism -> None
        let standard = marketplace.find_best_offer(660, 500.0);
        assert!(standard.is_none());

        // With nepotism, lender is a friend (closeness = 1.0, bias = 0.5 -> leniency = 75 points)
        // 700 - 75 = 625 <= 660 -> Accepted!
        let result =
            marketplace.find_best_offer_with_nepotism(660, 500.0, true, 0.5, false, |lender_id| {
                if lender_id == 10 {
                    1.0
                } else {
                    0.0
                }
            });
        assert!(result.is_some());
        let (offer, is_nepotistic) = result.unwrap();
        assert_eq!(offer.lender_id, 10);
        assert!(is_nepotistic);
    }

    #[test]
    fn test_marketplace_find_best_offer_with_nepotism_rate_preference() {
        let mut marketplace = LendingMarketplace::new(0.01);
        marketplace.add_offer(1, 1000.0, 0.020, Some(600), 0); // Stranger: 2.0%
        marketplace.add_offer(2, 1000.0, 0.024, Some(600), 0); // Friend: 2.4%

        // Without nepotism -> Stranger offer 1 wins (2.0% < 2.4%)
        let standard = marketplace.find_best_offer(650, 500.0).unwrap();
        assert_eq!(standard.lender_id, 1);

        // With nepotism (bias 0.5, lender 2 is friend) -> Friend offer 2 wins!
        let (offer, is_nepotistic) = marketplace
            .find_best_offer_with_nepotism(650, 500.0, true, 0.5, false, |id| {
                if id == 2 {
                    1.0
                } else {
                    0.0
                }
            })
            .unwrap();
        assert_eq!(offer.lender_id, 2);
        assert!(is_nepotistic);
    }

    #[test]
    fn test_marketplace_find_best_offer_with_nepotism_inner_circle_only() {
        let mut marketplace = LendingMarketplace::new(0.01);
        marketplace.add_offer(1, 1000.0, 0.01, Some(600), 0); // Stranger: very cheap 1.0%
        marketplace.add_offer(2, 1000.0, 0.05, Some(600), 0); // Friend: expensive 5.0%

        // With inner_circle_only = true, stranger is excluded
        let (offer, is_nepotistic) = marketplace
            .find_best_offer_with_nepotism(650, 500.0, true, 0.3, true, |id| {
                if id == 2 {
                    1.0
                } else {
                    0.0
                }
            })
            .unwrap();
        assert_eq!(offer.lender_id, 2);
        assert!(is_nepotistic);
    }

    #[test]
    fn test_marketplace_find_best_offer_with_nepotism_out_of_range_bias_and_closeness() {
        let mut marketplace = LendingMarketplace::new(0.01);
        marketplace.add_offer(1, 1000.0, 0.05, Some(700), 0);

        // Negative bias and negative closeness should not panic or cause division by zero / negative rates
        let res =
            marketplace.find_best_offer_with_nepotism(600, 500.0, true, -1.0, false, |_| -0.5);
        assert!(res.is_none()); // 600 < 700 and forgiveness is clamped to 0

        // Excessively high bias (> 1.0) and closeness (> 1.0) should be clamped safely to 1.0
        let (offer, is_nepotistic) = marketplace
            .find_best_offer_with_nepotism(550, 500.0, true, 2.5, false, |_| 3.0)
            .expect("Should clamp and forgive up to 150 points (700 - 150 = 550)");
        assert_eq!(offer.id, 0);
        assert!(is_nepotistic);
    }
}
