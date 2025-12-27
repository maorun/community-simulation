#[cfg(test)]
mod engine_tests {
    use crate::{scenario::Scenario, SimulationConfig, SimulationEngine};

    fn get_test_config() -> SimulationConfig {
        SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 50.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
        }
    }

    #[test]
    fn test_simulation_engine_new() {
        let config = get_test_config();
        let engine = SimulationEngine::new(config);

        assert_eq!(engine.get_active_entity_count(), 10);
        assert_eq!(engine.current_step, 0);
    }

    #[test]
    fn test_simulation_engine_step() {
        let config = get_test_config();
        let mut engine = SimulationEngine::new(config);

        engine.step();

        assert_eq!(engine.current_step, 1);
        // Further assertions can be added to check the state of entities and the market
    }

    #[test]
    fn test_simulation_engine_run() {
        let mut config = get_test_config();
        config.max_steps = 2; // Keep the test fast
        let mut engine = SimulationEngine::new(config);

        let result = engine.run();

        assert_eq!(result.total_steps, 2);
        assert_eq!(engine.current_step, 2);
        assert!(!result.final_money_distribution.is_empty());
    }
}
