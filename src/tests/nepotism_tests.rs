use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::lending_marketplace::LendingMarketplace;
use crate::nepotism::{
    calculate_candidate_score, calculate_social_closeness, select_seller_with_nepotism,
    CandidateSeller, NepotismStats,
};
use crate::trust_network::TrustLevel;

#[test]
fn test_nepotism_disabled_by_default() {
    let config = SimulationConfig { entity_count: 10, max_steps: 10, ..Default::default() };
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    assert!(result.nepotism_statistics.is_none());
}

#[test]
fn test_nepotism_enabled_initialization_and_results() {
    let config = SimulationConfig {
        entity_count: 20,
        max_steps: 15,
        enable_friendships: true,
        enable_nepotism: true,
        nepotism_bias: 0.4,
        nepotism_outgroup_penalty: 0.15,
        ..Default::default()
    };
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    assert!(result.nepotism_statistics.is_some());
    let stats = result.nepotism_statistics.unwrap();
    // Verify stats structure and rate calculations
    assert!(stats.nepotism_rate() >= 0.0 && stats.nepotism_rate() <= 1.0);
    assert!(stats.inner_circle_ratio() >= 0.0 && stats.inner_circle_ratio() <= 1.0);
}

#[test]
fn test_nepotism_preferential_trading_and_welfare_loss() {
    // Construct candidates: candidate 1 is cheap outsider; candidate 2 is expensive friend
    let stranger = CandidateSeller {
        entity_idx: 1,
        person_id: 101,
        effective_price: 10.0,
        reputation: 1.0,
        is_friend: false,
        trust_level: TrustLevel::None,
        social_closeness: 0.0,
        preference_score: calculate_candidate_score(10.0, 1.0, 0.0, 0.5),
    };
    let friend = CandidateSeller {
        entity_idx: 2,
        person_id: 102,
        effective_price: 13.0, // 30% more expensive
        reputation: 1.0,
        is_friend: true,
        trust_level: TrustLevel::Direct,
        social_closeness: 1.0,
        preference_score: calculate_candidate_score(13.0, 1.0, 1.0, 0.5),
    };

    let candidates = vec![stranger, friend];

    // Under nepotism bias 0.5, the friend is chosen despite higher price
    let (selected, is_nepotistic, welfare_loss) =
        select_seller_with_nepotism(&candidates, true, false).unwrap();

    assert_eq!(selected.entity_idx, 2);
    assert_eq!(selected.person_id, 102);
    assert!(is_nepotistic);
    assert_eq!(welfare_loss, 3.0); // 13.0 - 10.0
}

#[test]
fn test_nepotism_meritocratic_zero_bias() {
    let stranger = CandidateSeller {
        entity_idx: 1,
        person_id: 101,
        effective_price: 10.0,
        reputation: 1.0,
        is_friend: false,
        trust_level: TrustLevel::None,
        social_closeness: 0.0,
        preference_score: calculate_candidate_score(10.0, 1.0, 0.0, 0.0),
    };
    let friend = CandidateSeller {
        entity_idx: 2,
        person_id: 102,
        effective_price: 12.0,
        reputation: 1.0,
        is_friend: true,
        trust_level: TrustLevel::Direct,
        social_closeness: 1.0,
        preference_score: calculate_candidate_score(12.0, 1.0, 1.0, 0.0),
    };

    let candidates = vec![stranger, friend];

    // With zero bias, the cheaper candidate wins
    let (selected, is_nepotistic, welfare_loss) =
        select_seller_with_nepotism(&candidates, true, false).unwrap();

    assert_eq!(selected.entity_idx, 1);
    assert!(!is_nepotistic);
    assert_eq!(welfare_loss, 0.0);
}

#[test]
fn test_nepotism_inner_circle_only_exclusivity() {
    let outsider = CandidateSeller {
        entity_idx: 1,
        person_id: 101,
        effective_price: 5.0, // Very cheap outsider
        reputation: 2.0,      // High reputation
        is_friend: false,
        trust_level: TrustLevel::None,
        social_closeness: 0.0,
        preference_score: 999.0,
    };
    let friend = CandidateSeller {
        entity_idx: 2,
        person_id: 102,
        effective_price: 25.0, // Expensive friend
        reputation: 0.8,
        is_friend: true,
        trust_level: TrustLevel::Direct,
        social_closeness: 1.0,
        preference_score: 50.0,
    };

    let candidates = vec![outsider, friend];

    // With inner_circle_only = true, outsider is excluded completely
    let (selected, is_nepotistic, welfare_loss) =
        select_seller_with_nepotism(&candidates, true, true).unwrap();

    assert_eq!(selected.entity_idx, 2);
    assert!(is_nepotistic);
    assert_eq!(welfare_loss, 20.0);
}

#[test]
fn test_nepotism_outgroup_penalty_applied_in_simulation() {
    let config = SimulationConfig {
        entity_count: 20,
        max_steps: 25,
        enable_friendships: true,
        enable_nepotism: true,
        nepotism_bias: 0.3,
        nepotism_outgroup_penalty: 0.20, // 20% outgroup surcharge
        ..Default::default()
    };
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert!(result.nepotism_statistics.is_some());
    let stats = result.nepotism_statistics.unwrap();
    // In a network where friendships are sparse, outgroup penalties should be applied
    assert!(stats.outgroup_penalties_applied > 0);
    assert!(stats.outgroup_trades_count > 0);
}

