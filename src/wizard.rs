/// Interactive configuration wizard for creating simulation scenarios
///
/// This module provides a user-friendly command-line wizard that guides users
/// through creating simulation configurations step-by-step, with validation
/// and dependency checking.
use crate::config::{PresetName, SimulationConfig};
use crate::error::{Result, SimulationError};
use crate::wizard_helpers;
use inquire::{Confirm, CustomType, Select, Text};
use std::path::PathBuf;

/// Run the interactive configuration wizard
///
/// Guides the user through creating a simulation configuration interactively,
/// providing help text, validation, and dependency checking along the way.
///
/// # Returns
///
/// Returns a tuple of `(SimulationConfig, Option<PathBuf>)` where the second element
/// is the path to save the configuration file to (if requested by the user).
///
/// # Examples
///
/// ```no_run
/// use community_simulation::wizard::run_wizard;
///
/// let (config, output_path) = run_wizard()?;
/// println!("Configuration created with {} persons", config.entity_count);
/// # Ok::<(), community_simulation::error::SimulationError>(())
/// ```
pub fn run_wizard() -> Result<(SimulationConfig, Option<PathBuf>)> {
    println!("\n🎯 Interactive Simulation Configuration Wizard");
    println!("═══════════════════════════════════════════════\n");
    println!("This wizard will help you create a simulation configuration.");
    println!("You can always press Ctrl+C to exit.\n");

    // Step 1: Choose between preset or custom configuration
    let start_mode = Select::new(
        "How would you like to start?",
        vec!["Use a preset configuration", "Create a custom configuration from scratch"],
    )
    .prompt()
    .map_err(|e| SimulationError::ValidationError(format!("Failed to get start mode: {}", e)))?;

    let config = if start_mode == "Use a preset configuration" {
        // Show presets with descriptions
        let presets = PresetName::all();
        let preset_choices: Vec<String> =
            presets.iter().map(|p| format!("{}: {}", p.as_str(), p.description())).collect();

        let selected = Select::new("Select a preset:", preset_choices).prompt().map_err(|e| {
            SimulationError::ValidationError(format!("Failed to select preset: {}", e))
        })?;

        // Extract preset name from selection
        let preset_name = wizard_helpers::extract_preset_name(&selected);
        let preset = preset_name
            .parse::<PresetName>()
            .map_err(|e| SimulationError::ValidationError(format!("Invalid preset: {}", e)))?;

        // Load preset and allow customization
        let mut config = SimulationConfig::from_preset(preset.clone());

        println!("\n✅ Loaded preset: {}", preset.description());
        let customize = Confirm::new("Would you like to customize this preset?")
            .with_default(false)
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
            })?;

        if customize {
            customize_config(&mut config)?;
        }

        config
    } else {
        // Create from scratch
        create_custom_config()?
    };

    // Ask if user wants to save to a file
    let save_to_file = Confirm::new("Would you like to save this configuration to a file?")
        .with_default(true)
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get save confirmation: {}", e))
        })?;

    let output_path =
        if save_to_file {
            let format = Select::new("Choose configuration file format:", vec!["YAML", "TOML"])
                .prompt()
                .map_err(|e| {
                    SimulationError::ValidationError(format!("Failed to select format: {}", e))
                })?;

            let default_name = wizard_helpers::get_default_config_filename(format);

            let path = Text::new("Enter file path:").with_default(&default_name).prompt().map_err(
                |e| SimulationError::ValidationError(format!("Failed to get file path: {}", e)),
            )?;

            Some(PathBuf::from(path))
        } else {
            None
        };

    println!("\n✅ Configuration complete!");
    println!("📊 Summary:");
    println!("  - Simulation steps: {}", config.max_steps);
    println!("  - Number of persons: {}", config.entity_count);
    println!("  - Scenario: {:?}", config.scenario);
    println!("  - Seed: {}", config.seed);

    Ok((config, output_path))
}

/// Customize an existing configuration
fn customize_config(config: &mut SimulationConfig) -> Result<()> {
    println!("\n📝 Customization Options");
    println!("═══════════════════════");

    // Basic parameters
    let change_basic = Confirm::new("Would you like to change basic parameters (steps, persons)?")
        .with_default(false)
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    if change_basic {
        config.max_steps = CustomType::<usize>::new("Number of simulation steps:")
            .with_default(config.max_steps)
            .with_error_message("Please enter a valid positive number")
            .prompt()
            .map_err(|e| SimulationError::ValidationError(format!("Failed to get steps: {}", e)))?;

        config.entity_count = CustomType::<usize>::new("Number of persons:")
            .with_default(config.entity_count)
            .with_error_message("Please enter a valid positive number")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get entity count: {}", e))
            })?;

        config.initial_money_per_person = CustomType::<f64>::new("Initial money per person:")
            .with_default(config.initial_money_per_person)
            .with_error_message("Please enter a valid positive number")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get initial money: {}", e))
            })?;
    }

    // Scenario selection
    let change_scenario = Confirm::new("Would you like to change the pricing scenario?")
        .with_default(false)
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    if change_scenario {
        let scenarios = wizard_helpers::get_scenario_choices();

        let selected =
            Select::new("Select pricing scenario:", scenarios).prompt().map_err(|e| {
                SimulationError::ValidationError(format!("Failed to select scenario: {}", e))
            })?;

        config.scenario = wizard_helpers::parse_scenario_selection(selected);
    }

    // Advanced features
    let configure_advanced = Confirm::new("Would you like to configure advanced features?")
        .with_default(false)
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    if configure_advanced {
        configure_advanced_features(config)?;
    }

    Ok(())
}

