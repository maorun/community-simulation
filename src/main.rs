use clap::{Parser, Subcommand};
use colored::Colorize;
use community_simulation::{PresetName, SimulationConfig, SimulationEngine};
use log::{debug, info, warn};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use std::time::Instant;

use community_simulation::completion;
use community_simulation::list_commands;
use community_simulation::scenario::Scenario;
use community_simulation::utils::certification_duration_from_arg;

#[derive(Parser)]
#[command(name = "community-simulation")]
#[command(about = "Economic simulation framework with configurable agent-based modeling")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the economic simulation
    #[command(visible_alias = "simulate")]
    Run(Box<RunArgs>),

    /// Launch interactive configuration wizard
    Wizard {
        /// Disable colored terminal output
        #[arg(long, default_value_t = false)]
        no_color: bool,
    },

    /// List available presets, scenarios, or other options
    List {
        #[command(subcommand)]
        list_type: ListType,
    },

    /// Generate shell completion scripts
    #[command(name = "completion")]
    Completion {
        /// Shell type (bash, zsh, fish, powershell)
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}

#[derive(Subcommand)]
enum ListType {
    /// List available preset configurations
    Presets,
    /// List available pricing scenarios
    Scenarios,
}

#[derive(Parser)]
#[command(name = "run")]
struct RunArgs {
    /// Path to configuration file (YAML or TOML). CLI arguments override config file values.
    #[arg(short, long)]
    config: Option<String>,

    /// Use a preset configuration (e.g., 'small_economy', 'crisis_scenario', 'quick_test')
    /// Use 'list presets' subcommand to see all available presets
    #[arg(long)]
    preset: Option<String>,

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

    /// Path to SQLite database file for exporting simulation results
    #[arg(long)]
    sqlite_output: Option<String>,

    /// Path to time-series CSV output file for time-series analysis
    /// Exports data in long format (step,metric,value) suitable for time-series databases
    /// and analysis tools like Grafana, InfluxDB, Jupyter notebooks, or Excel
    #[arg(long)]
    timeseries_output: Option<String>,

    /// Path to Parquet output file for big-data analytics
    /// Exports in Apache Parquet format for efficient analysis with pandas, DuckDB, Spark
    #[arg(long)]
    parquet_output: Option<String>,

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

    /// Demand generation strategy (Uniform, Concentrated, Cyclical)
    /// Controls how many skills each person needs per step
    /// - Uniform: Random 2-5 needs (default, balanced)
    /// - Concentrated: Most have low demand, few have high (inequality)
    /// - Cyclical: Demand varies over time (business cycles)
    #[arg(long)]
    demand_strategy: Option<community_simulation::scenario::DemandStrategy>,

    /// Technology growth rate per step (e.g., 0.001 = 0.1% per step)
    /// Simulates productivity improvements over time
    #[arg(long)]
    tech_growth_rate: Option<f64>,

    /// Enable technology breakthrough events (sudden positive innovations)
    /// When enabled, random breakthroughs boost specific skill efficiencies
    #[arg(long, default_value_t = false)]
    enable_technology_breakthroughs: bool,

    /// Probability per step that a technology breakthrough occurs (0.0-1.0, e.g., 0.01 = 1%)
    /// Only used when --enable-technology-breakthroughs is set
    #[arg(long)]
    tech_breakthrough_probability: Option<f64>,

    /// Minimum efficiency boost from a breakthrough (e.g., 1.2 = 20% boost)
    /// Only used when --enable-technology-breakthroughs is set
    #[arg(long)]
    tech_breakthrough_min_effect: Option<f64>,

    /// Maximum efficiency boost from a breakthrough (e.g., 1.5 = 50% boost)
    /// Only used when --enable-technology-breakthroughs is set
    #[arg(long)]
    tech_breakthrough_max_effect: Option<f64>,

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

    /// Disable ASCII histogram visualization of wealth distribution in terminal output
    #[arg(long, default_value_t = false)]
    no_histogram: bool,

    /// Show ASCII price history chart for top skills in output
    /// Displays a terminal-based chart showing how the most valuable skills' prices evolved over time
    #[arg(long, default_value_t = false)]
    show_price_chart: bool,

    /// Cost multiplier for learning a skill based on market price (e.g., 3.0 = 3x market price)
    /// Only used when --enable-education is set
    #[arg(long)]
    learning_cost_multiplier: Option<f64>,

    /// Probability per step that a person attempts to learn a skill (0.0-1.0, e.g., 0.1 = 10%)
    /// Only used when --enable-education is set
    #[arg(long)]
    learning_probability: Option<f64>,

    /// Enable mentorship system where experienced persons can mentor others for reduced learning costs.
    /// Requires education to be enabled. Mentors must have high-quality skills to teach others.
    /// Mentees pay reduced learning costs and mentors gain reputation bonuses.
    #[arg(long, default_value_t = false)]
    enable_mentorship: bool,

    /// Cost reduction for mentored learning as a fraction (0.0-1.0, e.g., 0.5 = 50% discount)
    /// Only used when --enable-mentorship is set
    #[arg(long)]
    mentorship_cost_reduction: Option<f64>,

    /// Minimum skill quality required to be eligible as a mentor (0.0-5.0 scale, default: 3.5)
    /// Only used when --enable-mentorship is set
    #[arg(long)]
    min_mentor_quality: Option<f64>,

    /// Reputation bonus awarded to mentors for successful mentoring (default: 0.05)
    /// Only used when --enable-mentorship is set
    #[arg(long)]
    mentor_reputation_bonus: Option<f64>,

    /// Enable random crisis events that create economic shocks during the simulation
    /// When enabled, crises like market crashes, demand shocks, supply shocks, and currency devaluations can occur
    #[arg(long, default_value_t = false)]
    enable_crisis_events: bool,

    /// Probability per step that a crisis event will occur (0.0-1.0, e.g., 0.02 = 2%)
    /// Only used when --enable-crisis-events is set. Lower values = rarer crises
    #[arg(long)]
    crisis_probability: Option<f64>,

    /// Crisis severity level (0.0-1.0, e.g., 0.5 = moderate severity)
    /// Controls how severe crisis effects are. 0.0 = minimal impact, 1.0 = maximum impact
    /// Only used when --enable-crisis-events is set
    #[arg(long)]
    crisis_severity: Option<f64>,

    /// Enable loan system where persons can borrow and lend money
    /// When enabled, persons can request loans from others when they lack money for purchases
    #[arg(long, default_value_t = false)]
    enable_loans: bool,

    /// Interest rate per step for loans (0.0-1.0, e.g., 0.01 = 1% per step)
    /// Only used when --enable-loans is set
    #[arg(long)]
    loan_interest_rate: Option<f64>,

    /// Repayment period for loans in simulation steps (e.g., 20 = repay over 20 steps)
    /// Only used when --enable-loans is set
    #[arg(long)]
    loan_repayment_period: Option<usize>,

    /// Minimum money threshold for a person to be eligible to lend
    /// Only used when --enable-loans is set
    #[arg(long)]
    min_money_to_lend: Option<f64>,

    /// Enable insurance system where persons can purchase insurance policies
    /// When enabled, persons can buy insurance to protect against economic risks
    /// Available types: Credit (loan defaults), Income (low earnings), Crisis (economic shocks)
    #[arg(long, default_value_t = false)]
    enable_insurance: bool,

