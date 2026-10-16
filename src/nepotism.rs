//! # Nepotism and Cronyism Module (Nepotismus & Vetternwirtschaft)
//!
//! Models nepotism, favoritism, and cronyism in community interactions.
//! Agents prioritize trading partners and loan recipients from their friendship circle
//! or trust network, even when economically suboptimal (higher prices, lower quality,
//! or lower credit scores). Non-friends / out-group members face penalties or exclusions.
//!
//! ## Core Features
//!
//! - **Preferential Partner Selection**: When multiple providers exist for a skill,
//!   buyers with nepotistic preferences weight social connections (friendship, trust level)
//!   alongside economic merit (price, quality, reputation).
//! - **Out-Group Penalty (Malus für Nicht-Freunde)**: Sellers apply a surcharge to
//!   buyers outside their trust / friendship network, representing discrimination or out-group distrust.
//! - **Inner Circle Exclusivity**: If configured (`nepotism_inner_circle_only`), agents
//!   restrict transactions to their inner circle (friends / trust connections) when available.
//! - **Leniency in Credit / Lending**: In P2P lending, lenders can accept lower credit scores
//!   or give preferential matching to friends.
//! - **Welfare & Inefficiency Metrics**: Tracks nepotistic vs meritocratic decisions,
//!   inner-circle vs out-group trade volume, outgroup penalties applied, and welfare loss
//!   resulting from paying friendship premiums over cheaper alternative providers.

use crate::person::PersonId;
use crate::trust_network::TrustLevel;
use serde::{Deserialize, Serialize};

/// Statistics tracking nepotism, cronyism, and social favoritism in the simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NepotismStats {
    /// Total number of trade opportunities evaluated where multiple providers were available
    pub total_trades_evaluated: usize,
    /// Number of trades where a friend / trust connection was chosen over a strictly cheaper/higher-merit non-friend
    pub nepotistic_trades_count: usize,
    /// Number of trades where the purely optimal economic candidate was chosen
    pub merit_trades_count: usize,
    /// Total trades executed within the inner circle (friends or 1st-3rd degree trust)
    pub inner_circle_trades_count: usize,
    /// Total trades executed with out-group members (strangers)
    pub outgroup_trades_count: usize,
    /// Number of times out-group price penalties were applied to non-friends
    pub outgroup_penalties_applied: usize,
    /// Cumulative welfare loss (extra money spent by choosing more expensive friends over cheaper non-friends)
    pub total_welfare_loss: f64,
    /// Number of loans granted to friends despite sub-standard credit scores or preferred over better market rates
    pub nepotistic_loans_count: usize,
}

impl Default for NepotismStats {
    fn default() -> Self {
        Self::new()
    }
}

impl NepotismStats {
    /// Creates a new, empty NepotismStats instance.
    pub fn new() -> Self {
        Self {
            total_trades_evaluated: 0,
            nepotistic_trades_count: 0,
            merit_trades_count: 0,
            inner_circle_trades_count: 0,
            outgroup_trades_count: 0,
            outgroup_penalties_applied: 0,
            total_welfare_loss: 0.0,
            nepotistic_loans_count: 0,
        }
    }

    /// Records an evaluated trade decision.
    ///
    /// # Arguments
    /// * `is_nepotistic` - Whether a social connection was chosen over a cheaper/better alternative
    /// * `is_inner_circle` - Whether the transaction partner is a friend or in the trust network
    /// * `welfare_loss` - Extra money spent due to choosing the friend over a cheaper alternative (>= 0.0)
    pub fn record_trade(&mut self, is_nepotistic: bool, is_inner_circle: bool, welfare_loss: f64) {
        self.total_trades_evaluated += 1;
        if is_nepotistic {
            self.nepotistic_trades_count += 1;
            if welfare_loss > 0.0 {
                self.total_welfare_loss += welfare_loss;
            }
        } else {
            self.merit_trades_count += 1;
        }

        if is_inner_circle {
            self.inner_circle_trades_count += 1;
        } else {
            self.outgroup_trades_count += 1;
        }
    }

    /// Records an applied outgroup penalty.
    pub fn record_outgroup_penalty(&mut self) {
        self.outgroup_penalties_applied += 1;
    }

    /// Records a nepotistic loan match.
    pub fn record_nepotistic_loan(&mut self) {
        self.nepotistic_loans_count += 1;
    }

    /// Calculates the rate of nepotistic choices among evaluated competitive trades (0.0 - 1.0).
    pub fn nepotism_rate(&self) -> f64 {
        if self.total_trades_evaluated == 0 {
            0.0
        } else {
            self.nepotistic_trades_count as f64 / self.total_trades_evaluated as f64
        }
    }

    /// Calculates the proportion of trades conducted within the inner circle (0.0 - 1.0).
    pub fn inner_circle_ratio(&self) -> f64 {
        let total = self.inner_circle_trades_count + self.outgroup_trades_count;
        if total == 0 {
            0.0
        } else {
            self.inner_circle_trades_count as f64 / total as f64
        }
    }
}

