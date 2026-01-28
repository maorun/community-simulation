/// Parameter sweep functionality for sensitivity analysis
///
/// This module provides automated parameter sweeping (grid search) to understand
/// how different parameter values affect simulation outcomes. It enables researchers
/// to systematically explore the parameter space and identify robust configurations.
use crate::error::{Result, SimulationError};
use crate::result::{calculate_statistics, MonteCarloStats, SimulationResult};
use crate::{SimulationConfig, SimulationEngine};
use colored::Colorize;
use log::info;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

/// Specification for a parameter sweep over a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterRange {
    /// Sweep over initial money per person values
    InitialMoney { min: f64, max: f64, steps: usize },
    /// Sweep over base skill price values
    BasePrice { min: f64, max: f64, steps: usize },
    /// Sweep over savings rate values
    SavingsRate { min: f64, max: f64, steps: usize },
    /// Sweep over transaction fee values
    TransactionFee { min: f64, max: f64, steps: usize },
}

impl ParameterRange {
    /// Get the parameter name for display
    pub fn name(&self) -> &str {
        match self {
            ParameterRange::InitialMoney { .. } => "initial_money",
            ParameterRange::BasePrice { .. } => "base_price",
            ParameterRange::SavingsRate { .. } => "savings_rate",
            ParameterRange::TransactionFee { .. } => "transaction_fee",
        }
    }

    /// Generate the list of values to test for this parameter
    pub fn values(&self) -> Vec<f64> {
        match self {
            ParameterRange::InitialMoney { min, max, steps }
            | ParameterRange::BasePrice { min, max, steps }
            | ParameterRange::SavingsRate { min, max, steps }
            | ParameterRange::TransactionFee { min, max, steps } => {
                if *steps <= 1 {
                    vec![*min]
                } else {
                    let step_size = (max - min) / (*steps - 1) as f64;
                    (0..*steps).map(|i| min + i as f64 * step_size).collect()
                }
            },
        }
    }

    /// Apply this parameter value to a configuration
    pub fn apply_to_config(&self, config: &mut SimulationConfig, value: f64) {
        match self {
            ParameterRange::InitialMoney { .. } => config.initial_money_per_person = value,
            ParameterRange::BasePrice { .. } => config.base_skill_price = value,
            ParameterRange::SavingsRate { .. } => config.savings_rate = value,
            ParameterRange::TransactionFee { .. } => config.transaction_fee = value,
        }
    }
}

/// Result from a single parameter configuration in a sweep
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterSweepPoint {
    /// The parameter value tested
    pub parameter_value: f64,
    /// The parameter name
    pub parameter_name: String,
    /// Results from all runs at this parameter value
    pub results: Vec<SimulationResult>,
    /// Aggregated statistics across runs
    pub avg_money_stats: MonteCarloStats,
    pub gini_coefficient_stats: MonteCarloStats,
    pub total_trades_stats: MonteCarloStats,
    pub avg_reputation_stats: MonteCarloStats,
}

/// Complete results from a parameter sweep analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterSweepResult {
    /// Name of the parameter being swept
    pub parameter_name: String,
    /// Number of runs per parameter value
    pub runs_per_point: usize,
    /// Base seed used (each run uses seed + offset)
    pub base_seed: u64,
    /// Results for each parameter value tested
    pub sweep_points: Vec<ParameterSweepPoint>,
    /// Total number of simulations run
    pub total_simulations: usize,
}