    /// Insurance premium rate as percentage of coverage (0.0-1.0, e.g., 0.05 = 5%)
    /// Premium is calculated as: coverage * premium_rate, then adjusted for reputation
    /// Only used when --enable-insurance is set
    #[arg(long)]
    insurance_premium_rate: Option<f64>,

    /// Insurance policy duration in simulation steps (e.g., 100 = policy lasts 100 steps)
    /// After this duration, policies expire and must be renewed
    /// Set to 0 for indefinite coverage
    /// Only used when --enable-insurance is set
    #[arg(long)]
    insurance_duration: Option<usize>,

    /// Probability per step that a person attempts to purchase insurance (0.0-1.0, e.g., 0.05 = 5%)
    /// Only used when --enable-insurance is set
    #[arg(long)]
    insurance_purchase_probability: Option<f64>,

    /// Default coverage amount for insurance policies (e.g., 50.0 = max payout of 50)
    /// Higher coverage provides more protection but costs more in premiums
    /// Only used when --enable-insurance is set
    #[arg(long)]
    insurance_coverage_amount: Option<f64>,

    /// Number of groups/organizations to create in the simulation
    /// When set, persons are assigned to groups for collective behavior analysis
    /// Valid range: 1 to number of persons
    #[arg(long)]
    num_groups: Option<usize>,

    /// Distance cost multiplier for geographic trade costs (0.0-1.0, e.g., 0.01 = 1% cost per distance unit)
    /// Controls the impact of geographic distance on trade costs
    /// final_cost = base_cost * (1 + distance * distance_cost_factor)
    /// Set to 0.0 to disable distance-based costs (default)
    #[arg(long)]
    distance_cost_factor: Option<f64>,

    /// Price elasticity factor (0.0-1.0) controlling sensitivity to supply/demand imbalances
    /// Higher values = more volatile prices, Lower values = more stable prices
    /// Default: 0.1 (10% price adjustment per unit imbalance)
    #[arg(long)]
    price_elasticity: Option<f64>,

    /// Volatility percentage (0.0-0.5) for random price fluctuations each step
    /// Simulates unpredictable market forces and sentiment changes
    /// Default: 0.02 (±2% random variation)
    #[arg(long)]
    volatility: Option<f64>,

    /// Enable event tracking during simulation
    /// When enabled, collects events for trades, price updates, reputation changes, and step completions
    /// Events are included in simulation results for detailed analysis and debugging
    /// Minimal performance overhead when disabled (default)
    #[arg(long, default_value_t = false)]
    enable_events: bool,

    /// Enable production system where persons can combine skills to create new skills
    /// When enabled, persons use recipes to combine two input skills into more valuable output skills
    /// Simulates supply chains, skill composition, and economic specialization
    #[arg(long, default_value_t = false)]
    enable_production: bool,

    /// Probability per step that a person attempts production (0.0-1.0, e.g., 0.05 = 5%)
    /// Only used when --enable-production is set. Higher values = more active production
    #[arg(long)]
    production_probability: Option<f64>,

    /// Enable satisficing decision-making (bounded rationality)
    /// When enabled, buyers accept the first "good enough" purchase option that meets the satisficing threshold
    /// instead of always seeking the optimal purchase. Models real-world cognitive limitations and heuristics.
    /// This can lead to different market dynamics and emergent "good enough" equilibria.
    #[arg(long, default_value_t = false)]
    enable_satisficing: bool,

    /// Threshold for satisficing decisions (0.0-1.0, default: 0.5)
    /// Buyers accept the first purchase option with priority score >= this threshold
    /// Higher values = more selective (near-optimal), lower values = less selective (faster decisions)
    /// Only used when --enable-satisficing is set
    /// Examples: 0.3=lenient, 0.5=balanced, 0.7=selective
    #[arg(long)]
    satisficing_threshold: Option<f64>,

    /// Run simulation in interactive mode (REPL)
    /// Allows step-by-step execution with commands for debugging and exploration
    /// Available commands: step, run N, stats, save <path>, help, exit
    #[arg(long, default_value_t = false)]
    interactive: bool,

    /// Record simulation actions to a JSON file for replay and debugging
    /// When enabled, logs all trades, failed trades, price updates, and crisis events
    /// The action log can be used for bug reproduction, debugging, and analysis
    #[arg(long)]
    record_actions: Option<String>,

    /// Enable quality rating system for skills (0.0-5.0 scale)
    /// Skills have quality that improves with successful trades and decays when not used
    /// Higher quality enables higher prices, creating quality competition in the market
    #[arg(long, default_value_t = false)]
    enable_quality: bool,

    /// Rate at which skill quality improves per successful trade (0.0-1.0, default: 0.1)
    /// Each successful sale increases quality by this amount (capped at 5.0)
    /// Only used when --enable-quality is set
    #[arg(long)]
    quality_improvement_rate: Option<f64>,

    /// Rate at which unused skill quality decays per step (0.0-1.0, default: 0.05)
    /// Skills that are not sold lose this much quality per step (minimum 0.0)
    /// Only used when --enable-quality is set
    #[arg(long)]
    quality_decay_rate: Option<f64>,

    /// Initial quality rating for all skills at simulation start (0.0-5.0, default: 3.0)
    /// All skills begin with this quality rating (3.0 = average quality)
    /// Only used when --enable-quality is set
    #[arg(long)]
    initial_quality: Option<f64>,

    /// Enable certification system for skills
    /// Persons can get their skills certified by paying a fee, which increases skill prices
    /// Certifications have levels (1-5) and can expire after a certain duration
    #[arg(long, default_value_t = false)]
    enable_certification: bool,

    /// Cost multiplier for obtaining skill certification (0.1-10.0, default: 2.0)
    /// Certification cost = base_skill_price * multiplier * certification_level
    /// Only used when --enable-certification is set
    #[arg(long)]
    certification_cost_multiplier: Option<f64>,

    /// Duration in steps before certification expires (default: 200)
    /// Set to 0 for certifications that never expire
    /// Only used when --enable-certification is set
    #[arg(long)]
    certification_duration: Option<usize>,

    /// Probability that a person attempts to certify a skill each step (0.0-1.0, default: 0.05)
    /// Only used when --enable-certification is set
    #[arg(long)]
    certification_probability: Option<f64>,

    /// Enable market segmentation system
    /// Persons are categorized into Budget (bottom 40%), Mittelklasse (40-85%), or Luxury (top 15%)
    /// based on wealth percentiles. Segments affect price acceptance ranges and quality expectations.
    #[arg(long, default_value_t = false)]
    enable_market_segments: bool,

    /// Enable auction-based trading for price discovery
    /// When enabled, a portion of trades use auctions where buyers submit bids
    /// and the highest bidder wins (English auction). Provides an alternative to bilateral trading.
    #[arg(long, default_value_t = false)]
    enable_auctions: bool,

    /// Percentage of trades using auctions instead of bilateral trading (0.0-1.0, default: 0.2)
    /// For example, 0.2 means 20% of trades go through auctions
    /// Only used when --enable-auctions is set
    #[arg(long)]
    auction_participation_rate: Option<f64>,

    /// Enable community resource pools for groups (requires --num-groups)
    /// Groups maintain shared pools where members contribute money each step
    /// Pools provide collective support and mutual aid to members in need
    #[arg(long, default_value_t = false)]
    enable_resource_pools: bool,

