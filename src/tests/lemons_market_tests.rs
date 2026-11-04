//! Tests for the lemons market exit of high-quality sellers (information asymmetry).

use crate::tests::test_helpers::test_config;
use crate::{SimulationConfig, SimulationEngine, SimulationResult};

fn lemons_config(threshold: usize, certification: bool) -> SimulationConfig {
    let mut config = test_config().entity_count(40).max_steps(60).seed(7).build();
    config.enable_information_asymmetry = true;
    config.inspection_cost = 0.0;
    config.lemons_exit_threshold_steps = threshold;
    config.lemons_exit_quality_margin = 0.2;
    if certification {
        config.enable_certification = true;
        config.certification_probability = 1.0;
        config.certification_duration = None;
        config.certification_cost = 0.1;
    }
    config
}

fn run(config: SimulationConfig) -> SimulationResult {
    let mut engine = SimulationEngine::new(config);
    engine.run()
}

#[test]
fn uncertified_high_quality_sellers_leave_the_market() {
    let result = run(lemons_config(3, false));
    let stats = result.lemons_market_statistics.expect("lemons statistics must exist");

    assert!(stats.exited_sellers > 0, "high-quality sellers must leave the market");
    assert!(stats.avg_true_quality_active_providers.is_finite());
}

#[test]
fn exit_lowers_the_average_true_quality_of_active_providers() {
    let before_run = SimulationEngine::new(lemons_config(3, false));
    // Same weighting as the statistic: true quality of each provider and skill pair.
    let mut baseline_sum = 0.0;
    let mut baseline_count = 0usize;
    for entity in before_run.get_entities() {
        for skill in entity.person_data.all_skills() {
            if let Some(market_skill) = before_run.get_market().skills.get(&skill.id) {
                baseline_sum += market_skill.true_quality;
                baseline_count += 1;
            }
        }
    }
    let baseline_avg = baseline_sum / baseline_count as f64;

    let result = run(lemons_config(3, false));
    let stats = result.lemons_market_statistics.expect("lemons statistics must exist");
    assert!(
        stats.avg_true_quality_active_providers < baseline_avg,
        "active providers {} must have lower quality than all providers {}",
        stats.avg_true_quality_active_providers,
        baseline_avg
    );
}

#[test]
fn certification_keeps_more_high_quality_sellers_in_the_market() {
    let uncertified = run(lemons_config(3, false)).lemons_market_statistics.expect("stats");
    let certified = run(lemons_config(3, true)).lemons_market_statistics.expect("stats");

    assert!(
        certified.exited_sellers < uncertified.exited_sellers,
        "certified exits {} must be below uncertified exits {}",
        certified.exited_sellers,
        uncertified.exited_sellers
    );
    assert!(
        certified.avg_true_quality_active_providers > uncertified.avg_true_quality_active_providers,
        "certified market quality {} must be above uncertified {}",
        certified.avg_true_quality_active_providers,
        uncertified.avg_true_quality_active_providers
    );
}

#[test]
fn threshold_zero_gives_the_same_result_as_without_the_feature() {
    let mut plain = lemons_config(0, false);
    plain.lemons_exit_quality_margin = 0.0;
    let with_margin = lemons_config(0, false);

    let a = run(plain);
    let b = run(with_margin);

    assert!(a.lemons_market_statistics.is_none());
    assert!(b.lemons_market_statistics.is_none());
    assert_eq!(a.trade_volume_statistics.total_trades, b.trade_volume_statistics.total_trades);
    assert_eq!(a.trade_volume_statistics.total_volume, b.trade_volume_statistics.total_volume);
    assert_eq!(a.money_statistics.average, b.money_statistics.average);
}

#[test]
fn lemons_market_run_is_deterministic() {
    let a = run(lemons_config(3, false));
    let b = run(lemons_config(3, false));

    let sa = a.lemons_market_statistics.expect("stats");
    let sb = b.lemons_market_statistics.expect("stats");
    assert_eq!(sa.exited_sellers, sb.exited_sellers);
    assert_eq!(sa.avg_true_quality_active_providers, sb.avg_true_quality_active_providers);
    assert_eq!(a.trade_volume_statistics.total_trades, b.trade_volume_statistics.total_trades);
}

#[test]
fn lemons_exit_margin_must_be_finite_and_non_negative() {
    let mut config = lemons_config(3, false);
    config.lemons_exit_quality_margin = -0.1;
    assert!(config.validate().is_err());
    config.lemons_exit_quality_margin = f64::NAN;
    assert!(config.validate().is_err());
    config.lemons_exit_quality_margin = 0.5;
    assert!(config.validate().is_ok());
}

#[test]
fn positive_threshold_requires_information_asymmetry() {
    let mut config = lemons_config(3, false);
    config.enable_information_asymmetry = false;
    assert!(config.validate().is_err());
}

#[test]
fn disabled_threshold_leaves_the_default_run_unchanged() {
    let mut default_run = test_config().entity_count(40).max_steps(60).seed(7).build();
    default_run.enable_information_asymmetry = true;
    default_run.inspection_cost = 0.0;
    let mut zero_run = default_run.clone();
    zero_run.lemons_exit_threshold_steps = 0;
    zero_run.lemons_exit_quality_margin = 1.5;

    let a = run(default_run);
    let b = run(zero_run);

    assert_eq!(a.trade_volume_statistics.total_volume, b.trade_volume_statistics.total_volume);
    assert_eq!(a.money_statistics.average, b.money_statistics.average);
    assert_eq!(a.final_money_distribution, b.final_money_distribution);
}

#[test]
fn interactive_result_reports_lemons_statistics() {
    let mut engine = SimulationEngine::new(lemons_config(3, false));
    for _ in 0..30 {
        engine.step();
    }
    assert!(engine.get_current_result().lemons_market_statistics.is_some());
}
