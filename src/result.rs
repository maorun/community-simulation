use crate::error::{Result, SimulationError};
use crate::{Entity, SkillId}; // Entity now wraps Person
use colored::Colorize;
use flate2::write::GzEncoder;
use flate2::Compression;
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
    /// Herfindahl-Hirschman Index: measure of market concentration for wealth (0 = perfect competition, 10000 = monopoly)
    /// Values < 1500 indicate competitive distribution, 1500-2500 moderate concentration, > 2500 high concentration
    pub herfindahl_index: f64,
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

/// Statistics about trade volume and economic activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeVolumeStats {
    /// Total number of successful trades across all steps
    pub total_trades: usize,
    /// Total money exchanged across all trades
    pub total_volume: f64,
    /// Average number of trades per step
    pub avg_trades_per_step: f64,
    /// Average money exchanged per step
    pub avg_volume_per_step: f64,
    /// Average transaction value (total volume / total trades)
    pub avg_transaction_value: f64,
    /// Minimum trades in a single step
    pub min_trades_per_step: usize,
    /// Maximum trades in a single step
    pub max_trades_per_step: usize,
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

    // Trade volume analysis
    pub trade_volume_statistics: TradeVolumeStats,
    /// Number of trades executed at each step
    pub trades_per_step: Vec<usize>,
    /// Total money volume exchanged at each step
    pub volume_per_step: Vec<f64>,
    /// Total transaction fees collected across all trades
    pub total_fees_collected: f64,

    // final_entities might be too verbose if Person struct grows large with transaction history.
    // Consider summarizing person data if needed, or providing it under a flag.
    // For now, let's keep it as it contains all person data including transaction history.
    pub final_persons_data: Vec<Entity>, // Renamed from final_entities
}

