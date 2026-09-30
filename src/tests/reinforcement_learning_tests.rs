// Tests for reinforcement learning functionality

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::person::{Strategy, StrategyParameters};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn test_rl_strategy_parameters_initialization() {
    let initial_money = 100.0;
    let params = StrategyParameters::new(initial_money);

    assert_eq!(params.initial_money, 100.0);
    assert_eq!(params.previous_money, 100.0);
    assert_eq!(params.adjustment_factor, 1.0);
    assert_eq!(params.current_epsilon, 0.1); // Default value
    assert_eq!(params.previous_reward, 0.0);
    assert_eq!(params.total_reward, 0.0);
}

#[test]
fn test_rl_update_increases_adjustment_factor_on_positive_reward() {
    let mut params = StrategyParameters::new(100.0);

    let learning_rate = 0.1;
    let discount_factor = 0.9;
    let positive_reward = 0.5;

    params.apply_rl_update(positive_reward, learning_rate, discount_factor);

    // adjustment_factor should increase (1.0 + 0.1 * (0.5 - 0.0) = 1.05)
    assert!(params.adjustment_factor > 1.0);
    assert_eq!(params.previous_reward, positive_reward);
}

#[test]
fn test_rl_update_decreases_adjustment_factor_on_negative_reward() {
    let mut params = StrategyParameters::new(100.0);
    params.previous_reward = 0.5; // Start with positive previous reward

    let learning_rate = 0.1;
    let discount_factor = 0.9;
    let negative_reward = -0.5;

    params.apply_rl_update(negative_reward, learning_rate, discount_factor);

    // adjustment_factor should decrease (1.0 + 0.1 * (-0.5 - 0.5) = 0.9)
    assert!(params.adjustment_factor < 1.0);
}

#[test]
fn test_rl_update_clamps_adjustment_factor() {
    let mut params = StrategyParameters::new(100.0);

    let learning_rate = 1.0; // Very high learning rate
    let discount_factor = 0.9;

    // Try to push adjustment_factor above 2.0
    for _ in 0..10 {
        params.apply_rl_update(10.0, learning_rate, discount_factor);
    }

    // Should be clamped at 2.0
    assert!(params.adjustment_factor <= 2.0);

    // Try to push adjustment_factor below 0.1
    let mut params2 = StrategyParameters::new(100.0);
    params2.previous_reward = 10.0;
    for _ in 0..10 {
        params2.apply_rl_update(-10.0, learning_rate, discount_factor);
    }

    // Should be clamped at 0.1
    assert!(params2.adjustment_factor >= 0.1);
}

#[test]
fn test_reward_calculation_positive_growth() {
    let mut params = StrategyParameters::new(100.0);
    params.update_previous_money(100.0);

    let current_money = 110.0; // 10% growth
    let reputation = 1.0;
    let successful_trades = 2;
    let failed_trades = 0;

    let reward = params.calculate_reward(
        current_money,
        reputation,
        successful_trades,
        failed_trades,
        1.0, // success multiplier
        0.5, // failure multiplier
    );

    // Should be positive due to growth and successful trades
    assert!(reward > 0.0);
}

#[test]
fn test_reward_calculation_negative_growth() {
    let mut params = StrategyParameters::new(100.0);
    params.update_previous_money(100.0);

    let current_money = 90.0; // -10% decline
    let reputation = 1.0;
    let successful_trades = 0;
    let failed_trades = 2;

    let reward = params.calculate_reward(
        current_money,
        reputation,
        successful_trades,
        failed_trades,
        1.0, // success multiplier
        0.5, // failure multiplier
    );

    // Should be negative due to decline and failed trades
    assert!(reward < 0.0);
}

#[test]
fn test_epsilon_decay() {
    let mut params = StrategyParameters::new(100.0);
    params.current_epsilon = 0.5;

    let decay_rate = 0.9;
    params.decay_epsilon(decay_rate);

    // Epsilon should decay to 0.5 * 0.9 = 0.45
    assert!((params.current_epsilon - 0.45).abs() < 1e-10);

    // Decay many times
    for _ in 0..100 {
        params.decay_epsilon(decay_rate);
    }

    // Should not go below minimum (0.01)
    assert!(params.current_epsilon >= 0.01);
}

#[test]
fn test_should_explore() {
    let mut params = StrategyParameters::new(100.0);
    params.current_epsilon = 0.5; // 50% exploration

    let mut rng = StdRng::seed_from_u64(42);

    let mut explore_count = 0;
    let trials = 1000;

    for _ in 0..trials {
        if params.should_explore(&mut rng) {
            explore_count += 1;
        }
    }

    // Should explore roughly 50% of the time (allow some variance)
    let explore_rate = explore_count as f64 / trials as f64;
    assert!(explore_rate > 0.45 && explore_rate < 0.55);
}

#[test]
fn test_rl_enabled_simulation() {
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 5,
        enable_reinforcement_learning: true,
        rl_learning_rate: 0.1,
        rl_epsilon: 0.2,
        rl_epsilon_decay: 0.95,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run simulation
    for _ in 0..10 {
        engine.step();
    }

    // Simulation should complete without errors
    assert_eq!(engine.current_step, 10);
}

