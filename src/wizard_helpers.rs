//! Helper functions for the wizard module
//!
//! This module contains pure functions that support the wizard without
//! requiring interactive input/output, making them easier to test.

use crate::scenario::Scenario;

/// Parse a scenario selection string into a Scenario enum
///
/// Converts user-friendly scenario descriptions into the corresponding
/// Scenario enum variant.
///
/// # Arguments
///
/// * `selected` - The scenario description string from user selection
///
/// # Returns
///
/// The corresponding `Scenario` enum variant, defaulting to `Scenario::Original`
/// if the selection doesn't match any known scenario.
///
/// # Examples
///
/// ```
/// use simulation_framework::wizard_helpers::parse_scenario_selection;
/// use simulation_framework::scenario::Scenario;
///
/// assert_eq!(
///     parse_scenario_selection("Original (supply/demand-based)"),
///     Scenario::Original
/// );
/// assert_eq!(
///     parse_scenario_selection("DynamicPricing (sales-based)"),
///     Scenario::DynamicPricing
/// );
/// assert_eq!(
///     parse_scenario_selection("AdaptivePricing (gradual adaptation)"),
///     Scenario::AdaptivePricing
/// );
/// assert_eq!(
///     parse_scenario_selection("AuctionPricing (competitive bidding)"),
///     Scenario::AuctionPricing
/// );
/// assert_eq!(
///     parse_scenario_selection("unknown"),
///     Scenario::Original
/// );
/// ```
pub fn parse_scenario_selection(selected: &str) -> Scenario {
    match selected {
        "Original (supply/demand-based)" => Scenario::Original,
        "DynamicPricing (sales-based)" => Scenario::DynamicPricing,
        "AdaptivePricing (gradual adaptation)" => Scenario::AdaptivePricing,
        "AuctionPricing (competitive bidding)" => Scenario::AuctionPricing,
        _ => Scenario::Original,
    }
}

/// Get available scenario choices as formatted strings
///
/// Returns a list of user-friendly scenario descriptions that can be
/// presented to users for selection.
///
/// # Returns
///
/// A vector of strings describing each available scenario.
///
/// # Examples
///
/// ```
/// use simulation_framework::wizard_helpers::get_scenario_choices;
///
/// let choices = get_scenario_choices();
/// assert_eq!(choices.len(), 4);
/// assert!(choices.contains(&"Original (supply/demand-based)"));
/// ```
pub fn get_scenario_choices() -> Vec<&'static str> {
    vec![
        "Original (supply/demand-based)",
        "DynamicPricing (sales-based)",
        "AdaptivePricing (gradual adaptation)",
        "AuctionPricing (competitive bidding)",
    ]
}

/// Extract preset name from a formatted preset choice string
///
/// Preset choices are formatted as "name: description". This function
/// extracts just the name portion.
///
/// # Arguments
///
/// * `selection` - The formatted preset selection string
///
/// # Returns
///
/// The preset name (everything before the first colon)
///
/// # Examples
///
/// ```
/// use simulation_framework::wizard_helpers::extract_preset_name;
///
/// assert_eq!(
///     extract_preset_name("small_economy: A small economic system"),
///     "small_economy"
/// );
/// assert_eq!(
///     extract_preset_name("quick_test: Fast test configuration"),
///     "quick_test"
/// );
/// assert_eq!(
///     extract_preset_name("no_colon"),
///     "no_colon"
/// );
/// ```
pub fn extract_preset_name(selection: &str) -> &str {
    selection.split(':').next().unwrap_or(selection).trim()
}

/// Determine the default filename based on format choice
///
/// # Arguments
///
/// * `format` - Either "YAML" or "TOML"
///
/// # Returns
///
/// A filename string with the appropriate extension
///
/// # Examples
///
/// ```
/// use simulation_framework::wizard_helpers::get_default_config_filename;
///
/// assert_eq!(
///     get_default_config_filename("YAML"),
///     "simulation_config.yaml"
/// );
/// assert_eq!(
///     get_default_config_filename("TOML"),
///     "simulation_config.toml"
/// );
/// assert_eq!(
///     get_default_config_filename("yaml"),
///     "simulation_config.yaml"
/// );
/// ```
pub fn get_default_config_filename(format: &str) -> String {
    if format.to_uppercase() == "YAML" {
        "simulation_config.yaml".to_string()
    } else {
        "simulation_config.toml".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scenario_selection_all_variants() {
        assert_eq!(
            parse_scenario_selection("Original (supply/demand-based)"),
            Scenario::Original
        );
        assert_eq!(
            parse_scenario_selection("DynamicPricing (sales-based)"),
            Scenario::DynamicPricing
        );
        assert_eq!(
            parse_scenario_selection("AdaptivePricing (gradual adaptation)"),
            Scenario::AdaptivePricing
        );
        assert_eq!(
            parse_scenario_selection("AuctionPricing (competitive bidding)"),
            Scenario::AuctionPricing
        );
    }

    #[test]
    fn test_parse_scenario_selection_unknown_defaults_to_original() {
        assert_eq!(parse_scenario_selection("unknown"), Scenario::Original);
        assert_eq!(parse_scenario_selection(""), Scenario::Original);
        assert_eq!(parse_scenario_selection("random text"), Scenario::Original);
    }

    #[test]
    fn test_get_scenario_choices() {
        let choices = get_scenario_choices();
        assert_eq!(choices.len(), 4);
        assert_eq!(choices[0], "Original (supply/demand-based)");
        assert_eq!(choices[1], "DynamicPricing (sales-based)");
        assert_eq!(choices[2], "AdaptivePricing (gradual adaptation)");
        assert_eq!(choices[3], "AuctionPricing (competitive bidding)");
    }

    #[test]
    fn test_extract_preset_name() {
        assert_eq!(
            extract_preset_name("small_economy: A small economic system"),
            "small_economy"
        );
        assert_eq!(
            extract_preset_name("quick_test: Fast test configuration"),
            "quick_test"
        );
        assert_eq!(extract_preset_name("no_colon"), "no_colon");
        assert_eq!(extract_preset_name("multiple:colons:here"), "multiple");
    }

    #[test]
    fn test_get_default_config_filename() {
        assert_eq!(get_default_config_filename("YAML"), "simulation_config.yaml");
        assert_eq!(get_default_config_filename("TOML"), "simulation_config.toml");
        assert_eq!(get_default_config_filename("yaml"), "simulation_config.yaml");
        assert_eq!(get_default_config_filename("toml"), "simulation_config.toml");
        assert_eq!(get_default_config_filename("YaMl"), "simulation_config.yaml");
    }
}
