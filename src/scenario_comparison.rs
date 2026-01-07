/// Scenario comparison functionality for A/B testing of economic policies
///
/// This module provides automated comparison of different simulation scenarios to understand
/// how different pricing mechanisms and market behaviors affect outcomes. It enables researchers
/// to perform rigorous A/B testing and identify the most effective policies.
use crate::error::{Result, SimulationError};
use crate::result::{calculate_statistics, MonteCarloStats, SimulationResult};
use crate::scenario::Scenario;
use crate::{SimulationConfig, SimulationEngine};
use colored::Colorize;
use log::info;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;

/// Result from a single scenario in a comparison
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioComparisonPoint {
    /// The scenario being tested
    pub scenario: Scenario,
    /// Results from all runs with this scenario
    pub results: Vec<SimulationResult>,
    /// Aggregated statistics across runs
    pub avg_money_stats: MonteCarloStats,
    pub gini_coefficient_stats: MonteCarloStats,
    pub total_trades_stats: MonteCarloStats,
    pub avg_reputation_stats: MonteCarloStats,
}

/// Complete results from a scenario comparison analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioComparisonResult {
    /// Scenarios being compared
    pub scenarios: Vec<Scenario>,
    /// Number of runs per scenario
    pub runs_per_scenario: usize,
    /// Base seed used (each run uses seed + offset)
    pub base_seed: u64,
    /// Results for each scenario tested
    pub comparison_points: Vec<ScenarioComparisonPoint>,
    /// Total number of simulations run
    pub total_simulations: usize,
    /// Winner based on different criteria
    pub winners: ComparisonWinners,
}

/// Winners across different comparison criteria
#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonWinners {
    /// Scenario with highest average wealth
    pub highest_avg_wealth: Scenario,
    /// Scenario with lowest wealth inequality (Gini coefficient)
    pub lowest_inequality: Scenario,
    /// Scenario with highest trade volume
    pub highest_trade_volume: Scenario,
    /// Scenario with highest average reputation
    pub highest_reputation: Scenario,
}

impl ScenarioComparisonResult {
    /// Create a new ScenarioComparisonResult by running simulations across multiple scenarios
    pub fn run(
        base_config: SimulationConfig,
        scenarios: Vec<Scenario>,
        runs_per_scenario: usize,
    ) -> Result<Self> {
        if scenarios.is_empty() {
            return Err(SimulationError::ValidationError(
                "At least one scenario must be provided for comparison".to_string(),
            ));
        }

        if scenarios.len() < 2 {
            return Err(SimulationError::ValidationError(
                "At least two different scenarios must be provided for comparison".to_string(),
            ));
        }

        if runs_per_scenario < 1 {
            return Err(SimulationError::ValidationError(
                "Runs per scenario must be at least 1".to_string(),
            ));
        }

        info!(
            "{}",
            format!(
                "Starting scenario comparison: {} scenarios × {} runs = {} total simulations",
                scenarios.len(),
                runs_per_scenario,
                scenarios.len() * runs_per_scenario
            )
            .bright_cyan()
        );

        let base_seed = base_config.seed;
        let total_simulations = scenarios.len() * runs_per_scenario;

        // Run simulations for each scenario in parallel
        let comparison_points: Vec<_> = scenarios
            .par_iter()
            .map(|scenario| {
                info!(
                    "{}",
                    format!(
                        "Running {} runs for scenario: {:?}",
                        runs_per_scenario, scenario
                    )
                    .bright_yellow()
                );

                // Run multiple simulations with different seeds for this scenario
                let results: Vec<_> = (0..runs_per_scenario)
                    .into_par_iter()
                    .map(|run_idx| {
                        let mut config = base_config.clone();
                        config.scenario = scenario.clone();
                        config.seed = base_seed + run_idx as u64;

                        let mut engine = SimulationEngine::new(config);
                        engine.run_with_progress(false)
                    })
                    .collect();

                // Calculate aggregate statistics for this scenario
                let avg_money_values: Vec<f64> =
                    results.iter().map(|r| r.money_statistics.average).collect();
                let avg_money_stats = calculate_statistics(&avg_money_values);

                let gini_values: Vec<f64> = results
                    .iter()
                    .map(|r| r.money_statistics.gini_coefficient)
                    .collect();
                let gini_coefficient_stats = calculate_statistics(&gini_values);

                let total_trades_values: Vec<f64> = results
                    .iter()
                    .map(|r| r.trade_volume_statistics.total_trades as f64)
                    .collect();
                let total_trades_stats = calculate_statistics(&total_trades_values);

                let avg_reputation_values: Vec<f64> = results
                    .iter()
                    .map(|r| r.reputation_statistics.average)
                    .collect();
                let avg_reputation_stats = calculate_statistics(&avg_reputation_values);

                ScenarioComparisonPoint {
                    scenario: scenario.clone(),
                    results,
                    avg_money_stats,
                    gini_coefficient_stats,
                    total_trades_stats,
                    avg_reputation_stats,
                }
            })
            .collect();

        // Determine winners based on different criteria
        let winners = Self::determine_winners(&comparison_points);

        info!(
            "{}",
            "Scenario comparison completed successfully!".bright_green()
        );

        Ok(ScenarioComparisonResult {
            scenarios,
            runs_per_scenario,
            base_seed,
            comparison_points,
            total_simulations,
            winners,
        })
    }

