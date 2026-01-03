use crate::error::{Result, SimulationError};
use crate::scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Preset configuration names for typical simulation scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum PresetName {
    Default,
    SmallEconomy,
    LargeEconomy,
    CrisisScenario,
    HighInflation,
    TechGrowth,
    QuickTest,
}

impl PresetName {
    /// Get all available preset names
    pub fn all() -> Vec<PresetName> {
        vec![
            PresetName::Default,
            PresetName::SmallEconomy,
            PresetName::LargeEconomy,
            PresetName::CrisisScenario,
            PresetName::HighInflation,
            PresetName::TechGrowth,
            PresetName::QuickTest,
        ]
    }

    /// Get the string identifier for this preset
    pub fn as_str(&self) -> &str {
        match self {
            PresetName::Default => "default",
            PresetName::SmallEconomy => "small_economy",
            PresetName::LargeEconomy => "large_economy",
            PresetName::CrisisScenario => "crisis_scenario",
            PresetName::HighInflation => "high_inflation",
            PresetName::TechGrowth => "tech_growth",
            PresetName::QuickTest => "quick_test",
        }
    }

    /// Get a description of this preset
    pub fn description(&self) -> &str {
        match self {
            PresetName::Default => "Standard economy with 100 persons, 500 steps",
            PresetName::SmallEconomy => "Small economy with 20 persons for quick testing",
            PresetName::LargeEconomy => "Large economy with 500 persons for detailed analysis",
            PresetName::CrisisScenario => "Economic crisis with low initial money and high prices",
            PresetName::HighInflation => "High inflation scenario with dynamic pricing",
            PresetName::TechGrowth => "Technology growth scenario with high initial capital",
            PresetName::QuickTest => "Very small economy for rapid testing (10 persons, 50 steps)",
        }
    }
}

/// Implement FromStr trait for parsing preset names from strings
impl FromStr for PresetName {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(PresetName::Default),
            "small_economy" | "small" => Ok(PresetName::SmallEconomy),
            "large_economy" | "large" => Ok(PresetName::LargeEconomy),
            "crisis_scenario" | "crisis" => Ok(PresetName::CrisisScenario),
            "high_inflation" | "inflation" => Ok(PresetName::HighInflation),
            "tech_growth" | "tech" => Ok(PresetName::TechGrowth),
            "quick_test" | "quick" => Ok(PresetName::QuickTest),
            _ => Err(format!("Unknown preset: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    // General simulation parameters
    pub max_steps: usize,
    pub entity_count: usize, // This will be our number of persons
    pub seed: u64,

    // Economic simulation specific parameters
    pub initial_money_per_person: f64,
    pub base_skill_price: f64,
    // num_unique_skills will be equal to entity_count as each person has one unique skill

    // time_step might not be directly relevant for a turn-based economic sim,
    // but we can keep it or remove it later. For now, let's keep it.
    pub time_step: f64,
    pub scenario: Scenario,

    /// Technology growth rate per simulation step.
    ///
    /// This rate determines how quickly skills become more efficient over time,
    /// simulating technological progress and productivity improvements.
    /// A rate of 0.001 means skills improve by 0.1% per step.
    /// Set to 0.0 to disable technological progress (default).
    #[serde(default)]
    pub tech_growth_rate: f64,

    /// Seasonal demand amplitude (0.0 = no seasonality, 0.0-1.0 = variation strength).
    ///
    /// Controls the strength of seasonal fluctuations in skill demand.
    /// A value of 0.5 means demand can vary ±50% from the base level.
    /// Set to 0.0 to disable seasonal effects (default).
    #[serde(default)]
    pub seasonal_amplitude: f64,

    /// Seasonal cycle period in simulation steps.
    ///
    /// Determines how many steps it takes for demand to complete one seasonal cycle.
    /// For example, a value of 100 means demand patterns repeat every 100 steps.
    /// Only used when seasonal_amplitude > 0.0.
    #[serde(default = "default_seasonal_period")]
    pub seasonal_period: usize,

    /// Transaction fee rate as a percentage of the transaction value.
    ///
    /// This represents the cost of conducting trade in the market.
    /// The fee is deducted from the seller's proceeds.
    /// A value of 0.05 means a 5% fee is charged on each transaction.
    /// Set to 0.0 to disable transaction fees (default).
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default)]
    pub transaction_fee: f64,