impl ParameterSweepResult {
    /// Create a new ParameterSweepResult by running simulations across a parameter range
    pub fn run_sweep(
        base_config: SimulationConfig,
        parameter_range: ParameterRange,
        runs_per_point: usize,
        _show_progress: bool,
    ) -> Self {
        let parameter_name = parameter_range.name().to_string();
        let values = parameter_range.values();
        let base_seed = base_config.seed;

        info!(
            "Starting parameter sweep: {} with {} values, {} runs each",
            parameter_name,
            values.len(),
            runs_per_point
        );

        // Run simulations for each parameter value in parallel
        let sweep_points: Vec<ParameterSweepPoint> = values
            .par_iter()
            .enumerate()
            .map(|(idx, &value)| {
                info!(
                    "Testing {} = {:.4} ({}/{})...",
                    parameter_name,
                    value,
                    idx + 1,
                    values.len()
                );

                // Create configuration with this parameter value
                let mut config = base_config.clone();
                parameter_range.apply_to_config(&mut config, value);

                // Run multiple simulations with different seeds
                let results: Vec<SimulationResult> = (0..runs_per_point)
                    .into_par_iter()
                    .map(|run_idx| {
                        let mut run_config = config.clone();
                        run_config.seed = base_seed + (idx * runs_per_point + run_idx) as u64;

                        let mut engine = SimulationEngine::new(run_config);
                        // Disable progress bar for individual runs in parameter sweep
                        engine.run_with_progress(false)
                    })
                    .collect();

                // Calculate aggregated statistics
                let avg_moneys: Vec<f64> =
                    results.iter().map(|r| r.money_statistics.average).collect();
                let gini_coefficients: Vec<f64> =
                    results.iter().map(|r| r.money_statistics.gini_coefficient).collect();
                let total_trades: Vec<f64> =
                    results.iter().map(|r| r.trade_volume_statistics.total_trades as f64).collect();
                let avg_reputations: Vec<f64> =
                    results.iter().map(|r| r.reputation_statistics.average).collect();

                info!(
                    "Completed {} = {:.4} (avg money: {:.2})",
                    parameter_name,
                    value,
                    avg_moneys.iter().sum::<f64>() / avg_moneys.len() as f64
                );

                ParameterSweepPoint {
                    parameter_value: value,
                    parameter_name: parameter_name.clone(),
                    results,
                    avg_money_stats: calculate_statistics(&avg_moneys),
                    gini_coefficient_stats: calculate_statistics(&gini_coefficients),
                    total_trades_stats: calculate_statistics(&total_trades),
                    avg_reputation_stats: calculate_statistics(&avg_reputations),
                }
            })
            .collect();

        let total_simulations = values.len() * runs_per_point;

        Self { parameter_name, runs_per_point, base_seed, sweep_points, total_simulations }
    }

    /// Save parameter sweep results to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;

        let mut file = File::create(path)?;
        file.write_all(json_str.as_bytes())?;

