//! Regression tests: custom recipes from the configuration must reach the production system.

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::error::SimulationError;

const OUTPUT: &str = "CustomOutput";

fn config_toml(with_recipe: bool) -> String {
    let mut toml = String::from(
        "entity_count = 10\nmax_steps = 30\nseed = 42\nskills_per_person = 3\n\
         initial_money_per_person = 1000.0\nenable_production = true\n\
         production_probability = 1.0\n",
    );
    if with_recipe {
        toml.push_str(&format!(
            "\n[[custom_recipes]]\ninput_skill_1 = \"Skill0\"\ninput_skill_2 = \"Skill1\"\n\
             output_skill = \"{OUTPUT}\"\ncost_multiplier = 1.5\nbatch_size = 3\n\
             yield_multiplier = 2.0\n"
        ));
    }
    toml
}

fn load_from_temp_file(contents: &str, name: &str) -> SimulationConfig {
    let path =
        std::env::temp_dir().join(format!("custom_recipes_{}_{}.toml", name, std::process::id()));
    std::fs::write(&path, contents).expect("temp file must be writable");
    let result = SimulationConfig::from_file(&path);
    let _ = std::fs::remove_file(&path);
    result.expect("config must load")
}

/// Runs the engine after person 0 receives both input skills of the custom recipe.
fn run_with_skill_owner(config: SimulationConfig) -> crate::result::SimulationResult {
    let mut engine = SimulationEngine::new(config);
    let price = engine.get_config().base_skill_price;
    let person = &mut engine.get_entities_mut()[0].person_data;
    person.own_skills = vec![
        crate::skill::Skill::new("Skill0".to_string(), price),
        crate::skill::Skill::new("Skill1".to_string(), price),
    ];
    engine.run()
}

#[test]
fn toml_custom_recipe_changes_production_result() {
    let default_config = load_from_temp_file(&config_toml(false), "default");
    let custom_config = load_from_temp_file(&config_toml(true), "custom");
    assert!(default_config.custom_recipes.is_empty());
    assert_eq!(custom_config.custom_recipes.len(), 1);
    assert_eq!(custom_config.custom_recipes[0].batch_size, 3);
    assert!(custom_config.validate().is_ok());

    let default_result = run_with_skill_owner(default_config);
    let custom_result = run_with_skill_owner(custom_config);

    let has_output =
        |r: &crate::result::SimulationResult| r.final_skill_prices.iter().any(|s| s.id == OUTPUT);
    assert!(!has_output(&default_result));
    assert!(has_output(&custom_result));
    assert_ne!(default_result.final_money_distribution, custom_result.final_money_distribution);
}

#[test]
fn invalid_custom_recipe_fails_validation() {
    let mut config = load_from_temp_file(&config_toml(true), "invalid");
    config.custom_recipes[0].batch_size = 0;
    let error = config.validate().expect_err("batch_size 0 must be rejected");
    assert!(matches!(error, SimulationError::ValidationError(_)));
    assert!(error.to_string().contains("custom_recipes[0]"));
}

#[test]
fn custom_recipes_require_enable_production() {
    let mut config = load_from_temp_file(&config_toml(true), "disabled");
    config.enable_production = false;
    assert!(matches!(config.validate(), Err(SimulationError::ValidationError(_))));
}

#[test]
fn old_config_without_field_loads_with_empty_list() {
    let config: SimulationConfig =
        toml::from_str("entity_count = 5\nmax_steps = 10\n").expect("old config must parse");
    assert!(config.custom_recipes.is_empty());
}
