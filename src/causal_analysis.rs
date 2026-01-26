/// Causal Analysis Framework for rigorous policy evaluation
///
/// This module provides tools for causal inference in economic simulations,
/// enabling researchers to evaluate policy interventions and mechanism designs
/// with statistical rigor. It supports A/B testing with treatment and control groups,
/// calculating effect sizes, confidence intervals, and statistical significance.
///
/// # Example
///
/// ```no_run
/// use simulation_framework::causal_analysis::*;
/// use simulation_framework::{SimulationConfig, SimulationEngine};
///
/// // Run treatment group simulations
/// let mut treatment_config = SimulationConfig::default();
/// treatment_config.savings_rate = 0.2; // 20% savings rate
///
/// let mut treatment_results = vec![];
/// for seed in 0..10 {
///     treatment_config.seed = seed;
///     let mut engine = SimulationEngine::new(treatment_config.clone());
///     treatment_results.push(engine.run());
/// }
///
/// // Run control group simulations
/// let mut control_config = SimulationConfig::default();
/// control_config.savings_rate = 0.0; // No savings
///
/// let mut control_results = vec![];
/// for seed in 0..10 {
///     control_config.seed = seed;
///     let mut engine = SimulationEngine::new(control_config.clone());
///     control_results.push(engine.run());
/// }
///
/// // Perform causal analysis
/// let config = CausalAnalysisConfig {
///     treatment_name: "20% Savings Rate".to_string(),
///     control_name: "No Savings".to_string(),
///     ..Default::default()
/// };
///
/// let analysis = CausalAnalysisResult::analyze(&treatment_results, &control_results, config).unwrap();
/// analysis.print_summary();
/// ```
use crate::error::{Result, SimulationError};
use crate::result::SimulationResult;
use colored::Colorize;
use serde::{Deserialize, Serialize};

/// Configuration for causal analysis experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAnalysisConfig {
    /// Name of the treatment (intervention) being tested
    pub treatment_name: String,

    /// Name of the control (baseline) condition
    pub control_name: String,

    /// Confidence level for statistical tests (e.g., 0.95 for 95%)
    #[serde(default = "default_confidence_level")]
    pub confidence_level: f64,

    /// Number of bootstrap samples for confidence intervals
    #[serde(default = "default_bootstrap_samples")]
    pub bootstrap_samples: usize,
}

fn default_confidence_level() -> f64 {
    0.95
}

fn default_bootstrap_samples() -> usize {
    1000
}

impl Default for CausalAnalysisConfig {
    fn default() -> Self {
        Self {
            treatment_name: "Treatment".to_string(),
            control_name: "Control".to_string(),
            confidence_level: 0.95,
            bootstrap_samples: 1000,
        }
    }
}

/// Statistical test result for a single metric comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTest {
    /// Name of the metric being tested
    pub metric_name: String,

    /// Mean value in treatment group
    pub treatment_mean: f64,

    /// Mean value in control group
    pub control_mean: f64,

    /// Absolute effect size (treatment - control)
    pub effect_size: f64,

    /// Relative effect size ((treatment - control) / control)
    pub relative_effect: f64,

    /// Standard error of the difference
    pub standard_error: f64,

    /// T-statistic
    pub t_statistic: f64,

    /// Degrees of freedom
    pub degrees_of_freedom: usize,

    /// Two-tailed p-value
    pub p_value: f64,

    /// Lower bound of confidence interval
    pub ci_lower: f64,

    /// Upper bound of confidence interval
    pub ci_upper: f64,

    /// Whether the result is statistically significant
    pub is_significant: bool,
}

/// Complete results from a causal analysis experiment
#[derive(Debug, Serialize, Deserialize)]
pub struct CausalAnalysisResult {
    /// Configuration used for this analysis
    pub config: CausalAnalysisConfig,

    /// Number of simulations in treatment group
    pub treatment_n: usize,

    /// Number of simulations in control group
    pub control_n: usize,

    /// Statistical tests for each metric
    pub tests: Vec<StatisticalTest>,

    /// Overall assessment
    pub summary: String,
}

