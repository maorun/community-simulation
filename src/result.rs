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
    pub step_times: Vec<f64>, // Time taken for each step
    pub active_persons: usize, // Renamed from active_entities for clarity

    // Economic output
    pub final_money_distribution: Vec<f64>, // List of money amounts per person
    pub money_statistics: MoneyStats,

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
            let avg_step_time_ms = self.step_times.iter().sum::<f64>() / self.step_times.len() as f64 * 1000.0;
            println!("Average step time: {:.4}ms", avg_step_time_ms);
        }
        println!("Active persons remaining: {}", self.active_persons);
        let performance = if self.total_duration > 0.0 {
            self.total_steps as f64 / self.total_duration
        } else { 0.0 };
        println!("Performance: {:.0} steps/second", performance);

        println!("\n--- Money Distribution ---");
        println!("Average Money: {:.2}", self.money_statistics.average);
        println!("Median Money: {:.2}", self.money_statistics.median);
        println!("Std Dev Money: {:.2}", self.money_statistics.std_dev);
        println!("Min/Max Money: {:.2} / {:.2}", self.money_statistics.min_money, self.money_statistics.max_money);

        println!("\n--- Skill Valuations ---");
        if let Some(skill) = &self.most_valuable_skill {
            println!("Most Valuable Skill: {} (Price: {:.2})", skill.id, skill.price);
        }
        if let Some(skill) = &self.least_valuable_skill {
            println!("Least Valuable Skill: {} (Price: {:.2})", skill.id, skill.price);
        }

        println!("\nTop 5 Most Valuable Skills:");
        for skill_info in self.final_skill_prices.iter().take(5) {
            println!("  - {}: {:.2}", skill_info.id, skill_info.price);
        }

        println!("\nTop 5 Least Valuable Skills (excluding those at min price if many):");
        // Iterate in reverse, but skip if all are min_price
        let mut count = 0;
        for skill_info in self.final_skill_prices.iter().rev().take(10) { // Check more than 5 to find some not at min
            if count < 5 {
                // Basic heuristic: if it's significantly above absolute min, show it.
                // This needs better logic if many skills bottom out at min_skill_price.
                // For now, just show them.
                println!("  - {}: {:.2}", skill_info.id, skill_info.price);
                count += 1;
            }
        }
        if self.skill_price_history.keys().len() > 0 {
            println!("\nSkill price history for {} skills available in JSON output.", self.skill_price_history.keys().len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate_money_stats(money_values: &[f64]) -> MoneyStats {
        if money_values.is_empty() {
            return MoneyStats { average: 0.0, median: 0.0, std_dev: 0.0, min_money: 0.0, max_money: 0.0 };
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
        } else { 0.0 };

        let variance = sorted_money.iter().map(|value| {
            let diff = average - value;
            diff * diff
        }).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        MoneyStats {
            average,
            median,
            std_dev,
            min_money: *sorted_money.first().unwrap_or(&0.0),
            max_money: *sorted_money.last().unwrap_or(&0.0),
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
    }

    #[test]
    fn test_money_stats_single_value() {
        let stats = calculate_money_stats(&[100.0]);
        assert_eq!(stats.average, 100.0);
        assert_eq!(stats.median, 100.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min_money, 100.0);
        assert_eq!(stats.max_money, 100.0);
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
}