use serde::{Deserialize, Serialize};

/// Government entity that collects taxes and optionally redistributes wealth.
///
/// The government acts as a central authority in the simulation that can:
/// - Collect taxes from persons based on their income or wealth
/// - Redistribute collected taxes to all persons equally (if enabled)
/// - Track tax statistics for analysis
///
/// # Examples
///
/// ```
/// use simulation_framework::government::Government;
///
/// let mut gov = Government::new(0.1, true); // 10% tax rate with redistribution
/// assert_eq!(gov.get_tax_rate(), 0.1);
/// assert_eq!(gov.get_total_collected(), 0.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Government {
    /// Tax rate as a fraction (0.0 to 1.0).
    /// For example, 0.1 represents a 10% tax rate.
    tax_rate: f64,
    /// Current treasury balance (taxes collected but not yet redistributed).
    total_collected: f64,
    /// Cumulative total of all taxes ever collected (for statistics).
    cumulative_collected: f64,
    /// Total taxes redistributed during the simulation.
    total_redistributed: f64,
    /// Whether to redistribute collected taxes back to the population.
    redistribution_enabled: bool,
}

impl Government {
    /// Creates a new Government with the specified tax rate and redistribution setting.
    ///
    /// # Arguments
    ///
    /// * `tax_rate` - The tax rate as a fraction (0.0 to 1.0)
    /// * `redistribution_enabled` - Whether to redistribute collected taxes
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::government::Government;
    ///
    /// let gov = Government::new(0.15, true);
    /// assert_eq!(gov.get_tax_rate(), 0.15);
    /// ```
    pub fn new(tax_rate: f64, redistribution_enabled: bool) -> Self {
        Government {
            tax_rate,
            total_collected: 0.0,
            cumulative_collected: 0.0,
            total_redistributed: 0.0,
            redistribution_enabled,
        }
    }

    /// Gets the current tax rate.
    pub fn get_tax_rate(&self) -> f64 {
        self.tax_rate
    }

    /// Gets the total amount of taxes collected.
    pub fn get_total_collected(&self) -> f64 {
        self.total_collected
    }

    /// Gets the cumulative total of all taxes ever collected.
    pub fn get_cumulative_collected(&self) -> f64 {
        self.cumulative_collected
    }

    /// Gets the total amount of taxes redistributed.
    pub fn get_total_redistributed(&self) -> f64 {
        self.total_redistributed
    }

    /// Checks if redistribution is enabled.
    pub fn is_redistribution_enabled(&self) -> bool {
        self.redistribution_enabled
    }

    /// Collects tax from a single amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to tax
    ///
    /// # Returns
    ///
    /// The tax amount collected (amount * tax_rate)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::government::Government;
    ///
    /// let mut gov = Government::new(0.1, true);
    /// let tax = gov.collect_tax(100.0);
    /// assert_eq!(tax, 10.0);
    /// assert_eq!(gov.get_total_collected(), 10.0);
    /// ```
    pub fn collect_tax(&mut self, amount: f64) -> f64 {
        let tax = amount * self.tax_rate;
        self.total_collected += tax;
        self.cumulative_collected += tax;
        tax
    }

    /// Redistributes collected taxes equally among all persons.
    ///
    /// # Arguments
    ///
    /// * `num_persons` - The number of persons to redistribute to
    ///
    /// # Returns
    ///
    /// The amount each person receives, or 0.0 if redistribution is disabled
    /// or no persons exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::government::Government;
    ///
    /// let mut gov = Government::new(0.1, true);
    /// gov.collect_tax(100.0); // Collects 10.0 in taxes
    /// let per_person = gov.redistribute(5);
    /// assert_eq!(per_person, 2.0); // 10.0 / 5 = 2.0
    /// assert_eq!(gov.get_total_redistributed(), 10.0);
    /// ```
    pub fn redistribute(&mut self, num_persons: usize) -> f64 {
        if !self.redistribution_enabled || num_persons == 0 || self.total_collected == 0.0 {
            return 0.0;
        }

        let per_person = self.total_collected / num_persons as f64;
        self.total_redistributed += self.total_collected;
        self.total_collected = 0.0; // Clear the treasury after redistribution
        per_person
    }