impl CausalAnalysisResult {
    /// Perform causal analysis comparing treatment and control groups
    ///
    /// # Arguments
    ///
    /// * `treatment` - Simulation results from treatment group
    /// * `control` - Simulation results from control group
    /// * `config` - Configuration for the analysis
    ///
    /// # Returns
    ///
    /// A CausalAnalysisResult containing statistical comparisons
    pub fn analyze(
        treatment: &[SimulationResult],
        control: &[SimulationResult],
        config: CausalAnalysisConfig,
    ) -> Result<Self> {
        if treatment.is_empty() {
            return Err(SimulationError::ValidationError(
                "Treatment group cannot be empty".to_string(),
            ));
        }
        if control.is_empty() {
            return Err(SimulationError::ValidationError(
                "Control group cannot be empty".to_string(),
            ));
        }

        let tests = vec![
            // Compare average money
            Self::compare_metric(
                "Average Money",
                treatment
                    .iter()
                    .map(|r| r.money_statistics.average)
                    .collect::<Vec<_>>()
                    .as_slice(),
                control
                    .iter()
                    .map(|r| r.money_statistics.average)
                    .collect::<Vec<_>>()
                    .as_slice(),
                config.confidence_level,
            ),
            // Compare Gini coefficient
            Self::compare_metric(
                "Gini Coefficient",
                treatment
                    .iter()
                    .map(|r| r.money_statistics.gini_coefficient)
                    .collect::<Vec<_>>()
                    .as_slice(),
                control
                    .iter()
                    .map(|r| r.money_statistics.gini_coefficient)
                    .collect::<Vec<_>>()
                    .as_slice(),
                config.confidence_level,
            ),
            // Compare total trades
            Self::compare_metric(
                "Total Trades",
                treatment
                    .iter()
                    .map(|r| r.trade_volume_statistics.total_trades as f64)
                    .collect::<Vec<_>>()
                    .as_slice(),
                control
                    .iter()
                    .map(|r| r.trade_volume_statistics.total_trades as f64)
                    .collect::<Vec<_>>()
                    .as_slice(),
                config.confidence_level,
            ),
            // Compare average reputation
            Self::compare_metric(
                "Average Reputation",
                treatment
                    .iter()
                    .map(|r| r.reputation_statistics.average)
                    .collect::<Vec<_>>()
                    .as_slice(),
                control
                    .iter()
                    .map(|r| r.reputation_statistics.average)
                    .collect::<Vec<_>>()
                    .as_slice(),
                config.confidence_level,
            ),
        ];

        // Generate summary
        let significant_tests = tests.iter().filter(|t| t.is_significant).count();
        let summary = format!(
            "{} out of {} metrics show statistically significant differences at {}% confidence level",
            significant_tests,
            tests.len(),
            (config.confidence_level * 100.0) as usize
        );

        Ok(CausalAnalysisResult {
            config,
            treatment_n: treatment.len(),
            control_n: control.len(),
            tests,
            summary,
        })
    }

    /// Compare a single metric between two groups using Welch's t-test
    fn compare_metric(
        name: &str,
        treatment: &[f64],
        control: &[f64],
        confidence_level: f64,
    ) -> StatisticalTest {
        let treatment_mean = mean(treatment);
        let control_mean = mean(control);
        let treatment_var = variance(treatment, treatment_mean);
        let control_var = variance(control, control_mean);

        let n1 = treatment.len() as f64;
        let n2 = control.len() as f64;

        // Welch's t-test (unequal variances)
        let standard_error = ((treatment_var / n1) + (control_var / n2)).sqrt();
        let effect_size = treatment_mean - control_mean;
        let relative_effect = if control_mean.abs() > 1e-10 {
            effect_size / control_mean
        } else {
            0.0
        };

        let t_statistic = if standard_error > 1e-10 {
            effect_size / standard_error
        } else {
            0.0
        };

        // Welch-Satterthwaite degrees of freedom
        let df = if treatment_var > 1e-10 && control_var > 1e-10 {
            let numerator = ((treatment_var / n1) + (control_var / n2)).powi(2);
            let denominator = ((treatment_var / n1).powi(2) / (n1 - 1.0))
                + ((control_var / n2).powi(2) / (n2 - 1.0));
            (numerator / denominator).floor() as usize
        } else {
            (n1 + n2 - 2.0) as usize
        };

        // Simple p-value approximation using normal distribution
        // For large samples, t-distribution ≈ normal distribution
        let p_value = 2.0 * (1.0 - normal_cdf(t_statistic.abs()));

        // Confidence interval
        let critical_value = inverse_normal_cdf(1.0 - (1.0 - confidence_level) / 2.0);
        let margin_of_error = critical_value * standard_error;
        let ci_lower = effect_size - margin_of_error;
        let ci_upper = effect_size + margin_of_error;

        let alpha = 1.0 - confidence_level;
        let is_significant = p_value < alpha;

        StatisticalTest {
            metric_name: name.to_string(),
            treatment_mean,
            control_mean,
            effect_size,
            relative_effect,
            standard_error,
            t_statistic,
            degrees_of_freedom: df,
            p_value,
            ci_lower,
            ci_upper,
            is_significant,
        }
    }

