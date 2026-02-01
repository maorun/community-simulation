use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simulation_framework::scenario::Scenario;
use simulation_framework::{SimulationConfig, SimulationEngine};
use std::hint::black_box;

/// Benchmark the simulation engine initialization
fn bench_engine_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_initialization");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let config = SimulationConfig {
                    entity_count: size,
                    max_steps: 100,
                    initial_money_per_person: 100.0,
                    base_skill_price: 10.0,
                    seed: 42,
                    scenario: Scenario::Original,
                    time_step: 1.0,
                    tech_growth_rate: 0.0,
                    ..Default::default()
                };
                black_box(SimulationEngine::new(config));
            });
        });
    }

    group.finish();
}

/// Benchmark a single simulation step
fn bench_single_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_step");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let config = SimulationConfig {
                entity_count: size,
                max_steps: 100,
                initial_money_per_person: 100.0,
                base_skill_price: 10.0,
                seed: 42,
                scenario: Scenario::Original,
                time_step: 1.0,
                tech_growth_rate: 0.0,
                ..Default::default()
            };

            b.iter_batched(
                || SimulationEngine::new(config.clone()),
                |mut engine| {
                    engine.step();
                    black_box(engine);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark a full simulation run
fn bench_full_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_simulation");
    group.sample_size(10); // Reduce sample size for longer benchmarks

    for (persons, steps) in [(10, 50), (50, 50), (100, 100)].iter() {
        let param = format!("{}p_{}s", persons, steps);
        group.bench_with_input(
            BenchmarkId::from_parameter(&param),
            &(*persons, *steps),
            |b, &(p, s)| {
                b.iter(|| {
                    let config = SimulationConfig {
                        entity_count: p,
                        max_steps: s,
                        initial_money_per_person: 100.0,
                        base_skill_price: 10.0,
                        seed: 42,
                        scenario: Scenario::Original,
                        time_step: 1.0,
                        tech_growth_rate: 0.0,
                        ..Default::default()
                    };

                    let mut engine = SimulationEngine::new(config);
                    black_box(engine.run());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different scenarios
fn bench_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios");
    group.sample_size(10);

    for scenario in [Scenario::Original, Scenario::DynamicPricing].iter() {
        let scenario_name = format!("{:?}", scenario);
        let scenario_copy = scenario.clone(); // Clone the scenario for use in closure
        group.bench_with_input(
            BenchmarkId::from_parameter(&scenario_name),
            scenario,
            |b, _scenario| {
                b.iter(|| {
                    let config = SimulationConfig {
                        entity_count: 50,
                        max_steps: 50,
                        initial_money_per_person: 100.0,
                        base_skill_price: 10.0,
                        seed: 42,
                        scenario: scenario_copy.clone(),
                        time_step: 1.0,
                        tech_growth_rate: 0.0,
                        ..Default::default()
                    };

                    let mut engine = SimulationEngine::new(config);
                    black_box(engine.run());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark statistics calculations (Gini, Lorenz curve, Herfindahl index)
fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    // Generate sample wealth data of different sizes
    let small_data: Vec<f64> = (0..100).map(|i| (i as f64) * 1.5 + 50.0).collect();
    let medium_data: Vec<f64> = (0..1000).map(|i| (i as f64) * 1.5 + 50.0).collect();
    let large_data: Vec<f64> = (0..10000).map(|i| (i as f64) * 1.5 + 50.0).collect();

    // Benchmark Gini coefficient calculation
    for (name, data) in [
        ("gini_100", &small_data),
        ("gini_1000", &medium_data),
        ("gini_10000", &large_data),
    ] {
        let sum: f64 = data.iter().sum();
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(simulation_framework::result::calculate_gini_coefficient(&sorted, sum))
            });
        });
    }

    // Benchmark Lorenz curve calculation
    for (name, data) in [("lorenz_100", &small_data), ("lorenz_1000", &medium_data)] {
        let sum: f64 = data.iter().sum();
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(simulation_framework::result::calculate_lorenz_curve(&sorted, sum))
            });
        });
    }

    // Benchmark Herfindahl index calculation
    for (name, data) in [
        ("herfindahl_100", &small_data),
        ("herfindahl_1000", &medium_data),
        ("herfindahl_10000", &large_data),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(simulation_framework::result::calculate_herfindahl_index(data)));
        });
    }

    // Benchmark wealth concentration calculation
    for (name, data) in [
        ("wealth_concentration_100", &small_data),
        ("wealth_concentration_1000", &medium_data),
    ] {
        let sum: f64 = data.iter().sum();
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(simulation_framework::result::calculate_wealth_concentration(
                    &sorted, sum,
                ))
            });
        });
    }

    group.finish();
}

/// Benchmark market operations (price updates, supply/demand tracking)
fn bench_market_operations(c: &mut Criterion) {
    use simulation_framework::scenario::PriceUpdater;
    use simulation_framework::{Market, Skill};

    let mut group = c.benchmark_group("market_operations");

    // Benchmark market creation with different numbers of skills
    for num_skills in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_market", num_skills),
            num_skills,
            |b, &num| {
                b.iter(|| {
                    let price_updater = PriceUpdater::from(Scenario::Original);
                    let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);
                    for i in 0..num {
                        let skill = Skill::new(format!("Skill{}", i), 10.0);
                        market.add_skill(skill);
                    }
                    black_box(market);
                });
            },
        );
    }

    // Benchmark supply/demand tracking
    group.bench_function("supply_demand_tracking", |b| {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);
        for i in 0..100 {
            let skill = Skill::new(format!("Skill{}", i), 10.0);
            market.add_skill(skill);
        }

        b.iter(|| {
            for i in 0..100 {
                let skill_id = format!("Skill{}", i);
                market.increment_skill_supply(&skill_id);
                market.increment_demand(&skill_id);
            }
            black_box(&market);
        });
    });

    group.finish();
}

/// Benchmark result serialization (JSON output)
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Create a simulation result to serialize
    let config = SimulationConfig {
        entity_count: 100,
        max_steps: 100,
        initial_money_per_person: 100.0,
        base_skill_price: 10.0,
        seed: 42,
        scenario: Scenario::Original,
        time_step: 1.0,
        tech_growth_rate: 0.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Benchmark JSON serialization
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&result).unwrap();
            black_box(json);
        });
    });

    // Benchmark JSON pretty serialization
    group.bench_function("json_serialize_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(&result).unwrap();
            black_box(json);
        });
    });

    group.finish();
}

/// Benchmark incremental statistics calculator
fn bench_incremental_stats(c: &mut Criterion) {
    use simulation_framework::result::IncrementalStats;

    let mut group = c.benchmark_group("incremental_stats");

    // Benchmark updating stats with different data sizes
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut stats = IncrementalStats::new();
                for i in 0..size {
                    stats.update(i as f64 * 1.5 + 50.0);
                }
                black_box((stats.mean(), stats.variance(), stats.std_dev()));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_engine_initialization,
    bench_single_step,
    bench_full_simulation,
    bench_scenarios,
    bench_statistics,
    bench_market_operations,
    bench_serialization,
    bench_incremental_stats
);
criterion_main!(benches);