    /// Savings rate as a percentage of current money to save each step.
    ///
    /// Each simulation step, persons will save this percentage of their current money.
    /// The saved money is moved from available cash to savings, affecting spending capacity.
    /// A value of 0.05 means 5% of current money is saved each step.
    /// Set to 0.0 to disable savings (default).
    /// Valid range: 0.0 to 1.0 (0% to 100%)
    #[serde(default)]
    pub savings_rate: f64,
}

fn default_seasonal_period() -> usize {
    100
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_steps: 500,    // Default to 500 steps for market convergence
            entity_count: 100, // 100 persons
            seed: 42,
            initial_money_per_person: 100.0, // 100 Euros
            base_skill_price: 10.0,          // 10 Euros base price for skills
            time_step: 1.0,                  // Represents one discrete step or turn
            scenario: Scenario::Original,
            tech_growth_rate: 0.0,   // Disabled by default
            seasonal_amplitude: 0.0, // Disabled by default
            seasonal_period: 100,    // Default cycle length
            transaction_fee: 0.0,    // Disabled by default
            savings_rate: 0.0,       // Disabled by default
        }
    }
}

impl SimulationConfig {
    /// Validates the configuration parameters to ensure they are within acceptable ranges.
    ///
    /// # Returns
    /// * `Ok(())` if all parameters are valid
    /// * `Err(SimulationError::ValidationError)` with a descriptive error message if validation fails
    ///
    /// # Validation Rules
    /// - `max_steps` must be greater than 0
    /// - `entity_count` must be greater than 0
    /// - `initial_money_per_person` must be non-negative
    /// - `base_skill_price` must be greater than 0
    /// - `time_step` must be greater than 0
    /// - `tech_growth_rate` must be non-negative
    /// - `seasonal_amplitude` must be between 0.0 and 1.0 (inclusive)
    /// - `seasonal_period` must be greater than 0
    ///
    /// # Examples
    /// ```
    /// use simulation_framework::SimulationConfig;
    ///
    /// let mut config = SimulationConfig::default();
    /// assert!(config.validate().is_ok());
    ///
    /// config.max_steps = 0;
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.max_steps == 0 {
            return Err(SimulationError::ValidationError(
                "max_steps must be greater than 0".to_string(),
            ));
        }

        if self.entity_count == 0 {
            return Err(SimulationError::ValidationError(
                "entity_count (number of persons) must be greater than 0".to_string(),
            ));
        }