/// Calculates the social closeness score between two individuals.
///
/// Returns a value between 0.0 (no connection / out-group) and 1.0 (direct friend).
pub fn calculate_social_closeness(is_direct_friend: bool, trust_level: TrustLevel) -> f64 {
    if is_direct_friend {
        1.0
    } else {
        match trust_level {
            TrustLevel::Direct => 1.0,
            TrustLevel::SecondDegree => 0.5,
            TrustLevel::ThirdDegree => 0.25,
            TrustLevel::None => 0.0,
        }
    }
}

/// Evaluates a candidate seller by combining economic merit and nepotistic social bias.
///
/// * `effective_price` - The price the buyer would pay (including discounts or penalties)
/// * `reputation` - Seller reputation (typically 0.0 to 2.0, neutral 1.0)
/// * `social_closeness` - Social closeness score in [0.0, 1.0]
/// * `nepotism_bias` - Weight of social connection vs economic merit [0.0, 1.0]
///
/// Returns a composite preference score. Higher score = more preferred candidate.
pub fn calculate_candidate_score(
    effective_price: f64,
    reputation: f64,
    social_closeness: f64,
    nepotism_bias: f64,
) -> f64 {
    // Economic merit: inverse of price multiplied by reputation
    let price_factor = 100.0 / effective_price.max(0.1);
    let merit_score = price_factor * (reputation / 1.0).max(0.1);

    // If nepotism_bias is 0.0, purely meritocratic
    if nepotism_bias <= 0.0 {
        return merit_score;
    }

    // Social bonus scales with nepotism_bias and social_closeness:
    // When social_closeness is 1.0 and nepotism_bias is 0.5, score is boosted by 2.5x
    merit_score * (1.0 + nepotism_bias * social_closeness * 3.0)
}

/// Candidate seller information for nepotistic partner selection.
#[derive(Debug, Clone)]
pub struct CandidateSeller {
    pub entity_idx: usize,
    pub person_id: PersonId,
    pub effective_price: f64,
    pub reputation: f64,
    pub is_friend: bool,
    pub trust_level: TrustLevel,
    pub social_closeness: f64,
    pub preference_score: f64,
}

