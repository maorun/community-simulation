//! Golden-file regression tests for deterministic simulation output.
//!
//! The simulation is deterministic for a fixed RNG seed, so results of a set of
//! representative preset scenarios can be captured as "golden" snapshots. Each
//! snapshot stores a small set of key metrics (not the full JSON output) in
//! `tests/golden/<fixture>.json`. If a refactor silently changes simulation
//! behaviour, these tests fail with a metric-by-metric diff.
//!
//! ## Updating snapshots
//!
//! When a behaviour change is intentional, regenerate the snapshots with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test --test golden_regression
//! ```
//!
//! and review the resulting diff carefully before committing it.

use community_simulation::{PresetName, SimulationConfig, SimulationEngine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Relative tolerance used when comparing floating point metrics.
///
/// The simulation is fully deterministic for a fixed seed, so this tolerance only
/// absorbs last-bit floating point noise (for example from reordered summations
/// after a refactor) on the same platform. It is intentionally tight so that real
/// behavioural drift is reported. Snapshots are recorded on the CI platform
/// (x86_64 Linux); results produced by a different target may legitimately differ
/// by more and then need to be regenerated there.
const RELATIVE_TOLERANCE: f64 = 1e-9;

/// Environment variable used to regenerate the committed snapshots.
const UPDATE_ENV_VAR: &str = "UPDATE_GOLDEN";

/// A fixture describing one deterministic simulation run.
struct Fixture {
    /// File name (without extension) of the golden snapshot.
    name: &'static str,
    /// Preset the simulation configuration is derived from.
    preset: PresetName,
    /// RNG seed used for the run.
    seed: u64,
    /// Number of steps to run (overrides the preset to keep CI runtimes short).
    max_steps: usize,
}

/// The committed golden snapshot format.
#[derive(Debug, Serialize, Deserialize)]
struct GoldenSnapshot {
    preset: String,
    seed: u64,
    max_steps: usize,
    /// Key output metrics, kept in a `BTreeMap` for stable ordering in the file.
    metrics: BTreeMap<String, f64>,
}

/// Build the configuration for a fixture.
fn fixture_config(fixture: &Fixture) -> SimulationConfig {
    SimulationConfig {
        seed: fixture.seed,
        max_steps: fixture.max_steps,
        ..SimulationConfig::from_preset(fixture.preset.clone())
    }
}

/// Run a fixture and extract the key metrics that are tracked in the snapshot.
fn run_fixture(fixture: &Fixture) -> BTreeMap<String, f64> {
    let mut engine = SimulationEngine::new(fixture_config(fixture));
    let result = engine.run();

    let mut metrics = BTreeMap::new();
    metrics.insert("total_steps".to_string(), result.total_steps as f64);
    metrics.insert("active_persons".to_string(), result.active_persons as f64);
    metrics.insert("failed_steps".to_string(), result.failed_steps as f64);

    metrics.insert("money_average".to_string(), result.money_statistics.average);
    metrics.insert("money_median".to_string(), result.money_statistics.median);
    metrics.insert("money_std_dev".to_string(), result.money_statistics.std_dev);
    metrics.insert("money_min".to_string(), result.money_statistics.min_money);
    metrics.insert("money_max".to_string(), result.money_statistics.max_money);
    metrics.insert("gini_coefficient".to_string(), result.money_statistics.gini_coefficient);

    metrics.insert("total_trades".to_string(), result.trade_volume_statistics.total_trades as f64);
    metrics.insert("total_volume".to_string(), result.trade_volume_statistics.total_volume);
    metrics.insert(
        "avg_transaction_value".to_string(),
        result.trade_volume_statistics.avg_transaction_value,
    );
    metrics.insert("total_fees_collected".to_string(), result.total_fees_collected);
    metrics.insert(
        "failed_trade_attempts".to_string(),
        result.failed_trade_statistics.total_failed_attempts as f64,
    );

    if let Some(skill) = result.most_valuable_skill.as_ref() {
        metrics.insert("most_valuable_skill_price".to_string(), skill.price);
    }
    if let Some(skill) = result.least_valuable_skill.as_ref() {
        metrics.insert("least_valuable_skill_price".to_string(), skill.price);
    }

    metrics
}

/// Path of the golden snapshot belonging to a fixture.
fn golden_path(fixture: &Fixture) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{}.json", fixture.name))
}

/// Whether the snapshots should be regenerated instead of compared.
fn update_mode() -> bool {
    matches!(std::env::var(UPDATE_ENV_VAR).as_deref(), Ok("1") | Ok("true"))
}

/// Compare two metric values using a relative tolerance.
fn metrics_match(expected: f64, actual: f64) -> bool {
    if expected == actual {
        return true;
    }
    if !expected.is_finite() || !actual.is_finite() {
        return false;
    }
    let scale = expected.abs().max(actual.abs()).max(1.0);
    (expected - actual).abs() <= RELATIVE_TOLERANCE * scale
}