    /// Print a human-readable summary of the causal analysis
    pub fn print_summary(&self) {
        println!("\n{}", "=== Causal Analysis Results ===".bold());
        println!(
            "Treatment: {} (n={})",
            self.config.treatment_name, self.treatment_n
        );
        println!(
            "Control: {} (n={})",
            self.config.control_name, self.control_n
        );
        println!(
            "Confidence Level: {}%",
            (self.config.confidence_level * 100.0) as usize
        );
        println!();

        for test in &self.tests {
            println!("{}", test.metric_name.bold());
            println!("  Treatment Mean: {:.4}", test.treatment_mean);
            println!("  Control Mean:   {:.4}", test.control_mean);
            println!(
                "  Effect Size:    {:.4} ({:+.2}%)",
                test.effect_size,
                test.relative_effect * 100.0
            );
            println!(
                "  95% CI:         [{:.4}, {:.4}]",
                test.ci_lower, test.ci_upper
            );
            println!("  t-statistic:    {:.4}", test.t_statistic);
            println!("  p-value:        {:.4}", test.p_value);
            println!(
                "  Significant:    {}",
                if test.is_significant { "YES" } else { "NO" }
            );
            println!();
        }

        println!("{}", self.summary);
    }

    /// Save results to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            SimulationError::ValidationError(format!("JSON serialization failed: {}", e))
        })?;
        std::fs::write(path, json).map_err(SimulationError::from)?;
        Ok(())
    }
}

// Helper statistical functions

/// Calculate mean of a slice
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate sample variance
fn variance(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let sum_squared_diff: f64 = values.iter().map(|x| (x - mean).powi(2)).sum();
    sum_squared_diff / (values.len() - 1) as f64
}

/// Cumulative distribution function of standard normal distribution
/// Using approximation from Abramowitz and Stegun
fn normal_cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989423 * (-x * x / 2.0).exp();
    let prob =
        d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

    if x >= 0.0 {
        1.0 - prob
    } else {
        prob
    }
}

/// Inverse CDF of standard normal distribution
/// Using Beasley-Springer-Moro algorithm approximation
fn inverse_normal_cdf(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // Use rational approximation for central region
    if p > 0.02425 && p < 0.97575 {
        let q = p - 0.5;
        let r = q * q;
        return q
            * ((((-25.44106049637 * r + 41.39119773534) * r - 18.61500062529) * r
                + 2.50662823884)
                / ((((3.13082909833 * r - 21.06224101826) * r + 23.08336743743) * r
                    - 8.47351093090)
                    * r
                    + 1.0));
    }

    // Use tail approximation
    let q = if p < 0.5 { p } else { 1.0 - p };
    let r = (-q.ln()).sqrt();
    let val = (((2.32121276858 * r + 4.85014127135) * r - 2.29796479134)
        / ((1.63706781897 * r + 3.54388924762) * r + 1.0))
        - r;

    if p < 0.5 {
        -val
    } else {
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(mean(&[]), 0.0);
        assert_eq!(mean(&[42.0]), 42.0);
    }

    #[test]
    fn test_variance() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let m = mean(&values);
        let v = variance(&values, m);
        // Sample variance of 1,2,3,4,5 is 2.5
        assert!((v - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_normal_cdf() {
        // Test some known values
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_cdf(1.96) - 0.975).abs() < 1e-3);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 1e-3);
    }

    #[test]
    fn test_inverse_normal_cdf() {
        // Test some known values
        assert!((inverse_normal_cdf(0.5) - 0.0).abs() < 1e-6);
        assert!((inverse_normal_cdf(0.975) - 1.96).abs() < 0.01);
        assert!((inverse_normal_cdf(0.025) + 1.96).abs() < 0.01);
    }
}
