use crate::skill::SkillId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an externality effect (positive or negative) from a transaction.
///
/// Externalities are costs or benefits that affect third parties not directly
/// involved in a transaction. Positive externalities provide benefits to society
/// (e.g., education creating an informed citizenry), while negative externalities
/// impose costs (e.g., pollution from production).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Externality {
    /// The skill that generated this externality
    pub skill_id: SkillId,
    /// The simulation step when this externality occurred
    pub step: usize,
    /// The private cost/benefit (what buyer and seller exchanged)
    pub private_value: f64,
    /// The external cost (negative) or benefit (positive) to society
    /// Positive values are positive externalities, negative values are negative externalities
    pub external_value: f64,
    /// The social value (private + external)
    pub social_value: f64,
}

impl Externality {
    /// Creates a new externality record from a transaction.
    ///
    /// # Arguments
    /// * `skill_id` - The skill involved in the transaction
    /// * `step` - The current simulation step
    /// * `private_value` - The private transaction amount
    /// * `externality_rate` - Rate of externality as percentage of private value
    ///
    /// # Returns
    /// A new Externality instance
    pub fn new(skill_id: SkillId, step: usize, private_value: f64, externality_rate: f64) -> Self {
        let external_value = private_value * externality_rate;
        let social_value = private_value + external_value;

        Self { skill_id, step, private_value, external_value, social_value }
    }

    /// Returns true if this is a positive externality (benefit to society).
    pub fn is_positive(&self) -> bool {
        self.external_value > 0.0
    }

    /// Returns true if this is a negative externality (cost to society).
    pub fn is_negative(&self) -> bool {
        self.external_value < 0.0
    }

    /// Calculates the optimal Pigovian tax/subsidy for this externality.
    ///
    /// A Pigovian tax is a corrective tax equal to the external cost, designed to
    /// internalize the externality. For negative externalities, this is a positive tax;
    /// for positive externalities, this is a negative tax (subsidy).
    ///
    /// # Returns
    /// The recommended tax/subsidy amount (positive = tax, negative = subsidy)
    pub fn optimal_pigovian_correction(&self) -> f64 {
        -self.external_value // Negative of external value to correct it
    }
}

/// Aggregated statistics about externalities over the simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalityStats {
    /// Total count of externalities recorded
    pub total_count: usize,
    /// Count of positive externalities
    pub positive_count: usize,
    /// Count of negative externalities
    pub negative_count: usize,
    /// Sum of all private values (total trade volume for tracked transactions)
    pub total_private_value: f64,
    /// Sum of all external values (positive - negative)
    pub total_external_value: f64,
    /// Sum of all positive external values (benefits)
    pub total_positive_externalities: f64,
    /// Sum of absolute values of negative external values (total cost magnitude)
    pub total_negative_externalities: f64,
    /// Sum of all social values (private + external)
    pub total_social_value: f64,
    /// Average externality per transaction
    pub avg_external_value: f64,
    /// Ratio of external to private value (externality intensity)
    pub externality_intensity: f64,
    /// Optimal total Pigovian tax revenue (for negative externalities)
    pub optimal_pigovian_tax_total: f64,
    /// Optimal total Pigovian subsidy (for positive externalities)
    pub optimal_pigovian_subsidy_total: f64,
    /// Per-skill externality tracking
    pub per_skill_externalities: HashMap<SkillId, SkillExternalityStats>,
}

/// Externality statistics for a specific skill.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillExternalityStats {
    /// Number of transactions with externalities for this skill
    pub count: usize,
    /// Total private value for this skill
    pub total_private_value: f64,
    /// Total external value for this skill
    pub total_external_value: f64,
    /// Average external value per transaction for this skill
    pub avg_external_value: f64,
}

impl ExternalityStats {
    /// Creates empty externality statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an externality to the statistics.
    pub fn record(&mut self, externality: &Externality) {
        self.total_count += 1;

        if externality.is_positive() {
            self.positive_count += 1;
            self.total_positive_externalities += externality.external_value;
            // For positive externalities, optimal subsidy equals the external benefit
            self.optimal_pigovian_subsidy_total += externality.external_value;
        } else if externality.is_negative() {
            self.negative_count += 1;
            self.total_negative_externalities += externality.external_value.abs();
            self.optimal_pigovian_tax_total += externality.external_value.abs();
        }

        self.total_private_value += externality.private_value;
        self.total_external_value += externality.external_value;
        self.total_social_value += externality.social_value;

        // Update per-skill statistics
        let skill_stats =
            self.per_skill_externalities.entry(externality.skill_id.clone()).or_default();

        skill_stats.count += 1;
        skill_stats.total_private_value += externality.private_value;
        skill_stats.total_external_value += externality.external_value;
    }

