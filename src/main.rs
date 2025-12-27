use clap::Parser;
use log::{debug, info};
use simulation_framework::{SimulationConfig, SimulationEngine};
use std::time::Instant;

use simulation_framework::scenario::Scenario;

#[derive(Parser)]
#[command(name = "economic_simulation")]
#[command(about = "Runs an economic simulation with persons, skills, and a market.")]
struct Args {
    #[arg(short, long, default_value_t = SimulationConfig::default().max_steps)]
    steps: usize,

    #[arg(short, long, default_value_t = SimulationConfig::default().entity_count)]
    persons: usize, // Changed from entities to persons for clarity

    #[arg(long, default_value_t = SimulationConfig::default().initial_money_per_person)]
    initial_money: f64,

    #[arg(long, default_value_t = SimulationConfig::default().base_skill_price)]
    base_price: f64,

    #[arg(short, long)]
    output: Option<String>,

    // Rayon will use a default number of threads based on CPU cores if not set.
    // We can remove this CLI arg to simplify, or keep it for advanced users.
    // For now, let's keep it but make it optional.
    #[arg(long)]
    threads: Option<usize>,

    #[arg(long, default_value_t = SimulationConfig::default().seed)]
    seed: u64,

    #[arg(long, default_value_t = Scenario::default())]
    scenario: Scenario,

    /// Disable the progress bar during simulation
    #[arg(long, default_value_t = false)]
    no_progress: bool,

    /// Set the log level (error, warn, info, debug, trace)
    /// Can also be set via RUST_LOG environment variable
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    // If RUST_LOG is not set, use the CLI argument
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &args.log_level);
    }
    env_logger::init();

    if let Some(num_threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()?;
    } else {
        // Initialize Rayon with default number of threads (usually number of logical cores)
        rayon::ThreadPoolBuilder::new().build_global()?;
    }

    let config = SimulationConfig {
        max_steps: args.steps,
        entity_count: args.persons, // Use 'persons' from CLI for entity_count
        time_step: SimulationConfig::default().time_step, // Using default, not exposed via CLI for now
        seed: args.seed,
        initial_money_per_person: args.initial_money,
        base_skill_price: args.base_price,
        scenario: args.scenario,
    };

    info!(
        "Initializing economic simulation with {} persons for {} steps",
        config.entity_count, config.max_steps
    );
    debug!(
        "Configuration: initial_money={}, base_skill_price={}, seed={}, scenario={:?}",
        config.initial_money_per_person, config.base_skill_price, config.seed, config.scenario
    );

    let start_time = Instant::now();
    // SimulationEngine::new will need to be updated to handle the new config and setup persons/market
    let mut engine = SimulationEngine::new(config);
    let show_progress = !args.no_progress;
    let result = engine.run_with_progress(show_progress);
    let duration = start_time.elapsed();

    info!("Simulation completed in {:.2}s", duration.as_secs_f64());
    let steps_per_second = if duration.as_secs_f64() > 0.0 {
        args.steps as f64 / duration.as_secs_f64()
    } else {
        0.0
    };
    info!("Performance: {:.0} steps/second", steps_per_second);

    if let Some(output_path) = args.output {
        // result.save_to_file will need to be adapted for economic data
        result.save_to_file(&output_path)?;
        info!("Results saved to {}", output_path);
    }

    // result.print_summary will need to be adapted for economic data
    result.print_summary();

    Ok(())
}
