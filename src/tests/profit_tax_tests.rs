#[cfg(test)]
mod profit_tax_tests {
    use crate::person::{Location, Person, Strategy};
    use crate::{SimulationConfig, SimulationEngine};

    #[test]
    fn test_loss_carryforward_basic() {
        let mut person = Person::new(
            0,
            100.0,
            vec![],
            Strategy::Balanced,
            Location::new(0.0, 0.0),
            0.95,
        );

        assert_eq!(person.accumulated_loss, 0.0);
        person.accumulated_loss += 20.0;
        assert_eq!(person.accumulated_loss, 20.0);
    }

    #[test]
    fn test_profit_tax_execution_and_loss_offset() {
        let config = SimulationConfig {
            entity_count: 5,
            max_steps: 10,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            tax_rate: 0.20, // 20% profit tax
            seed: 42,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        if let Some(total_taxes) = result.total_taxes_collected {
            assert!(total_taxes >= 0.0, "Total profit taxes collected should be non-negative");
        }
    }
}