/// Selects the best seller among candidate providers for a skill.
///
/// Returns `(selected_candidate, is_nepotistic, welfare_loss)` if candidates exist.
/// `is_nepotistic` is true if a social connection was chosen despite a strictly cheaper
/// alternative being available. `welfare_loss` is the price premium paid over that cheaper alternative.
pub fn select_seller_with_nepotism(
    candidates: &[CandidateSeller],
    nepotism_enabled: bool,
    inner_circle_only: bool,
) -> Option<(&CandidateSeller, bool, f64)> {
    if candidates.is_empty() {
        return None;
    }

    // If inner_circle_only is enabled, filter to inner circle if any exist
    let eligible_candidates: Vec<&CandidateSeller> = if nepotism_enabled && inner_circle_only {
        let inner_circle: Vec<&CandidateSeller> =
            candidates.iter().filter(|c| c.social_closeness > 0.0).collect();
        if !inner_circle.is_empty() {
            inner_circle
        } else {
            candidates.iter().collect()
        }
    } else {
        candidates.iter().collect()
    };

    // Find the purely cheapest / best economic candidate among all candidates
    let cheapest_candidate = candidates.iter().min_by(|a, b| {
        a.effective_price
            .partial_cmp(&b.effective_price)
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    // Select candidate with the highest preference score
    let selected = eligible_candidates
        .iter()
        .max_by(|a, b| {
            a.preference_score
                .partial_cmp(&b.preference_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()?;

    // Check if nepotism caused a suboptimal economic choice
    let is_nepotistic = nepotism_enabled
        && selected.entity_idx != cheapest_candidate.entity_idx
        && selected.social_closeness > 0.0
        && selected.effective_price > cheapest_candidate.effective_price;

    let welfare_loss = if is_nepotistic {
        (selected.effective_price - cheapest_candidate.effective_price).max(0.0)
    } else {
        0.0
    };

    Some((selected, is_nepotistic, welfare_loss))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nepotism_stats_new() {
        let stats = NepotismStats::new();
        assert_eq!(stats.total_trades_evaluated, 0);
        assert_eq!(stats.nepotistic_trades_count, 0);
        assert_eq!(stats.merit_trades_count, 0);
        assert_eq!(stats.inner_circle_trades_count, 0);
        assert_eq!(stats.outgroup_trades_count, 0);
        assert_eq!(stats.outgroup_penalties_applied, 0);
        assert_eq!(stats.total_welfare_loss, 0.0);
        assert_eq!(stats.nepotistic_loans_count, 0);
        assert_eq!(stats.nepotism_rate(), 0.0);
        assert_eq!(stats.inner_circle_ratio(), 0.0);
    }

    #[test]
    fn test_nepotism_stats_record_trade() {
        let mut stats = NepotismStats::new();
        stats.record_trade(true, true, 2.5);
        stats.record_trade(false, false, 0.0);
        stats.record_outgroup_penalty();
        stats.record_nepotistic_loan();

        assert_eq!(stats.total_trades_evaluated, 2);
        assert_eq!(stats.nepotistic_trades_count, 1);
        assert_eq!(stats.merit_trades_count, 1);
        assert_eq!(stats.inner_circle_trades_count, 1);
        assert_eq!(stats.outgroup_trades_count, 1);
        assert_eq!(stats.outgroup_penalties_applied, 1);
        assert_eq!(stats.total_welfare_loss, 2.5);
        assert_eq!(stats.nepotistic_loans_count, 1);
        assert_eq!(stats.nepotism_rate(), 0.5);
        assert_eq!(stats.inner_circle_ratio(), 0.5);
    }

    #[test]
    fn test_calculate_social_closeness() {
        assert_eq!(calculate_social_closeness(true, TrustLevel::None), 1.0);
        assert_eq!(calculate_social_closeness(false, TrustLevel::Direct), 1.0);
        assert_eq!(calculate_social_closeness(false, TrustLevel::SecondDegree), 0.5);
        assert_eq!(calculate_social_closeness(false, TrustLevel::ThirdDegree), 0.25);
        assert_eq!(calculate_social_closeness(false, TrustLevel::None), 0.0);
    }

    #[test]
    fn test_calculate_candidate_score_zero_bias() {
        // Pure meritocracy when bias is 0.0
        let score_cheaper = calculate_candidate_score(10.0, 1.0, 0.0, 0.0);
        let score_expensive_friend = calculate_candidate_score(15.0, 1.0, 1.0, 0.0);
        assert!(score_cheaper > score_expensive_friend);
    }

    #[test]
    fn test_calculate_candidate_score_high_bias() {
        // Friend wins despite higher price when nepotism bias is high
        let score_cheaper = calculate_candidate_score(10.0, 1.0, 0.0, 0.5);
        let score_expensive_friend = calculate_candidate_score(12.0, 1.0, 1.0, 0.5);
        assert!(score_expensive_friend > score_cheaper);
    }

    #[test]
    fn test_select_seller_with_nepotism_merit() {
        let candidates = vec![
            CandidateSeller {
                entity_idx: 1,
                person_id: 1,
                effective_price: 10.0,
                reputation: 1.0,
                is_friend: false,
                trust_level: TrustLevel::None,
                social_closeness: 0.0,
                preference_score: calculate_candidate_score(10.0, 1.0, 0.0, 0.0),
            },
            CandidateSeller {
                entity_idx: 2,
                person_id: 2,
                effective_price: 15.0,
                reputation: 1.0,
                is_friend: true,
                trust_level: TrustLevel::Direct,
                social_closeness: 1.0,
                preference_score: calculate_candidate_score(15.0, 1.0, 1.0, 0.0),
            },
        ];

        let (selected, is_nepotistic, welfare_loss) =
            select_seller_with_nepotism(&candidates, false, false).unwrap();
        assert_eq!(selected.entity_idx, 1);
        assert!(!is_nepotistic);
        assert_eq!(welfare_loss, 0.0);
    }

    #[test]
    fn test_select_seller_with_nepotism_favoritism() {
        let candidates = vec![
            CandidateSeller {
                entity_idx: 1,
                person_id: 1,
                effective_price: 10.0,
                reputation: 1.0,
                is_friend: false,
                trust_level: TrustLevel::None,
                social_closeness: 0.0,
                preference_score: calculate_candidate_score(10.0, 1.0, 0.0, 0.5),
            },
            CandidateSeller {
                entity_idx: 2,
                person_id: 2,
                effective_price: 12.0,
                reputation: 1.0,
                is_friend: true,
                trust_level: TrustLevel::Direct,
                social_closeness: 1.0,
                preference_score: calculate_candidate_score(12.0, 1.0, 1.0, 0.5),
            },
        ];

        let (selected, is_nepotistic, welfare_loss) =
            select_seller_with_nepotism(&candidates, true, false).unwrap();
        assert_eq!(selected.entity_idx, 2);
        assert!(is_nepotistic);
        assert_eq!(welfare_loss, 2.0);
    }

    #[test]
    fn test_select_seller_inner_circle_only() {
        let candidates = vec![
            CandidateSeller {
                entity_idx: 1,
                person_id: 1,
                effective_price: 5.0, // Very cheap outsider
                reputation: 1.0,
                is_friend: false,
                trust_level: TrustLevel::None,
                social_closeness: 0.0,
                preference_score: 100.0,
            },
            CandidateSeller {
                entity_idx: 2,
                person_id: 2,
                effective_price: 20.0, // Expensive friend
                reputation: 1.0,
                is_friend: true,
                trust_level: TrustLevel::Direct,
                social_closeness: 1.0,
                preference_score: 50.0,
            },
        ];

        // With inner_circle_only = true, the outsider is excluded completely
        let (selected, is_nepotistic, welfare_loss) =
            select_seller_with_nepotism(&candidates, true, true).unwrap();
        assert_eq!(selected.entity_idx, 2);
        assert!(is_nepotistic);
        assert_eq!(welfare_loss, 15.0);
    }
}