impl SimulationResult {
    /// Save simulation results to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the output file
    /// * `compress` - If true, compress the output using gzip and append .gz to the filename
    ///
    /// # Returns
    /// * `Result<()>` - Success or a SimulationError
    ///
    /// # Examples
    /// ```no_run
    /// # use simulation_framework::result::SimulationResult;
    /// # let result = SimulationResult {
    /// #     total_steps: 0,
    /// #     total_duration: 0.0,
    /// #     step_times: vec![],
    /// #     active_persons: 0,
    /// #     final_money_distribution: vec![],
    /// #     money_statistics: simulation_framework::result::MoneyStats {
    /// #         average: 0.0, median: 0.0, std_dev: 0.0,
    /// #         min_money: 0.0, max_money: 0.0, gini_coefficient: 0.0, herfindahl_index: 0.0,
    /// #     },
    /// #     final_reputation_distribution: vec![],
    /// #     reputation_statistics: simulation_framework::result::ReputationStats {
    /// #         average: 0.0, median: 0.0, std_dev: 0.0,
    /// #         min_reputation: 0.0, max_reputation: 0.0,
    /// #     },
    /// #     final_skill_prices: vec![],
    /// #     most_valuable_skill: None,
    /// #     least_valuable_skill: None,
    /// #     skill_price_history: std::collections::HashMap::new(),
    /// #     trade_volume_statistics: simulation_framework::result::TradeVolumeStats {
    /// #         total_trades: 0, total_volume: 0.0,
    /// #         avg_trades_per_step: 0.0, avg_volume_per_step: 0.0,
    /// #         avg_transaction_value: 0.0,
    /// #         min_trades_per_step: 0, max_trades_per_step: 0,
    /// #     },
    /// #     trades_per_step: vec![],
    /// #     volume_per_step: vec![],
    /// #     total_fees_collected: 0.0,
    /// #     final_persons_data: vec![],
    /// # };
    /// // Save uncompressed JSON
    /// result.save_to_file("results.json", false).unwrap();
    ///
    /// // Save compressed JSON
    /// result.save_to_file("results.json", true).unwrap(); // Creates results.json.gz
    /// ```
    pub fn save_to_file(&self, path: &str, compress: bool) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SimulationError::JsonSerialize(e.to_string()))?;

        if compress {
            // Add .gz extension if not already present
            let output_path = if path.ends_with(".gz") {
                path.to_string()
            } else {
                format!("{}.gz", path)
            };

            let file = File::create(&output_path)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(json.as_bytes())?;
            encoder.finish()?;
        } else {
            let mut file = File::create(path)?;
            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }

    /// Save simulation results to CSV files.
    /// Creates multiple CSV files with the given path as prefix:
    /// - {path}_summary.csv: Summary statistics
    /// - {path}_money.csv: Money distribution per person
    /// - {path}_reputation.csv: Reputation distribution per person
    /// - {path}_skill_prices.csv: Final skill prices
    /// - {path}_price_history.csv: Skill price history over time (if available)
    /// - {path}_trade_volume.csv: Trade volume history over time
    ///
    /// # Returns
    /// * `Result<()>` - Success or a SimulationError
    pub fn save_to_csv(&self, path_prefix: &str) -> Result<()> {
        // Save summary statistics
        self.save_summary_csv(&format!("{}_summary.csv", path_prefix))?;

        // Save money distribution
        self.save_money_csv(&format!("{}_money.csv", path_prefix))?;

        // Save reputation distribution
        self.save_reputation_csv(&format!("{}_reputation.csv", path_prefix))?;

        // Save skill prices
        self.save_skill_prices_csv(&format!("{}_skill_prices.csv", path_prefix))?;

        // Save price history if available
        if !self.skill_price_history.is_empty() {
            self.save_price_history_csv(&format!("{}_price_history.csv", path_prefix))?;
        }

        // Save trade volume history
        self.save_trade_volume_csv(&format!("{}_trade_volume.csv", path_prefix))?;

        Ok(())
    }

    fn save_summary_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Metric,Value")?;
        writeln!(file, "Total Steps,{}", self.total_steps)?;
        writeln!(file, "Total Duration (s),{:.4}", self.total_duration)?;
        writeln!(file, "Active Persons,{}", self.active_persons)?;

        if !self.step_times.is_empty() {
            let avg_step_time = self.step_times.iter().sum::<f64>() / self.step_times.len() as f64;
            writeln!(file, "Average Step Time (s),{:.6}", avg_step_time)?;
        }

        writeln!(file)?;
        writeln!(file, "Money Statistics")?;
        writeln!(file, "Average Money,{:.4}", self.money_statistics.average)?;
        writeln!(file, "Median Money,{:.4}", self.money_statistics.median)?;
        writeln!(file, "Std Dev Money,{:.4}", self.money_statistics.std_dev)?;
        writeln!(file, "Min Money,{:.4}", self.money_statistics.min_money)?;
        writeln!(file, "Max Money,{:.4}", self.money_statistics.max_money)?;
        writeln!(
            file,
            "Gini Coefficient,{:.6}",
            self.money_statistics.gini_coefficient
        )?;
        writeln!(
            file,
            "Herfindahl Index,{:.2}",
            self.money_statistics.herfindahl_index
        )?;

        writeln!(file)?;
        writeln!(file, "Reputation Statistics")?;
        writeln!(
            file,
            "Average Reputation,{:.6}",
            self.reputation_statistics.average
        )?;
        writeln!(
            file,
            "Median Reputation,{:.6}",
            self.reputation_statistics.median
        )?;
        writeln!(
            file,
            "Std Dev Reputation,{:.6}",
            self.reputation_statistics.std_dev
        )?;
        writeln!(
            file,
            "Min Reputation,{:.6}",
            self.reputation_statistics.min_reputation
        )?;
        writeln!(
            file,
            "Max Reputation,{:.6}",
            self.reputation_statistics.max_reputation
        )?;

        writeln!(file)?;
        writeln!(file, "Trade Volume Statistics")?;
        writeln!(
            file,
            "Total Trades,{}",
            self.trade_volume_statistics.total_trades
        )?;
        writeln!(
            file,
            "Total Volume,{:.4}",
            self.trade_volume_statistics.total_volume
        )?;
        writeln!(
            file,
            "Avg Trades Per Step,{:.4}",
            self.trade_volume_statistics.avg_trades_per_step
        )?;
        writeln!(
            file,
            "Avg Volume Per Step,{:.4}",
            self.trade_volume_statistics.avg_volume_per_step
        )?;
        writeln!(
            file,
            "Avg Transaction Value,{:.4}",
            self.trade_volume_statistics.avg_transaction_value
        )?;
        writeln!(
            file,
            "Min Trades Per Step,{}",
            self.trade_volume_statistics.min_trades_per_step
        )?;
        writeln!(
            file,
            "Max Trades Per Step,{}",
            self.trade_volume_statistics.max_trades_per_step
        )?;

        Ok(())
    }

    fn save_money_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Person_ID,Money")?;
        for (id, money) in self.final_money_distribution.iter().enumerate() {
            writeln!(file, "{},{:.4}", id, money)?;
        }

        Ok(())
    }

    fn save_reputation_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Person_ID,Reputation")?;
        for (id, reputation) in self.final_reputation_distribution.iter().enumerate() {
            writeln!(file, "{},{:.6}", id, reputation)?;
        }

        Ok(())
    }

    fn save_skill_prices_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Skill_ID,Final_Price")?;
        for skill_info in &self.final_skill_prices {
            writeln!(file, "{},{:.4}", skill_info.id, skill_info.price)?;
        }

        Ok(())
    }

    fn save_price_history_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        // Collect all skill IDs and sort them for consistent output
        let mut skill_ids: Vec<_> = self.skill_price_history.keys().collect();
        skill_ids.sort();

        // Write header
        write!(file, "Step")?;
        for skill_id in &skill_ids {
            write!(file, ",Skill_{}", skill_id)?;
        }
        writeln!(file)?;

        // Determine max number of steps (should be the same for all skills)
        let max_steps = self
            .skill_price_history
            .values()
            .map(|prices| prices.len())
            .max()
            .unwrap_or(0);

        // Write data rows
        for step in 0..max_steps {
            write!(file, "{}", step)?;
            for skill_id in &skill_ids {
                let price = self
                    .skill_price_history
                    .get(*skill_id)
                    .and_then(|prices| prices.get(step));

                if let Some(&price) = price {
                    write!(file, ",{:.4}", price)?;
                } else {
                    write!(file, ",")?;
                }
            }
            writeln!(file)?;
        }

        Ok(())
    }

    fn save_trade_volume_csv(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "Step,Trades_Count,Volume_Exchanged")?;
        for (step, (&trades, &volume)) in self
            .trades_per_step
            .iter()
            .zip(self.volume_per_step.iter())
            .enumerate()
        {
            writeln!(file, "{},{},{:.4}", step, trades, volume)?;
        }

        Ok(())
    }

    pub fn print_summary(&self) {
        println!(
            "\n{}",
            "=== Economic Simulation Summary ===".bright_cyan().bold()
        );
        println!("{} {}", "Total steps:".bold(), self.total_steps);
        println!("{} {:.2}s", "Total duration:".bold(), self.total_duration);
        if !self.step_times.is_empty() {
            let avg_step_time_ms =
                self.step_times.iter().sum::<f64>() / self.step_times.len() as f64 * 1000.0;
            println!("{} {:.4}ms", "Average step time:".bold(), avg_step_time_ms);
        }
        println!(
            "{} {}",
            "Active persons remaining:".bold(),
            self.active_persons
        );
        let performance = if self.total_duration > 0.0 {
            self.total_steps as f64 / self.total_duration
        } else {
            0.0
        };
        println!(
            "{} {}",
            "Performance:".bold(),
            format!("{:.0} steps/second", performance).bright_yellow()
        );

        println!("\n{}", "--- Money Distribution ---".bright_green().bold());
        println!(
            "{} {:.2}",
            "Average Money:".bold(),
            self.money_statistics.average
        );
        println!(
            "{} {:.2}",
            "Median Money:".bold(),
            self.money_statistics.median
        );
        println!(
            "{} {:.2}",
            "Std Dev Money:".bold(),
            self.money_statistics.std_dev
        );
        println!(
            "{} {:.2} / {:.2}",
            "Min/Max Money:".bold(),
            self.money_statistics.min_money,
            self.money_statistics.max_money
        );

        // Color code Gini coefficient based on inequality level
        let gini_str = format!("{:.4}", self.money_statistics.gini_coefficient);
        let gini_colored = if self.money_statistics.gini_coefficient < 0.3 {
            gini_str.bright_green()
        } else if self.money_statistics.gini_coefficient < 0.5 {
            gini_str.bright_yellow()
        } else {
            gini_str.bright_red()
        };
        println!(
            "{} {} {}",
            "Gini Coefficient:".bold(),
            gini_colored,
            "(0 = perfect equality, 1 = perfect inequality)".dimmed()
        );

        // Color code HHI based on concentration level
        let hhi = self.money_statistics.herfindahl_index;
        let hhi_str = format!("{:.2}", hhi);
        let hhi_colored = if hhi < 1500.0 {
            hhi_str.bright_green()
        } else if hhi < 2500.0 {
            hhi_str.bright_yellow()
        } else {
            hhi_str.bright_red()
        };
        println!(
            "{} {} {}",
            "Herfindahl Index:".bold(),
            hhi_colored,
            "(< 1500 = competitive, 1500-2500 = moderate, > 2500 = high concentration)".dimmed()
        );

        println!(
            "\n{}",
            "--- Reputation Distribution ---".bright_magenta().bold()
        );
        println!(
            "{} {:.4}",
            "Average Reputation:".bold(),
            self.reputation_statistics.average
        );
        println!(
            "{} {:.4}",
            "Median Reputation:".bold(),
            self.reputation_statistics.median
        );
        println!(
            "{} {:.4}",
            "Std Dev Reputation:".bold(),
            self.reputation_statistics.std_dev
        );
        println!(
            "{} {:.4} / {:.4}",
            "Min/Max Reputation:".bold(),
            self.reputation_statistics.min_reputation,
            self.reputation_statistics.max_reputation
        );

        println!("\n{}", "--- Skill Valuations ---".bright_blue().bold());
        if let Some(skill) = &self.most_valuable_skill {
            println!(
                "{} {} {}",
                "Most Valuable Skill:".bold(),
                skill.id.to_string().bright_cyan(),
                format!("(Price: {:.2})", skill.price).bright_green()
            );
        }
        if let Some(skill) = &self.least_valuable_skill {
            println!(
                "{} {} {}",
                "Least Valuable Skill:".bold(),
                skill.id.to_string().bright_cyan(),
                format!("(Price: {:.2})", skill.price).bright_red()
            );
        }

        println!(
            "\n{}",
            "--- Trade Volume Statistics ---".bright_yellow().bold()
        );
        println!(
            "{} {}",
            "Total Trades:".bold(),
            self.trade_volume_statistics.total_trades
        );
        println!(
            "{} {:.2}",
            "Total Volume Exchanged:".bold(),
            self.trade_volume_statistics.total_volume
        );
        println!(
            "{} {:.2}",
            "Avg Trades Per Step:".bold(),
            self.trade_volume_statistics.avg_trades_per_step
        );
        println!(
            "{} {:.2}",
            "Avg Volume Per Step:".bold(),
            self.trade_volume_statistics.avg_volume_per_step
        );
        println!(
            "{} {:.2}",
            "Avg Transaction Value:".bold(),
            self.trade_volume_statistics.avg_transaction_value
        );
        println!(
            "{} {} / {}",
            "Min/Max Trades Per Step:".bold(),
            self.trade_volume_statistics.min_trades_per_step,
            self.trade_volume_statistics.max_trades_per_step
        );

        println!("\n{}", "Top 5 Most Valuable Skills:".bright_cyan().bold());
        for skill_info in self.final_skill_prices.iter().take(5) {
            println!(
                "  {} {} {:.2}",
                "-".dimmed(),
                format!("{}:", skill_info.id).bright_white(),
                skill_info.price
            );
        }

        println!(
            "\n{}",
            "Top 5 Least Valuable Skills (excluding those at min price if many):"
                .bright_cyan()
                .bold()
        );
        // Iterate in reverse, but skip if all are min_price
        let mut count = 0;
        for skill_info in self.final_skill_prices.iter().rev().take(10) {
            // Check more than 5 to find some not at min
            if count < 5 {
                // Basic heuristic: if it's significantly above absolute min, show it.
                // This needs better logic if many skills bottom out at min_skill_price.
                // For now, just show them.
                println!(
                    "  {} {} {:.2}",
                    "-".dimmed(),
                    format!("{}:", skill_info.id).bright_white(),
                    skill_info.price
                );
                count += 1;
            }
        }
        if self.skill_price_history.keys().len() > 0 {
            println!(
                "\n{} {} {}",
                "Skill price history for".dimmed(),
                self.skill_price_history
                    .keys()
                    .len()
                    .to_string()
                    .bright_white(),
                "skills available in JSON output.".dimmed()
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

/// Calculate the Herfindahl-Hirschman Index (HHI) for a given distribution of values.
///
/// The HHI measures market concentration by summing the squared market shares.
/// It ranges from near 0 (perfect competition with many equal participants) to 10,000 (monopoly).
///
/// # Interpretation
/// * HHI < 1,500: Competitive market (low concentration)
/// * HHI 1,500-2,500: Moderate concentration
/// * HHI > 2,500: High concentration (potential oligopoly/monopoly concerns)
///
/// # Arguments
/// * `values` - A slice of values representing shares (e.g., money, market share)
///
/// # Formula
/// HHI = sum((share_i * 100)^2) for all i
/// where share_i = value_i / total_value
///
/// # Returns
/// The HHI as f64, scaled to 0-10,000 range
///
/// # Examples
/// ```
/// use simulation_framework::result::calculate_herfindahl_index;
///
/// // Perfect equality (4 participants with 25% each): HHI = 2,500
/// let equal_shares = vec![25.0, 25.0, 25.0, 25.0];
/// let hhi = calculate_herfindahl_index(&equal_shares);
/// assert!((hhi - 2500.0).abs() < 0.1);
///
/// // Monopoly (1 participant with 100%): HHI = 10,000
/// let monopoly = vec![100.0];
/// let hhi = calculate_herfindahl_index(&monopoly);
/// assert!((hhi - 10000.0).abs() < 0.1);
/// ```
pub fn calculate_herfindahl_index(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let total: f64 = values.iter().sum();
    if total == 0.0 {
        return 0.0;
    }

    // Calculate HHI: sum of squared market shares (as percentages)
    values
        .iter()
        .map(|&value| {
            let share_percentage = (value / total) * 100.0;
            share_percentage * share_percentage
        })
        .sum()
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
                herfindahl_index: 2200.0,
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
            trade_volume_statistics: TradeVolumeStats {
                total_trades: 100,
                total_volume: 1000.0,
                avg_trades_per_step: 10.0,
                avg_volume_per_step: 100.0,
                avg_transaction_value: 10.0,
                min_trades_per_step: 5,
                max_trades_per_step: 15,
            },
            trades_per_step: vec![10, 12, 8, 10, 15, 9, 11, 10, 5, 10],
            volume_per_step: vec![
                100.0, 120.0, 80.0, 100.0, 150.0, 90.0, 110.0, 100.0, 50.0, 100.0,
            ],
            total_fees_collected: 0.0,
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

        result.save_to_file(path, false).unwrap();

        let mut contents = String::new();
        file.reopen()
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
    }

    #[test]
    fn test_save_to_file_compressed() {
        use flate2::read::GzDecoder;

        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_output.json");
        let path_str = path.to_str().unwrap();

        // Save compressed file
        result.save_to_file(path_str, true).unwrap();

        // Verify .gz file was created
        let gz_path = format!("{}.gz", path_str);
        assert!(std::path::Path::new(&gz_path).exists());

        // Decompress and verify contents
        let file = File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
        assert!(contents.contains("\"active_persons\": 5"));
    }

    #[test]
    fn test_save_to_file_compressed_with_gz_extension() {
        use flate2::read::GzDecoder;

        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_output.json.gz");
        let path_str = path.to_str().unwrap();

        // Save compressed file with .gz already in the path
        result.save_to_file(path_str, true).unwrap();

        // Verify file was created without double .gz extension
        assert!(std::path::Path::new(path_str).exists());
        let double_gz_path = format!("{}.gz", path_str);
        assert!(!std::path::Path::new(&double_gz_path).exists());

        // Decompress and verify contents
        let file = File::open(path_str).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("\"total_steps\": 10"));
        assert!(contents.contains("\"total_duration\": 1.23"));
    }

    #[test]
    fn test_compressed_file_is_smaller() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();

        // Save uncompressed
        let uncompressed_path = temp_dir.path().join("uncompressed.json");
        result
            .save_to_file(uncompressed_path.to_str().unwrap(), false)
            .unwrap();

        // Save compressed
        let compressed_path = temp_dir.path().join("compressed.json");
        result
            .save_to_file(compressed_path.to_str().unwrap(), true)
            .unwrap();
        let compressed_gz_path = format!("{}.gz", compressed_path.to_str().unwrap());

        // Compare file sizes
        let uncompressed_size = std::fs::metadata(&uncompressed_path).unwrap().len();
        let compressed_size = std::fs::metadata(&compressed_gz_path).unwrap().len();

        // Compressed should be smaller (for this test data)
        assert!(
            compressed_size < uncompressed_size,
            "Compressed size {} should be less than uncompressed size {}",
            compressed_size,
            uncompressed_size
        );
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
                herfindahl_index: 0.0,
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

        // Calculate Herfindahl Index using the shared utility function
        let herfindahl_index = calculate_herfindahl_index(&sorted_money);

        MoneyStats {
            average,
            median,
            std_dev,
            min_money: *sorted_money.first().unwrap_or(&0.0),
            max_money: *sorted_money.last().unwrap_or(&0.0),
            gini_coefficient,
            herfindahl_index,
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
        assert_eq!(stats.herfindahl_index, 0.0);
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
        // Single person = monopoly = HHI of 10,000
        assert!((stats.herfindahl_index - 10000.0).abs() < 0.1);
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

    #[test]
    fn test_save_to_csv_summary() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        // Check that summary file was created and contains expected content
        let summary_path = format!("{}_summary.csv", path_prefix);
        let mut contents = String::new();
        File::open(&summary_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("Metric,Value"));
        assert!(contents.contains("Total Steps,10"));
        assert!(contents.contains("Active Persons,5"));
        assert!(contents.contains("Average Money,100"));
        assert!(contents.contains("Gini Coefficient,0.2"));
    }

    #[test]
    fn test_save_to_csv_money_distribution() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        // Check money distribution file
        let money_path = format!("{}_money.csv", path_prefix);
        let mut contents = String::new();
        File::open(&money_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("Person_ID,Money"));
        assert!(contents.contains("0,50."));
        assert!(contents.contains("1,80."));
        assert!(contents.contains("2,100."));
        assert!(contents.contains("3,120."));
        assert!(contents.contains("4,150."));
    }

    #[test]
    fn test_save_to_csv_reputation_distribution() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        // Check reputation distribution file
        let reputation_path = format!("{}_reputation.csv", path_prefix);
        let mut contents = String::new();
        File::open(&reputation_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("Person_ID,Reputation"));
        assert!(contents.contains("0,0.95"));
        assert!(contents.contains("2,1.0"));
        assert!(contents.contains("4,1.1"));
    }

    #[test]
    fn test_save_to_csv_price_history() {
        let mut result = get_test_result();

        // Add price history data (SkillId is String type)
        let mut price_history = HashMap::new();
        price_history.insert("Skill_0".to_string(), vec![10.0, 11.0, 12.0]);
        price_history.insert("Skill_1".to_string(), vec![15.0, 14.5, 14.0]);
        result.skill_price_history = price_history;

        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        // Check price history file
        let history_path = format!("{}_price_history.csv", path_prefix);
        let mut contents = String::new();
        File::open(&history_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("Step,Skill_"));
        assert!(contents.contains("0,10."));
        assert!(contents.contains("1,11."));
        assert!(contents.contains("2,12."));
    }

    #[test]
    fn test_trade_volume_statistics() {
        let result = get_test_result();

        // Verify trade volume statistics are calculated correctly
        assert_eq!(result.trade_volume_statistics.total_trades, 100);
        assert_eq!(result.trade_volume_statistics.total_volume, 1000.0);
        assert_eq!(result.trade_volume_statistics.avg_trades_per_step, 10.0);
        assert_eq!(result.trade_volume_statistics.avg_volume_per_step, 100.0);
        assert_eq!(result.trade_volume_statistics.avg_transaction_value, 10.0);
        assert_eq!(result.trade_volume_statistics.min_trades_per_step, 5);
        assert_eq!(result.trade_volume_statistics.max_trades_per_step, 15);
    }

    #[test]
    fn test_save_to_csv_trade_volume() {
        let result = get_test_result();
        let temp_dir = tempfile::tempdir().unwrap();
        let path_prefix = temp_dir
            .path()
            .join("test_output")
            .to_str()
            .unwrap()
            .to_string();

        result.save_to_csv(&path_prefix).unwrap();

        // Check trade volume file
        let volume_path = format!("{}_trade_volume.csv", path_prefix);
        let mut contents = String::new();
        File::open(&volume_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("Step,Trades_Count,Volume_Exchanged"));
        assert!(contents.contains("0,10,100."));
        assert!(contents.contains("4,15,150."));
        assert!(contents.contains("8,5,50."));
    }

    #[test]
    fn test_herfindahl_index_empty() {
        let values: Vec<f64> = vec![];
        let hhi = calculate_herfindahl_index(&values);
        assert_eq!(hhi, 0.0);
    }

    #[test]
    fn test_herfindahl_index_monopoly() {
        // One participant with everything = HHI of 10,000
        let values = vec![100.0];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 10000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_perfect_equality() {
        // 4 participants with 25% each = HHI of 2,500
        let values = vec![25.0, 25.0, 25.0, 25.0];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 2500.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_ten_equal() {
        // 10 participants with 10% each = HHI of 1,000
        let values = vec![10.0; 10];
        let hhi = calculate_herfindahl_index(&values);
        assert!((hhi - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_high_concentration() {
        // One large player (60%) and 4 small players (10% each) = HHI of 4,000
        let values = vec![60.0, 10.0, 10.0, 10.0, 10.0];
        let hhi = calculate_herfindahl_index(&values);
        // HHI = 60^2 + 10^2 + 10^2 + 10^2 + 10^2 = 3600 + 100 + 100 + 100 + 100 = 4000
        assert!((hhi - 4000.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_moderate_concentration() {
        // Moderate concentration
        let values = vec![30.0, 25.0, 20.0, 15.0, 10.0];
        let hhi = calculate_herfindahl_index(&values);
        // HHI = 30^2 + 25^2 + 20^2 + 15^2 + 10^2 = 900 + 625 + 400 + 225 + 100 = 2250
        assert!((hhi - 2250.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_low_concentration() {
        // Many small players = low HHI
        let values = vec![5.0; 20];
        let hhi = calculate_herfindahl_index(&values);
        // 20 participants with 5% each = HHI of 500
        assert!((hhi - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_herfindahl_index_zero_sum() {
        // Zero total should return 0
        let values = vec![0.0, 0.0, 0.0];
        let hhi = calculate_herfindahl_index(&values);
        assert_eq!(hhi, 0.0);
    }

    #[test]
    fn test_money_stats_includes_hhi() {
        // Test that calculate_money_stats includes HHI
        let money = vec![25.0, 25.0, 25.0, 25.0];
        let stats = calculate_money_stats(&money);
        // Perfect equality: HHI should be 2500
        assert!((stats.herfindahl_index - 2500.0).abs() < 0.1);
    }

    #[test]
    fn test_money_stats_hhi_monopoly() {
        // One person has all money
        let money = vec![0.0, 0.0, 0.0, 100.0];
        let stats = calculate_money_stats(&money);
        // Monopoly: HHI should be 10000
        assert!((stats.herfindahl_index - 10000.0).abs() < 0.1);
    }
}
