#[cfg(test)]
mod tests {
    use crate::config::{PresetName, SimulationConfig};
    use crate::engine::SimulationEngine;
    use crate::person::{Location, Person, Strategy};

    #[test]
    fn test_person_productivity_by_age() {
        let mut person =
            Person::new(1, 100.0, vec![], Strategy::Balanced, Location::new(0.0, 0.0), 0.95);
        person.retirement_age = 65;
        person.max_age = 80;

        // Young adult
        person.age = 10;
        assert!((person.productivity_factor() - 0.75).abs() < 1e-5);

        // Prime working age
        person.age = 40;
        assert_eq!(person.productivity_factor(), 1.0);

        // Retired
        person.age = 65;
        assert_eq!(person.productivity_factor(), 1.0);
        assert!(person.is_retired());

        person.age = 80;
        assert!((person.productivity_factor() - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_demographic_transition_preset() {
        let config = SimulationConfig::from_preset(PresetName::DemographicTransition);
        assert!(config.enable_demographics);
        assert_eq!(config.default_retirement_age, 65);
        assert_eq!(config.default_max_age, 80);
        assert_eq!(config.pension_contribution_rate, 0.08);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_demographic_aging_and_succession() {
        let config = SimulationConfig {
            enable_demographics: true,
            default_retirement_age: 65,
            default_max_age: 80,
            entity_count: 10,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);

        // Set entity near max age
        if let Some(entity) = engine.get_entities_mut().get_mut(0) {
            entity.person_data.age = 79;
            entity.person_data.money = 1000.0;
        }

        // Run one step -> triggers process_demographics which increments age to 80, triggers rebirth & succession
        engine.step();

        // Entity 0 should have reached max_age (80) and been succeeded at age 18
        let entity_0 = &engine.get_entities()[0];
        assert_eq!(entity_0.person_data.age, 18); // Reborn at age 18
        assert!(entity_0.person_data.money < 1000.0); // Money modified by pension deduction & inheritance
    }
}
