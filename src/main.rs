use clap::Parser;
use colored::Colorize;
use log::{debug, info, warn};
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

    /// Minimum price floor for skills (must be positive and ≤ base_price)
    /// Prevents skills from becoming worthless, models price controls/minimum wage
    #[arg(long)]
    min_skill_price: Option<f64>,

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

    /// Technology growth rate per step (e.g., 0.001 = 0.1% per step)
    /// Simulates productivity improvements over time
    #[arg(long)]
    tech_growth_rate: Option<f64>,

    /// Seasonal demand amplitude (0.0 = no seasonality, 0.0-1.0 = variation strength)
    /// Controls strength of seasonal fluctuations in skill demand
    #[arg(long)]
    seasonal_amplitude: Option<f64>,

    /// Seasonal cycle period in simulation steps (default: 100)
    /// Number of steps for one complete seasonal cycle (must be > 0)
    #[arg(long)]
    seasonal_period: Option<usize>,

    /// Transaction fee rate as a percentage (0.0-1.0, e.g., 0.05 = 5% fee)
    /// Fee is deducted from seller's proceeds on each transaction
    #[arg(long)]
    transaction_fee: Option<f64>,

    /// Savings rate as a percentage (0.0-1.0, e.g., 0.05 = 5% savings rate)
    /// Persons save this percentage of their money each step
    #[arg(long)]
    savings_rate: Option<f64>,

    /// Disable the progress bar during simulation
    #[arg(long, default_value_t = false)]
    no_progress: bool,

    /// Set the log level (error, warn, info, debug, trace)
    /// Can also be set via RUST_LOG environment variable
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Disable colored terminal output
    #[arg(long, default_value_t = false)]
    no_color: bool,

    /// Number of Monte Carlo simulation runs with different random seeds
    /// Each run uses seed, seed+1, seed+2, etc. Results are aggregated with statistics
    #[arg(long)]
    monte_carlo_runs: Option<usize>,

    /// Interval (in steps) between automatic checkpoint saves
    /// Set to 0 to disable auto-checkpointing (default)
    #[arg(long)]
    checkpoint_interval: Option<usize>,

    /// Path to the checkpoint file for saving/loading simulation state
    /// Defaults to "checkpoint.json" if not specified
    #[arg(long)]
    checkpoint_file: Option<String>,

    /// Resume the simulation from a previously saved checkpoint
    /// The checkpoint file must exist
    #[arg(long, default_value_t = false)]
    resume: bool,

    /// Run parameter sweep analysis over a parameter range
    /// Format: "parameter:min:max:steps" (e.g., "initial_money:50:150:5")
    /// Available parameters: initial_money, base_price, savings_rate, transaction_fee
    #[arg(long)]
    parameter_sweep: Option<String>,

    /// Number of simulation runs per parameter value in parameter sweep (default: 3)
    /// Each run uses a different random seed for statistical robustness
    #[arg(long)]
    sweep_runs: Option<usize>,

    /// Tax rate as a percentage of trade income (0.0-1.0, e.g., 0.10 = 10% tax)
    /// Tax is deducted from seller's proceeds after transaction fee
    #[arg(long)]
    tax_rate: Option<f64>,

    /// Enable redistribution of collected taxes to all persons
    /// When enabled, taxes are distributed equally among all persons at the end of each step
    #[arg(long, default_value_t = false)]
    enable_tax_redistribution: bool,

    /// Number of skills each person can provide (default: 1)
    /// Higher values create more versatile persons who can participate in multiple markets
    #[arg(long)]
    skills_per_person: Option<usize>,

    /// Path to stream step-by-step simulation data in JSONL (JSON Lines) format
    /// When enabled, simulation appends one JSON object per line after each step
    /// Useful for real-time monitoring and reduced memory usage
    #[arg(long)]
    stream_output: Option<String>,

    /// Compare multiple simulation scenarios to analyze policy effects
    /// Provide comma-separated scenario names (e.g., "Original,DynamicPricing,AdaptivePricing")
    /// Each scenario will be run multiple times with different seeds for statistical robustness
    #[arg(long)]
    compare_scenarios: Option<String>,

    /// Number of simulation runs per scenario in comparison mode (default: 3)
    /// Higher values provide more reliable statistics but take longer to execute
    #[arg(long)]
    comparison_runs: Option<usize>,

    /// Enable contract system for long-term agreements between persons
    /// When enabled, persons can form contracts that lock in prices for multiple steps
    #[arg(long, default_value_t = false)]
    enable_contracts: bool,

    /// Maximum duration for contracts in simulation steps (default: 50)
    /// Determines how long a contract can remain active
    #[arg(long)]
    max_contract_duration: Option<usize>,

    /// Minimum duration for contracts in simulation steps (default: 10)
    /// Contracts must last at least this many steps
    #[arg(long)]
    min_contract_duration: Option<usize>,

    /// Price discount for contract trades as a percentage (0.0-1.0, e.g., 0.05 = 5%)
    /// Contracts offer stability and this discount incentivizes their formation
    #[arg(long)]
    contract_price_discount: Option<f64>,

    /// Enable education system where persons can learn new skills
    /// Persons invest money to learn skills, simulating human capital formation
    #[arg(long, default_value_t = false)]
    enable_education: bool,

    /// Cost multiplier for learning a skill based on market price (e.g., 3.0 = 3x market price)
    /// Only used when --enable-education is set
    #[arg(long)]
    learning_cost_multiplier: Option<f64>,

    /// Probability per step that a person attempts to learn a skill (0.0-1.0, e.g., 0.1 = 10%)
    /// Only used when --enable-education is set
    #[arg(long)]
    learning_probability: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle --no-color flag to disable colored output globally
    if args.no_color {
        colored::control::set_override(false);
    }

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
        let preset = PresetName::from_str(preset_name)
            .map_err(|e| format!("{}. Use --list-presets to see available presets.", e))?;
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
        if let Some(min_skill_price) = args.min_skill_price {
            cfg.min_skill_price = min_skill_price;
        }
        if let Some(scenario) = args.scenario.clone() {
            cfg.scenario = scenario;
        }
        if let Some(tech_growth_rate) = args.tech_growth_rate {
            cfg.tech_growth_rate = tech_growth_rate;
        }
        if let Some(seasonal_amplitude) = args.seasonal_amplitude {
            cfg.seasonal_amplitude = seasonal_amplitude;
        }
        if let Some(seasonal_period) = args.seasonal_period {
            cfg.seasonal_period = seasonal_period;
        }
        if let Some(transaction_fee) = args.transaction_fee {
            cfg.transaction_fee = transaction_fee;
        }
        if let Some(savings_rate) = args.savings_rate {
            cfg.savings_rate = savings_rate;
        }
        if let Some(checkpoint_interval) = args.checkpoint_interval {
            cfg.checkpoint_interval = checkpoint_interval;
        }
        if let Some(checkpoint_file) = &args.checkpoint_file {
            cfg.checkpoint_file = Some(checkpoint_file.clone());
        }
        if args.resume {
            cfg.resume_from_checkpoint = true;
        }
        if let Some(tax_rate) = args.tax_rate {
            cfg.tax_rate = tax_rate;
        }
        if args.enable_tax_redistribution {
            cfg.enable_tax_redistribution = true;
        }
        if let Some(skills_per_person) = args.skills_per_person {
            cfg.skills_per_person = skills_per_person;
        }
        if let Some(stream_output) = &args.stream_output {
            cfg.stream_output_path = Some(stream_output.clone());
        }
        if args.enable_contracts {
            cfg.enable_contracts = true;
        }
        if let Some(max_duration) = args.max_contract_duration {
            cfg.max_contract_duration = max_duration;
        }
        if let Some(min_duration) = args.min_contract_duration {
            cfg.min_contract_duration = min_duration;
        }
        if let Some(discount) = args.contract_price_discount {
            cfg.contract_price_discount = discount;
        }
        if args.enable_education {
            cfg.enable_education = true;
        }
        if let Some(cost_multiplier) = args.learning_cost_multiplier {
            cfg.learning_cost_multiplier = cost_multiplier;
        }
        if let Some(probability) = args.learning_probability {
            cfg.learning_probability = probability;
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
            if let Some(min_skill_price) = args.min_skill_price {
                cfg.min_skill_price = min_skill_price;
            }
            if let Some(scenario) = args.scenario.clone() {
                cfg.scenario = scenario;
            }
            if let Some(tech_growth_rate) = args.tech_growth_rate {
                cfg.tech_growth_rate = tech_growth_rate;
            }
            if let Some(seasonal_amplitude) = args.seasonal_amplitude {
                cfg.seasonal_amplitude = seasonal_amplitude;
            }
            if let Some(seasonal_period) = args.seasonal_period {
                cfg.seasonal_period = seasonal_period;
            }
            if let Some(transaction_fee) = args.transaction_fee {
                cfg.transaction_fee = transaction_fee;
            }
            if let Some(savings_rate) = args.savings_rate {
                cfg.savings_rate = savings_rate;
            }
            if let Some(checkpoint_interval) = args.checkpoint_interval {
                cfg.checkpoint_interval = checkpoint_interval;
            }
            if let Some(checkpoint_file) = &args.checkpoint_file {
                cfg.checkpoint_file = Some(checkpoint_file.clone());
            }
            if args.resume {
                cfg.resume_from_checkpoint = true;
            }
            if let Some(tax_rate) = args.tax_rate {
                cfg.tax_rate = tax_rate;
            }
            if args.enable_tax_redistribution {
                cfg.enable_tax_redistribution = true;
            }
            if let Some(skills_per_person) = args.skills_per_person {
                cfg.skills_per_person = skills_per_person;
            }
            if let Some(stream_output) = &args.stream_output {
                cfg.stream_output_path = Some(stream_output.clone());
            }
            if args.enable_contracts {
                cfg.enable_contracts = true;
            }
            if let Some(max_duration) = args.max_contract_duration {
                cfg.max_contract_duration = max_duration;
            }
            if let Some(min_duration) = args.min_contract_duration {
                cfg.min_contract_duration = min_duration;
            }
            if let Some(discount) = args.contract_price_discount {
                cfg.contract_price_discount = discount;
            }
            if args.enable_education {
                cfg.enable_education = true;
            }
            if let Some(cost_multiplier) = args.learning_cost_multiplier {
                cfg.learning_cost_multiplier = cost_multiplier;
            }
            if let Some(probability) = args.learning_probability {
                cfg.learning_probability = probability;
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
            min_skill_price: args
                .min_skill_price
                .unwrap_or(SimulationConfig::default().min_skill_price),
            scenario: args
                .scenario
                .unwrap_or(SimulationConfig::default().scenario),
            tech_growth_rate: args
                .tech_growth_rate
                .unwrap_or(SimulationConfig::default().tech_growth_rate),
            seasonal_amplitude: args
                .seasonal_amplitude
                .unwrap_or(SimulationConfig::default().seasonal_amplitude),
            seasonal_period: args
                .seasonal_period
                .unwrap_or(SimulationConfig::default().seasonal_period),
            transaction_fee: args
                .transaction_fee
                .unwrap_or(SimulationConfig::default().transaction_fee),
            savings_rate: args
                .savings_rate
                .unwrap_or(SimulationConfig::default().savings_rate),
            enable_loans: SimulationConfig::default().enable_loans,
            loan_interest_rate: SimulationConfig::default().loan_interest_rate,
            loan_repayment_period: SimulationConfig::default().loan_repayment_period,
            min_money_to_lend: SimulationConfig::default().min_money_to_lend,
            checkpoint_interval: args
                .checkpoint_interval
                .unwrap_or(SimulationConfig::default().checkpoint_interval),
            checkpoint_file: args.checkpoint_file.clone(),
            resume_from_checkpoint: args.resume,
            tax_rate: args
                .tax_rate
                .unwrap_or(SimulationConfig::default().tax_rate),
            enable_tax_redistribution: args.enable_tax_redistribution,
            skills_per_person: args
                .skills_per_person
                .unwrap_or(SimulationConfig::default().skills_per_person),
            stream_output_path: args.stream_output.clone(),
            priority_urgency_weight: SimulationConfig::default().priority_urgency_weight,
            priority_affordability_weight: SimulationConfig::default()
                .priority_affordability_weight,
            priority_efficiency_weight: SimulationConfig::default().priority_efficiency_weight,
            priority_reputation_weight: SimulationConfig::default().priority_reputation_weight,
            enable_black_market: SimulationConfig::default().enable_black_market,
            black_market_price_multiplier: SimulationConfig::default()
                .black_market_price_multiplier,
            black_market_participation_rate: SimulationConfig::default()
                .black_market_participation_rate,
            enable_contracts: args.enable_contracts,
            max_contract_duration: args
                .max_contract_duration
                .unwrap_or(SimulationConfig::default().max_contract_duration),
            min_contract_duration: args
                .min_contract_duration
                .unwrap_or(SimulationConfig::default().min_contract_duration),
            contract_price_discount: args
                .contract_price_discount
                .unwrap_or(SimulationConfig::default().contract_price_discount),
            enable_education: args.enable_education,
            learning_cost_multiplier: args
                .learning_cost_multiplier
                .unwrap_or(SimulationConfig::default().learning_cost_multiplier),
            learning_probability: args
                .learning_probability
                .unwrap_or(SimulationConfig::default().learning_probability),
            enable_crisis_events: SimulationConfig::default().enable_crisis_events,
            crisis_probability: SimulationConfig::default().crisis_probability,
            crisis_severity: SimulationConfig::default().crisis_severity,
        }
    };

    // Validate configuration before proceeding
    config.validate()?;

    // Check if scenario comparison mode is enabled
    if let Some(scenario_spec) = args.compare_scenarios {
        let comparison_runs = args.comparison_runs.unwrap_or(3);
        if comparison_runs < 1 {
            return Err("Comparison runs must be at least 1".into());
        }

        run_scenario_comparison(config, &scenario_spec, comparison_runs, args.output)?;
    } else if let Some(sweep_spec) = args.parameter_sweep {
        let sweep_runs = args.sweep_runs.unwrap_or(3);
        if sweep_runs < 1 {
            return Err("Parameter sweep runs must be at least 1".into());
        }

        run_parameter_sweep(config, &sweep_spec, sweep_runs, args.output)?;
    } else if let Some(num_runs) = args.monte_carlo_runs {
        if num_runs < 2 {
            return Err("Monte Carlo runs must be at least 2".into());
        }

        info!(
            "{}",
            format!(
                "Running Monte Carlo simulation: {} runs with {} persons for {} steps each",
                num_runs, config.entity_count, config.max_steps
            )
            .bright_cyan()
        );

        run_monte_carlo(
            config,
            num_runs,
            args.output,
            args.csv_output,
            args.compress,
        )?;
    } else {
        // Single simulation run (original behavior)
        info!(
            "{}",
            format!(
                "Initializing economic simulation with {} persons for {} steps",
                config.entity_count, config.max_steps
            )
            .bright_cyan()
        );
        debug!(
            "Configuration: initial_money={}, base_skill_price={}, seed={}, scenario={:?}",
            config.initial_money_per_person, config.base_skill_price, config.seed, config.scenario
        );

        let start_time = Instant::now();
        let max_steps = config.max_steps; // Store max_steps before moving config

        // Initialize engine - either from checkpoint or fresh start
        let mut engine = if config.resume_from_checkpoint {
            let checkpoint_path = config
                .checkpoint_file
                .clone()
                .unwrap_or_else(|| "checkpoint.json".to_string());

            info!(
                "{}",
                format!("Resuming simulation from checkpoint: {}", checkpoint_path).bright_cyan()
            );

            SimulationEngine::load_checkpoint(&checkpoint_path)
                .map_err(|e| format!("Failed to load checkpoint from {}: {}", checkpoint_path, e))?
        } else {
            SimulationEngine::new(config)
        };

        let show_progress = !args.no_progress;
        let result = engine.run_with_progress(show_progress);
        let duration = start_time.elapsed();

        info!(
            "{}",
            format!("Simulation completed in {:.2}s", duration.as_secs_f64()).bright_green()
        );
        let steps_per_second = if duration.as_secs_f64() > 0.0 {
            max_steps as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        info!(
            "{}",
            format!("Performance: {:.0} steps/second", steps_per_second).bright_yellow()
        );

        if let Some(output_path) = args.output {
            result.save_to_file(&output_path, args.compress)?;
            if args.compress {
                info!(
                    "{}",
                    format!("Compressed results saved to {}.gz", output_path).bright_blue()
                );
            } else {
                info!(
                    "{}",
                    format!("Results saved to {}", output_path).bright_blue()
                );
            }
        }

        if let Some(csv_prefix) = args.csv_output {
            result.save_to_csv(&csv_prefix)?;
            info!(
                "{}",
                format!("CSV results saved with prefix: {}", csv_prefix).bright_blue()
            );
        }

        result.print_summary();
    }

    Ok(())
}

