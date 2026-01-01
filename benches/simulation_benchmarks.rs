use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use simulation_framework::scenario::Scenario;
use simulation_framework::{SimulationConfig, SimulationEngine};

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
                    };

                    let mut engine = SimulationEngine::new(config);
                    black_box(engine.run());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_engine_initialization,
    bench_single_step,
    bench_full_simulation,
    bench_scenarios
);
criterion_main!(benches);
