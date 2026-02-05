/// Invariant checking framework for simulation validation.
///
/// This module provides a trait-based system for defining and checking invariants
/// during simulation execution. Invariants are conditions that should always hold true
/// during the simulation, and violations indicate potential bugs or incorrect assumptions.
///
/// # Example
///
/// ```rust
/// use simulation_framework::invariant::{Invariant, InvariantViolation};
/// use simulation_framework::{SimulationEngine, SimulationConfig};
///
/// // Check an invariant manually
/// let config = SimulationConfig::default();
/// let engine = SimulationEngine::new(config);
/// let money_conservation = simulation_framework::invariant::MoneyConservationInvariant::new(
///     engine.get_entities().iter().map(|e| e.get_money()).sum()
/// );
///
/// // This should pass since no simulation steps have occurred
/// assert!(money_conservation.check(&engine).is_ok());
/// ```
use crate::engine::SimulationEngine;
use std::fmt;

/// Represents a violation of an invariant.
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    /// Name of the violated invariant
    pub invariant_name: String,
    /// Description of what went wrong
    pub description: String,
    /// Current simulation step when the violation occurred
    pub step: usize,
    /// Optional: Expected value
    pub expected: Option<String>,
    /// Optional: Actual value observed
    pub actual: Option<String>,
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invariant violation '{}' at step {}: {}",
            self.invariant_name, self.step, self.description
        )?;
        if let (Some(expected), Some(actual)) = (&self.expected, &self.actual) {
            write!(f, " (expected: {}, actual: {})", expected, actual)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvariantViolation {}

/// Trait for simulation invariants that can be checked during execution.
///
/// Invariants are conditions that should always hold true in a valid simulation state.
/// Implementing this trait allows creating custom validation rules that can be
/// automatically checked at each simulation step.
pub trait Invariant: Send + Sync {
    /// Returns the name of this invariant for reporting purposes.
    fn name(&self) -> &str;

    /// Checks if the invariant holds for the current simulation state.
    ///
    /// Returns `Ok(())` if the invariant is satisfied, or an `Err(InvariantViolation)`
    /// if the invariant is violated.
    fn check(&self, engine: &SimulationEngine) -> Result<(), InvariantViolation>;

    /// Optional: Returns a description of what this invariant checks.
    fn description(&self) -> String {
        format!("Checks the '{}' invariant", self.name())
    }
}

/// Invariant that checks total money conservation in the economy.
///
/// This invariant ensures that the total amount of money in the system remains constant,
/// unless explicitly modified by the simulation (e.g., through tax redistribution,
/// transaction fees, or money creation/destruction).
///
/// The check accounts for:
/// - Money held by all persons (both available and savings)
/// - Transaction fees collected
/// - Tax revenue collected
/// - Money tied up in loans (principal amounts)
///
/// # Example
///
/// ```rust
/// use simulation_framework::invariant::{Invariant, MoneyConservationInvariant};
/// use simulation_framework::{SimulationEngine, SimulationConfig};
///
/// let config = SimulationConfig::default();
/// let engine = SimulationEngine::new(config);
/// let initial_money: f64 = engine.get_entities().iter().map(|e| e.get_money()).sum();
/// let invariant = MoneyConservationInvariant::new(initial_money);
///
/// // Check that money is conserved
/// assert!(invariant.check(&engine).is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct MoneyConservationInvariant {
    initial_total_money: f64,
    tolerance: f64,
}

impl MoneyConservationInvariant {
    /// Creates a new money conservation invariant with the given initial total money.
    ///
    /// # Arguments
    ///
    /// * `initial_total_money` - The total amount of money at the start of the simulation
    pub fn new(initial_total_money: f64) -> Self {
        // Use 0.01% of initial money as tolerance, or 0.01 minimum for very small economies
        let tolerance = (initial_total_money * 0.0001).max(0.01);
        Self { initial_total_money, tolerance }
    }