/// Run multiple simulations in parallel with different seeds (Monte Carlo method)
fn run_monte_carlo(
    base_config: SimulationConfig,
    num_runs: usize,
    output: Option<String>,
    csv_output: Option<String>,
    compress: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use rayon::prelude::*;
    use simulation_framework::MonteCarloResult;

    let start_time = Instant::now();
    let base_seed = base_config.seed;

    info!("Starting {} parallel simulation runs...", num_runs);

    // Run simulations in parallel using Rayon
    let results: Vec<_> = (0..num_runs)
        .into_par_iter()
        .map(|run_idx| {
            let mut config = base_config.clone();
            config.seed = base_seed + run_idx as u64;

            info!(
                "Starting run {}/{} (seed: {})",
                run_idx + 1,
                num_runs,
                config.seed
            );

            let mut engine = SimulationEngine::new(config);
            // Disable progress bar for individual runs in Monte Carlo mode
            let result = engine.run_with_progress(false);

            info!("Completed run {}/{}", run_idx + 1, num_runs);
            result
        })
        .collect();

    let total_duration = start_time.elapsed();

    info!(
        "{}",
        format!(
            "All Monte Carlo runs completed in {:.2}s",
            total_duration.as_secs_f64()
        )
        .bright_green()
    );
    info!(
        "{}",
        format!(
            "Average time per run: {:.2}s",
            total_duration.as_secs_f64() / num_runs as f64
        )
        .bright_yellow()
    );

    // Create aggregated results
    let mc_result = MonteCarloResult::from_runs(results, base_seed);

    // Save results if output path specified
    if let Some(output_path) = output {
        mc_result.save_to_file(&output_path, compress)?;
        if compress {
            info!(
                "{}",
                format!("Compressed Monte Carlo results saved to {}.gz", output_path).bright_blue()
            );
        } else {
            info!(
                "{}",
                format!("Monte Carlo results saved to {}", output_path).bright_blue()
            );
        }
    }

    // CSV export not yet supported for Monte Carlo results
    if csv_output.is_some() {
        warn!("CSV export is not yet supported for Monte Carlo results");
    }

    // Print summary
    mc_result.print_summary();

    Ok(())
}