/// Render a human-readable diff of the mismatching metrics.
fn format_diff(expected: &BTreeMap<String, f64>, actual: &BTreeMap<String, f64>) -> Option<String> {
    let mut lines = Vec::new();

    for (key, expected_value) in expected {
        match actual.get(key) {
            Some(actual_value) if metrics_match(*expected_value, *actual_value) => {},
            Some(actual_value) => lines.push(format!(
                "  {}: expected {} but got {} (delta {:+})",
                key,
                expected_value,
                actual_value,
                actual_value - expected_value
            )),
            None => lines.push(format!("  {}: missing from current output", key)),
        }
    }

    for key in actual.keys() {
        if !expected.contains_key(key) {
            lines.push(format!("  {}: new metric not present in snapshot", key));
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Run a fixture and compare it against (or regenerate) its golden snapshot.
fn check_fixture(fixture: &Fixture) {
    let metrics = run_fixture(fixture);
    let path = golden_path(fixture);

    if update_mode() {
        let snapshot = GoldenSnapshot {
            preset: fixture.preset.as_str().to_string(),
            seed: fixture.seed,
            max_steps: fixture.max_steps,
            metrics,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create golden directory");
        }
        let json =
            serde_json::to_string_pretty(&snapshot).expect("failed to serialize golden snapshot");
        std::fs::write(&path, format!("{}\n", json)).expect("failed to write golden snapshot");
        return;
    }

    let contents = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Missing golden snapshot {}: {}\nRun `{}=1 cargo test --test golden_regression` to create it.",
            path.display(),
            e,
            UPDATE_ENV_VAR
        )
    });
    let snapshot: GoldenSnapshot = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Invalid golden snapshot {}: {}", path.display(), e));

    assert_eq!(
        snapshot.preset,
        fixture.preset.as_str(),
        "Golden snapshot {} was recorded for a different preset",
        path.display()
    );
    assert_eq!(
        snapshot.seed,
        fixture.seed,
        "Golden snapshot {} was recorded with a different seed",
        path.display()
    );
    assert_eq!(
        snapshot.max_steps,
        fixture.max_steps,
        "Golden snapshot {} was recorded with a different step count",
        path.display()
    );

    if let Some(diff) = format_diff(&snapshot.metrics, &metrics) {
        panic!(
            "Golden-file regression for fixture '{}' (preset {}, seed {}):\n{}\n\n\
             If this change is intentional, regenerate the snapshots with \
             `{}=1 cargo test --test golden_regression` and review the diff.",
            fixture.name,
            fixture.preset.as_str(),
            fixture.seed,
            diff,
            UPDATE_ENV_VAR
        );
    }
}

#[test]
fn golden_quick_test() {
    check_fixture(&Fixture {
        name: "quick_test",
        preset: PresetName::QuickTest,
        seed: 42,
        max_steps: 50,
    });
}

#[test]
fn golden_small_economy() {
    check_fixture(&Fixture {
        name: "small_economy",
        preset: PresetName::SmallEconomy,
        seed: 42,
        max_steps: 100,
    });
}

#[test]
fn golden_gig_economy() {
    check_fixture(&Fixture {
        name: "gig_economy",
        preset: PresetName::GigEconomy,
        seed: 42,
        max_steps: 200,
    });
}

#[test]
fn golden_crisis_scenario() {
    check_fixture(&Fixture {
        name: "crisis_scenario",
        preset: PresetName::CrisisScenario,
        seed: 42,
        max_steps: 200,
    });
}

#[test]
fn golden_high_inflation() {
    check_fixture(&Fixture {
        name: "high_inflation",
        preset: PresetName::HighInflation,
        seed: 42,
        max_steps: 200,
    });
}

#[test]
fn golden_runs_are_deterministic() {
    // Guards the premise of the golden-file tests: identical configuration and
    // seed must produce identical metrics.
    let fixture =
        Fixture { name: "quick_test", preset: PresetName::QuickTest, seed: 42, max_steps: 50 };

    let first = run_fixture(&fixture);
    let second = run_fixture(&fixture);

    assert_eq!(format_diff(&first, &second), None, "Repeated runs with the same seed diverged");
}

#[test]
fn golden_different_seeds_produce_different_results() {
    // Ensures the snapshots actually depend on the seed and are not constant.
    let first = run_fixture(&Fixture {
        name: "quick_test",
        preset: PresetName::QuickTest,
        seed: 42,
        max_steps: 50,
    });
    let second = run_fixture(&Fixture {
        name: "quick_test",
        preset: PresetName::QuickTest,
        seed: 1234,
        max_steps: 50,
    });

    assert!(
        format_diff(&first, &second).is_some(),
        "Different seeds unexpectedly produced identical metrics"
    );
}
