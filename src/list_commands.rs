//! List commands for displaying available presets and scenarios
//!
//! This module provides functions for listing presets and scenarios
//! that are available in the simulation framework.

use crate::config::{PresetName, SimulationConfig};
use crate::scenario::Scenario;

/// List all available preset configurations with their details
///
/// Prints a formatted list of all available presets including their
/// descriptions and key configuration parameters.
///
/// # Examples
///
/// ```no_run
/// use community_simulation::list_commands::list_presets;
///
/// list_presets().expect("Failed to list presets");
/// ```
pub fn list_presets() -> Result<(), Box<dyn std::error::Error>> {
    println!("Available preset configurations:\n");
    for preset in PresetName::all() {
        let config = SimulationConfig::from_preset(preset.clone());
        println!("  {}", preset.as_str());
        println!("    Description: {}", preset.description());
        println!(
            "    Parameters: {} persons, {} steps, ${:.0} initial money, ${:.0} base price, scenario: {:?}",
            config.entity_count,
            config.max_steps,
            config.initial_money_per_person,
            config.base_skill_price,
            config.scenario
        );
        println!();
    }
    Ok(())
}

/// List all available pricing scenarios with their details
///
/// Prints a formatted list of all available scenarios including their
/// descriptions, mechanisms, and use cases.
///
/// # Examples
///
/// ```no_run
/// use community_simulation::list_commands::list_scenarios;
///
/// list_scenarios().expect("Failed to list scenarios");
/// ```
pub fn list_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("Available pricing scenarios:\n");

    for scenario in Scenario::all() {
        let default_marker = if scenario.is_default() {
            " (default)"
        } else {
            ""
        };
        println!("  {}{}", scenario, default_marker);
        println!("    Description: {}", scenario.description());
        println!("    Mechanism: {}", scenario.mechanism());
        println!("    Best for: {}\n", scenario.use_case());
    }

    println!("Usage: community_simulation run --scenario <SCENARIO>");
    println!("Example: community_simulation run --scenario AdaptivePricing -s 500 -p 100");

    Ok(())
}

/// Format preset information as a string
///
/// Creates a formatted string representation of a preset's details
/// for display or testing purposes.
///
/// # Arguments
///
/// * `preset` - The preset to format
///
/// # Returns
///
/// A string containing the preset's formatted information
///
/// # Examples
///
/// ```
/// use community_simulation::list_commands::format_preset_info;
/// use community_simulation::config::PresetName;
///
/// let info = format_preset_info(&PresetName::SmallEconomy);
/// assert!(info.contains("small_economy"));
/// assert!(info.contains("Description:"));
/// ```
pub fn format_preset_info(preset: &PresetName) -> String {
    let config = SimulationConfig::from_preset(preset.clone());
    format!(
        "  {}\n    Description: {}\n    Parameters: {} persons, {} steps, ${:.0} initial money, ${:.0} base price, scenario: {:?}\n",
        preset.as_str(),
        preset.description(),
        config.entity_count,
        config.max_steps,
        config.initial_money_per_person,
        config.base_skill_price,
        config.scenario
    )
}