/// Parse and run a parameter sweep analysis
fn run_parameter_sweep(
    base_config: SimulationConfig,
    sweep_spec: &str,
    runs_per_point: usize,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use simulation_framework::ParameterRange;
    use simulation_framework::ParameterSweepResult;

    // Parse sweep specification: "parameter:min:max:steps"
    let parts: Vec<&str> = sweep_spec.split(':').collect();
    if parts.len() != 4 {
        return Err(format!(
            "Invalid parameter sweep format: '{}'. Expected format: 'parameter:min:max:steps'",
            sweep_spec
        )
        .into());
    }

    let parameter_name = parts[0];
    let min: f64 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid min value: '{}'", parts[1]))?;
    let max: f64 = parts[2]
        .parse()
        .map_err(|_| format!("Invalid max value: '{}'", parts[2]))?;
    let steps: usize = parts[3]
        .parse()
        .map_err(|_| format!("Invalid steps value: '{}'", parts[3]))?;

    if steps < 1 {
        return Err("Number of steps must be at least 1".into());
    }

    if min > max {
        return Err(format!("Min value ({}) must be <= max value ({})", min, max).into());
    }

    // Create parameter range based on parameter name
    let parameter_range = match parameter_name {
        "initial_money" => ParameterRange::InitialMoney { min, max, steps },
        "base_price" => ParameterRange::BasePrice { min, max, steps },
        "savings_rate" => ParameterRange::SavingsRate { min, max, steps },
        "transaction_fee" => ParameterRange::TransactionFee { min, max, steps },
        _ => {
            return Err(format!(
                "Unknown parameter: '{}'. Available: initial_money, base_price, savings_rate, transaction_fee",
                parameter_name
            )
            .into());
        }
    };

    info!(
        "{}",
        format!(
            "Starting parameter sweep: {} from {:.2} to {:.2} with {} steps, {} runs per value",
            parameter_name, min, max, steps, runs_per_point
        )
        .bright_cyan()
    );
    info!("Total simulations to run: {}", steps * runs_per_point);

    let start_time = Instant::now();

    // Run the parameter sweep
    let result =
        ParameterSweepResult::run_sweep(base_config, parameter_range, runs_per_point, false);

    let duration = start_time.elapsed();

    info!(
        "{}",
        format!(
            "Parameter sweep completed in {:.2}s ({:.1}s per parameter value)",
            duration.as_secs_f64(),
            duration.as_secs_f64() / steps as f64
        )
        .bright_green()
    );

    // Save results if output path specified
    if let Some(output_path) = output {
        result.save_to_file(&output_path)?;
        info!(
            "{}",
            format!("Parameter sweep results saved to {}", output_path).bright_blue()
        );
    }

    // Print summary
    result.print_summary();

    Ok(())
}