    /// Contribution rate to group resource pool as percentage of money (0.0-0.5, default: 0.02)
    /// Each step, group members contribute this percentage to their pool
    /// Only used when --enable-resource-pools is set
    #[arg(long)]
    pool_contribution_rate: Option<f64>,

    /// Minimum money threshold for pool support (default: 30.0)
    /// Members with less than this amount can receive equal distributions from their group's pool
    /// Only used when --enable-resource-pools is set
    #[arg(long)]
    pool_withdrawal_threshold: Option<f64>,

    /// Enable health system with disease transmission and economic impacts
    /// Sick persons have reduced productivity and can spread illness through trades
    #[arg(long, default_value_t = false)]
    enable_health: bool,

    /// Disease transmission rate per trade with sick person (0.0-1.0, default: 0.05 = 5%)
    /// Only used when --enable-health is set
    #[arg(long)]
    disease_transmission_rate: Option<f64>,

    /// Number of steps a person remains sick before recovering (default: 10)
    /// Only used when --enable-health is set
    #[arg(long)]
    disease_recovery_duration: Option<usize>,

    /// Number of persons who start sick at simulation start (default: 0)
    /// Seeds the disease spread. Set to 0 for no initial infections
    /// Only used when --enable-health is set
    #[arg(long)]
    initial_sick_persons: Option<usize>,

    /// Enable invariant checking during simulation to validate correctness
    /// Invariants check conditions that should always hold true (e.g., money conservation)
    /// Useful for debugging and ensuring simulation validity
    #[arg(long, default_value_t = false)]
    enable_invariant_checking: bool,

    /// Use strict mode for invariant violations (panic on first violation)
    /// When true, the simulation aborts on the first violation
    /// When false (default), violations are logged but simulation continues
    /// Only used when --enable-invariant-checking is set
    #[arg(long, default_value_t = false)]
    strict_invariant_mode: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_simulation(*args),
        Commands::Wizard { no_color } => run_wizard(no_color),
        Commands::List { list_type } => run_list(list_type),
        Commands::Completion { shell } => run_completion(&shell),
    }
}

/// Run the list subcommand
fn run_list(list_type: ListType) -> Result<(), Box<dyn std::error::Error>> {
    match list_type {
        ListType::Presets => list_commands::list_presets(),
        ListType::Scenarios => list_commands::list_scenarios(),
    }
}

/// Generate shell completion script
fn run_completion(shell_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shell = match completion::parse_shell_name(shell_name) {
        Some(s) => s,
        None => {
            eprintln!("Error: Unsupported shell '{}'", shell_name);
            eprintln!("Supported shells: {}", completion::get_supported_shells().join(", "));
            std::process::exit(1);
        },
    };

    let bin_name = community_simulation::utils::get_binary_name("community-simulation");

    completion::generate_completion::<Cli>(shell, &bin_name, &mut io::stdout());
    Ok(())
}

/// Run the interactive configuration wizard
fn run_wizard(no_color: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Handle --no-color flag to disable colored output globally
    if no_color {
        colored::control::set_override(false);
    }

    let (config, output_path) = community_simulation::wizard::run_wizard()?;

    // Save config if requested
    if let Some(path) = &output_path {
        let content =
            community_simulation::wizard_helpers::serialize_config_by_extension(&config, path)?;

        std::fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
        println!("\n✅ Configuration saved to: {}", path.display());
    }

    // Ask if user wants to run the simulation now
    use inquire::Confirm;
    let run_now = Confirm::new("Would you like to run the simulation now?")
        .with_default(false)
        .prompt()
        .map_err(|e| format!("Failed to get confirmation: {}", e))?;

    if !run_now {
        println!("\n👋 Configuration complete! You can run the simulation later using:");
        if let Some(path) = output_path {
            println!("   community-simulation run --config {}", path.display());
        } else {
            println!("   community-simulation run [with your chosen parameters]");
        }
        return Ok(());
    }

    // Continue with simulation using the wizard-generated config
    let mut engine = SimulationEngine::new(config.clone());

    // Run the simulation
    let start = Instant::now();
    // Always show progress in wizard mode for better interactivity (wizard is inherently interactive)
    let result = engine.run_with_progress(true);
    let duration = start.elapsed();

    // Print results
    // Always show histogram in wizard mode for immediate visual feedback
    result.print_summary(true);
    println!("\n⏱️  Simulation completed in {:.2}s", duration.as_secs_f64());
    println!(
        "⚡ Performance: {:.0} steps/second",
        result.total_steps as f64 / duration.as_secs_f64()
    );

    Ok(())
}