#[test]
fn test_rl_disabled_simulation() {
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 5,
        enable_reinforcement_learning: false, // RL disabled
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run simulation
    for _ in 0..10 {
        engine.step();
    }

    // Simulation should complete without errors
    assert_eq!(engine.current_step, 10);
}

#[test]
fn test_rl_reward_accumulation() {
    let mut params = StrategyParameters::new(100.0);

    assert_eq!(params.total_reward, 0.0);

    // Apply several updates
    params.apply_rl_update(0.5, 0.1, 0.9);
    assert!(params.total_reward > 0.0);

    let reward1 = params.total_reward;
    params.apply_rl_update(0.3, 0.1, 0.9);

    // Total reward should accumulate
    assert!(params.total_reward > reward1);
}

#[test]
fn test_rl_with_different_learning_rates() {
    let mut params_fast = StrategyParameters::new(100.0);
    let mut params_slow = StrategyParameters::new(100.0);

    let fast_lr = 0.5;
    let slow_lr = 0.01;
    let reward = 1.0;

    params_fast.apply_rl_update(reward, fast_lr, 0.9);
    params_slow.apply_rl_update(reward, slow_lr, 0.9);

    // Fast learning should change adjustment_factor more
    let fast_change = (params_fast.adjustment_factor - 1.0).abs();
    let slow_change = (params_slow.adjustment_factor - 1.0).abs();

    assert!(fast_change > slow_change);
}

#[test]
fn test_q_values_are_deterministic_per_seed() {
    let first = StrategyParameters::new_with_seed(100.0, 42);
    let second = StrategyParameters::new_with_seed(100.0, 42);
    let different = StrategyParameters::new_with_seed(100.0, 43);

    assert_eq!(first.q_values, second.q_values);
    assert_ne!(first.q_values, different.q_values);
}

#[test]
fn test_q_update_changes_only_each_active_strategy_value() {
    for (active_index, strategy) in Strategy::all_variants().into_iter().enumerate() {
        let mut parameters = StrategyParameters::new_with_seed(100.0, 7);
        let before = parameters.q_values;
        parameters.apply_rl_update_for_strategy(strategy, 1.0, 0.5, 0.9);

        for (index, value) in parameters.q_values.iter().enumerate() {
            if index == active_index {
                assert_ne!(*value, before[index]);
            } else {
                assert_eq!(*value, before[index]);
            }
        }
    }
}

#[test]
fn test_exploitation_selects_highest_value_strategy() {
    let mut parameters = StrategyParameters::new_with_seed(100.0, 7);
    parameters.current_epsilon = 0.0;
    parameters.q_values = [-1.0, 0.25, 2.0, 0.5];
    let mut rng = StdRng::seed_from_u64(99);

    assert_eq!(parameters.select_strategy(&mut rng), Strategy::Aggressive);
}

#[test]
fn test_seeded_tie_breaking_is_reproducible_and_uses_only_best_strategies() {
    let mut parameters = StrategyParameters::new_with_seed(100.0, 7);
    parameters.current_epsilon = 0.0;
    parameters.q_values = [-1.0, 2.0, 2.0, 0.5];
    let mut first_rng = StdRng::seed_from_u64(123);
    let mut second_rng = StdRng::seed_from_u64(123);

    let first: Vec<_> = (0..8).map(|_| parameters.select_strategy(&mut first_rng)).collect();
    let second: Vec<_> = (0..8).map(|_| parameters.select_strategy(&mut second_rng)).collect();

    assert_eq!(first, second);
    assert!(first
        .iter()
        .all(|strategy| matches!(strategy, Strategy::Balanced | Strategy::Aggressive)));
}

#[test]
fn test_strategy_parameters_deserialize_without_q_learning_fields() {
    let parameters = StrategyParameters::new(100.0);
    let mut value = serde_json::to_value(parameters).expect("serialize strategy parameters");
    let object = value.as_object_mut().expect("strategy parameters object");
    object.remove("q_values");
    object.remove("q_seed");

    let restored: StrategyParameters =
        serde_json::from_value(value).expect("deserialize legacy strategy parameters");
    assert_eq!(restored.q_values, [0.0; 4]);
    assert_eq!(restored.q_seed, 0);
}

#[test]
fn test_rl_state_and_trajectory_follow_simulation_seed() {
    fn run(seed: u64) -> (Vec<[f64; 4]>, Vec<Strategy>, Vec<f64>) {
        let config = SimulationConfig {
            seed,
            max_steps: 5,
            entity_count: 6,
            enable_reinforcement_learning: true,
            rl_epsilon: 0.2,
            ..Default::default()
        };
        let mut engine = SimulationEngine::new(config);
        let initial_q_values = engine
            .get_entities()
            .iter()
            .map(|entity| entity.person_data.strategy_params.q_values)
            .collect();
        for _ in 0..5 {
            engine.step();
        }
        let strategies =
            engine.get_entities().iter().map(|entity| entity.person_data.strategy).collect();
        let wealth = engine.get_entities().iter().map(|entity| entity.person_data.money).collect();
        (initial_q_values, strategies, wealth)
    }

    assert_eq!(run(1234), run(1234));
    assert_ne!(run(1234).0, run(1235).0);
}
