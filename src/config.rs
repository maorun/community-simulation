use crate::scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
        }
    }
}

impl SimulationConfig {
    /// Load configuration from a YAML or TOML file.
    /// File format is auto-detected based on file extension.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file (.yaml, .yml, or .toml)
    ///
    /// # Returns
    /// * `Result<SimulationConfig, Box<dyn std::error::Error>>` - The loaded config or an error
    ///
    /// # Examples
    /// ```no_run
    /// use simulation_framework::SimulationConfig;
    ///
    /// let config = SimulationConfig::from_file("config.yaml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;

        // Detect format based on file extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or("Unable to determine file extension")?;

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                let config: SimulationConfig = serde_yaml::from_str(&contents)?;
                Ok(config)
            }
            "toml" => {
                let config: SimulationConfig = toml::from_str(&contents)?;
                Ok(config)
            }
            _ => Err(format!("Unsupported config file format: .{}", extension).into()),
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
    /// * `Result<SimulationConfig, Box<dyn std::error::Error>>` - The merged config or an error
    pub fn from_file_with_overrides<P: AsRef<Path>, F>(
        path: P,
        cli_overrides: F,
    ) -> Result<Self, Box<dyn std::error::Error>>
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
    }

    #[test]
    fn test_missing_file() {
        let result = SimulationConfig::from_file("/nonexistent/config.yaml");
        assert!(result.is_err());
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
}
