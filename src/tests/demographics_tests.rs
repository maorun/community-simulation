use crate::{
    person::{Location, Person, Strategy},
    skill::Skill,
    PresetName, SimulationConfig, SimulationEngine,
};

#[test]
fn test_person_aging_and_productivity_factor() {
    let skill = Skill::new("Programming".to_string(), 50.0);
    let mut person = Person::new(
        1,
        100.0,
        vec![skill],
        Strategy::Balanced,
        Location::new(10.0, 10.0),
        0.95,
    );

    // Initial default age is 20
    assert_eq!(person.age, 20);
    assert_eq!(person.retirement_age, 65);
    assert_eq!(person.max_age, 80);
    assert!(!person.is_retired());
    assert_eq!(person.productivity_factor(), 1.0);

    // Youth (< 20)
    person.age = 10;
    assert_eq!(person.productivity_factor(), 0.75); // 0.5 + 0.5 * (10 / 20)

    // Retirement age
    person.age = 65;
    assert!(person.is_retired());
    assert_eq!(person.productivity_factor(), 1.0);

    // Old age (towards max age)
    person.age = 80;
    assert_eq!(person.productivity_factor(), 0.2); // Floored at 0.2
}

#[test]
fn test_engine_demographics_pension_and_aging() {
    let mut config = SimulationConfig::default();
    config.enable_demographics = true;
    config.entity_count = 10;
    config.max_steps = 10;
    config.default_retirement_age = 65;
    config.default_max_age = 80;
    config.pension_contribution_rate = 0.05;

    let mut engine = SimulationEngine::new(config);

    // Initial age is 20
    let entities = engine.get_entities();
    assert_eq!(entities[0].person_data.age, 20);

    // Run 5 steps
    for _ in 0..5 {
        engine.step();
    }

    // Ages should increase by 5
    let updated_entities = engine.get_entities();
    assert_eq!(updated_entities[0].person_data.age, 25);
}

#[test]
fn test_engine_demographics_preset_and_succession() {
    let mut config = SimulationConfig::from_preset(PresetName::DemographicTransition);
    config.entity_count = 5;
    config.max_steps = 10;

    let mut engine = SimulationEngine::new(config);
    assert!(engine.get_config().enable_demographics);

    let result = engine.run();
    assert_eq!(result.total_steps, 10);
    assert_eq!(result.active_persons, 5);
}