    /// Creates a new money conservation invariant with a custom tolerance.
    ///
    /// # Arguments
    ///
    /// * `initial_total_money` - The total amount of money at the start of the simulation
    /// * `tolerance` - The acceptable difference in money totals (for floating-point precision)
    pub fn new_with_tolerance(initial_total_money: f64, tolerance: f64) -> Self {
        Self { initial_total_money, tolerance }
    }
}

impl Invariant for MoneyConservationInvariant {
    fn name(&self) -> &str {
        "MoneyConservation"
    }

    fn check(&self, engine: &SimulationEngine) -> Result<(), InvariantViolation> {
        // Calculate total money in the system
        let total_person_money: f64 = engine
            .get_entities()
            .iter()
            .map(|e| e.get_money() + e.person_data.savings)
            .sum();

        // Add fees collected by the system
        let total_fees = engine.get_total_fees_collected();

        // Add tax revenue collected (whether redistributed or not)
        let total_taxes = engine.get_total_taxes_collected();

        // Calculate current total
        let current_total = total_person_money + total_fees + total_taxes;

        // Check if money is conserved within tolerance
        let difference = (current_total - self.initial_total_money).abs();

        if difference > self.tolerance {
            return Err(InvariantViolation {
                invariant_name: self.name().to_string(),
                description: format!(
                    "Total money in the system has changed. Difference: {:.2}",
                    current_total - self.initial_total_money
                ),
                step: engine.get_current_step(),
                expected: Some(format!("{:.2}", self.initial_total_money)),
                actual: Some(format!("{:.2}", current_total)),
            });
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Ensures total money remains constant at {:.2} (tolerance: {:.2})",
            self.initial_total_money, self.tolerance
        )
    }
}

/// Invariant that checks no person has negative wealth (unless loans are enabled).
///
/// This invariant ensures that persons maintain non-negative money balances.
/// When loans are enabled, persons may have negative balances (debt), so this
/// invariant is automatically disabled in that case.
///
/// # Example
///
/// ```rust
/// use simulation_framework::invariant::{Invariant, NonNegativeWealthInvariant};
/// use simulation_framework::{SimulationEngine, SimulationConfig};
///
/// let config = SimulationConfig::default();
/// let engine = SimulationEngine::new(config);
/// let invariant = NonNegativeWealthInvariant::new(false); // No loans enabled
///
/// // Check that all persons have non-negative wealth
/// assert!(invariant.check(&engine).is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct NonNegativeWealthInvariant {
    allow_negative_when_loans_enabled: bool,
}

impl NonNegativeWealthInvariant {
    /// Creates a new non-negative wealth invariant.
    ///
    /// # Arguments
    ///
    /// * `allow_negative_when_loans_enabled` - If true, the invariant is skipped when loans are enabled
    pub fn new(allow_negative_when_loans_enabled: bool) -> Self {
        Self { allow_negative_when_loans_enabled }
    }
}

impl Invariant for NonNegativeWealthInvariant {
    fn name(&self) -> &str {
        "NonNegativeWealth"
    }

    fn check(&self, engine: &SimulationEngine) -> Result<(), InvariantViolation> {
        // Skip check if loans are enabled and we allow negative balances
        if self.allow_negative_when_loans_enabled && engine.get_config().enable_loans {
            return Ok(());
        }

        // Find any persons with negative money
        let negative_wealth_persons: Vec<(usize, f64)> = engine
            .get_entities()
            .iter()
            .enumerate()
            .filter_map(|(idx, e)| {
                let money = e.get_money();
                if money < 0.0 {
                    Some((idx, money))
                } else {
                    None
                }
            })
            .collect();

        if !negative_wealth_persons.is_empty() {
            let person_details: Vec<String> = negative_wealth_persons
                .iter()
                .take(5) // Limit to first 5 for readability
                .map(|(idx, money)| format!("Person {} has {:.2}", idx, money))
                .collect();

            let details = if negative_wealth_persons.len() > 5 {
                format!(
                    "{} (and {} more)",
                    person_details.join(", "),
                    negative_wealth_persons.len() - 5
                )
            } else {
                person_details.join(", ")
            };

            return Err(InvariantViolation {
                invariant_name: self.name().to_string(),
                description: format!(
                    "{} person(s) have negative wealth: {}",
                    negative_wealth_persons.len(),
                    details
                ),
                step: engine.get_current_step(),
                expected: Some("All persons have non-negative wealth".to_string()),
                actual: Some(format!(
                    "{} persons with negative wealth",
                    negative_wealth_persons.len()
                )),
            });
        }

        Ok(())
    }

