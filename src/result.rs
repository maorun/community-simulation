use crate::{Entity, SkillId}; // Entity now wraps Person
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyStats {
    pub average: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min_money: f64,
    pub max_money: f64,
    /// Gini coefficient: measure of wealth inequality (0 = perfect equality, 1 = perfect inequality)
    pub gini_coefficient: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReputationStats {
    pub average: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min_reputation: f64,
    pub max_reputation: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillPriceInfo {
    pub id: SkillId,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    // Core simulation metrics
    pub total_steps: usize,
    pub total_duration: f64,
    pub step_times: Vec<f64>,  // Time taken for each step
    pub active_persons: usize, // Renamed from active_entities for clarity

    // Economic output
    pub final_money_distribution: Vec<f64>, // List of money amounts per person
    pub money_statistics: MoneyStats,

    // Reputation metrics
    pub final_reputation_distribution: Vec<f64>, // List of reputation scores per person
    pub reputation_statistics: ReputationStats,

    pub final_skill_prices: Vec<SkillPriceInfo>, // Sorted by price
    pub most_valuable_skill: Option<SkillPriceInfo>,
    pub least_valuable_skill: Option<SkillPriceInfo>,

    // For graphical representation of price development over time
    // Key: SkillId, Value: Vec of prices, one entry per step
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub skill_price_history: HashMap<SkillId, Vec<f64>>,

    // final_entities might be too verbose if Person struct grows large with transaction history.
    // Consider summarizing person data if needed, or providing it under a flag.
    // For now, let's keep it as it contains all person data including transaction history.
    pub final_persons_data: Vec<Entity>, // Renamed from final_entities
}

impl SimulationResult {
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn print_summary(&self) {
        println!("\n=== Economic Simulation Summary ===");
        println!("Total steps: {}", self.total_steps);
        println!("Total duration: {:.2}s", self.total_duration);
        if !self.step_times.is_empty() {
            let avg_step_time_ms =
                self.step_times.iter().sum::<f64>() / self.step_times.len() as f64 * 1000.0;
            println!("Average step time: {:.4}ms", avg_step_time_ms);
        }
        println!("Active persons remaining: {}", self.active_persons);
        let performance = if self.total_duration > 0.0 {
            self.total_steps as f64 / self.total_duration
        } else {
            0.0
        };
        println!("Performance: {:.0} steps/second", performance);

        println!("\n--- Money Distribution ---");
        println!("Average Money: {:.2}", self.money_statistics.average);
        println!("Median Money: {:.2}", self.money_statistics.median);
        println!("Std Dev Money: {:.2}", self.money_statistics.std_dev);
        println!(
            "Min/Max Money: {:.2} / {:.2}",
            self.money_statistics.min_money, self.money_statistics.max_money
        );
        println!(
            "Gini Coefficient: {:.4} (0 = perfect equality, 1 = perfect inequality)",
            self.money_statistics.gini_coefficient
        );

        println!("\n--- Reputation Distribution ---");
        println!(
            "Average Reputation: {:.4}",
            self.reputation_statistics.average
        );
        println!(
            "Median Reputation: {:.4}",
            self.reputation_statistics.median
        );
        println!(
            "Std Dev Reputation: {:.4}",
            self.reputation_statistics.std_dev
        );
        println!(
            "Min/Max Reputation: {:.4} / {:.4}",
            self.reputation_statistics.min_reputation, self.reputation_statistics.max_reputation
        );

        println!("\n--- Skill Valuations ---");
        if let Some(skill) = &self.most_valuable_skill {
            println!(
                "Most Valuable Skill: {} (Price: {:.2})",
                skill.id, skill.price
            );
        }
        if let Some(skill) = &self.least_valuable_skill {
            println!(
                "Least Valuable Skill: {} (Price: {:.2})",
                skill.id, skill.price
            );
        }

        println!("\nTop 5 Most Valuable Skills:");
        for skill_info in self.final_skill_prices.iter().take(5) {
            println!("  - {}: {:.2}", skill_info.id, skill_info.price);
        }

        println!("\nTop 5 Least Valuable Skills (excluding those at min price if many):");
        // Iterate in reverse, but skip if all are min_price
        let mut count = 0;
        for skill_info in self.final_skill_prices.iter().rev().take(10) {
            // Check more than 5 to find some not at min
            if count < 5 {
                // Basic heuristic: if it's significantly above absolute min, show it.
                // This needs better logic if many skills bottom out at min_skill_price.
                // For now, just show them.
                println!("  - {}: {:.2}", skill_info.id, skill_info.price);
                count += 1;
            }
        }
        if self.skill_price_history.keys().len() > 0 {
            println!(
                "\nSkill price history for {} skills available in JSON output.",
                self.skill_price_history.keys().len()
            );
        }
    }
}

/// Calculate the Gini coefficient for a given distribution of values.
///
/// The Gini coefficient is a measure of inequality ranging from 0 (perfect equality)
/// to 1 (perfect inequality). Values above 1 can occur when negative values exist.
///
/// # Arguments
/// * `sorted_values` - A slice of values sorted in ascending order
/// * `sum` - The sum of all values
///
/// # Formula
/// G = (2 * sum(i * x_i)) / (n * sum(x_i)) - (n + 1) / n
/// where x_i are sorted values and i is the rank (1-indexed)
///
/// # Returns
/// The Gini coefficient as f64
pub fn calculate_gini_coefficient(sorted_values: &[f64], sum: f64) -> f64 {
    if sorted_values.is_empty() || sum == 0.0 {
        return 0.0;
    }

    let n = sorted_values.len();
    let weighted_sum: f64 = sorted_values
        .iter()
        .enumerate()
        .map(|(i, &value)| (i + 1) as f64 * value)
        .sum();
    (2.0 * weighted_sum) / (n as f64 * sum) - (n as f64 + 1.0) / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    fn get_test_result() -> SimulationResult {
        SimulationResult {
            total_steps: 10,
            total_duration: 1.23,
            step_times: vec![0.1, 0.12, 0.1, 0.13, 0.1, 0.11, 0.1, 0.14, 0.1, 0.13],
            active_persons: 5,
            final_money_distribution: vec![50.0, 80.0, 100.0, 120.0, 150.0],
            money_statistics: MoneyStats {
                average: 100.0,
                median: 100.0,
                std_dev: 31.62,
                min_money: 50.0,
                max_money: 150.0,
                gini_coefficient: 0.2,
            },
            final_reputation_distribution: vec![0.95, 1.0, 1.0, 1.05, 1.1],
            reputation_statistics: ReputationStats {
                average: 1.02,
                median: 1.0,
                std_dev: 0.05,
                min_reputation: 0.95,
                max_reputation: 1.1,
            },
            final_skill_prices: vec![],
            most_valuable_skill: None,
            least_valuable_skill: None,
            skill_price_history: HashMap::new(),
            final_persons_data: vec![],
        }
    }

    #[test]
    fn test_print_summary() {
        let result = get_test_result();
        // This test just checks that print_summary doesn't panic.
        result.print_summary();
    }

    #[test]
    fn test_save_to_file() {
        let result = get_test_result();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        result.save_to_file(path).unwrap();

        let mut contents = String::new();
        file.reopen()
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
    }

    fn calculate_money_stats(money_values: &[f64]) -> MoneyStats {
        if money_values.is_empty() {
            return MoneyStats {
                average: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min_money: 0.0,
                max_money: 0.0,
                gini_coefficient: 0.0,
            };
        }

        let mut sorted_money = money_values.to_vec();
        sorted_money.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = sorted_money.iter().sum();
        let count = sorted_money.len() as f64;
        let average = sum / count;

        let median = if count > 0.0 {
            if count as usize % 2 == 1 {
                sorted_money[count as usize / 2]
            } else {
                (sorted_money[count as usize / 2 - 1] + sorted_money[count as usize / 2]) / 2.0
            }
        } else {
            0.0
        };

        let variance = sorted_money
            .iter()
            .map(|value| {
                let diff = average - value;
                diff * diff
            })
            .sum::<f64>()
            / count;
        let std_dev = variance.sqrt();

        // Calculate Gini coefficient using the shared utility function
        let gini_coefficient = calculate_gini_coefficient(&sorted_money, sum);

        MoneyStats {
            average,
            median,
            std_dev,
            min_money: *sorted_money.first().unwrap_or(&0.0),
            max_money: *sorted_money.last().unwrap_or(&0.0),
            gini_coefficient,
        }
    }

    #[test]
    fn test_money_stats_empty() {
        let stats = calculate_money_stats(&[]);
        assert_eq!(stats.average, 0.0);
        assert_eq!(stats.median, 0.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min_money, 0.0);
        assert_eq!(stats.max_money, 0.0);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_money_stats_single_value() {
        let stats = calculate_money_stats(&[100.0]);
        assert_eq!(stats.average, 100.0);
        assert_eq!(stats.median, 100.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min_money, 100.0);
        assert_eq!(stats.max_money, 100.0);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_money_stats_multiple_values_odd() {
        let money = [10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.average, 30.0);
        assert_eq!(stats.median, 30.0);
        assert_eq!(stats.min_money, 10.0);
        assert_eq!(stats.max_money, 50.0);
        // Std dev for [10,20,30,40,50] is sqrt(((20^2 + 10^2 + 0^2 + 10^2 + 20^2)/5)) = sqrt((400+100+0+100+400)/5) = sqrt(1000/5) = sqrt(200) = 14.1421356
        assert!((stats.std_dev - 14.1421356).abs() < 1e-6);
    }

    #[test]
    fn test_money_stats_multiple_values_even() {
        let money = [10.0, 20.0, 30.0, 60.0]; // Avg = 30, Median = 25
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.average, 30.0);
        assert_eq!(stats.median, 25.0); // (20+30)/2
        assert_eq!(stats.min_money, 10.0);
        assert_eq!(stats.max_money, 60.0);
        // Std dev for [10,20,30,60] (avg 30) is sqrt(((20^2 + 10^2 + 0^2 + 30^2)/4)) = sqrt((400+100+0+900)/4) = sqrt(1400/4) = sqrt(350) = 18.7082869
        assert!((stats.std_dev - 18.7082869).abs() < 1e-6);
    }

    #[test]
    fn test_gini_coefficient_perfect_equality() {
        // All persons have equal money - should be 0 (perfect equality)
        let money = [100.0, 100.0, 100.0, 100.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_perfect_inequality() {
        // One person has all money, others have nothing - should be close to 1
        let money = [0.0, 0.0, 0.0, 100.0];
        let stats = calculate_money_stats(&money);
        // For n=4, perfect inequality Gini = (n-1)/n = 3/4 = 0.75
        assert!((stats.gini_coefficient - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_gini_coefficient_moderate_inequality() {
        // Some inequality but not extreme
        let money = [10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = calculate_money_stats(&money);
        // For linearly increasing values, Gini should be around 0.2667
        // G = (2 * sum(i * x_i)) / (n * sum(x_i)) - (n + 1) / n
        // sum(i * x_i) = 1*10 + 2*20 + 3*30 + 4*40 + 5*50 = 10 + 40 + 90 + 160 + 250 = 550
        // sum(x_i) = 150, n = 5
        // G = (2 * 550) / (5 * 150) - 6 / 5 = 1100 / 750 - 1.2 = 1.4667 - 1.2 = 0.2667
        assert!((stats.gini_coefficient - 0.26666667).abs() < 1e-6);
    }

    #[test]
    fn test_gini_coefficient_empty_distribution() {
        let money: Vec<f64> = vec![];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_single_person() {
        let money = [100.0];
        let stats = calculate_money_stats(&money);
        // Single person: Gini should be 0 (no inequality possible)
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_two_persons_equal() {
        let money = [50.0, 50.0];
        let stats = calculate_money_stats(&money);
        assert_eq!(stats.gini_coefficient, 0.0);
    }

    #[test]
    fn test_gini_coefficient_two_persons_unequal() {
        let money = [25.0, 75.0];
        let stats = calculate_money_stats(&money);
        // For n=2: G = (2 * (1*25 + 2*75)) / (2 * 100) - 3/2
        // = (2 * (25 + 150)) / 200 - 1.5 = 350 / 200 - 1.5 = 1.75 - 1.5 = 0.25
        assert!((stats.gini_coefficient - 0.25).abs() < 1e-10);
    }
}
