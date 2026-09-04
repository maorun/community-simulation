//! WebAssembly bindings for running simulations in a browser.

use crate::{Scenario, SimulationConfig, SimulationEngine};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct WasmSimulationOptions {
    steps: usize,
    persons: usize,
    seed: u64,
    initial_money: f64,
    base_price: f64,
    scenario: Scenario,
    enable_technology_breakthroughs: bool,
    enable_loans: bool,
    enable_auctions: bool,
    enable_contracts: bool,
    enable_education: bool,
    enable_crisis_events: bool,
    enable_insurance: bool,
}

impl Default for WasmSimulationOptions {
    fn default() -> Self {
        let config = SimulationConfig::default();
        Self {
            steps: config.max_steps,
            persons: config.entity_count,
            seed: config.seed,
            initial_money: config.initial_money_per_person,
            base_price: config.base_skill_price,
            scenario: config.scenario,
            enable_technology_breakthroughs: false,
            enable_loans: false,
            enable_auctions: false,
            enable_contracts: false,
            enable_education: false,
            enable_crisis_events: false,
            enable_insurance: false,
        }
    }
}

impl TryFrom<WasmSimulationOptions> for SimulationConfig {
    type Error = String;

    fn try_from(options: WasmSimulationOptions) -> Result<Self, Self::Error> {
        if options.steps == 0 || options.persons == 0 {
            return Err("'steps' and 'persons' must be greater than zero".to_string());
        }
        if !options.initial_money.is_finite()
            || !options.base_price.is_finite()
            || options.initial_money < 0.0
            || options.base_price <= 0.0
        {
            return Err(
                "'initial_money' must be non-negative and 'base_price' must be positive finite numbers"
                    .to_string(),
            );
        }

        Ok(SimulationConfig {
            max_steps: options.steps,
            entity_count: options.persons,
            seed: options.seed,
            initial_money_per_person: options.initial_money,
            base_skill_price: options.base_price,
            scenario: options.scenario,
            enable_technology_breakthroughs: options.enable_technology_breakthroughs,
            enable_loans: options.enable_loans,
            enable_auctions: options.enable_auctions,
            enable_contracts: options.enable_contracts,
            enable_education: options.enable_education,
            enable_crisis_events: options.enable_crisis_events,
            enable_insurance: options.enable_insurance,
            ..SimulationConfig::default()
        })
    }
}

/// Runs a simulation from a JSON options object and returns its JSON result.
#[wasm_bindgen]
pub fn run_simulation(options_json: &str) -> Result<String, JsValue> {
    run_simulation_json(options_json).map_err(|error| JsValue::from_str(&error))
}

fn run_simulation_json(options_json: &str) -> Result<String, String> {
    let options = serde_json::from_str::<WasmSimulationOptions>(options_json)
        .map_err(|error| format!("Invalid simulation options: {error}"))?;
    let config = SimulationConfig::try_from(options)?;
    serde_json::to_string(&SimulationEngine::new(config).run())
        .map_err(|error| format!("Could not serialize result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_simulation_returns_json_result() {
        let result =
            run_simulation_json(r#"{"steps":2,"persons":3,"seed":42,"scenario":"Original"}"#)
                .unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(result["total_steps"], 2);
        assert_eq!(result["active_persons"], 3);
    }

    #[test]
    fn run_simulation_rejects_zero_persons() {
        assert!(run_simulation_json(r#"{"persons":0}"#).is_err());
    }
}
