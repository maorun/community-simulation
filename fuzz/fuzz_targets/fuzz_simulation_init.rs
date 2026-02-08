#![no_main]

use libfuzzer_sys::fuzz_target;
use community_simulation::{config::SimulationConfig, engine::SimulationEngine, scenario::Scenario};

fuzz_target!(|data: &[u8]| {
    // Fuzzes SimulationEngine initialization with arbitrary numeric inputs
    // This tests the robustness of validation and initialization logic
    
    if data.len() < 48 {
        return;
    }
    
    // Extract fuzzed parameters from the input data
    let max_steps = u16::from_le_bytes([data[0], data[1]]) as usize;
    let entity_count = u16::from_le_bytes([data[2], data[3]]) as usize;
    let seed = u64::from_le_bytes([
        data[4], data[5], data[6], data[7],
        data[8], data[9], data[10], data[11]
    ]);
    let initial_money = f64::from_le_bytes([
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19]
    ]);
    let base_price = f64::from_le_bytes([
        data[20], data[21], data[22], data[23],
        data[24], data[25], data[26], data[27]
    ]);
    let min_price = f64::from_le_bytes([
        data[28], data[29], data[30], data[31],
        data[32], data[33], data[34], data[35]
    ]);
    let time_step = f64::from_le_bytes([
        data[36], data[37], data[38], data[39],
        data[40], data[41], data[42], data[43]
    ]);
    let seasonal_period = u16::from_le_bytes([data[44], data[45]]) as usize;
    let skills_per_person = u16::from_le_bytes([data[46], data[47]]) as usize;
    
    // Skip invalid floating point values (NaN, infinity) to focus on testing
    // the validation logic with realistic but potentially invalid ranges
    if !initial_money.is_finite() || !base_price.is_finite() 
        || !min_price.is_finite() || !time_step.is_finite() {
        return;
    }
    
    // Create config with fuzzed values
    let config = SimulationConfig {
        max_steps,
        entity_count,
        seed,
        initial_money_per_person: initial_money,
        base_skill_price: base_price,
        min_skill_price: min_price,
        time_step,
        scenario: Scenario::Original,
        demand_strategy: Default::default(),
        tech_growth_rate: 0.0,
        seasonal_amplitude: 0.0,
        seasonal_period,
        transaction_fee: 0.0,
        savings_rate: 0.0,
        tax_rate: 0.0,
        enable_tax_redistribution: false,
        skills_per_person,
        enable_friendships: false,
        friendship_probability: 0.0,
        friendship_discount: 0.0,
        enable_loans: false,
        loan_interest_rate: 0.0,
        loan_repayment_period: 1,
        min_money_to_lend: 0.0,
        enable_contracts: false,
        max_contract_duration: 1,
        min_contract_duration: 1,
        contract_price_discount: 0.0,
        enable_black_market: false,
        black_market_price_multiplier: 1.0,
        black_market_participation_rate: 0.0,
        enable_education: false,
        learning_cost_multiplier: 1.0,
        learning_probability: 0.0,
        enable_crisis_events: false,
        crisis_probability: 0.0,
        crisis_severity: 0.0,
        checkpoint_interval: 0,
        checkpoint_file: None,
        resume_from_checkpoint: false,
        stream_output_path: None,
        priority_urgency_weight: 0.5,
        priority_affordability_weight: 0.3,
        priority_efficiency_weight: 0.1,
        priority_reputation_weight: 0.1,
    };
    
    // Try to validate config - this should not panic
    // Store the validation result to avoid redundant computation
    let validation_result = config.validate();
    
    // If validation succeeds, try to create engine - this should also not panic
    if validation_result.is_ok() {
        let _ = SimulationEngine::new(config);
    }
});