    /// Finalizes statistics by calculating averages and ratios.
    pub fn finalize(&mut self) {
        if self.total_count > 0 {
            self.avg_external_value = self.total_external_value / self.total_count as f64;
        }

        if self.total_private_value > 0.0 {
            self.externality_intensity = self.total_external_value / self.total_private_value;
        }

        // Calculate per-skill averages
        for skill_stats in self.per_skill_externalities.values_mut() {
            if skill_stats.count > 0 {
                skill_stats.avg_external_value =
                    skill_stats.total_external_value / skill_stats.count as f64;
            }
        }
    }

    /// Returns a summary text description of the externality statistics.
    pub fn summary(&self) -> String {
        format!(
            "Externalities: {} total ({} positive, {} negative)\n  \
             Private value: {:.2}, External value: {:.2}, Social value: {:.2}\n  \
             Externality intensity: {:.2}% of private value\n  \
             Optimal Pigovian tax: {:.2}, Optimal subsidy: {:.2}",
            self.total_count,
            self.positive_count,
            self.negative_count,
            self.total_private_value,
            self.total_external_value,
            self.total_social_value,
            self.externality_intensity * 100.0,
            self.optimal_pigovian_tax_total,
            self.optimal_pigovian_subsidy_total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_externality_creation() {
        // Positive externality: education (20% positive externality)
        let ext = Externality::new("Education".to_string(), 1, 100.0, 0.2);
        assert_eq!(ext.private_value, 100.0);
        assert_eq!(ext.external_value, 20.0);
        assert_eq!(ext.social_value, 120.0);
        assert!(ext.is_positive());
        assert!(!ext.is_negative());
        assert_eq!(ext.optimal_pigovian_correction(), -20.0); // Subsidy
    }

    #[test]
    fn test_negative_externality() {
        // Negative externality: pollution (-30% negative externality)
        let ext = Externality::new("Manufacturing".to_string(), 1, 100.0, -0.3);
        assert_eq!(ext.private_value, 100.0);
        assert_eq!(ext.external_value, -30.0);
        assert_eq!(ext.social_value, 70.0);
        assert!(!ext.is_positive());
        assert!(ext.is_negative());
        assert_eq!(ext.optimal_pigovian_correction(), 30.0); // Tax
    }

    #[test]
    fn test_externality_stats() {
        let mut stats = ExternalityStats::new();

        // Add positive externality
        let ext1 = Externality::new("Education".to_string(), 1, 100.0, 0.2);
        stats.record(&ext1);

        // Add negative externality
        let ext2 = Externality::new("Manufacturing".to_string(), 2, 50.0, -0.3);
        stats.record(&ext2);

        // Add another positive externality
        let ext3 = Externality::new("Healthcare".to_string(), 3, 80.0, 0.15);
        stats.record(&ext3);

        stats.finalize();

        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.positive_count, 2);
        assert_eq!(stats.negative_count, 1);
        assert_eq!(stats.total_private_value, 230.0);

        // Total external: 20.0 - 15.0 + 12.0 = 17.0
        assert_eq!(stats.total_external_value, 17.0);

        // Total positive: 20.0 + 12.0 = 32.0
        assert_eq!(stats.total_positive_externalities, 32.0);

        // Total negative: 15.0
        assert_eq!(stats.total_negative_externalities, 15.0);

        // Optimal Pigovian tax should equal magnitude of negative externalities
        assert_eq!(stats.optimal_pigovian_tax_total, 15.0);

        // Optimal subsidy should equal total positive externalities
        assert_eq!(stats.optimal_pigovian_subsidy_total, 32.0);

        // Average: 17.0 / 3 = 5.67 (approx)
        assert!((stats.avg_external_value - 5.666).abs() < 0.01);

        // Per-skill stats
        assert_eq!(stats.per_skill_externalities.len(), 3);
    }
}
