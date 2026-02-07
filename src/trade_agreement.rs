use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a trade agreement between two or more persons
/// Trade agreements provide mutual discounts on trades between partners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreement {
    /// Unique identifier for this agreement
    pub id: usize,

    /// Set of person IDs who are part of this agreement
    pub partners: HashSet<usize>,

    /// Discount rate applied to trades between partners (0.0 - 1.0)
    /// e.g., 0.1 means 10% discount
    pub discount_rate: f64,

    /// Simulation step when the agreement was created
    pub created_at: usize,

    /// Duration of the agreement in simulation steps
    /// Agreement expires at created_at + duration
    pub duration: usize,

    /// Number of trades that have occurred under this agreement
    pub trade_count: usize,

    /// Total value of trades conducted under this agreement
    pub total_trade_value: f64,
}

impl TradeAgreement {
    /// Create a new bilateral trade agreement between two persons
    pub fn new_bilateral(
        id: usize,
        person1: usize,
        person2: usize,
        discount_rate: f64,
        created_at: usize,
        duration: usize,
    ) -> Self {
        let mut partners = HashSet::new();
        partners.insert(person1);
        partners.insert(person2);

        TradeAgreement {
            id,
            partners,
            discount_rate,
            created_at,
            duration,
            trade_count: 0,
            total_trade_value: 0.0,
        }
    }

    /// Create a new multilateral trade agreement between multiple persons
    pub fn new_multilateral(
        id: usize,
        partners: HashSet<usize>,
        discount_rate: f64,
        created_at: usize,
        duration: usize,
    ) -> Self {
        TradeAgreement {
            id,
            partners,
            discount_rate,
            created_at,
            duration,
            trade_count: 0,
            total_trade_value: 0.0,
        }
    }

    /// Check if this agreement is still active at the given simulation step
    pub fn is_active(&self, current_step: usize) -> bool {
        current_step < self.created_at + self.duration
    }

    /// Check if this agreement has expired at the given simulation step
    pub fn is_expired(&self, current_step: usize) -> bool {
        !self.is_active(current_step)
    }

    /// Check if two persons are both partners in this agreement
    pub fn includes_both(&self, person1: usize, person2: usize) -> bool {
        self.partners.contains(&person1) && self.partners.contains(&person2)
    }

    /// Record a trade that occurred under this agreement
    pub fn record_trade(&mut self, trade_value: f64) {
        self.trade_count += 1;
        self.total_trade_value += trade_value;
    }

    /// Get the number of partners in this agreement
    pub fn partner_count(&self) -> usize {
        self.partners.len()
    }

    /// Check if a person is a partner in this agreement
    pub fn is_partner(&self, person_id: usize) -> bool {
        self.partners.contains(&person_id)
    }
}

/// Statistics about trade agreements in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreementStatistics {
    /// Total number of trade agreements ever formed
    pub total_agreements_formed: usize,

    /// Number of currently active agreements
    pub active_agreements: usize,

    /// Number of expired agreements
    pub expired_agreements: usize,

    /// Number of bilateral agreements (2 partners)
    pub bilateral_agreements: usize,

    /// Number of multilateral agreements (3+ partners)
    pub multilateral_agreements: usize,

    /// Total trades conducted under agreements
    pub total_agreement_trades: usize,

    /// Total value of trades under agreements
    pub total_agreement_trade_value: f64,

    /// Average discount rate across all agreements
    pub average_discount_rate: f64,

    /// Average agreement duration in steps
    pub average_duration: f64,

    /// Average number of trades per agreement
    pub average_trades_per_agreement: f64,
}

impl TradeAgreementStatistics {
    /// Create empty statistics
    pub fn new() -> Self {
        TradeAgreementStatistics {
            total_agreements_formed: 0,
            active_agreements: 0,
            expired_agreements: 0,
            bilateral_agreements: 0,
            multilateral_agreements: 0,
            total_agreement_trades: 0,
            total_agreement_trade_value: 0.0,
            average_discount_rate: 0.0,
            average_duration: 0.0,
            average_trades_per_agreement: 0.0,
        }
    }
}

impl Default for TradeAgreementStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bilateral_agreement_creation() {
        let agreement = TradeAgreement::new_bilateral(1, 10, 20, 0.1, 0, 100);

        assert_eq!(agreement.id, 1);
        assert_eq!(agreement.partner_count(), 2);
        assert!(agreement.is_partner(10));
        assert!(agreement.is_partner(20));
        assert!(!agreement.is_partner(30));
        assert_eq!(agreement.discount_rate, 0.1);
        assert_eq!(agreement.created_at, 0);
        assert_eq!(agreement.duration, 100);
    }

    #[test]
    fn test_multilateral_agreement_creation() {
        let mut partners = HashSet::new();
        partners.insert(10);
        partners.insert(20);
        partners.insert(30);

        let agreement = TradeAgreement::new_multilateral(2, partners, 0.15, 5, 200);

        assert_eq!(agreement.id, 2);
        assert_eq!(agreement.partner_count(), 3);
        assert!(agreement.is_partner(10));
        assert!(agreement.is_partner(20));
        assert!(agreement.is_partner(30));
        assert_eq!(agreement.discount_rate, 0.15);
    }

    #[test]
    fn test_agreement_expiration() {
        let agreement = TradeAgreement::new_bilateral(1, 10, 20, 0.1, 0, 100);

        assert!(agreement.is_active(0));
        assert!(agreement.is_active(50));
        assert!(agreement.is_active(99));
        assert!(!agreement.is_active(100));
        assert!(!agreement.is_active(150));

        assert!(!agreement.is_expired(99));
        assert!(agreement.is_expired(100));
    }

    #[test]
    fn test_includes_both() {
        let agreement = TradeAgreement::new_bilateral(1, 10, 20, 0.1, 0, 100);

        assert!(agreement.includes_both(10, 20));
        assert!(agreement.includes_both(20, 10));
        assert!(!agreement.includes_both(10, 30));
        assert!(!agreement.includes_both(30, 20));
    }

    #[test]
    fn test_record_trade() {
        let mut agreement = TradeAgreement::new_bilateral(1, 10, 20, 0.1, 0, 100);

        assert_eq!(agreement.trade_count, 0);
        assert_eq!(agreement.total_trade_value, 0.0);

        agreement.record_trade(50.0);
        assert_eq!(agreement.trade_count, 1);
        assert_eq!(agreement.total_trade_value, 50.0);

        agreement.record_trade(30.0);
        assert_eq!(agreement.trade_count, 2);
        assert_eq!(agreement.total_trade_value, 80.0);
    }

    #[test]
    fn test_trade_agreement_statistics_new() {
        let stats = TradeAgreementStatistics::new();
        assert_eq!(stats.total_agreements_formed, 0);
        assert_eq!(stats.active_agreements, 0);
        assert_eq!(stats.bilateral_agreements, 0);
    }

    #[test]
    fn test_trade_agreement_statistics_default() {
        let stats = TradeAgreementStatistics::default();
        assert_eq!(stats.total_agreements_formed, 0);
        assert_eq!(stats.average_discount_rate, 0.0);
    }
}