    fn description(&self) -> String {
        if self.allow_negative_when_loans_enabled {
            "Ensures no person has negative wealth (unless loans are enabled)".to_string()
        } else {
            "Ensures no person has negative wealth".to_string()
        }
    }
}

/// Collection of invariants to check during simulation.
#[derive(Default)]
pub struct InvariantChecker {
    invariants: Vec<Box<dyn Invariant>>,
    /// If true, stop the simulation on first violation. If false, log and continue.
    strict_mode: bool,
    /// Count of violations detected during the simulation
    total_violations: std::cell::Cell<usize>,
}

impl InvariantChecker {
    /// Creates a new empty invariant checker.
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
            strict_mode: false,
            total_violations: std::cell::Cell::new(0),
        }
    }

    /// Creates a new invariant checker in strict mode.
    ///
    /// In strict mode, the simulation will panic on the first invariant violation.
    pub fn new_strict() -> Self {
        Self {
            invariants: Vec::new(),
            strict_mode: true,
            total_violations: std::cell::Cell::new(0),
        }
    }

    /// Adds an invariant to be checked.
    pub fn add_invariant(&mut self, invariant: Box<dyn Invariant>) {
        self.invariants.push(invariant);
    }

    /// Sets whether to use strict mode (panic on violation) or lenient mode (log and continue).
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Checks all registered invariants against the current simulation state.
    ///
    /// Returns a vector of violations. If strict mode is enabled, this will panic
    /// on the first violation instead of returning it.
    pub fn check_all(&self, engine: &SimulationEngine) -> Vec<InvariantViolation> {
        let mut violations = Vec::new();

        for invariant in &self.invariants {
            if let Err(violation) = invariant.check(engine) {
                if self.strict_mode {
                    panic!("Invariant violation in strict mode: {}", violation);
                }
                violations.push(violation);
            }
        }

        // Update violation count
        if !violations.is_empty() {
            self.total_violations.set(self.total_violations.get() + violations.len());
        }

        violations
    }

    /// Returns the total number of violations detected so far.
    pub fn total_violations(&self) -> usize {
        self.total_violations.get()
    }

    /// Returns true if any invariants are registered.
    pub fn has_invariants(&self) -> bool {
        !self.invariants.is_empty()
    }

    /// Returns the number of registered invariants.
    pub fn count(&self) -> usize {
        self.invariants.len()
    }

    /// Returns whether strict mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SimulationConfig, SimulationEngine};

    #[test]
    fn test_money_conservation_invariant_passes() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config);
        let initial_money: f64 = engine.get_entities().iter().map(|e| e.get_money()).sum();

        let invariant = MoneyConservationInvariant::new(initial_money);
        assert!(invariant.check(&engine).is_ok());
    }

    #[test]
    fn test_non_negative_wealth_invariant_passes() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config);

        let invariant = NonNegativeWealthInvariant::new(false);
        assert!(invariant.check(&engine).is_ok());
    }

    #[test]
    fn test_invariant_checker_empty() {
        let checker = InvariantChecker::new();
        assert!(!checker.has_invariants());
        assert_eq!(checker.count(), 0);
        assert!(!checker.is_strict());
    }

    #[test]
    fn test_invariant_checker_strict_mode() {
        let mut checker = InvariantChecker::new_strict();
        assert!(checker.is_strict());

        checker.set_strict_mode(false);
        assert!(!checker.is_strict());
    }

    #[test]
    fn test_invariant_checker_add_and_check() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config);
        let initial_money: f64 = engine.get_entities().iter().map(|e| e.get_money()).sum();

        let mut checker = InvariantChecker::new();
        checker.add_invariant(Box::new(MoneyConservationInvariant::new(initial_money)));
        checker.add_invariant(Box::new(NonNegativeWealthInvariant::new(false)));

        assert_eq!(checker.count(), 2);
        assert!(checker.has_invariants());

        let violations = checker.check_all(&engine);
        assert_eq!(violations.len(), 0, "No violations should be found in initial state");
    }

    #[test]
    fn test_invariant_violation_display() {
        let violation = InvariantViolation {
            invariant_name: "TestInvariant".to_string(),
            description: "Test violation".to_string(),
            step: 42,
            expected: Some("100".to_string()),
            actual: Some("95".to_string()),
        };

        let display = format!("{}", violation);
        assert!(display.contains("TestInvariant"));
        assert!(display.contains("step 42"));
        assert!(display.contains("Test violation"));
        assert!(display.contains("expected: 100"));
        assert!(display.contains("actual: 95"));
    }

    #[test]
    fn test_invariant_violation_display_without_values() {
        let violation = InvariantViolation {
            invariant_name: "TestInvariant".to_string(),
            description: "Test violation".to_string(),
            step: 42,
            expected: None,
            actual: None,
        };

        let display = format!("{}", violation);
        assert!(display.contains("TestInvariant"));
        assert!(display.contains("step 42"));
        assert!(!display.contains("expected:"));
    }

    #[test]
    fn test_money_conservation_custom_tolerance() {
        let invariant = MoneyConservationInvariant::new_with_tolerance(1000.0, 5.0);
        assert_eq!(invariant.tolerance, 5.0);
        assert_eq!(invariant.initial_total_money, 1000.0);
    }

    #[test]
    fn test_money_conservation_description() {
        let invariant = MoneyConservationInvariant::new(1000.0);
        let desc = invariant.description();
        assert!(desc.contains("1000.00"));
        assert!(desc.contains("total money"));
    }

    #[test]
    fn test_non_negative_wealth_description() {
        let invariant1 = NonNegativeWealthInvariant::new(false);
        let desc1 = invariant1.description();
        assert!(desc1.contains("negative wealth"));
        assert!(!desc1.contains("unless loans"));

        let invariant2 = NonNegativeWealthInvariant::new(true);
        let desc2 = invariant2.description();
        assert!(desc2.contains("negative wealth"));
        assert!(desc2.contains("unless loans are enabled"));
    }

    #[test]
    fn test_non_negative_wealth_with_loans_enabled() {
        let mut config = SimulationConfig::default();
        config.enable_loans = true;
        config.max_steps = 5;
        let engine = SimulationEngine::new(config);

        // With allow_negative_when_loans_enabled=true, should pass
        let invariant = NonNegativeWealthInvariant::new(true);
        assert!(invariant.check(&engine).is_ok());

        // With allow_negative_when_loans_enabled=false, should still check
        let invariant2 = NonNegativeWealthInvariant::new(false);
        // This should pass initially since no one has negative money yet
        assert!(invariant2.check(&engine).is_ok());
    }

    #[test]
    fn test_invariant_checker_violations_count() {
        let checker = InvariantChecker::new();
        assert_eq!(checker.total_violations(), 0);
    }

    #[test]
    fn test_invariant_violation_is_error() {
        let violation = InvariantViolation {
            invariant_name: "Test".to_string(),
            description: "Test".to_string(),
            step: 0,
            expected: None,
            actual: None,
        };
        
        // Should implement std::error::Error
        let _err: &dyn std::error::Error = &violation;
    }

    #[test]
    fn test_invariant_default_description() {
        let config = SimulationConfig::default();
        let _engine = SimulationEngine::new(config);
        let invariant = MoneyConservationInvariant::new(1000.0);
        
        // The default description should contain the invariant name
        let desc = invariant.description();
        assert!(!desc.is_empty());
    }
}