/// Run the main simulation
fn run_simulation(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
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

    if let Some(num_threads) = args.threads {
        rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global()?;
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
        if let Some(demand_strategy) = args.demand_strategy.clone() {
            cfg.demand_strategy = demand_strategy;
        }
        if let Some(tech_growth_rate) = args.tech_growth_rate {
            cfg.tech_growth_rate = tech_growth_rate;
        }
        if args.enable_technology_breakthroughs {
            cfg.enable_technology_breakthroughs = true;
        }
        if let Some(tech_breakthrough_probability) = args.tech_breakthrough_probability {
            cfg.tech_breakthrough_probability = tech_breakthrough_probability;
        }
        if let Some(tech_breakthrough_min_effect) = args.tech_breakthrough_min_effect {
            cfg.tech_breakthrough_min_effect = tech_breakthrough_min_effect;
        }
        if let Some(tech_breakthrough_max_effect) = args.tech_breakthrough_max_effect {
            cfg.tech_breakthrough_max_effect = tech_breakthrough_max_effect;
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
        if args.enable_mentorship {
            cfg.enable_mentorship = true;
        }
        if let Some(cost_reduction) = args.mentorship_cost_reduction {
            cfg.mentorship_cost_reduction = cost_reduction;
        }
        if let Some(min_quality) = args.min_mentor_quality {
            cfg.min_mentor_quality = min_quality;
        }
        if let Some(bonus) = args.mentor_reputation_bonus {
            cfg.mentor_reputation_bonus = bonus;
        }
        if args.enable_loans {
            cfg.enable_loans = true;
        }
        if let Some(interest_rate) = args.loan_interest_rate {
            cfg.loan_interest_rate = interest_rate;
        }
        if let Some(repayment_period) = args.loan_repayment_period {
            cfg.loan_repayment_period = repayment_period;
        }
        if let Some(min_money) = args.min_money_to_lend {
            cfg.min_money_to_lend = min_money;
        }
        if args.enable_insurance {
            cfg.enable_insurance = true;
        }
        if let Some(premium_rate) = args.insurance_premium_rate {
            cfg.insurance_premium_rate = premium_rate;
        }
        if let Some(duration) = args.insurance_duration {
            cfg.insurance_duration = duration;
        }
        if let Some(probability) = args.insurance_purchase_probability {
            cfg.insurance_purchase_probability = probability;
        }
        if let Some(coverage) = args.insurance_coverage_amount {
            cfg.insurance_coverage_amount = coverage;
        }
        if let Some(num_groups) = args.num_groups {
            cfg.num_groups = Some(num_groups);
        }
        if let Some(price_elasticity) = args.price_elasticity {
            cfg.price_elasticity_factor = price_elasticity;
        }
        if let Some(volatility) = args.volatility {
            cfg.volatility_percentage = volatility;
        }
        if args.enable_production {
            cfg.enable_production = true;
        }
        if let Some(production_prob) = args.production_probability {
            cfg.production_probability = production_prob;
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
            if let Some(demand_strategy) = args.demand_strategy.clone() {
                cfg.demand_strategy = demand_strategy;
            }
            if let Some(tech_growth_rate) = args.tech_growth_rate {
                cfg.tech_growth_rate = tech_growth_rate;
            }
            if args.enable_technology_breakthroughs {
                cfg.enable_technology_breakthroughs = true;
            }
            if let Some(tech_breakthrough_probability) = args.tech_breakthrough_probability {
                cfg.tech_breakthrough_probability = tech_breakthrough_probability;
            }
            if let Some(tech_breakthrough_min_effect) = args.tech_breakthrough_min_effect {
                cfg.tech_breakthrough_min_effect = tech_breakthrough_min_effect;
            }
            if let Some(tech_breakthrough_max_effect) = args.tech_breakthrough_max_effect {
                cfg.tech_breakthrough_max_effect = tech_breakthrough_max_effect;
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
            if args.enable_mentorship {
                cfg.enable_mentorship = true;
            }
            if let Some(cost_reduction) = args.mentorship_cost_reduction {
                cfg.mentorship_cost_reduction = cost_reduction;
            }
            if let Some(min_quality) = args.min_mentor_quality {
                cfg.min_mentor_quality = min_quality;
            }
            if let Some(bonus) = args.mentor_reputation_bonus {
                cfg.mentor_reputation_bonus = bonus;
            }
            if args.enable_crisis_events {
                cfg.enable_crisis_events = true;
            }
            if let Some(crisis_prob) = args.crisis_probability {
                cfg.crisis_probability = crisis_prob;
            }
            if let Some(severity) = args.crisis_severity {
                cfg.crisis_severity = severity;
            }
            if args.enable_loans {
                cfg.enable_loans = true;
            }
            if let Some(interest_rate) = args.loan_interest_rate {
                cfg.loan_interest_rate = interest_rate;
            }
            if let Some(repayment_period) = args.loan_repayment_period {
                cfg.loan_repayment_period = repayment_period;
            }
            if let Some(min_money) = args.min_money_to_lend {
                cfg.min_money_to_lend = min_money;
            }
            if args.enable_insurance {
                cfg.enable_insurance = true;
            }
            if let Some(premium_rate) = args.insurance_premium_rate {
                cfg.insurance_premium_rate = premium_rate;
            }
            if let Some(duration) = args.insurance_duration {
                cfg.insurance_duration = duration;
            }
            if let Some(probability) = args.insurance_purchase_probability {
                cfg.insurance_purchase_probability = probability;
            }
            if let Some(coverage) = args.insurance_coverage_amount {
                cfg.insurance_coverage_amount = coverage;
            }
            if let Some(num_groups) = args.num_groups {
                cfg.num_groups = Some(num_groups);
            }
            if let Some(distance_cost_factor) = args.distance_cost_factor {
                cfg.distance_cost_factor = distance_cost_factor;
            }
            if let Some(price_elasticity) = args.price_elasticity {
                cfg.price_elasticity_factor = price_elasticity;
            }
            if let Some(volatility) = args.volatility {
                cfg.volatility_percentage = volatility;
            }
            if args.enable_production {
                cfg.enable_production = true;
            }
            if let Some(production_prob) = args.production_probability {
                cfg.production_probability = production_prob;
            }
            if args.enable_certification {
                cfg.enable_certification = true;
            }
            if let Some(cost_multiplier) = args.certification_cost_multiplier {
                cfg.certification_cost_multiplier = cost_multiplier;
            }
            if let Some(duration) = args.certification_duration {
                cfg.certification_duration = certification_duration_from_arg(duration);
            }
            if let Some(probability) = args.certification_probability {
                cfg.certification_probability = probability;
            }
            if args.enable_resource_pools {
                cfg.enable_resource_pools = true;
            }
            if let Some(contribution_rate) = args.pool_contribution_rate {
                cfg.pool_contribution_rate = contribution_rate;
            }
            if let Some(threshold) = args.pool_withdrawal_threshold {
                cfg.pool_withdrawal_threshold = threshold;
            }
            if args.enable_health {
                cfg.enable_health = true;
            }
            if let Some(rate) = args.disease_transmission_rate {
                cfg.disease_transmission_rate = rate;
            }
            if let Some(duration) = args.disease_recovery_duration {
                cfg.disease_recovery_duration = duration;
            }
            if let Some(count) = args.initial_sick_persons {
                cfg.initial_sick_persons = count;
            }
        })?
    } else {
        // No config file or preset, use CLI arguments or defaults
        SimulationConfig {
            max_steps: args.steps.unwrap_or(SimulationConfig::default().max_steps),
            entity_count: args.persons.unwrap_or(SimulationConfig::default().entity_count),
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
            per_skill_price_limits: HashMap::new(), // Not configurable via CLI
            scenario: args.scenario.unwrap_or(SimulationConfig::default().scenario),
            demand_strategy: args
                .demand_strategy
                .unwrap_or(SimulationConfig::default().demand_strategy),
            tech_growth_rate: args
                .tech_growth_rate
                .unwrap_or(SimulationConfig::default().tech_growth_rate),
            enable_technology_breakthroughs: args.enable_technology_breakthroughs,
            tech_breakthrough_probability: args
                .tech_breakthrough_probability
                .unwrap_or(SimulationConfig::default().tech_breakthrough_probability),
            tech_breakthrough_min_effect: args
                .tech_breakthrough_min_effect
                .unwrap_or(SimulationConfig::default().tech_breakthrough_min_effect),
            tech_breakthrough_max_effect: args
                .tech_breakthrough_max_effect
                .unwrap_or(SimulationConfig::default().tech_breakthrough_max_effect),
            seasonal_amplitude: args
                .seasonal_amplitude
                .unwrap_or(SimulationConfig::default().seasonal_amplitude),
            seasonal_period: args
                .seasonal_period
                .unwrap_or(SimulationConfig::default().seasonal_period),
            transaction_fee: args
                .transaction_fee
                .unwrap_or(SimulationConfig::default().transaction_fee),
            savings_rate: args.savings_rate.unwrap_or(SimulationConfig::default().savings_rate),
            enable_loans: args.enable_loans,
            enable_credit_rating: SimulationConfig::default().enable_credit_rating,
            loan_interest_rate: args
                .loan_interest_rate
                .unwrap_or(SimulationConfig::default().loan_interest_rate),
            loan_repayment_period: args
                .loan_repayment_period
                .unwrap_or(SimulationConfig::default().loan_repayment_period),
            min_money_to_lend: args
                .min_money_to_lend
                .unwrap_or(SimulationConfig::default().min_money_to_lend),
            enable_p2p_lending: SimulationConfig::default().enable_p2p_lending,
            p2p_platform_fee_rate: SimulationConfig::default().p2p_platform_fee_rate,
            enable_investments: SimulationConfig::default().enable_investments,
            investment_return_rate: SimulationConfig::default().investment_return_rate,
            investment_duration: SimulationConfig::default().investment_duration,
            investment_probability: SimulationConfig::default().investment_probability,
            min_money_to_invest: SimulationConfig::default().min_money_to_invest,
            checkpoint_interval: args
                .checkpoint_interval
                .unwrap_or(SimulationConfig::default().checkpoint_interval),
            checkpoint_file: args.checkpoint_file.clone(),
            resume_from_checkpoint: args.resume,
            tax_rate: args.tax_rate.unwrap_or(SimulationConfig::default().tax_rate),
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
            enable_auctions: args.enable_auctions,
            auction_participation_rate: args
                .auction_participation_rate
                .unwrap_or(SimulationConfig::default().auction_participation_rate),
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
            enable_mentorship: args.enable_mentorship,
            mentorship_cost_reduction: args
                .mentorship_cost_reduction
                .unwrap_or(SimulationConfig::default().mentorship_cost_reduction),
            min_mentor_quality: args
                .min_mentor_quality
                .unwrap_or(SimulationConfig::default().min_mentor_quality),
            mentor_reputation_bonus: args
                .mentor_reputation_bonus
                .unwrap_or(SimulationConfig::default().mentor_reputation_bonus),
            enable_crisis_events: args.enable_crisis_events,
            crisis_probability: args
                .crisis_probability
                .unwrap_or(SimulationConfig::default().crisis_probability),
            crisis_severity: args
                .crisis_severity
                .unwrap_or(SimulationConfig::default().crisis_severity),
            enable_insurance: args.enable_insurance,
            insurance_premium_rate: args
                .insurance_premium_rate
                .unwrap_or(SimulationConfig::default().insurance_premium_rate),
            insurance_duration: args
                .insurance_duration
                .unwrap_or(SimulationConfig::default().insurance_duration),
            insurance_purchase_probability: args
                .insurance_purchase_probability
                .unwrap_or(SimulationConfig::default().insurance_purchase_probability),
            insurance_coverage_amount: args
                .insurance_coverage_amount
                .unwrap_or(SimulationConfig::default().insurance_coverage_amount),
            enable_friendships: SimulationConfig::default().enable_friendships,
            friendship_probability: SimulationConfig::default().friendship_probability,
            friendship_discount: SimulationConfig::default().friendship_discount,
            enable_trade_agreements: SimulationConfig::default().enable_trade_agreements,
            trade_agreement_probability: SimulationConfig::default().trade_agreement_probability,
            trade_agreement_discount: SimulationConfig::default().trade_agreement_discount,
            trade_agreement_duration: SimulationConfig::default().trade_agreement_duration,
            enable_trust_networks: SimulationConfig::default().enable_trust_networks,
            enable_influence: SimulationConfig::default().enable_influence,
            num_groups: args.num_groups,
            distance_cost_factor: args
                .distance_cost_factor
                .unwrap_or(SimulationConfig::default().distance_cost_factor),
            price_elasticity_factor: args
                .price_elasticity
                .unwrap_or(SimulationConfig::default().price_elasticity_factor),
            volatility_percentage: args
                .volatility
                .unwrap_or(SimulationConfig::default().volatility_percentage),
            enable_events: args.enable_events,
            enable_production: args.enable_production,
            production_probability: args
                .production_probability
                .unwrap_or(SimulationConfig::default().production_probability),
            enable_satisficing: args.enable_satisficing,
            satisficing_threshold: args
                .satisficing_threshold
                .unwrap_or(SimulationConfig::default().satisficing_threshold),
            enable_environment: SimulationConfig::default().enable_environment,
            resource_cost_per_transaction: SimulationConfig::default()
                .resource_cost_per_transaction,
            custom_resource_reserves: None,
            enable_voting: SimulationConfig::default().enable_voting,
            voting_method: SimulationConfig::default().voting_method,
            proposal_duration: SimulationConfig::default().proposal_duration,
            proposal_probability: SimulationConfig::default().proposal_probability,
            voting_participation_rate: SimulationConfig::default().voting_participation_rate,
            enable_quality: args.enable_quality,
            quality_improvement_rate: args
                .quality_improvement_rate
                .unwrap_or(SimulationConfig::default().quality_improvement_rate),
            quality_decay_rate: args
                .quality_decay_rate
                .unwrap_or(SimulationConfig::default().quality_decay_rate),
            initial_quality: args
                .initial_quality
                .unwrap_or(SimulationConfig::default().initial_quality),
            enable_certification: args.enable_certification,
            certification_cost_multiplier: args
                .certification_cost_multiplier
                .unwrap_or(SimulationConfig::default().certification_cost_multiplier),
            certification_duration: args.certification_duration.map_or_else(
                || SimulationConfig::default().certification_duration,
                certification_duration_from_arg,
            ),
            certification_probability: args
                .certification_probability
                .unwrap_or(SimulationConfig::default().certification_probability),
            enable_market_segments: args.enable_market_segments,
            enable_resource_pools: args.enable_resource_pools,
            pool_contribution_rate: args
                .pool_contribution_rate
                .unwrap_or(SimulationConfig::default().pool_contribution_rate),
            pool_withdrawal_threshold: args
                .pool_withdrawal_threshold
                .unwrap_or(SimulationConfig::default().pool_withdrawal_threshold),
            enable_adaptive_strategies: SimulationConfig::default().enable_adaptive_strategies,
            adaptation_rate: SimulationConfig::default().adaptation_rate,
            exploration_rate: SimulationConfig::default().exploration_rate,
            enable_strategy_evolution: SimulationConfig::default().enable_strategy_evolution,
            evolution_update_frequency: SimulationConfig::default().evolution_update_frequency,
            imitation_probability: SimulationConfig::default().imitation_probability,
            mutation_rate: SimulationConfig::default().mutation_rate,
            enable_specialization: SimulationConfig::default().enable_specialization,
            enable_parallel_trades: SimulationConfig::default().enable_parallel_trades,
            enable_externalities: SimulationConfig::default().enable_externalities,
            externality_rate: SimulationConfig::default().externality_rate,
            externality_rates_per_skill: HashMap::new(), // Not configurable via CLI
            enable_health: args.enable_health,
            disease_transmission_rate: args
                .disease_transmission_rate
                .unwrap_or(SimulationConfig::default().disease_transmission_rate),
            disease_recovery_duration: args
                .disease_recovery_duration
                .unwrap_or(SimulationConfig::default().disease_recovery_duration),
            initial_sick_persons: args
                .initial_sick_persons
                .unwrap_or(SimulationConfig::default().initial_sick_persons),
            enable_automation: SimulationConfig::default().enable_automation,
            automation_rate: SimulationConfig::default().automation_rate,
            automation_risks_per_skill: HashMap::new(), // Not configurable via CLI
            enable_reinforcement_learning: SimulationConfig::default()
                .enable_reinforcement_learning,
            rl_learning_rate: SimulationConfig::default().rl_learning_rate,
            rl_discount_factor: SimulationConfig::default().rl_discount_factor,
            rl_epsilon: SimulationConfig::default().rl_epsilon,
            rl_epsilon_decay: SimulationConfig::default().rl_epsilon_decay,
            rl_reward_success_multiplier: SimulationConfig::default().rl_reward_success_multiplier,
            rl_reward_failure_multiplier: SimulationConfig::default().rl_reward_failure_multiplier,
            enable_invariant_checking: args.enable_invariant_checking,
            strict_invariant_mode: args.strict_invariant_mode,
            check_money_conservation: SimulationConfig::default().check_money_conservation,
            check_non_negative_wealth: SimulationConfig::default().check_non_negative_wealth,
            enable_assets: SimulationConfig::default().enable_assets,
            asset_purchase_probability: SimulationConfig::default().asset_purchase_probability,
            min_money_for_asset_purchase: SimulationConfig::default().min_money_for_asset_purchase,
            property_appreciation_rate: SimulationConfig::default().property_appreciation_rate,
            equipment_depreciation_rate: SimulationConfig::default().equipment_depreciation_rate,
            rental_income_rate: SimulationConfig::default().rental_income_rate,
            stock_return_rate: SimulationConfig::default().stock_return_rate,
            asset_price_multiplier: SimulationConfig::default().asset_price_multiplier,
        }
    };

    // Validate configuration before proceeding
    config.validate()?;

    // Check if interactive mode is enabled
    if args.interactive {
        // Interactive mode is incompatible with batch modes
        if args.monte_carlo_runs.is_some() {
            return Err("Interactive mode cannot be combined with Monte Carlo runs".into());
        }
        if args.parameter_sweep.is_some() {
            return Err("Interactive mode cannot be combined with parameter sweep".into());
        }
        if args.compare_scenarios.is_some() {
            return Err("Interactive mode cannot be combined with scenario comparison".into());
        }

        return run_interactive_mode(config);
    }

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

        run_monte_carlo(config, num_runs, args.output, args.csv_output, args.compress)?;
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
            let checkpoint_path =
                config.checkpoint_file.clone().unwrap_or_else(|| "checkpoint.json".to_string());

            info!(
                "{}",
                format!("Resuming simulation from checkpoint: {}", checkpoint_path).bright_cyan()
            );

            SimulationEngine::load_checkpoint(&checkpoint_path)
                .map_err(|e| format!("Failed to load checkpoint from {}: {}", checkpoint_path, e))?
        } else {
            SimulationEngine::new(config)
        };

        // Enable action recording if requested
        if args.record_actions.is_some() {
            engine.enable_action_recording();
        }

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
                info!("{}", format!("Results saved to {}", output_path).bright_blue());
            }
        }

        if let Some(csv_prefix) = args.csv_output {
            result.save_to_csv(&csv_prefix)?;
            info!("{}", format!("CSV results saved with prefix: {}", csv_prefix).bright_blue());
        }

        if let Some(action_log_path) = args.record_actions {
            engine.save_action_log(&action_log_path)?;
            info!("{}", format!("Action log saved to: {}", action_log_path).bright_blue());
        }

        if let Some(sqlite_path) = args.sqlite_output {
            community_simulation::database::export_to_sqlite(&result, &sqlite_path)?;
            info!("{}", format!("SQLite database saved to: {}", sqlite_path).bright_blue());
        }

        if let Some(timeseries_path) = args.timeseries_output {
            result.save_timeseries_csv(&timeseries_path)?;
            info!("{}", format!("Time-series data saved to: {}", timeseries_path).bright_blue());
        }

        if let Some(parquet_path) = args.parquet_output {
            result.export_to_parquet(&parquet_path)?;
            info!("{}", format!("Parquet data saved to: {}", parquet_path).bright_blue());
        }

        result.print_summary_with_options(!args.no_histogram, args.show_price_chart);
    }

    Ok(())
}