/// Parse and run a scenario comparison analysis
fn run_scenario_comparison(
    base_config: SimulationConfig,
    scenario_spec: &str,
    runs_per_scenario: usize,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use simulation_framework::{Scenario, ScenarioComparisonResult};
    use std::str::FromStr;

    // Parse scenario specification: comma-separated list of scenario names
    let scenario_names: Vec<&str> = scenario_spec.split(',').map(|s| s.trim()).collect();

    if scenario_names.is_empty() {
        return Err("At least one scenario must be provided for comparison".into());
    }

    // Parse each scenario name
    let mut scenarios = Vec::new();
    for name in scenario_names {
        let scenario = Scenario::from_str(name).map_err(|e| {
            format!(
                "Invalid scenario '{}': {}. Available: Original, DynamicPricing, AdaptivePricing",
                name, e
            )
        })?;
        scenarios.push(scenario);
    }

    // Remove duplicates while preserving order
    scenarios.dedup();

    info!(
        "{}",
        format!(
            "Starting scenario comparison: {} scenarios with {} runs each",
            scenarios.len(),
            runs_per_scenario
        )
        .bright_cyan()
    );
    info!(
        "Scenarios to compare: {}",
        scenarios
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<_>>()
            .join(", ")
    );
    info!(
        "Total simulations to run: {}",
        scenarios.len() * runs_per_scenario
    );

    let start_time = Instant::now();

    // Run the scenario comparison
    let result = ScenarioComparisonResult::run(base_config, scenarios, runs_per_scenario)?;

    let duration = start_time.elapsed();

    info!(
        "{}",
        format!(
            "Scenario comparison completed in {:.2}s ({:.1}s per scenario)",
            duration.as_secs_f64(),
            duration.as_secs_f64() / result.scenarios.len() as f64
        )
        .bright_green()
    );

    // Save results if output path specified
    if let Some(output_path) = output {
        result.save_to_file(&output_path)?;
        info!(
            "{}",
            format!("Scenario comparison results saved to {}", output_path).bright_blue()
        );
    }

    // Print summary
    result.print_summary();

    Ok(())
}
