use clap::Parser;
use log::{debug, info};
use simulation_framework::{PresetName, SimulationConfig, SimulationEngine};
use std::str::FromStr;
use std::time::Instant;

use simulation_framework::scenario::Scenario;

#[derive(Parser)]
#[command(name = "economic_simulation")]
#[command(about = "Runs an economic simulation with persons, skills, and a market.")]
struct Args {
    /// Path to configuration file (YAML or TOML). CLI arguments override config file values.
    #[arg(short, long)]
    config: Option<String>,

    /// Use a preset configuration (e.g., 'small_economy', 'crisis_scenario', 'quick_test')
    /// Use --list-presets to see all available presets
    #[arg(long)]
    preset: Option<String>,

    /// List all available preset configurations and exit
    #[arg(long, default_value_t = false)]
    list_presets: bool,

    #[arg(short, long)]
    steps: Option<usize>,

    #[arg(short, long)]
    persons: Option<usize>, // Changed from entities to persons for clarity

    #[arg(long)]
    initial_money: Option<f64>,

    #[arg(long)]
    base_price: Option<f64>,

    #[arg(short, long)]
    output: Option<String>,

    /// Path prefix for CSV output files (creates multiple .csv files with this prefix)
    #[arg(long)]
    csv_output: Option<String>,

    /// Compress JSON output using gzip (.gz extension will be added automatically)
    #[arg(long, default_value_t = false)]
    compress: bool,

    // Rayon will use a default number of threads based on CPU cores if not set.
    // We can remove this CLI arg to simplify, or keep it for advanced users.
    // For now, let's keep it but make it optional.
    #[arg(long)]
    threads: Option<usize>,

    #[arg(long)]
    seed: Option<u64>,

    #[arg(long)]
    scenario: Option<Scenario>,

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

    // Handle --list-presets flag
    if args.list_presets {
        println!("Available preset configurations:\n");
        for preset in PresetName::all() {
            let config = SimulationConfig::from_preset(preset.clone());
            println!("  {}", preset.as_str());
            println!("    Description: {}", preset.description());
            println!(
                "    Parameters: {} persons, {} steps, ${:.0} initial money, ${:.0} base price, scenario: {:?}",
                config.entity_count,
                config.max_steps,
                config.initial_money_per_person,
                config.base_skill_price,
                config.scenario
            );
            println!();
        }
        return Ok(());
    }

    if let Some(num_threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()?;
    } else {
        // Initialize Rayon with default number of threads (usually number of logical cores)
        rayon::ThreadPoolBuilder::new().build_global()?;
    }

    // Load configuration: priority order is preset -> file -> CLI arguments
    // CLI arguments only override preset/file values when explicitly provided
    let config = if let Some(preset_name) = &args.preset {
        // Load from preset
        let preset = PresetName::from_str(preset_name).map_err(|e| {
            format!(
                "{}. Use --list-presets to see available presets.",
                e
            )
        })?;
        info!("Loading preset configuration: {}", preset.as_str());
        let mut cfg = SimulationConfig::from_preset(preset);
        // Apply CLI overrides only if provided
        if let Some(steps) = args.steps {
            cfg.max_steps = steps;
        }
        if let Some(persons) = args.persons {
            cfg.entity_count = persons;
        }
        if let Some(seed) = args.seed {
            cfg.seed = seed;
        }
        if let Some(initial_money) = args.initial_money {
            cfg.initial_money_per_person = initial_money;
        }
        if let Some(base_price) = args.base_price {
            cfg.base_skill_price = base_price;
        }
        if let Some(scenario) = args.scenario.clone() {
            cfg.scenario = scenario;
        }
        cfg
    } else if let Some(config_path) = &args.config {
        info!("Loading configuration from: {}", config_path);
        SimulationConfig::from_file_with_overrides(config_path, |cfg| {
            // CLI arguments only override config file values when provided
            if let Some(steps) = args.steps {
                cfg.max_steps = steps;
            }
            if let Some(persons) = args.persons {
                cfg.entity_count = persons;
            }
            if let Some(seed) = args.seed {
                cfg.seed = seed;
            }
            if let Some(initial_money) = args.initial_money {
                cfg.initial_money_per_person = initial_money;
            }
            if let Some(base_price) = args.base_price {
                cfg.base_skill_price = base_price;
            }
            if let Some(scenario) = args.scenario.clone() {
                cfg.scenario = scenario;
            }
        })?
    } else {
        // No config file or preset, use CLI arguments or defaults
        SimulationConfig {
            max_steps: args.steps.unwrap_or(SimulationConfig::default().max_steps),
            entity_count: args
                .persons
                .unwrap_or(SimulationConfig::default().entity_count),
            time_step: SimulationConfig::default().time_step,
            seed: args.seed.unwrap_or(SimulationConfig::default().seed),
            initial_money_per_person: args
                .initial_money
                .unwrap_or(SimulationConfig::default().initial_money_per_person),
            base_skill_price: args
                .base_price
                .unwrap_or(SimulationConfig::default().base_skill_price),
            scenario: args
                .scenario
                .unwrap_or(SimulationConfig::default().scenario),
        }
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
    let max_steps = config.max_steps; // Store max_steps before moving config
    // SimulationEngine::new will need to be updated to handle the new config and setup persons/market
    let mut engine = SimulationEngine::new(config);
    let show_progress = !args.no_progress;
    let result = engine.run_with_progress(show_progress);
    let duration = start_time.elapsed();

    info!("Simulation completed in {:.2}s", duration.as_secs_f64());
    let steps_per_second = if duration.as_secs_f64() > 0.0 {
        max_steps as f64 / duration.as_secs_f64()
    } else {
        0.0
    };
    info!("Performance: {:.0} steps/second", steps_per_second);

    if let Some(output_path) = args.output {
        // result.save_to_file will need to be adapted for economic data
        result.save_to_file(&output_path, args.compress)?;
        if args.compress {
            info!("Compressed results saved to {}.gz", output_path);
        } else {
            info!("Results saved to {}", output_path);
        }
    }

    if let Some(csv_prefix) = args.csv_output {
        result.save_to_csv(&csv_prefix)?;
        info!("CSV results saved with prefix: {}", csv_prefix);
    }

    // result.print_summary will need to be adapted for economic data
    result.print_summary();

    Ok(())
}