    /// Adds already-calculated tax amount to the government treasury.
    ///
    /// Use this method when taxes have been calculated elsewhere and you want
    /// to add them to the government's totals. Unlike `collect_tax`, this method
    /// does not apply the tax rate.
    ///
    /// # Arguments
    ///
    /// * `tax_amount` - The pre-calculated tax amount to add
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::government::Government;
    ///
    /// let mut gov = Government::new(0.1, true);
    /// // Taxes were calculated elsewhere as 10.0
    /// gov.add_tax(10.0);
    /// assert_eq!(gov.get_cumulative_collected(), 10.0);
    /// ```
    pub fn add_tax(&mut self, tax_amount: f64) {
        self.total_collected += tax_amount;
        self.cumulative_collected += tax_amount;
    }

    /// Resets the government's tax collection statistics.
    /// Useful for testing or starting a new simulation phase.
    pub fn reset(&mut self) {
        self.total_collected = 0.0;
        self.cumulative_collected = 0.0;
        self.total_redistributed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_government_new() {
        let gov = Government::new(0.15, true);
        assert_eq!(gov.get_tax_rate(), 0.15);
        assert_eq!(gov.get_total_collected(), 0.0);
        assert_eq!(gov.get_total_redistributed(), 0.0);
        assert!(gov.is_redistribution_enabled());
    }

    #[test]
    fn test_collect_tax() {
        let mut gov = Government::new(0.2, true);
        let tax = gov.collect_tax(100.0);
        assert_eq!(tax, 20.0);
        assert_eq!(gov.get_total_collected(), 20.0);
    }

    #[test]
    fn test_collect_tax_multiple() {
        let mut gov = Government::new(0.1, true);
        gov.collect_tax(100.0);
        gov.collect_tax(50.0);
        assert_eq!(gov.get_total_collected(), 15.0); // 10 + 5
    }

    #[test]
    fn test_redistribute_enabled() {
        let mut gov = Government::new(0.1, true);
        gov.collect_tax(100.0); // Collects 10.0
        let per_person = gov.redistribute(5);
        assert_eq!(per_person, 2.0);
        assert_eq!(gov.get_total_redistributed(), 10.0);
        assert_eq!(gov.get_total_collected(), 0.0); // Should be cleared
    }

    #[test]
    fn test_redistribute_disabled() {
        let mut gov = Government::new(0.1, false);
        gov.collect_tax(100.0);
        let per_person = gov.redistribute(5);
        assert_eq!(per_person, 0.0);
        assert_eq!(gov.get_total_redistributed(), 0.0);
    }

    #[test]
    fn test_redistribute_zero_persons() {
        let mut gov = Government::new(0.1, true);
        gov.collect_tax(100.0);
        let per_person = gov.redistribute(0);
        assert_eq!(per_person, 0.0);
    }

    #[test]
    fn test_redistribute_no_taxes() {
        let mut gov = Government::new(0.1, true);
        let per_person = gov.redistribute(5);
        assert_eq!(per_person, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut gov = Government::new(0.1, true);
        gov.collect_tax(100.0);
        gov.redistribute(5);
        gov.reset();
        assert_eq!(gov.get_total_collected(), 0.0);
        assert_eq!(gov.get_total_redistributed(), 0.0);
    }

    #[test]
    fn test_zero_tax_rate() {
        let mut gov = Government::new(0.0, true);
        let tax = gov.collect_tax(100.0);
        assert_eq!(tax, 0.0);
        assert_eq!(gov.get_total_collected(), 0.0);
    }

    #[test]
    fn test_high_tax_rate() {
        let mut gov = Government::new(0.9, true);
        let tax = gov.collect_tax(100.0);
        assert_eq!(tax, 90.0);
    }
}