    /// Determine the winning scenario for each criterion
    fn determine_winners(points: &[ScenarioComparisonPoint]) -> ComparisonWinners {
        let highest_avg_wealth = points
            .iter()
            .max_by(|a, b| {
                a.avg_money_stats
                    .mean
                    .partial_cmp(&b.avg_money_stats.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.scenario.clone())
            .unwrap_or_default();

        let lowest_inequality = points
            .iter()
            .min_by(|a, b| {
                a.gini_coefficient_stats
                    .mean
                    .partial_cmp(&b.gini_coefficient_stats.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.scenario.clone())
            .unwrap_or_default();

        let highest_trade_volume = points
            .iter()
            .max_by(|a, b| {
                a.total_trades_stats
                    .mean
                    .partial_cmp(&b.total_trades_stats.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.scenario.clone())
            .unwrap_or_default();

        let highest_reputation = points
            .iter()
            .max_by(|a, b| {
                a.avg_reputation_stats
                    .mean
                    .partial_cmp(&b.avg_reputation_stats.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.scenario.clone())
            .unwrap_or_default();

        ComparisonWinners {
            highest_avg_wealth,
            lowest_inequality,
            highest_trade_volume,
            highest_reputation,
        }
    }

    /// Save the comparison results to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let file = File::create(path).map_err(SimulationError::from)?;

        serde_json::to_writer_pretty(file, self).map_err(|e| {
            SimulationError::JsonSerialize(format!(
                "Failed to write comparison results to {}: {}",
                path, e
            ))
        })?;

        Ok(())
    }

    /// Print a summary of the comparison to the console
    pub fn print_summary(&self) {
        println!(
            "\n{}",
            "=== Scenario Comparison Summary ===".bright_cyan().bold()
        );
        println!(
            "Total simulations: {} ({} scenarios × {} runs)",
            self.total_simulations,
            self.scenarios.len(),
            self.runs_per_scenario
        );
        println!();

        println!("{}", "Results by Scenario:".bright_yellow());
        for point in &self.comparison_points {
            println!("\n  {} {:?}", "Scenario:".bright_green(), point.scenario);
            println!(
                "    Avg Money:       {:.2} ± {:.2} (min: {:.2}, max: {:.2})",
                point.avg_money_stats.mean,
                point.avg_money_stats.std_dev,
                point.avg_money_stats.min,
                point.avg_money_stats.max
            );
            println!(
                "    Gini Coeff:      {:.4} ± {:.4} (min: {:.4}, max: {:.4})",
                point.gini_coefficient_stats.mean,
                point.gini_coefficient_stats.std_dev,
                point.gini_coefficient_stats.min,
                point.gini_coefficient_stats.max
            );
            println!(
                "    Total Trades:    {:.0} ± {:.0} (min: {:.0}, max: {:.0})",
                point.total_trades_stats.mean,
                point.total_trades_stats.std_dev,
                point.total_trades_stats.min,
                point.total_trades_stats.max
            );
            println!(
                "    Avg Reputation:  {:.4} ± {:.4} (min: {:.4}, max: {:.4})",
                point.avg_reputation_stats.mean,
                point.avg_reputation_stats.std_dev,
                point.avg_reputation_stats.min,
                point.avg_reputation_stats.max
            );
        }

        println!("\n{}", "Winners by Criterion:".bright_magenta().bold());
        println!(
            "  Highest Avg Wealth:   {:?}",
            self.winners.highest_avg_wealth
        );
        println!(
            "  Lowest Inequality:    {:?}",
            self.winners.lowest_inequality
        );
        println!(
            "  Highest Trade Volume: {:?}",
            self.winners.highest_trade_volume
        );
        println!(
            "  Highest Reputation:   {:?}",
            self.winners.highest_reputation
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;

    #[test]
    fn test_scenario_comparison_run() {
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
            loan_repayment_period: 10,
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

        let scenarios = vec![Scenario::Original, Scenario::DynamicPricing];
        let result = ScenarioComparisonResult::run(config, scenarios, 2);

        assert!(result.is_ok());
        let comparison = result.unwrap();
        assert_eq!(comparison.scenarios.len(), 2);
        assert_eq!(comparison.runs_per_scenario, 2);
        assert_eq!(comparison.total_simulations, 4);
        assert_eq!(comparison.comparison_points.len(), 2);
    }

    #[test]
    fn test_scenario_comparison_empty_scenarios() {
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
            loan_repayment_period: 10,
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

        let scenarios = vec![];
        let result = ScenarioComparisonResult::run(config, scenarios, 2);

        assert!(result.is_err());
    }

    #[test]
    fn test_scenario_comparison_zero_runs() {
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
            loan_repayment_period: 10,
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

        let scenarios = vec![Scenario::Original];
        let result = ScenarioComparisonResult::run(config, scenarios, 0);

        assert!(result.is_err());
    }

    #[test]
    fn test_scenario_comparison_single_scenario() {
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
            loan_repayment_period: 10,
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

        let scenarios = vec![Scenario::Original];
        let result = ScenarioComparisonResult::run(config, scenarios, 2);

        assert!(result.is_err());
    }
}