/// Run simulation in interactive mode (REPL) for step-by-step execution
fn run_interactive_mode(config: SimulationConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "{}",
        "Starting interactive mode. Type 'help' for available commands.".bright_cyan()
    );
    info!(
        "{}",
        format!(
            "Simulation configured with {} persons for up to {} steps",
            config.entity_count, config.max_steps
        )
        .bright_blue()
    );

    // Initialize the simulation engine
    let mut engine = if config.resume_from_checkpoint {
        let checkpoint_path =
            config.checkpoint_file.clone().unwrap_or_else(|| "checkpoint.json".to_string());

        info!("{}", format!("Resuming from checkpoint: {}", checkpoint_path).bright_cyan());

        SimulationEngine::load_checkpoint(&checkpoint_path)
            .map_err(|e| format!("Failed to load checkpoint from {}: {}", checkpoint_path, e))?
    } else {
        SimulationEngine::new(config.clone())
    };

    // Create readline editor for interactive input
    let mut rl = DefaultEditor::new()?;

    // REPL loop
    loop {
        let current_step = engine.get_current_step();
        let max_steps = engine.get_max_steps();

        let prompt = format!("sim[{}/{}]> ", current_step, max_steps);
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Add command to history
                let _ = rl.add_history_entry(line);

                // Parse and execute command
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }

                let command = parts[0].to_lowercase();
                match command.as_str() {
                    "help" | "?" => {
                        println!("{}", "Available commands:".bright_yellow());
                        println!("  {}  - Execute one simulation step", "step".bright_green());
                        println!("  {} - Execute N simulation steps", "run <N>".bright_green());
                        println!(
                            "  {}  - Show current simulation statistics",
                            "stats".bright_green()
                        );
                        println!(
                            "  {} - Save current state to checkpoint file",
                            "save <path>".bright_green()
                        );
                        println!("  {} - Show current simulation status", "status".bright_green());
                        println!(
                            "  {} - Show detailed state of a specific person",
                            "inspect <id>".bright_green()
                        );
                        println!(
                            "  {} - List all persons with summary info",
                            "persons/list-persons".bright_green()
                        );
                        println!(
                            "  {} - Display current market state and prices",
                            "market".bright_green()
                        );
                        println!(
                            "  {} - Show top N wealthiest persons (default: 10)",
                            "find-rich [N]".bright_green()
                        );
                        println!(
                            "  {} - Show bottom N poorest persons (default: 10)",
                            "find-poor [N]".bright_green()
                        );
                        println!(
                            "  {} - List persons with a specific skill",
                            "filter-by-skill <name>".bright_green()
                        );
                        println!("  {}  - Show this help message", "help".bright_green());
                        println!("  {}  - Exit interactive mode", "exit/quit".bright_green());
                    },
                    "step" => {
                        if current_step >= max_steps {
                            println!(
                                "{}",
                                "Simulation has reached max steps. Cannot execute more steps."
                                    .bright_red()
                            );
                            continue;
                        }

                        let start = Instant::now();
                        engine.step();
                        let duration = start.elapsed();
                        println!(
                            "{}",
                            format!(
                                "Executed step {} in {:.3}s",
                                current_step + 1,
                                duration.as_secs_f64()
                            )
                            .bright_green()
                        );
                    },
                    "run" => {
                        if parts.len() < 2 {
                            println!("{}", "Usage: run <N>".bright_red());
                            continue;
                        }

                        let num_steps: usize = match parts[1].parse() {
                            Ok(n) => n,
                            Err(_) => {
                                println!("{}", "Invalid number of steps".bright_red());
                                continue;
                            },
                        };

                        if num_steps == 0 {
                            println!("{}", "Number of steps must be greater than 0".bright_red());
                            continue;
                        }

                        let remaining_steps = max_steps.saturating_sub(current_step);
                        let steps_to_run = num_steps.min(remaining_steps);

                        if steps_to_run == 0 {
                            println!(
                                "{}",
                                "Simulation has reached max steps. Cannot execute more steps."
                                    .bright_red()
                            );
                            continue;
                        }

                        if steps_to_run < num_steps {
                            println!(
                                "{}",
                                format!(
                                    "Warning: Only {} steps remaining, running {} steps",
                                    remaining_steps, steps_to_run
                                )
                                .bright_yellow()
                            );
                        }

                        let start = Instant::now();
                        for i in 0..steps_to_run {
                            engine.step();
                            if (i + 1) % 10 == 0 || (i + 1) == steps_to_run {
                                println!(
                                    "{}",
                                    format!("  Progress: {}/{} steps", i + 1, steps_to_run)
                                        .bright_blue()
                                );
                            }
                        }
                        let duration = start.elapsed();
                        println!(
                            "{}",
                            format!(
                                "Executed {} steps in {:.3}s ({:.1} steps/s)",
                                steps_to_run,
                                duration.as_secs_f64(),
                                if duration.as_secs_f64() > 0.0 {
                                    steps_to_run as f64 / duration.as_secs_f64()
                                } else {
                                    0.0
                                }
                            )
                            .bright_green()
                        );
                    },
                    "stats" => {
                        let result = engine.get_current_result();
                        println!("\n{}", "=== Current Statistics ===".bright_yellow());
                        result.print_summary(false); // Disable histogram in interactive mode
                    },
                    "status" => {
                        println!("\n{}", "=== Simulation Status ===".bright_yellow());
                        println!("  Current Step: {}/{}", current_step, max_steps);
                        println!(
                            "  Progress: {:.1}%",
                            (current_step as f64 / max_steps as f64) * 100.0
                        );
                        println!("  Active Persons: {}", engine.get_active_persons());
                        println!("  Scenario: {:?}", engine.get_scenario());
                    },
                    "save" => {
                        if parts.len() < 2 {
                            println!("{}", "Usage: save <path>".bright_red());
                            continue;
                        }

                        let save_path = parts[1];
                        match engine.save_checkpoint(save_path) {
                            Ok(_) => {
                                println!(
                                    "{}",
                                    format!("Checkpoint saved to {}", save_path).bright_green()
                                );
                            },
                            Err(e) => {
                                println!(
                                    "{}",
                                    format!("Error saving checkpoint: {}", e).bright_red()
                                );
                            },
                        }
                    },
                    "inspect" => {
                        if parts.len() < 2 {
                            println!("{}", "Usage: inspect <person_id>".bright_red());
                            continue;
                        }

                        let person_id: usize = match parts[1].parse() {
                            Ok(id) => id,
                            Err(_) => {
                                println!("{}", "Invalid person ID".bright_red());
                                continue;
                            },
                        };

                        // Get entities from engine to inspect
                        let entities = engine.get_entities();
                        if let Some(entity) = entities.iter().find(|e| e.id == person_id) {
                            let person = &entity.person_data;
                            println!(
                                "\n{}",
                                format!("=== Person {} Details ===", person_id).bright_yellow()
                            );
                            println!("  {} {:.2}", "Money:".bright_cyan(), person.money);
                            println!("  {} {:.2}", "Savings:".bright_cyan(), person.savings);
                            println!("  {} {:.3}", "Reputation:".bright_cyan(), person.reputation);
                            println!("  {} {:?}", "Strategy:".bright_cyan(), person.strategy);
                            println!(
                                "  {} {:?}",
                                "Specialization:".bright_cyan(),
                                person.specialization_strategy
                            );
                            println!(
                                "  {} ({:.2}, {:.2})",
                                "Location:".bright_cyan(),
                                person.location.x,
                                person.location.y
                            );
                            println!("  {} {}", "Friends:".bright_cyan(), person.friends.len());
                            println!(
                                "  {} {}",
                                "Active:".bright_cyan(),
                                if entity.active { "Yes" } else { "No" }
                            );

                            // Display skills
                            println!("\n  {}:", "Own Skills".bright_green());
                            for skill in &person.own_skills {
                                let quality = person.skill_qualities.get(&skill.id);
                                let quality_str = if let Some(q) = quality {
                                    format!(" (quality: {:.2})", q)
                                } else {
                                    String::new()
                                };
                                println!(
                                    "    - {}: ${:.2}{}",
                                    skill.id, skill.current_price, quality_str
                                );
                            }

                            if !person.learned_skills.is_empty() {
                                println!("\n  {}:", "Learned Skills".bright_green());
                                for skill in &person.learned_skills {
                                    let quality = person.skill_qualities.get(&skill.id);
                                    let quality_str = if let Some(q) = quality {
                                        format!(" (quality: {:.2})", q)
                                    } else {
                                        String::new()
                                    };
                                    println!(
                                        "    - {}: ${:.2}{}",
                                        skill.id, skill.current_price, quality_str
                                    );
                                }
                            }

                            // Display needed skills
                            if !person.needed_skills.is_empty() {
                                println!("\n  {}:", "Needed Skills".bright_blue());
                                for need in &person.needed_skills {
                                    println!("    - {} (urgency: {})", need.id, need.urgency);
                                }
                            }

                            // Display loans
                            if !person.borrowed_loans.is_empty() || !person.lent_loans.is_empty() {
                                println!("\n  {}:", "Loans".bright_magenta());
                                if !person.borrowed_loans.is_empty() {
                                    println!("    Borrowed: {} loans", person.borrowed_loans.len());
                                }
                                if !person.lent_loans.is_empty() {
                                    println!("    Lent: {} loans", person.lent_loans.len());
                                }
                            }

                            // Display recent transactions
                            let recent_transactions: Vec<_> =
                                person.transaction_history.iter().rev().take(5).collect();
                            if !recent_transactions.is_empty() {
                                println!("\n  {} (last 5):", "Recent Transactions".bright_yellow());
                                for tx in recent_transactions.iter().rev() {
                                    let tx_type = match tx.transaction_type {
                                        community_simulation::person::TransactionType::Buy => "Buy",
                                        community_simulation::person::TransactionType::Sell => {
                                            "Sell"
                                        },
                                    };
                                    println!(
                                        "    Step {}: {} {} for ${:.2}",
                                        tx.step, tx_type, tx.skill_id, tx.amount
                                    );
                                }
                            }
                        } else {
                            println!("{}", format!("Person {} not found", person_id).bright_red());
                        }
                    },
                    "persons" | "list-persons" => {
                        let entities = engine.get_entities();
                        println!(
                            "\n{}",
                            format!("=== All Persons ({} total) ===", entities.len())
                                .bright_yellow()
                        );
                        println!(
                            "{:>5} {:>10} {:>10} {:>10} {:>10} {:>8}",
                            "ID", "Money", "Savings", "Reputation", "Skills", "Active"
                        );
                        println!("{}", "─".repeat(70));

                        for entity in entities {
                            let person = &entity.person_data;
                            let total_skills =
                                person.own_skills.len() + person.learned_skills.len();
                            let active_str = if entity.active { "Yes" } else { "No" };
                            println!(
                                "{:>5} {:>10.2} {:>10.2} {:>10.3} {:>10} {:>8}",
                                entity.id,
                                person.money,
                                person.savings,
                                person.reputation,
                                total_skills,
                                active_str
                            );
                        }
                    },
                    "market" => {
                        let market = engine.get_market();
                        println!("\n{}", "=== Market State ===".bright_yellow());
                        println!("  Base Price: ${:.2}", market.base_skill_price);
                        println!("  Total Skills: {}", market.skills.len());
                        println!("  Volatility: {:.1}%", market.volatility_percentage * 100.0);
                        println!("  Price Elasticity: {:.2}", market.price_elasticity_factor);
                        println!(
                            "\n{:>20} {:>10} {:>10} {:>10}",
                            "Skill", "Price", "Supply", "Demand"
                        );
                        println!("{}", "─".repeat(54));

                        let mut skills: Vec<_> = market.skills.values().collect();
                        skills.sort_by(|a, b| {
                            b.current_price
                                .partial_cmp(&a.current_price)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        for skill in skills.iter().take(20) {
                            let supply = market.supply_counts.get(&skill.id).unwrap_or(&0);
                            let demand = market.demand_counts.get(&skill.id).unwrap_or(&0);
                            println!(
                                "{:>20} {:>10.2} {:>10} {:>10}",
                                &skill.id, skill.current_price, supply, demand
                            );
                        }

                        if market.skills.len() > 20 {
                            println!(
                                "\n  (Showing top 20 by price, {} total skills)",
                                market.skills.len()
                            );
                        }
                    },
                    "find-rich" => {
                        let count: usize = if parts.len() > 1 {
                            match parts[1].parse() {
                                Ok(n) => n,
                                Err(_) => {
                                    println!(
                                        "{}",
                                        format!(
                                            "Invalid number '{}', using default of 10",
                                            parts[1]
                                        )
                                        .bright_yellow()
                                    );
                                    10
                                },
                            }
                        } else {
                            10
                        };

                        let mut entities: Vec<_> = engine.get_entities().iter().collect();
                        entities.sort_by(|a, b| {
                            b.person_data
                                .money
                                .partial_cmp(&a.person_data.money)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        println!(
                            "\n{}",
                            format!("=== Top {} Wealthiest Persons ===", count).bright_yellow()
                        );
                        println!(
                            "{:>5} {:>5} {:>15} {:>10} {:>10}",
                            "Rank", "ID", "Money", "Savings", "Skills"
                        );
                        println!("{}", "─".repeat(52));

                        for (i, entity) in entities.iter().take(count).enumerate() {
                            let person = &entity.person_data;
                            let total_skills =
                                person.own_skills.len() + person.learned_skills.len();
                            println!(
                                "{:>5} {:>5} {:>15.2} {:>10.2} {:>10}",
                                i + 1,
                                entity.id,
                                person.money,
                                person.savings,
                                total_skills
                            );
                        }
                    },
                    "find-poor" => {
                        let count: usize = if parts.len() > 1 {
                            match parts[1].parse() {
                                Ok(n) => n,
                                Err(_) => {
                                    println!(
                                        "{}",
                                        format!(
                                            "Invalid number '{}', using default of 10",
                                            parts[1]
                                        )
                                        .bright_yellow()
                                    );
                                    10
                                },
                            }
                        } else {
                            10
                        };

                        let mut entities: Vec<_> = engine.get_entities().iter().collect();
                        entities.sort_by(|a, b| {
                            a.person_data
                                .money
                                .partial_cmp(&b.person_data.money)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        println!(
                            "\n{}",
                            format!("=== Bottom {} Poorest Persons ===", count).bright_yellow()
                        );
                        println!(
                            "{:>5} {:>5} {:>15} {:>10} {:>10}",
                            "Rank", "ID", "Money", "Savings", "Skills"
                        );
                        println!("{}", "─".repeat(52));

                        for (i, entity) in entities.iter().take(count).enumerate() {
                            let person = &entity.person_data;
                            let total_skills =
                                person.own_skills.len() + person.learned_skills.len();
                            println!(
                                "{:>5} {:>5} {:>15.2} {:>10.2} {:>10}",
                                i + 1,
                                entity.id,
                                person.money,
                                person.savings,
                                total_skills
                            );
                        }
                    },
                    "filter-by-skill" => {
                        if parts.len() < 2 {
                            println!("{}", "Usage: filter-by-skill <skill_name>".bright_red());
                            continue;
                        }

                        let skill_name = parts[1..].join(" ");
                        let entities = engine.get_entities();

                        let matching: Vec<_> = entities
                            .iter()
                            .filter(|e| {
                                e.person_data.own_skills.iter().any(|s| {
                                    s.id.to_lowercase().contains(&skill_name.to_lowercase())
                                }) || e.person_data.learned_skills.iter().any(|s| {
                                    s.id.to_lowercase().contains(&skill_name.to_lowercase())
                                })
                            })
                            .collect();

                        if matching.is_empty() {
                            println!(
                                "{}",
                                format!("No persons found with skill matching '{}'", skill_name)
                                    .bright_yellow()
                            );
                        } else {
                            println!(
                                "\n{}",
                                format!(
                                    "=== Persons with '{}' ({} found) ===",
                                    skill_name,
                                    matching.len()
                                )
                                .bright_yellow()
                            );
                            println!("{:>5} {:>15} {:>10}", "ID", "Money", "Reputation");
                            println!("{}", "─".repeat(33));

                            for entity in matching {
                                let person = &entity.person_data;
                                println!(
                                    "{:>5} {:>15.2} {:>10.3}",
                                    entity.id, person.money, person.reputation
                                );
                            }
                        }
                    },
                    "exit" | "quit" => {
                        println!("{}", "Exiting interactive mode...".bright_yellow());
                        break;
                    },
                    _ => {
                        println!(
                            "{}",
                            format!(
                                "Unknown command: '{}'. Type 'help' for available commands.",
                                command
                            )
                            .bright_red()
                        );
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C pressed
                println!("{}", "\nInterrupted. Type 'exit' or 'quit' to exit.".bright_yellow());
            },
            Err(ReadlineError::Eof) => {
                // Ctrl+D pressed
                println!("{}", "\nEOF received. Exiting...".bright_yellow());
                break;
            },
            Err(err) => {
                println!("{}", format!("Error: {:?}", err).bright_red());
                break;
            },
        }
    }

    println!("{}", "Interactive mode ended.".bright_green());
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
    use community_simulation::MonteCarloResult;
    use rayon::prelude::*;

    let start_time = Instant::now();
    let base_seed = base_config.seed;

    info!("Starting {} parallel simulation runs...", num_runs);

    // Run simulations in parallel using Rayon
    let results: Vec<_> = (0..num_runs)
        .into_par_iter()
        .map(|run_idx| {
            let mut config = base_config.clone();
            config.seed = base_seed + run_idx as u64;

            info!("Starting run {}/{} (seed: {})", run_idx + 1, num_runs, config.seed);

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
        format!("All Monte Carlo runs completed in {:.2}s", total_duration.as_secs_f64())
            .bright_green()
    );
    info!(
        "{}",
        format!("Average time per run: {:.2}s", total_duration.as_secs_f64() / num_runs as f64)
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
            info!("{}", format!("Monte Carlo results saved to {}", output_path).bright_blue());
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
    use community_simulation::ParameterRange;
    use community_simulation::ParameterSweepResult;

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
    let min: f64 = parts[1].parse().map_err(|_| format!("Invalid min value: '{}'", parts[1]))?;
    let max: f64 = parts[2].parse().map_err(|_| format!("Invalid max value: '{}'", parts[2]))?;
    let steps: usize =
        parts[3].parse().map_err(|_| format!("Invalid steps value: '{}'", parts[3]))?;

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
        },
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
        info!("{}", format!("Parameter sweep results saved to {}", output_path).bright_blue());
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
    use community_simulation::{Scenario, ScenarioComparisonResult};
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
        scenarios.iter().map(|s| format!("{:?}", s)).collect::<Vec<_>>().join(", ")
    );
    info!("Total simulations to run: {}", scenarios.len() * runs_per_scenario);

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