        Ok(())
    }

    /// Print a summary of the parameter sweep results to the console
    pub fn print_summary(&self) {
        println!("\n{}", "=== Parameter Sweep Results ===".bright_cyan().bold());
        println!("Parameter: {}", self.parameter_name);
        println!("Runs per point: {}", self.runs_per_point);
        println!("Total simulations: {}", self.total_simulations);
        println!("Base seed: {}\n", self.base_seed);

        println!("{}", "Results by Parameter Value:".bright_yellow());
        println!(
            "{:<15} {:<15} {:<15} {:<15} {:<15}",
            "Value", "Avg Money", "Gini Coeff", "Total Trades", "Avg Reputation"
        );
        println!("{}", "-".repeat(75));

        for point in &self.sweep_points {
            println!(
                "{:<15.4} {:<15.2} {:<15.4} {:<15.0} {:<15.4}",
                point.parameter_value,
                point.avg_money_stats.mean,
                point.gini_coefficient_stats.mean,
                point.total_trades_stats.mean,
                point.avg_reputation_stats.mean,
            );
        }

        // Find optimal parameter values based on different criteria
        println!("\n{}", "Optimal Parameter Values:".bright_green());

        // Highest average money
        if let Some(max_money_point) = self
            .sweep_points
            .iter()
            .max_by(|a, b| a.avg_money_stats.mean.partial_cmp(&b.avg_money_stats.mean).unwrap())
        {
            println!(
                "  Highest avg money:  {} = {:.4} (${:.2})",
                self.parameter_name,
                max_money_point.parameter_value,
                max_money_point.avg_money_stats.mean
            );
        }

        // Lowest inequality (Gini coefficient)
        if let Some(min_gini_point) = self.sweep_points.iter().min_by(|a, b| {
            a.gini_coefficient_stats
                .mean
                .partial_cmp(&b.gini_coefficient_stats.mean)
                .unwrap()
        }) {
            println!(
                "  Lowest inequality:  {} = {:.4} (Gini: {:.4})",
                self.parameter_name,
                min_gini_point.parameter_value,
                min_gini_point.gini_coefficient_stats.mean
            );
        }

        // Highest trade volume
        if let Some(max_trades_point) = self.sweep_points.iter().max_by(|a, b| {
            a.total_trades_stats.mean.partial_cmp(&b.total_trades_stats.mean).unwrap()
        }) {
            println!(
                "  Highest trade volume: {} = {:.4} ({:.0} trades)",
                self.parameter_name,
                max_trades_point.parameter_value,
                max_trades_point.total_trades_stats.mean
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;

    #[test]
    fn test_parameter_range_values_generation() {
        let range = ParameterRange::InitialMoney { min: 50.0, max: 150.0, steps: 3 };

        let values = range.values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 50.0);
        assert_eq!(values[1], 100.0);
        assert_eq!(values[2], 150.0);
    }

    #[test]
    fn test_parameter_range_single_step() {
        let range = ParameterRange::BasePrice { min: 10.0, max: 20.0, steps: 1 };

        let values = range.values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 10.0);
    }

    #[test]
    fn test_parameter_range_apply_to_config() {
        let mut config = SimulationConfig::default();

        let range = ParameterRange::InitialMoney { min: 50.0, max: 150.0, steps: 2 };
        range.apply_to_config(&mut config, 75.0);
        assert_eq!(config.initial_money_per_person, 75.0);

        let range = ParameterRange::SavingsRate { min: 0.0, max: 0.1, steps: 2 };
        range.apply_to_config(&mut config, 0.05);
        assert_eq!(config.savings_rate, 0.05);
    }

    #[test]
    fn test_parameter_range_names() {
        assert_eq!(
            ParameterRange::InitialMoney { min: 0.0, max: 100.0, steps: 2 }.name(),
            "initial_money"
        );
        assert_eq!(
            ParameterRange::BasePrice { min: 0.0, max: 100.0, steps: 2 }.name(),
            "base_price"
        );
        assert_eq!(
            ParameterRange::SavingsRate { min: 0.0, max: 0.1, steps: 2 }.name(),
            "savings_rate"
        );
        assert_eq!(
            ParameterRange::TransactionFee { min: 0.0, max: 0.1, steps: 2 }.name(),
            "transaction_fee"
        );
    }

    #[test]
    fn test_calculate_stats_empty() {
        let stats = calculate_statistics(&[]);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.median, 0.0);
        assert_eq!(stats.std_dev, 0.0);
    }

    #[test]
    fn test_calculate_stats_single_value() {
        let stats = calculate_statistics(&[42.0]);
        assert_eq!(stats.mean, 42.0);
        assert_eq!(stats.median, 42.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min, 42.0);
        assert_eq!(stats.max, 42.0);
    }

    #[test]
    fn test_calculate_stats_multiple_values() {
        let stats = calculate_statistics(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert!((stats.std_dev - 1.5811).abs() < 0.01); // Sample std dev
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_parameter_sweep_run_small() {
        // Small test with minimal parameters to verify the sweep logic
        let config = SimulationConfig {
            max_steps: 10,
            entity_count: 5,
            seed: 42,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            min_skill_price: 1.0,
            time_step: 1.0,
            scenario: Scenario::Original,
            tech_growth_rate: 0.0,
            seasonal_amplitude: 0.0,
            seasonal_period: 100,
            transaction_fee: 0.0,
            savings_rate: 0.0,
            enable_loans: false,
            loan_interest_rate: 0.01,
            loan_repayment_period: 20,
            min_money_to_lend: 50.0,
            checkpoint_interval: 0,
            checkpoint_file: None,
            resume_from_checkpoint: false,
            tax_rate: 0.0,
            enable_tax_redistribution: false,
            skills_per_person: 1,
            stream_output_path: None,
            priority_urgency_weight: 0.5,
            priority_affordability_weight: 0.3,
            priority_efficiency_weight: 0.1,
            priority_reputation_weight: 0.1,
            ..Default::default()
        };

        let parameter_range = ParameterRange::InitialMoney { min: 80.0, max: 120.0, steps: 2 };

        let result = ParameterSweepResult::run_sweep(config, parameter_range, 2, false);

        assert_eq!(result.parameter_name, "initial_money");
        assert_eq!(result.runs_per_point, 2);
        assert_eq!(result.sweep_points.len(), 2);
        assert_eq!(result.total_simulations, 4); // 2 points * 2 runs

        // Verify parameter values
        assert_eq!(result.sweep_points[0].parameter_value, 80.0);
        assert_eq!(result.sweep_points[1].parameter_value, 120.0);

        // Verify each point has correct number of results
        assert_eq!(result.sweep_points[0].results.len(), 2);
        assert_eq!(result.sweep_points[1].results.len(), 2);
    }
}