        if self.initial_money_per_person.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "initial_money_per_person must be non-negative, got: {}",
                self.initial_money_per_person
            )));
        }

        if self.base_skill_price <= 0.0 {
            return Err(SimulationError::ValidationError(format!(
                "base_skill_price must be greater than 0, got: {}",
                self.base_skill_price
            )));
        }

        if self.time_step <= 0.0 {
            return Err(SimulationError::ValidationError(format!(
                "time_step must be greater than 0, got: {}",
                self.time_step
            )));
        }

        if self.tech_growth_rate.is_sign_negative() {
            return Err(SimulationError::ValidationError(format!(
                "tech_growth_rate must be non-negative, got: {}",
                self.tech_growth_rate
            )));
        }

        if !(0.0..=1.0).contains(&self.seasonal_amplitude) {
            return Err(SimulationError::ValidationError(format!(
                "seasonal_amplitude must be between 0.0 and 1.0, got: {}",
                self.seasonal_amplitude
            )));
        }

        if self.seasonal_period == 0 {
            return Err(SimulationError::ValidationError(
                "seasonal_period must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.transaction_fee) {
            return Err(SimulationError::ValidationError(format!(
                "transaction_fee must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.transaction_fee
            )));
        }

        if !(0.0..=1.0).contains(&self.savings_rate) {
            return Err(SimulationError::ValidationError(format!(
                "savings_rate must be between 0.0 and 1.0 (0% to 100%), got: {}",
                self.savings_rate
            )));
        }

        // Additional sanity checks for extreme values
        if self.max_steps > 1_000_000 {
            return Err(SimulationError::ValidationError(format!(
                "max_steps is too large ({}), maximum recommended value is 1,000,000",
                self.max_steps
            )));
        }

        if self.entity_count > 100_000 {
            return Err(SimulationError::ValidationError(format!(
                "entity_count is too large ({}), maximum recommended value is 100,000",
                self.entity_count
            )));
        }

        if self.tech_growth_rate > 1.0 {
            return Err(SimulationError::ValidationError(format!(
                "tech_growth_rate is too large ({}), values above 1.0 (100% per step) are unrealistic",
                self.tech_growth_rate
            )));
        }

        Ok(())
    }

    /// Create a configuration from a preset.
    ///
    /// # Arguments
    /// * `preset` - The preset name to use
    ///
    /// # Returns
    /// * `SimulationConfig` - A configuration with preset values
    ///
    /// # Examples
    /// ```
    /// use simulation_framework::{SimulationConfig, PresetName};
    ///
    /// let config = SimulationConfig::from_preset(PresetName::SmallEconomy);
    /// assert_eq!(config.entity_count, 20);
    /// ```
    pub fn from_preset(preset: PresetName) -> Self {
        match preset {
            PresetName::Default => Self::default(),
            PresetName::SmallEconomy => Self {
                max_steps: 100,
                entity_count: 20,
                seed: 42,
                initial_money_per_person: 100.0,
                base_skill_price: 10.0,
                time_step: 1.0,
                scenario: Scenario::Original,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
            PresetName::LargeEconomy => Self {
                max_steps: 2000,
                entity_count: 500,
                seed: 42,
                initial_money_per_person: 200.0,
                base_skill_price: 10.0,
                time_step: 1.0,
                scenario: Scenario::Original,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
            PresetName::CrisisScenario => Self {
                max_steps: 1000,
                entity_count: 100,
                seed: 42,
                initial_money_per_person: 50.0,
                base_skill_price: 25.0,
                time_step: 1.0,
                scenario: Scenario::Original,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
            PresetName::HighInflation => Self {
                max_steps: 1000,
                entity_count: 100,
                seed: 42,
                initial_money_per_person: 100.0,
                base_skill_price: 15.0,
                time_step: 1.0,
                scenario: Scenario::DynamicPricing,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
            PresetName::TechGrowth => Self {
                max_steps: 1500,
                entity_count: 150,
                seed: 42,
                initial_money_per_person: 250.0,
                base_skill_price: 8.0,
                time_step: 1.0,
                scenario: Scenario::Original,
                tech_growth_rate: 0.001, // 0.1% growth per step - significant over 1500 steps
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
            PresetName::QuickTest => Self {
                max_steps: 50,
                entity_count: 10,
                seed: 42,
                initial_money_per_person: 100.0,
                base_skill_price: 10.0,
                time_step: 1.0,
                scenario: Scenario::Original,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
            },
        }
    }

    /// Load configuration from a YAML or TOML file.
    /// File format is auto-detected based on file extension.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file (.yaml, .yml, or .toml)
    ///
    /// # Returns
    /// * `Result<SimulationConfig>` - The loaded config or a SimulationError
    ///
    /// # Examples
    /// ```no_run
    /// use simulation_framework::SimulationConfig;
    ///
    /// let config = SimulationConfig::from_file("config.yaml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(SimulationError::ConfigFileRead)?;

        // Detect format based on file extension
        let extension = path.extension().and_then(|s| s.to_str()).ok_or_else(|| {
            SimulationError::UnsupportedConfigFormat("(no extension)".to_string())
        })?;

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                let config: SimulationConfig = serde_yaml::from_str(&contents)
                    .map_err(|e| SimulationError::YamlParse(e.to_string()))?;
                Ok(config)
            }
            "toml" => {
                let config: SimulationConfig = toml::from_str(&contents)
                    .map_err(|e| SimulationError::TomlParse(e.to_string()))?;
                Ok(config)
            }
            _ => Err(SimulationError::UnsupportedConfigFormat(
                extension.to_string(),
            )),
        }
    }

    /// Merge configuration from a file with CLI overrides.
    /// Values from CLI (if present) take precedence over file values.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    /// * `cli_overrides` - Function that applies CLI overrides to the config
    ///
    /// # Returns
    /// * `Result<SimulationConfig>` - The merged config or a SimulationError
    pub fn from_file_with_overrides<P: AsRef<Path>, F>(path: P, cli_overrides: F) -> Result<Self>
    where
        F: FnOnce(&mut SimulationConfig),
    {
        let mut config = Self::from_file(path)?;
        cli_overrides(&mut config);
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn test_load_yaml_config() {
        let yaml_content = r#"
max_steps: 1000
entity_count: 50
seed: 123
initial_money_per_person: 200.0
base_skill_price: 15.0
time_step: 1.0
scenario: Original
"#;
        let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.max_steps, 1000);
        assert_eq!(config.entity_count, 50);
        assert_eq!(config.seed, 123);
        assert_eq!(config.initial_money_per_person, 200.0);
        assert_eq!(config.base_skill_price, 15.0);
    }

    #[test]
    fn test_load_toml_config() {
        let toml_content = r#"
max_steps = 2000
entity_count = 75
seed = 456
initial_money_per_person = 300.0
base_skill_price = 20.0
time_step = 1.0
scenario = "DynamicPricing"
"#;
        let mut temp_file = Builder::new().suffix(".toml").tempfile().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.max_steps, 2000);
        assert_eq!(config.entity_count, 75);
        assert_eq!(config.seed, 456);
        assert_eq!(config.initial_money_per_person, 300.0);
        assert_eq!(config.base_skill_price, 20.0);
    }

    #[test]
    fn test_invalid_file_extension() {
        let mut temp_file = Builder::new().suffix(".txt").tempfile().unwrap();
        temp_file.write_all(b"invalid content").unwrap();
        temp_file.flush().unwrap();

        let result = SimulationConfig::from_file(temp_file.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Unsupported configuration file format"));
    }

    #[test]
    fn test_missing_file() {
        let result = SimulationConfig::from_file("/nonexistent/config.yaml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Failed to read configuration file"));
    }

    #[test]
    fn test_config_with_overrides() {
        let yaml_content = r#"
max_steps: 1000
entity_count: 50
seed: 123
initial_money_per_person: 200.0
base_skill_price: 15.0
time_step: 1.0
scenario: Original
"#;
        let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SimulationConfig::from_file_with_overrides(temp_file.path(), |cfg| {
            cfg.max_steps = 5000; // CLI override
            cfg.seed = 999; // CLI override
        })
        .unwrap();

        assert_eq!(config.max_steps, 5000); // Overridden
        assert_eq!(config.entity_count, 50); // From file
        assert_eq!(config.seed, 999); // Overridden
        assert_eq!(config.initial_money_per_person, 200.0); // From file
    }

    #[test]
    fn test_preset_default() {
        let config = SimulationConfig::from_preset(PresetName::Default);
        let default_config = SimulationConfig::default();

        assert_eq!(config.max_steps, default_config.max_steps);
        assert_eq!(config.entity_count, default_config.entity_count);
        assert_eq!(
            config.initial_money_per_person,
            default_config.initial_money_per_person
        );
    }

    #[test]
    fn test_preset_small_economy() {
        let config = SimulationConfig::from_preset(PresetName::SmallEconomy);

        assert_eq!(config.entity_count, 20);
        assert_eq!(config.max_steps, 100);
        assert_eq!(config.initial_money_per_person, 100.0);
    }

    #[test]
    fn test_preset_large_economy() {
        let config = SimulationConfig::from_preset(PresetName::LargeEconomy);

        assert_eq!(config.entity_count, 500);
        assert_eq!(config.max_steps, 2000);
        assert_eq!(config.initial_money_per_person, 200.0);
    }

    #[test]
    fn test_preset_crisis_scenario() {
        let config = SimulationConfig::from_preset(PresetName::CrisisScenario);

        assert_eq!(config.entity_count, 100);
        assert_eq!(config.max_steps, 1000);
        assert_eq!(config.initial_money_per_person, 50.0);
        assert_eq!(config.base_skill_price, 25.0);
    }

    #[test]
    fn test_preset_high_inflation() {
        let config = SimulationConfig::from_preset(PresetName::HighInflation);

        assert_eq!(config.scenario, Scenario::DynamicPricing);
        assert_eq!(config.entity_count, 100);
        assert_eq!(config.base_skill_price, 15.0);
    }

    #[test]
    fn test_preset_quick_test() {
        let config = SimulationConfig::from_preset(PresetName::QuickTest);

        assert_eq!(config.entity_count, 10);
        assert_eq!(config.max_steps, 50);
    }

    #[test]
    fn test_preset_name_from_str() {
        assert_eq!(
            PresetName::from_str("default").unwrap(),
            PresetName::Default
        );
        assert_eq!(
            PresetName::from_str("small_economy").unwrap(),
            PresetName::SmallEconomy
        );
        assert_eq!(
            PresetName::from_str("small").unwrap(),
            PresetName::SmallEconomy
        );
        assert_eq!(
            PresetName::from_str("crisis").unwrap(),
            PresetName::CrisisScenario
        );
        assert!(PresetName::from_str("nonexistent").is_err());
    }

    #[test]
    fn test_preset_name_as_str() {
        assert_eq!(PresetName::Default.as_str(), "default");
        assert_eq!(PresetName::SmallEconomy.as_str(), "small_economy");
        assert_eq!(PresetName::QuickTest.as_str(), "quick_test");
    }

    #[test]
    fn test_all_presets_are_valid() {
        // Ensure all presets can be created without panicking
        for preset in PresetName::all() {
            let config = SimulationConfig::from_preset(preset.clone());
            assert!(config.entity_count > 0);
            assert!(config.max_steps > 0);
            assert!(config.initial_money_per_person > 0.0);
            assert!(config.base_skill_price > 0.0);
        }
    }

    #[test]
    fn test_validate_default_config() {
        let config = SimulationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_all_presets() {
        // Ensure all preset configurations pass validation
        for preset in PresetName::all() {
            let config = SimulationConfig::from_preset(preset.clone());
            assert!(
                config.validate().is_ok(),
                "Preset {:?} should pass validation",
                preset
            );
        }
    }

    #[test]
    fn test_validate_zero_max_steps() {
        let config = SimulationConfig {
            max_steps: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("max_steps must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_entity_count() {
        let config = SimulationConfig {
            entity_count: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("entity_count (number of persons) must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_initial_money() {
        let config = SimulationConfig {
            initial_money_per_person: -10.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("initial_money_per_person must be non-negative"));
    }

    #[test]
    fn test_validate_zero_base_skill_price() {
        let config = SimulationConfig {
            base_skill_price: 0.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("base_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_base_skill_price() {
        let config = SimulationConfig {
            base_skill_price: -5.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("base_skill_price must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_time_step() {
        let config = SimulationConfig {
            time_step: 0.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("time_step must be greater than 0"));
    }

    #[test]
    fn test_validate_negative_tech_growth_rate() {
        let config = SimulationConfig {
            tech_growth_rate: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("tech_growth_rate must be non-negative"));
    }

    #[test]
    fn test_validate_excessive_tech_growth_rate() {
        let config = SimulationConfig {
            tech_growth_rate: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("tech_growth_rate is too large"));
    }

    #[test]
    fn test_validate_seasonal_amplitude_out_of_range() {
        let config = SimulationConfig {
            seasonal_amplitude: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("seasonal_amplitude must be between 0.0 and 1.0"));

        let config2 = SimulationConfig {
            seasonal_amplitude: -0.1,
            ..Default::default()
        };
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_validate_zero_seasonal_period() {
        let config = SimulationConfig {
            seasonal_period: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("seasonal_period must be greater than 0"));
    }

    #[test]
    fn test_validate_extreme_max_steps() {
        let config = SimulationConfig {
            max_steps: 2_000_000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("max_steps is too large"));
    }

    #[test]
    fn test_validate_extreme_entity_count() {
        let config = SimulationConfig {
            entity_count: 200_000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("entity_count is too large"));
    }

    #[test]
    fn test_validate_valid_edge_cases() {
        // Test that boundary values are accepted

        // Max valid tech growth rate
        let config1 = SimulationConfig {
            tech_growth_rate: 1.0,
            ..Default::default()
        };
        assert!(config1.validate().is_ok());

        // Max valid seasonal amplitude
        let config2 = SimulationConfig {
            seasonal_amplitude: 1.0,
            ..Default::default()
        };
        assert!(config2.validate().is_ok());

        // Min valid seasonal amplitude
        let config3 = SimulationConfig {
            seasonal_amplitude: 0.0,
            ..Default::default()
        };
        assert!(config3.validate().is_ok());

        // Zero initial money (allowed - represents starting with no money)
        let config4 = SimulationConfig {
            initial_money_per_person: 0.0,
            ..Default::default()
        };
        assert!(config4.validate().is_ok());

        // Single person
        let config5 = SimulationConfig {
            entity_count: 1,
            ..Default::default()
        };
        assert!(config5.validate().is_ok());

        // Single step
        let config6 = SimulationConfig {
            max_steps: 1,
            ..Default::default()
        };
        assert!(config6.validate().is_ok());
    }
}