/// Format scenario information as a string
///
/// Creates a formatted string representation of a scenario's details
/// for display or testing purposes.
///
/// # Arguments
///
/// * `scenario` - The scenario to format
///
/// # Returns
///
/// A string containing the scenario's formatted information
///
/// # Examples
///
/// ```
/// use community_simulation::list_commands::format_scenario_info;
/// use community_simulation::scenario::Scenario;
///
/// let info = format_scenario_info(&Scenario::Original);
/// assert!(info.contains("Original"));
/// assert!(info.contains("Description:"));
/// ```
pub fn format_scenario_info(scenario: &Scenario) -> String {
    let default_marker = if scenario.is_default() {
        " (default)"
    } else {
        ""
    };
    format!(
        "  {}{}\n    Description: {}\n    Mechanism: {}\n    Best for: {}\n",
        scenario,
        default_marker,
        scenario.description(),
        scenario.mechanism(),
        scenario.use_case()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_preset_info_contains_key_elements() {
        let info = format_preset_info(&PresetName::SmallEconomy);
        assert!(info.contains("small_economy"));
        assert!(info.contains("Description:"));
        assert!(info.contains("persons"));
        assert!(info.contains("steps"));
    }

    #[test]
    fn test_format_scenario_info_contains_key_elements() {
        let info = format_scenario_info(&Scenario::Original);
        assert!(info.contains("Original"));
        assert!(info.contains("Description:"));
        assert!(info.contains("Mechanism:"));
        assert!(info.contains("Best for:"));
    }

    #[test]
    fn test_format_scenario_info_shows_default_marker() {
        let info = format_scenario_info(&Scenario::Original);
        assert!(info.contains("(default)"));
    }

    #[test]
    fn test_format_scenario_info_no_default_marker_for_non_default() {
        let info = format_scenario_info(&Scenario::DynamicPricing);
        assert!(!info.contains("(default)"));
    }

    #[test]
    fn test_all_presets_can_be_formatted() {
        for preset in PresetName::all() {
            let info = format_preset_info(&preset);
            assert!(!info.is_empty());
            assert!(info.contains("Description:"));
        }
    }

    #[test]
    fn test_all_scenarios_can_be_formatted() {
        for scenario in Scenario::all() {
            let info = format_scenario_info(&scenario);
            assert!(!info.is_empty());
            assert!(info.contains("Description:"));
        }
    }

    #[test]
    fn test_list_presets_executes_without_error() {
        // list_presets prints to stdout, so we just verify it doesn't panic or return error
        let result = list_presets();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_scenarios_executes_without_error() {
        // list_scenarios prints to stdout, so we just verify it doesn't panic or return error
        let result = list_scenarios();
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_preset_info_includes_all_parameters() {
        let info = format_preset_info(&PresetName::Default);
        // Verify all key information is present
        assert!(info.contains("default"));
        assert!(info.contains("Description:"));
        assert!(info.contains("Parameters:"));
        assert!(info.contains("persons"));
        assert!(info.contains("steps"));
        assert!(info.contains("initial money"));
        assert!(info.contains("base price"));
        assert!(info.contains("scenario:"));
    }

    #[test]
    fn test_format_preset_info_different_presets() {
        // Test a few different presets to ensure formatting works for all
        let presets = vec![PresetName::Default, PresetName::SmallEconomy, PresetName::LargeEconomy];

        for preset in presets {
            let info = format_preset_info(&preset);
            assert!(!info.is_empty());
            assert!(info.contains(preset.as_str()));
            assert!(info.contains("Description:"));
            // Each preset should have numeric values
            assert!(info.chars().any(|c| c.is_numeric()));
        }
    }

    #[test]
    fn test_format_scenario_info_includes_all_fields() {
        let info = format_scenario_info(&Scenario::AdaptivePricing);
        // Verify all key information is present
        assert!(info.contains("AdaptivePricing"));
        assert!(info.contains("Description:"));
        assert!(info.contains("Mechanism:"));
        assert!(info.contains("Best for:"));
    }

    #[test]
    fn test_format_scenario_info_different_scenarios() {
        // Test all available scenarios
        let scenarios = Scenario::all();
        assert!(!scenarios.is_empty(), "Should have at least one scenario");

        for scenario in scenarios {
            let info = format_scenario_info(&scenario);
            assert!(!info.is_empty());
            assert!(info.contains("Description:"));
            assert!(info.contains("Mechanism:"));
            assert!(info.contains("Best for:"));
        }
    }

    #[test]
    fn test_format_preset_info_output_structure() {
        let info = format_preset_info(&PresetName::SmallEconomy);
        // Verify the output has the expected structure with proper indentation
        let lines: Vec<&str> = info.lines().collect();
        assert!(lines.len() >= 2, "Should have at least 2 lines");
        // First line should start with spaces (indentation)
        assert!(lines[0].starts_with("  "));
    }

    #[test]
    fn test_format_scenario_info_output_structure() {
        let info = format_scenario_info(&Scenario::Original);
        // Verify the output has the expected structure
        let lines: Vec<&str> = info.lines().collect();
        assert!(lines.len() >= 4, "Should have at least 4 lines");
        // Lines should contain proper field labels
        assert!(info.contains("Description:"));
        assert!(info.contains("Mechanism:"));
        assert!(info.contains("Best for:"));
    }

    #[test]
    fn test_only_default_scenario_has_default_marker() {
        let scenarios = Scenario::all();
        let mut default_count = 0;

        for scenario in scenarios {
            let info = format_scenario_info(&scenario);
            if info.contains("(default)") {
                default_count += 1;
                assert!(scenario.is_default(), "Only default scenario should have default marker");
            }
        }

        assert_eq!(default_count, 1, "Exactly one scenario should be marked as default");
    }
}