#[test]
fn test_nepotism_with_trust_networks_transitive_hierarchy() {
    let config = SimulationConfig {
        entity_count: 25,
        max_steps: 30,
        enable_friendships: true,
        enable_trust_networks: true,
        enable_nepotism: true,
        nepotism_bias: 0.5,
        nepotism_outgroup_penalty: 0.15,
        ..Default::default()
    };
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert!(result.nepotism_statistics.is_some());
    assert!(result.trust_network_statistics.is_some());
    let stats = result.nepotism_statistics.unwrap();
    assert!(stats.total_trades_evaluated > 0);
}

#[test]
fn test_nepotism_checkpoint_save_and_restore() {
    let config = SimulationConfig {
        entity_count: 15,
        max_steps: 10,
        enable_friendships: true,
        enable_nepotism: true,
        nepotism_bias: 0.4,
        ..Default::default()
    };
    let mut engine = SimulationEngine::new(config);
    engine.step();
    engine.step();

    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_str().unwrap();

    // Save checkpoint
    engine.save_checkpoint(temp_path).expect("Failed to create checkpoint");

    // Restore checkpoint
    let restored_engine =
        SimulationEngine::load_checkpoint(temp_path).expect("Failed to restore from checkpoint");

    assert!(restored_engine.get_nepotism_stats().is_some());
    assert_eq!(restored_engine.current_step, 2);
}

#[test]
fn test_nepotism_lending_marketplace_matching() {
    let mut marketplace = LendingMarketplace::new(0.01);
    // Lender 1: Stranger, requires 700 score, interest 2%
    marketplace.add_offer(1, 1000.0, 0.02, Some(700), 0);
    // Lender 2: Friend, requires 700 score, interest 3%
    marketplace.add_offer(2, 1000.0, 0.03, Some(700), 0);

    // Borrower has score 650 (below 700)
    // Pure merit rejects borrower
    assert!(marketplace.find_best_offer(650, 500.0).is_none());

    // With nepotism, friend lender 2 accepts borrower with leniency
    let (matched_offer, is_nepotistic) = marketplace
        .find_best_offer_with_nepotism(650, 500.0, true, 0.5, false, |lender_id| {
            if lender_id == 2 {
                1.0
            } else {
                0.0
            }
        })
        .expect("Should match friend lender with leniency");

    assert_eq!(matched_offer.lender_id, 2);
    assert!(is_nepotistic);
}

#[test]
fn test_nepotism_stats_comprehensive_methods() {
    let mut stats = NepotismStats::new();
    assert_eq!(stats.nepotism_rate(), 0.0);
    assert_eq!(stats.inner_circle_ratio(), 0.0);

    stats.record_trade(true, true, 5.0);
    stats.record_trade(false, true, 0.0);
    stats.record_trade(false, false, 0.0);
    stats.record_outgroup_penalty();
    stats.record_nepotistic_loan();

    assert_eq!(stats.total_trades_evaluated, 3);
    assert_eq!(stats.nepotistic_trades_count, 1);
    assert_eq!(stats.merit_trades_count, 2);
    assert_eq!(stats.inner_circle_trades_count, 2);
    assert_eq!(stats.outgroup_trades_count, 1);
    assert_eq!(stats.outgroup_penalties_applied, 1);
    assert_eq!(stats.total_welfare_loss, 5.0);
    assert_eq!(stats.nepotistic_loans_count, 1);

    assert!((stats.nepotism_rate() - (1.0 / 3.0)).abs() < 1e-6);
    assert!((stats.inner_circle_ratio() - (2.0 / 3.0)).abs() < 1e-6);
}

#[test]
fn test_calculate_social_closeness_all_tiers() {
    assert_eq!(calculate_social_closeness(true, TrustLevel::None), 1.0);
    assert_eq!(calculate_social_closeness(false, TrustLevel::Direct), 1.0);
    assert_eq!(calculate_social_closeness(false, TrustLevel::SecondDegree), 0.5);
    assert_eq!(calculate_social_closeness(false, TrustLevel::ThirdDegree), 0.25);
    assert_eq!(calculate_social_closeness(false, TrustLevel::None), 0.0);
}

#[test]
fn test_nepotism_stats_historical_tracking_and_validation() {
    let mut stats = NepotismStats::new();
    assert!(stats.validate().is_ok());

    stats.record_trade_with_step(1, true, true, 10.0);
    stats.record_outgroup_penalty_with_step(2);
    stats.record_nepotistic_loan_with_step(3);

    assert_eq!(stats.history.len(), 3);
    assert_eq!(stats.history[0].step, 1);
    assert_eq!(stats.history[0].event_type, "trade");
    assert!(stats.history[0].is_nepotistic);

    assert_eq!(stats.history[1].step, 2);
    assert_eq!(stats.history[1].event_type, "outgroup_penalty");

    assert_eq!(stats.history[2].step, 3);
    assert_eq!(stats.history[2].event_type, "nepotistic_loan");

    assert!(stats.validate().is_ok());

    // Test validation failure with inconsistent trades count
    let mut invalid_stats = NepotismStats::new();
    invalid_stats.total_trades_evaluated = 5;
    invalid_stats.nepotistic_trades_count = 1;
    invalid_stats.merit_trades_count = 1;
    assert!(invalid_stats.validate().is_err());
}
