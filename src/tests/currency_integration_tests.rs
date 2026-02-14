use crate::currency::{Currency, CurrencySystem};
use crate::tests::test_helpers::test_config;
use crate::SimulationEngine;

#[test]
fn test_currency_system_in_config() {
    // Test that currency system can be configured with multiple currencies
    let mut currency_system = CurrencySystem::default();
    currency_system.add_currency(Currency::new("EUR".to_string(), 1.2));
    currency_system.add_currency(Currency::new("JPY".to_string(), 0.01));

    let config = test_config()
        .max_steps(10)
        .entity_count(5)
        .currency_system(currency_system.clone())
        .build();

    assert_eq!(config.currency_system.currencies.len(), 3); // BASE, EUR, JPY
    assert_eq!(config.currency_system.base_currency_id, "BASE");
}

#[test]
fn test_simulation_with_default_currency() {
    // Test that simulation works with default single currency
    let config = test_config().max_steps(5).entity_count(5).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
    assert_eq!(result.active_persons, 5);

    // Check that all persons have the default BASE currency
    for entity in engine.get_entities().iter() {
        assert_eq!(entity.person_data.currency_id, "BASE");
    }
}

#[test]
fn test_multi_currency_system_integration() {
    // Test that simulation can be configured with multiple currencies
    let mut currency_system = CurrencySystem::default();
    currency_system.add_currency(Currency::new("USD".to_string(), 1.0));
    currency_system.add_currency(Currency::new("EUR".to_string(), 1.2));

    let config = test_config()
        .max_steps(10)
        .entity_count(10)
        .currency_system(currency_system)
        .enable_multi_currency(true)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 10);
    // Simulation should complete successfully with multi-currency enabled
}

#[test]
fn test_currency_conversion_accuracy() {
    // Test currency conversion with various exchange rates
    let mut system = CurrencySystem::default();
    system.add_currency(Currency::new("EUR".to_string(), 1.2));
    system.add_currency(Currency::new("GBP".to_string(), 1.3));
    system.add_currency(Currency::new("JPY".to_string(), 0.01));

    // Test BASE to EUR
    let base_to_eur = system.convert(100.0, "BASE", "EUR");
    assert!(base_to_eur.is_some());
    assert!((base_to_eur.unwrap() - 120.0).abs() < 0.001);

    // Test EUR to GBP
    let eur_to_gbp = system.convert(120.0, "EUR", "GBP");
    assert!(eur_to_gbp.is_some());
    // 120 EUR -> 100 BASE -> 130 GBP
    assert!((eur_to_gbp.unwrap() - 130.0).abs() < 0.001);

    // Test round trip: BASE -> EUR -> BASE
    let base_to_eur = system.convert(100.0, "BASE", "EUR").unwrap();
    let eur_to_base = system.convert(base_to_eur, "EUR", "BASE").unwrap();
    assert!((eur_to_base - 100.0).abs() < 0.001);
}

#[test]
fn test_currency_serialization_in_config() {
    // Test that currency system can be serialized and deserialized
    let mut currency_system = CurrencySystem::default();
    currency_system.add_currency(Currency::new("EUR".to_string(), 1.2));

    let config = test_config()
        .currency_system(currency_system)
        .build();

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("currency_system"));
    assert!(json.contains("EUR"));

    // Deserialize back
    let deserialized: crate::SimulationConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.currency_system.currencies.contains_key("EUR"));
}

#[test]
fn test_person_currency_field() {
    // Test that persons have currency_id field properly initialized
    let config = test_config().entity_count(5).build();
    let engine = SimulationEngine::new(config);

    for entity in engine.get_entities().iter() {
        // Default currency should be "BASE"
        assert_eq!(entity.person_data.currency_id, "BASE");
    }
}

#[test]
fn test_multiple_currencies_different_rates() {
    // Test currency system with diverse exchange rates
    let mut system = CurrencySystem::new(Currency::new("USD".to_string(), 1.0));
    system.add_currency(Currency::new("EUR".to_string(), 1.2));    // Strong currency
    system.add_currency(Currency::new("MXN".to_string(), 0.05));   // Weak currency
    system.add_currency(Currency::new("CHF".to_string(), 1.1));    // Medium currency

    // Test various conversions
    assert_eq!(system.currencies.len(), 4);

    // USD to MXN (strong to weak)
    let usd_to_mxn = system.convert(100.0, "USD", "MXN").unwrap();
    assert!((usd_to_mxn - 5.0).abs() < 0.001); // 100 USD -> 5 MXN

    // MXN to EUR (weak to strong)
    let mxn_to_eur = system.convert(5.0, "MXN", "EUR").unwrap();
    assert!((mxn_to_eur - 120.0).abs() < 0.001); // 5 MXN -> 100 USD -> 120 EUR
}

#[test]
fn test_currency_backward_compatibility() {
    // Test that simulations without multi-currency enabled work as before
    let config = test_config()
        .max_steps(20)
        .entity_count(10)
        .enable_multi_currency(false) // Explicitly disabled
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Should complete successfully
    assert_eq!(result.total_steps, 20);
    assert!(!result.final_money_distribution.is_empty());

    // All persons should have BASE currency
    for person_money in &result.final_money_distribution {
        // Money should be present (may be positive, zero, or negative depending on strategy)
        assert!(person_money.is_finite());
    }
}