/// Create a new custom configuration from scratch
#[allow(clippy::field_reassign_with_default)]
fn create_custom_config() -> Result<SimulationConfig> {
    println!("\n🔧 Creating Custom Configuration");
    println!("═══════════════════════════════");

    // Start with default config
    let mut config = SimulationConfig::default();

    // Basic parameters
    config.max_steps = CustomType::<usize>::new("Number of simulation steps:")
        .with_default(500)
        .with_error_message("Please enter a valid positive number")
        .with_help_message(
            "How many steps to simulate (e.g., 500 for standard, 100 for quick test)",
        )
        .prompt()
        .map_err(|e| SimulationError::ValidationError(format!("Failed to get steps: {}", e)))?;

    config.entity_count = CustomType::<usize>::new("Number of persons:")
        .with_default(100)
        .with_error_message("Please enter a valid positive number")
        .with_help_message("How many economic agents to simulate (e.g., 10-1000)")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get entity count: {}", e))
        })?;

    config.initial_money_per_person = CustomType::<f64>::new("Initial money per person:")
        .with_default(100.0)
        .with_error_message("Please enter a valid positive number")
        .with_help_message("Starting wealth for each person")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get initial money: {}", e))
        })?;

    // Scenario selection
    let scenarios = wizard_helpers::get_scenario_choices();

    let selected = Select::new("Select pricing scenario:", scenarios)
        .with_help_message("Different price update mechanisms create different market dynamics")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to select scenario: {}", e))
        })?;

    config.scenario = wizard_helpers::parse_scenario_selection(selected);

    // Ask about advanced features
    let configure_advanced = Confirm::new("Would you like to configure advanced features?")
        .with_default(false)
        .with_help_message("Enable loans, contracts, reputation, quality, etc.")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    if configure_advanced {
        configure_advanced_features(&mut config)?;
    }

    Ok(config)
}

/// Configure advanced features with dependency checking
fn configure_advanced_features(config: &mut SimulationConfig) -> Result<()> {
    println!("\n⚙️  Advanced Features");
    println!("══════════════════");

    // Quality system
    config.enable_quality = Confirm::new("Enable quality rating system?")
        .with_default(config.enable_quality)
        .with_help_message("Skills have quality ratings that affect prices and improve with use")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    // Loan system
    config.enable_loans = Confirm::new("Enable loan system?")
        .with_default(config.enable_loans)
        .with_help_message("Persons can borrow and lend money with interest")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    // Credit rating (depends on loans)
    if config.enable_loans {
        config.enable_credit_rating = Confirm::new("Enable credit rating system?")
            .with_default(config.enable_credit_rating)
            .with_help_message("Credit scores affect loan interest rates (requires loans enabled)")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
            })?;
    }

    // Contract system
    config.enable_contracts = Confirm::new("Enable contract system?")
        .with_default(config.enable_contracts)
        .with_help_message("Long-term agreements with locked prices for stable trading")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    // Friendship system
    config.enable_friendships = Confirm::new("Enable friendship system?")
        .with_default(config.enable_friendships)
        .with_help_message("Social network formation through trading, with price discounts")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    // Trade agreements (depends on friendships)
    if config.enable_friendships {
        config.enable_trade_agreements = Confirm::new("Enable trade agreements?")
            .with_default(config.enable_trade_agreements)
            .with_help_message("Bilateral agreements with mutual discounts (requires friendships)")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
            })?;
    }

    // Tax system (controlled by tax_rate, not a boolean flag)
    let enable_taxes = Confirm::new("Enable tax system?")
        .with_default(config.tax_rate > 0.0)
        .with_help_message("Income tax on trade proceeds with optional redistribution")
        .prompt()
        .map_err(|e| {
            SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
        })?;

    if enable_taxes {
        config.tax_rate = CustomType::<f64>::new("Tax rate (0.0-1.0, e.g., 0.10 for 10%):")
            .with_default(if config.tax_rate > 0.0 {
                config.tax_rate
            } else {
                0.10
            })
            .with_error_message("Please enter a value between 0.0 and 1.0")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get tax rate: {}", e))
            })?;

        config.enable_tax_redistribution = Confirm::new("Enable tax redistribution?")
            .with_default(config.enable_tax_redistribution)
            .with_help_message("Redistribute collected taxes equally among all persons")
            .prompt()
            .map_err(|e| {
                SimulationError::ValidationError(format!("Failed to get confirmation: {}", e))
            })?;
    } else {
        config.tax_rate = 0.0;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_wizard_module_exists() {
        // Basic smoke test to ensure module compiles
        // This test simply verifies the module can be instantiated
    }
}
