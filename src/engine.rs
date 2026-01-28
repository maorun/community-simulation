use crate::{
    contract::{Contract, ContractId},
    credit_rating::DEFAULT_CREDIT_SCORE,
    crisis::CrisisEvent,
    environment::Environment,
    event::EventBus,
    loan::{Loan, LoanId},
    person::{PersonId, Strategy},
    plugin::{PluginContext, PluginRegistry},
    result::{write_step_to_stream, StepData},
    scenario::{DemandGenerator, PriceUpdater},
    Entity, Market, SimulationConfig, SimulationResult, Skill, SkillId,
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, trace, warn};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::panic;
use std::path::Path;
use std::time::Instant;

// Technology shock crisis constants
const TECH_SHOCK_MIN_AFFECTED_PERCENTAGE: f64 = 0.20; // Minimum 20% of skills affected
const TECH_SHOCK_SEVERITY_RANGE: f64 = 0.20; // Additional 20% based on severity

/// Represents a positive technology breakthrough event.
///
/// Breakthroughs are sudden innovations that boost the efficiency of specific skills,
/// representing disruptive technologies, major discoveries, or breakthrough innovations
/// (e.g., AI tools boosting programmer productivity, new manufacturing techniques).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyBreakthrough {
    /// The skill ID that received the breakthrough
    pub skill_id: SkillId,
    /// The efficiency multiplier applied (e.g., 1.3 = 30% boost)
    pub efficiency_boost: f64,
    /// The simulation step when this breakthrough occurred
    pub step_occurred: usize,
}

/// Checkpoint structure for saving and restoring simulation state.
///
/// This structure captures all the stateful information needed to resume
/// a simulation from a specific point. The random number generator state
/// is not included; instead, the RNG is reseeded based on the current step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationCheckpoint {
    /// Configuration used for this simulation
    pub config: SimulationConfig,
    /// All entities in the simulation
    pub entities: Vec<Entity>,
    /// Market state including prices and history
    pub market: Market,
    /// Black market state (if enabled)
    pub black_market: Option<Market>,
    /// Current simulation step
    pub current_step: usize,
    /// All skill IDs in the market
    pub all_skill_ids: Vec<SkillId>,
    /// Trade volume per step history
    pub trades_per_step: Vec<usize>,
    /// Money volume per step history
    pub volume_per_step: Vec<f64>,
    /// Black market trade volume per step history
    pub black_market_trades_per_step: Vec<usize>,
    /// Black market money volume per step history
    pub black_market_volume_per_step: Vec<f64>,
    /// Total transaction fees collected
    pub total_fees_collected: f64,
    /// Number of failed steps (recovered from panics)
    pub failed_steps: usize,
    /// Total failed trade attempts (due to insufficient funds)
    pub failed_trade_attempts: usize,
    /// Failed trade attempts per step history
    pub failed_attempts_per_step: Vec<usize>,
    /// All loans in the system
    pub loans: HashMap<LoanId, Loan>,
    /// Total loans issued counter
    pub total_loans_issued: usize,
    /// Total loans repaid counter
    pub total_loans_repaid: usize,
    /// Total taxes collected during the simulation
    pub total_taxes_collected: f64,
    /// Total amount redistributed through tax system
    pub total_taxes_redistributed: f64,
    /// Per-skill trade tracking: (skill_id -> (trade_count, total_volume))
    pub per_skill_trades: HashMap<SkillId, (usize, f64)>,
    /// Per-seller, per-skill trade tracking for market concentration analysis
    /// (skill_id -> (seller_id -> total_volume))
    pub per_skill_seller_volumes: HashMap<SkillId, HashMap<usize, f64>>,
    /// All contracts in the system
    pub contracts: HashMap<ContractId, Contract>,
    /// Total contracts created counter
    pub total_contracts_created: usize,
    /// Total contracts completed counter
    pub total_contracts_completed: usize,
    /// Time-series of wealth distribution statistics
    pub wealth_stats_history: Vec<crate::result::WealthStatsSnapshot>,
    /// Incremental money statistics for O(1) retrieval
    pub money_incremental_stats: crate::result::IncrementalStats,
    pub min_money: f64,
    pub max_money: f64,
    /// Social mobility tracking: person_id -> Vec of quintile assignments at each step
    pub mobility_quintiles: HashMap<usize, Vec<usize>>,
    /// Environmental resource tracking (if enabled)
    pub environment: Option<Environment>,
    /// Voting system state (if enabled)
    pub voting_system: Option<crate::voting::VotingSystem>,
    /// Certification system tracking
    pub total_certifications_issued: usize,
    pub total_certifications_expired: usize,
    pub total_certification_cost: f64,
    /// Resource pool tracking: group_id -> (balance, total_contributions, total_withdrawals)
    pub resource_pools: HashMap<usize, (f64, f64, f64)>,
    /// Trade agreement system tracking
    pub trade_agreements: Vec<crate::trade_agreement::TradeAgreement>,
    pub total_trade_agreements_formed: usize,
    pub total_trade_agreements_expired: usize,
    pub trade_agreement_counter: usize,
    /// Trust network system (if enabled)
    pub trust_network: Option<crate::trust_network::TrustNetwork>,
    /// Insurance system tracking
    pub insurances: HashMap<crate::insurance::InsuranceId, crate::insurance::Insurance>,
    pub insurance_counter: usize,
    pub total_insurance_policies_issued: usize,
    pub total_insurance_claims_paid: usize,
    pub total_premiums_collected: f64,
    pub total_payouts_made: f64,
    /// Technology breakthrough tracking
    pub technology_breakthroughs: Vec<TechnologyBreakthrough>,
    /// Action log for replay recording (optional)
    pub action_log: Option<crate::replay::ActionLog>,
    /// Externality tracking (if enabled)
    pub externality_stats: crate::externality::ExternalityStats,
}

pub struct SimulationEngine {
    config: SimulationConfig,
    entities: Vec<Entity>,
    market: Market,
    /// Secondary market for black market trades (if enabled)
    black_market: Option<Market>,
    pub current_step: usize,
    rng: StdRng,
    all_skill_ids: Vec<SkillId>,
    /// Demand generator for determining number of needed skills per person
    demand_generator: DemandGenerator,
    // Trade volume tracking
    trades_per_step: Vec<usize>,
    volume_per_step: Vec<f64>,
    // Black market trade tracking
    black_market_trades_per_step: Vec<usize>,
    black_market_volume_per_step: Vec<f64>,
    // Transaction fees tracking
    total_fees_collected: f64,
    // Panic recovery tracking
    failed_steps: usize,
    // Failed trade attempts tracking
    failed_trade_attempts: usize,
    failed_attempts_per_step: Vec<usize>,
    // Loan system tracking
    loans: HashMap<LoanId, Loan>,
    total_loans_issued: usize,
    total_loans_repaid: usize,
    // Tax system tracking
    total_taxes_collected: f64,
    total_taxes_redistributed: f64,
    // Per-skill trade tracking: (skill_id -> (trade_count, total_volume))
    per_skill_trades: HashMap<SkillId, (usize, f64)>,
    // Per-seller, per-skill trade tracking for market concentration analysis
    // (skill_id -> (seller_id -> total_volume))
    per_skill_seller_volumes: HashMap<SkillId, HashMap<usize, f64>>,
    // Streaming output writer
    stream_writer: Option<BufWriter<File>>,
    // Contract system tracking
    contracts: HashMap<ContractId, Contract>,
    total_contracts_created: usize,
    total_contracts_completed: usize,
    // Mentorship system tracking
    mentorships: Vec<crate::person::Mentorship>,
    total_mentorships_formed: usize,
    successful_mentored_learnings: usize,
    total_mentorship_cost_savings: f64,
    unique_mentors: HashSet<usize>,
    unique_mentees: HashSet<usize>,
    // Certification system tracking
    total_certifications_issued: usize,
    total_certifications_expired: usize,
    total_certification_cost: f64,
    // Wealth statistics history tracking
    wealth_stats_history: Vec<crate::result::WealthStatsSnapshot>,
    // Incremental money statistics tracking for O(1) retrieval
    money_incremental_stats: crate::result::IncrementalStats,
    min_money: f64,
    max_money: f64,
    // Social mobility tracking: person_id -> Vec of quintile assignments (0-4) at each step
    mobility_quintiles: HashMap<usize, Vec<usize>>,
    // Plugin system for extending simulation
    plugin_registry: PluginRegistry,
    // Resource pool tracking: group_id -> (balance, total_contributions, total_withdrawals)
    resource_pools: HashMap<usize, (f64, f64, f64)>,
    // Production system recipes (cached for performance)
    production_recipes: Option<Vec<crate::production::Recipe>>,
    // Environmental resource tracking (if enabled)
    environment: Option<Environment>,
    // Voting system for governance and collective decision-making (if enabled)
    voting_system: Option<crate::voting::VotingSystem>,
    // Event bus for tracking simulation events (if enabled)
    event_bus: EventBus,
    // Trade agreement system tracking
    trade_agreements: Vec<crate::trade_agreement::TradeAgreement>,
    total_trade_agreements_formed: usize,
    total_trade_agreements_expired: usize,
    trade_agreement_counter: usize,
    // Trust network system (if enabled)
    trust_network: Option<crate::trust_network::TrustNetwork>,
    // Insurance system tracking
    insurances: HashMap<crate::insurance::InsuranceId, crate::insurance::Insurance>,
    insurance_counter: usize,
    total_insurance_policies_issued: usize,
    total_insurance_claims_paid: usize,
    total_premiums_collected: f64,
    total_payouts_made: f64,
    // Technology breakthrough tracking
    technology_breakthroughs: Vec<TechnologyBreakthrough>,
    // Action log for replay recording (optional)
    action_log: Option<crate::replay::ActionLog>,
    // Externality tracking (if enabled)
    externality_stats: crate::externality::ExternalityStats,
}

impl SimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let price_updater = PriceUpdater::from(config.scenario.clone());
        let demand_generator = DemandGenerator::from(config.demand_strategy.clone());
        let mut market = Market::new(
            config.base_skill_price,
            config.min_skill_price,
            config.price_elasticity_factor,
            config.volatility_percentage,
            price_updater.clone(),
        );

        // This is the version from feat/economic-simulation-model
        let entities = Self::initialize_entities(&config, &mut rng, &mut market);

        // Apply per-skill price limits from configuration
        if !config.per_skill_price_limits.is_empty() {
            // Create mapping from skill name to skill ID (they're both Strings, so this is an identity mapping)
            // We do this upfront to avoid borrowing issues when setting limits
            let skill_name_to_id: HashMap<String, SkillId> =
                market.skills.iter().map(|(id, skill)| (skill.id.clone(), id.clone())).collect();

            for (skill_name, (min_price, max_price)) in &config.per_skill_price_limits {
                if let Some(skill_id) = skill_name_to_id.get(skill_name) {
                    market.set_per_skill_price_limits(skill_id, *min_price, *max_price);
                    debug!(
                        "Set per-skill price limits for '{}': min={:?}, max={:?}",
                        skill_name, min_price, max_price
                    );
                }
            }
        }

        let all_skill_ids = market.skills.keys().cloned().collect::<Vec<SkillId>>();

        // Initialize black market if enabled
        let black_market = if config.enable_black_market {
            let mut bm = Market::new(
                config.base_skill_price * config.black_market_price_multiplier,
                config.min_skill_price * config.black_market_price_multiplier,
                config.price_elasticity_factor,
                config.volatility_percentage,
                price_updater,
            );
            // Add all skills to black market with adjusted prices
            for skill_id in &all_skill_ids {
                if let Some(skill) = market.skills.get(skill_id) {
                    let bm_skill = Skill::new(
                        skill.id.clone(),
                        skill.current_price * config.black_market_price_multiplier,
                    );
                    bm.add_skill(bm_skill);
                }
            }
            debug!(
                "Black market initialized with price multiplier: {}",
                config.black_market_price_multiplier
            );
            Some(bm)
        } else {
            None
        };

        // Initialize streaming output writer if path is provided
        let stream_writer = if let Some(path) = &config.stream_output_path {
            match File::create(path) {
                Ok(file) => Some(BufWriter::new(file)),
                Err(e) => {
                    warn!(
                        "Failed to create streaming output file: {}. Continuing without streaming.",
                        e
                    );
                    None
                },
            }
        } else {
            None
        };

        // Cache production recipes if production is enabled
        let production_recipes = if config.enable_production {
            Some(crate::production::generate_default_recipes())
        } else {
            None
        };

        // Initialize environment if enabled
        let environment = if config.enable_environment {
            use crate::environment::Resource;
            use std::collections::HashMap as StdHashMap;

            let env = if let Some(custom_reserves) = &config.custom_resource_reserves {
                // Parse custom reserves from string keys to Resource enum
                let mut reserves = StdHashMap::new();
                for (resource_name, amount) in custom_reserves {
                    match resource_name.to_lowercase().as_str() {
                        "energy" => reserves.insert(Resource::Energy, *amount),
                        "water" => reserves.insert(Resource::Water, *amount),
                        "materials" => reserves.insert(Resource::Materials, *amount),
                        "land" => reserves.insert(Resource::Land, *amount),
                        _ => {
                            warn!("Unknown resource type: {}, ignoring", resource_name);
                            continue;
                        },
                    };
                }
                Environment::new(reserves)
            } else {
                Environment::with_default_reserves()
            };
            debug!("Environment tracking initialized");
            Some(env)
        } else {
            None
        };

        // Initialize voting system if enabled
        let voting_system = if config.enable_voting {
            debug!("Voting system initialized with method: {:?}", config.voting_method);
            Some(crate::voting::VotingSystem::new(config.voting_method))
        } else {
            None
        };

        // Initialize event bus
        let event_bus = EventBus::new(config.enable_events);
        if config.enable_events {
            debug!("Event tracking system enabled");
        }

        // Initialize resource pools before moving config
        let resource_pools = {
            let mut pools = HashMap::new();
            if config.enable_resource_pools {
                if let Some(num_groups) = config.num_groups {
                    for group_id in 0..num_groups {
                        // Initialize pool with (balance, total_contributions, total_withdrawals)
                        pools.insert(group_id, (0.0, 0.0, 0.0));
                    }
                    debug!(
                        "Resource pools initialized for {} groups with {}% contribution rate",
                        num_groups,
                        config.pool_contribution_rate * 100.0
                    );
                }
            }
            pools
        };

        // Initialize trust network if enabled
        let trust_network = if config.enable_trust_networks {
            let mut network = crate::trust_network::TrustNetwork::new();
            // Add all persons to the trust network
            for entity in entities.iter() {
                network.add_person(entity.person_data.id);
            }
            debug!("Trust network initialized with {} persons", entities.len());
            Some(network)
        } else {
            None
        };

        Self {
            config,
            entities,
            market,
            black_market,
            current_step: 0,
            rng,
            all_skill_ids,
            demand_generator,
            trades_per_step: Vec::new(),
            volume_per_step: Vec::new(),
            black_market_trades_per_step: Vec::new(),
            black_market_volume_per_step: Vec::new(),
            total_fees_collected: 0.0,
            failed_steps: 0,
            failed_trade_attempts: 0,
            failed_attempts_per_step: Vec::new(),
            loans: HashMap::new(),
            total_loans_issued: 0,
            total_loans_repaid: 0,
            total_taxes_collected: 0.0,
            total_taxes_redistributed: 0.0,
            per_skill_trades: HashMap::new(),
            per_skill_seller_volumes: HashMap::new(),
            stream_writer,
            contracts: HashMap::new(),
            total_contracts_created: 0,
            total_contracts_completed: 0,
            mentorships: Vec::new(),
            total_mentorships_formed: 0,
            successful_mentored_learnings: 0,
            total_mentorship_cost_savings: 0.0,
            unique_mentors: HashSet::new(),
            unique_mentees: HashSet::new(),
            total_certifications_issued: 0,
            total_certifications_expired: 0,
            total_certification_cost: 0.0,
            wealth_stats_history: Vec::new(),
            money_incremental_stats: crate::result::IncrementalStats::new(),
            min_money: f64::INFINITY,
            max_money: f64::NEG_INFINITY,
            mobility_quintiles: HashMap::new(),
            plugin_registry: PluginRegistry::new(),
            resource_pools,
            production_recipes,
            environment,
            voting_system,
            event_bus,
            trade_agreements: Vec::new(),
            total_trade_agreements_formed: 0,
            total_trade_agreements_expired: 0,
            trade_agreement_counter: 0,
            trust_network,
            insurances: HashMap::new(),
            insurance_counter: 0,
            total_insurance_policies_issued: 0,
            total_insurance_claims_paid: 0,
            total_premiums_collected: 0.0,
            total_payouts_made: 0.0,
            technology_breakthroughs: Vec::new(),
            action_log: None, // Will be set via enable_action_recording if needed
            externality_stats: crate::externality::ExternalityStats::new(),
        }
    }

    /// Register a plugin with the simulation engine.
    ///
    /// Plugins can hook into various points in the simulation lifecycle
    /// to extend functionality without modifying core code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use simulation_framework::{SimulationEngine, SimulationConfig, Plugin, PluginContext};
    /// use std::any::Any;
    ///
    /// struct MyPlugin;
    /// impl Plugin for MyPlugin {
    ///     fn name(&self) -> &str { "MyPlugin" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn as_any_mut(&mut self) -> &mut dyn Any { self }
    /// }
    ///
    /// let config = SimulationConfig::default();
    /// let mut engine = SimulationEngine::new(config);
    /// engine.register_plugin(Box::new(MyPlugin));
    /// ```
    pub fn register_plugin(&mut self, plugin: Box<dyn crate::plugin::Plugin>) {
        self.plugin_registry.register(plugin);
    }

    /// Get a reference to the plugin registry.
    ///
    /// This allows external code to query registered plugins.
    pub fn plugin_registry(&self) -> &PluginRegistry {
        &self.plugin_registry
    }

    /// Get a mutable reference to the plugin registry.
    ///
    /// This allows external code to interact with plugins.
    pub fn plugin_registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.plugin_registry
    }

    /// Enable action recording for replay and debugging.
    ///
    /// When enabled, all simulation actions (trades, price updates, crisis events)
    /// are logged to an ActionLog that can be saved to a file.
    ///
    /// # Example
    ///
    /// ```rust
    /// use simulation_framework::{SimulationEngine, SimulationConfig};
    ///
    /// let config = SimulationConfig::default();
    /// let mut engine = SimulationEngine::new(config);
    /// engine.enable_action_recording();
    /// ```
    pub fn enable_action_recording(&mut self) {
        self.action_log = Some(crate::replay::ActionLog::new(
            self.config.seed,
            self.config.entity_count,
            self.config.max_steps,
        ));
        debug!("Action recording enabled for replay and debugging");
    }

    /// Save the action log to a file.
    ///
    /// Returns Ok(()) if successful, or an error if action recording is not enabled
    /// or if file I/O fails.
    pub fn save_action_log<P: AsRef<std::path::Path>>(&self, path: P) -> crate::error::Result<()> {
        if let Some(ref log) = self.action_log {
            log.save_to_file(path)?;
            info!("Action log saved successfully ({} actions recorded)", log.len());
            Ok(())
        } else {
            Err(crate::error::SimulationError::ActionLogWrite(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Action recording is not enabled",
            )))
        }
    }

    // This is the version from feat/economic-simulation-model
    fn initialize_entities(
        config: &SimulationConfig,
        rng: &mut StdRng,
        market: &mut Market,
    ) -> Vec<Entity> {
        // Create all unique skills for the market (one per person)
        let mut available_skills_for_market = Vec::new();
        for i in 0..config.entity_count {
            let skill_name = format!("Skill{}", i);
            let skill = Skill::new(skill_name.clone(), config.base_skill_price);
            available_skills_for_market.push(skill.clone());
            market.add_skill(skill);
        }

        let mut entities = Vec::with_capacity(config.entity_count);

        // Distribute skills to persons
        // Strategy: Cycle through skills, assigning skills_per_person skills to each person
        for i in 0..config.entity_count {
            let mut person_skills = Vec::with_capacity(config.skills_per_person);

            // Assign skills_per_person skills to this person
            for j in 0..config.skills_per_person {
                // Calculate which skill this person should get
                // Use a round-robin distribution: person i gets skills at indices
                // (i + j * entity_count) % total_skills
                let skill_index = (i + j * config.entity_count) % config.entity_count;
                let skill = available_skills_for_market[skill_index].clone();

                // Increment supply for this skill in the market
                market.increment_skill_supply(&skill.id);

                person_skills.push(skill);
            }

            // Assign a strategy to this person using round-robin distribution
            // This ensures equal distribution of strategies across the population
            let all_strategies = Strategy::all_variants();
            let strategy = all_strategies[i % all_strategies.len()];

            // Generate random location in 0.0-100.0 range for both x and y
            let location = crate::person::Location::new(
                rng.random_range(0.0..=100.0),
                rng.random_range(0.0..=100.0),
            );

            let mut entity =
                Entity::new(i, config.initial_money_per_person, person_skills, strategy, location);

            // Assign specialization strategy if enabled
            if config.enable_specialization {
                let all_specialization_strategies =
                    crate::person::SpecializationStrategy::all_variants();
                entity.person_data.specialization_strategy =
                    all_specialization_strategies[i % all_specialization_strategies.len()];
            }

            entities.push(entity);
        }

        // Assign groups if configured
        if let Some(num_groups) = config.num_groups {
            for (i, entity) in entities.iter_mut().enumerate() {
                // Round-robin group assignment
                entity.person_data.group_id = Some(i % num_groups);
            }
        }

        // Initialize skill qualities if quality system is enabled
        if config.enable_quality {
            for entity in entities.iter_mut() {
                // Initialize quality for all own skills
                for skill in &entity.person_data.own_skills {
                    entity
                        .person_data
                        .skill_qualities
                        .insert(skill.id.clone(), config.initial_quality);
                }
                // Note: Learned skills will have their quality initialized when learned
            }
            debug!(
                "Skill quality system initialized with initial quality: {}",
                config.initial_quality
            );
        }

        entities
    }

    /// Calculate seasonal demand factor for a specific skill at the current step.
    ///
    /// This function creates cyclical demand variations using sine waves,
    /// with different phase offsets for each skill to create diverse market dynamics.
    ///
    /// # Arguments
    /// * `skill_id` - The skill ID to calculate the seasonal factor for
    ///
    /// # Returns
    /// A multiplier in the range [1.0 - amplitude, 1.0 + amplitude]
    ///
    /// # Visibility
    /// Public for testing purposes
    pub fn calculate_seasonal_factor(&self, skill_id: &SkillId) -> f64 {
        if self.config.seasonal_amplitude == 0.0 || self.config.seasonal_period == 0 {
            return 1.0;
        }

        // Use skill ID hash to create a unique phase offset for each skill
        // This ensures different skills peak at different times
        let skill_hash = skill_id
            .chars()
            .fold(0u32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u32));
        // Scale hash to phase range: 0.01 scales the u32 hash to a reasonable phase offset
        // that distributes skills across the full 2π cycle without clustering.
        // This creates diverse seasonal patterns where different skills peak at different times.
        let phase_offset = (skill_hash as f64) * 0.01;

        // Calculate current position in the seasonal cycle
        let cycle_position = (self.current_step as f64 / self.config.seasonal_period as f64)
            * 2.0
            * std::f64::consts::PI;

        // Calculate sine wave with phase offset
        let sine_value = (cycle_position + phase_offset).sin();

        // Scale sine wave (-1 to 1) by amplitude and center around 1.0
        1.0 + sine_value * self.config.seasonal_amplitude
    }

    /// Check for and potentially trigger a crisis event.
    ///
    /// This method is called once per simulation step. It randomly determines
    /// whether a crisis occurs based on the configured probability, and if so,
    /// randomly selects a crisis type and applies its effects to the economy.
    fn check_and_trigger_crisis(&mut self) {
        if !self.config.enable_crisis_events {
            return;
        }

        // Check if a crisis should occur this step
        let random_value: f64 = self.rng.random();
        if random_value >= self.config.crisis_probability {
            return; // No crisis this step
        }

        // Select a random crisis type
        let crisis_types = CrisisEvent::all_types();
        let crisis = crisis_types.choose(&mut self.rng).unwrap();

        info!(
            "🚨 CRISIS EVENT at step {}: {} - {}",
            self.current_step,
            crisis.name(),
            crisis.description()
        );

        // Record crisis event action for replay (if enabled)
        if let Some(ref mut action_log) = self.action_log {
            action_log.record(crate::replay::SimulationAction::CrisisEvent {
                step: self.current_step,
                event_type: crisis.name().to_string(),
                severity: self.config.crisis_severity,
            });
        }

        // Apply crisis effects based on type
        match crisis {
            CrisisEvent::MarketCrash => {
                // Reduce all skill prices
                debug!("Applying market crash: reducing all skill prices");
                for (_skill_id, skill) in self.market.skills.iter_mut() {
                    let old_price = skill.current_price;
                    skill.current_price = crisis.apply_effect(
                        skill.current_price,
                        self.config.crisis_severity,
                        &mut self.rng,
                    );
                    // Respect minimum price floor
                    skill.current_price = skill.current_price.max(self.config.min_skill_price);
                    debug!(
                        "  Skill {}: ${:.2} -> ${:.2}",
                        skill.id, old_price, skill.current_price
                    );
                }
                // Also apply to black market if enabled
                if let Some(ref mut bm) = self.black_market {
                    for (_skill_id, skill) in bm.skills.iter_mut() {
                        skill.current_price = crisis.apply_effect(
                            skill.current_price,
                            self.config.crisis_severity,
                            &mut self.rng,
                        );
                        skill.current_price = skill.current_price.max(self.config.min_skill_price);
                    }
                }
            },
            CrisisEvent::DemandShock => {
                // Reduce demand by removing some needed skills from entities
                debug!("Applying demand shock: reducing skill demand");
                for entity in self.entities.iter_mut() {
                    if !entity.active {
                        continue;
                    }
                    let original_count = entity.person_data.needed_skills.len();
                    if original_count > 0 {
                        // Apply crisis effect to determine how many needs to keep
                        let reduction_factor =
                            crisis.apply_effect(1.0, self.config.crisis_severity, &mut self.rng);
                        let keep_ratio = 1.0 - reduction_factor;
                        let keep_count = ((original_count as f64) * keep_ratio).ceil() as usize;
                        entity.person_data.needed_skills.truncate(keep_count.max(0));
                    }
                }
            },
            CrisisEvent::SupplyShock => {
                // Temporarily reduce supply (simulated by temporarily disabling some entities)
                // For simplicity, we'll just log this - in a more complex implementation,
                // we could track "supply reduction" and reduce effective supply counts
                debug!("Applying supply shock: supply chain disruptions");
                // Apply effect to supply counts in the market
                for (_skill_id, count) in self.market.supply_counts.iter_mut() {
                    let old_supply = *count;
                    let reduction_factor =
                        crisis.apply_effect(1.0, self.config.crisis_severity, &mut self.rng);
                    *count = ((old_supply as f64) * reduction_factor) as usize;
                    debug!("  Supply reduced from {} to {}", old_supply, *count);
                }
            },
            CrisisEvent::CurrencyDevaluation => {
                // Reduce everyone's money holdings
                debug!("Applying currency devaluation: reducing all money holdings");
                for entity in self.entities.iter_mut() {
                    if !entity.active {
                        continue;
                    }
                    let old_money = entity.person_data.money;
                    entity.person_data.money = crisis.apply_effect(
                        entity.person_data.money,
                        self.config.crisis_severity,
                        &mut self.rng,
                    );
                    // Also affect savings
                    entity.person_data.savings = crisis.apply_effect(
                        entity.person_data.savings,
                        self.config.crisis_severity,
                        &mut self.rng,
                    );
                    debug!(
                        "  Person {}: ${:.2} -> ${:.2}",
                        entity.id, old_money, entity.person_data.money
                    );
                }
            },
            CrisisEvent::TechnologyShock => {
                // Technology shock: randomly select subset of skills to become obsolete
                debug!("Applying technology shock: making skills obsolete");

                // Randomly select 20-40% of skills to be affected (scaled by severity)
                let total_skills = self.market.skills.len();
                let affected_percentage = TECH_SHOCK_MIN_AFFECTED_PERCENTAGE
                    + (self.config.crisis_severity * TECH_SHOCK_SEVERITY_RANGE);
                let num_affected = ((total_skills as f64) * affected_percentage).ceil() as usize;

                // Collect skill IDs and shuffle them to randomly select affected skills
                let mut skill_ids: Vec<_> = self.market.skills.keys().cloned().collect();
                skill_ids.shuffle(&mut self.rng);

                // Take the first N skills as the affected ones
                let affected_skills: Vec<_> =
                    skill_ids.iter().take(num_affected).cloned().collect();

                // Apply massive price drops to affected skills
                for skill_id in &affected_skills {
                    if let Some(skill) = self.market.skills.get_mut(skill_id) {
                        let old_price = skill.current_price;
                        skill.current_price = crisis.apply_effect(
                            skill.current_price,
                            self.config.crisis_severity,
                            &mut self.rng,
                        );
                        // Respect minimum price floor
                        skill.current_price = skill.current_price.max(self.config.min_skill_price);

                        // Safe division by zero handling for percentage calculation
                        let drop_percentage = if old_price > 0.0 {
                            (old_price - skill.current_price) / old_price * 100.0
                        } else {
                            0.0
                        };

                        debug!(
                            "  Skill {} obsolete: ${:.2} -> ${:.2} ({:.0}% drop)",
                            skill.id, old_price, skill.current_price, drop_percentage
                        );
                    }
                }

                // Also apply to black market if enabled
                if let Some(ref mut bm) = self.black_market {
                    for skill_id in &affected_skills {
                        if let Some(skill) = bm.skills.get_mut(skill_id) {
                            skill.current_price = crisis.apply_effect(
                                skill.current_price,
                                self.config.crisis_severity,
                                &mut self.rng,
                            );
                            skill.current_price =
                                skill.current_price.max(self.config.min_skill_price);
                        }
                    }
                }

                info!(
                    "Technology shock affected {} out of {} skills",
                    affected_skills.len(),
                    total_skills
                );
            },
        }

        // Process insurance payouts for crisis events
        self.process_crisis_insurance_payouts(self.config.crisis_severity);
    }

    pub fn run(&mut self) -> SimulationResult {
        self.run_with_progress(false)
    }

    /// Run the simulation with optional progress bar display.
    ///
    /// # Arguments
    /// * `show_progress` - If true, displays a progress bar during simulation
    ///
    /// # Returns
    /// A `SimulationResult` containing all simulation metrics and data
    pub fn run_with_progress(&mut self, show_progress: bool) -> SimulationResult {
        let start_time = Instant::now();
        let mut step_times = Vec::new();

        info!("Starting economic simulation with {} persons", self.entities.len());
        debug!(
            "Simulation configuration: max_steps={}, scenario={:?}",
            self.config.max_steps, self.config.scenario
        );

        // Create plugin context for simulation start
        let persons: Vec<_> = self.entities.iter().map(|e| e.person_data.clone()).collect();
        let start_context = PluginContext {
            config: &self.config,
            current_step: 0,
            total_steps: self.config.max_steps,
            persons: &persons,
        };

        // Notify plugins that simulation is starting
        self.plugin_registry.on_simulation_start(&start_context);

        // Constants for progress bar configuration
        const PROGRESS_BAR_WIDTH: usize = 40;
        const PROGRESS_UPDATE_INTERVAL_STEPS: usize = 10;

        // Create progress bar if requested
        let progress_bar = if show_progress {
            let pb = ProgressBar::new(self.config.max_steps as u64);
            let template_str = format!(
                "{{msg}} [{{elapsed_precise}}] [{{bar:{}.cyan/blue}}] {{pos}}/{{len}} ({{percent}}%) ETA: {{eta}}",
                PROGRESS_BAR_WIDTH
            );
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&template_str)
                    .expect("Invalid progress bar template")
                    .progress_chars("=>-"),
            );
            pb.set_message("Simulating");
            Some(pb)
        } else {
            None
        };

        // Calculate update frequency: update stats every 1% of steps or every 10 steps, whichever is less frequent
        let stats_update_interval =
            (self.config.max_steps / 100).max(PROGRESS_UPDATE_INTERVAL_STEPS);

        for step in 0..self.config.max_steps {
            let step_start = Instant::now();

            // Create plugin context and notify plugins before step starts
            let persons: Vec<_> = self.entities.iter().map(|e| e.person_data.clone()).collect();
            let step_context = PluginContext {
                config: &self.config,
                current_step: self.current_step,
                total_steps: self.config.max_steps,
                persons: &persons,
            };
            self.plugin_registry.on_step_start(&step_context);

            // Catch panics during step execution for graceful degradation
            // Safety: We use AssertUnwindSafe here because:
            // 1. The simulation state is designed to be incrementally updated
            // 2. Failed steps are isolated - they don't affect other steps
            // 3. We explicitly handle the incomplete state by recording zero trades
            // 4. All collections (entities, market) use safe Rust with no raw pointers
            // Note: If a panic occurs, some mid-step state changes may be incomplete,
            // but the simulation can safely continue from the next step.
            let step_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                self.step();
            }));

            // Handle panic if it occurred
            if let Err(panic_info) = step_result {
                self.failed_steps += 1;

                // Extract panic message for logging
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic message".to_string()
                };

                warn!(
                    "Panic caught during step {}: {}. Simulation continues with graceful degradation.",
                    step + 1,
                    panic_msg
                );

                // Record this as a step with zero trades for statistics consistency
                self.trades_per_step.push(0);
                self.volume_per_step.push(0.0);

                // Increment current_step since step() panicked before reaching its increment
                // (step() normally increments at the end of its execution - see line 655)
                self.current_step += 1;
            }

            let step_duration = step_start.elapsed();
            step_times.push(step_duration.as_secs_f64());

            // Notify plugins after step completes
            let persons: Vec<_> = self.entities.iter().map(|e| e.person_data.clone()).collect();
            let step_end_context = PluginContext {
                config: &self.config,
                current_step: self.current_step,
                total_steps: self.config.max_steps,
                persons: &persons,
            };
            self.plugin_registry.on_step_end(&step_end_context);

            // Auto-checkpoint if enabled and at checkpoint interval
            #[allow(clippy::manual_is_multiple_of)] // is_multiple_of is not stable yet
            if self.config.checkpoint_interval > 0
                && self.current_step > 0
                && self.current_step % self.config.checkpoint_interval == 0
            {
                let checkpoint_path = self
                    .config
                    .checkpoint_file
                    .clone()
                    .unwrap_or_else(|| "checkpoint.json".to_string());

                if let Err(e) = self.save_checkpoint(&checkpoint_path) {
                    warn!("Failed to save checkpoint at step {}: {}", self.current_step, e);
                } else {
                    debug!("Auto-checkpoint saved at step {}", self.current_step);
                }
            }

            // Update progress bar if enabled
            if let Some(ref pb) = progress_bar {
                pb.inc(1);

                // Update message with additional info at calculated intervals
                if step % stats_update_interval == 0 || step == self.config.max_steps - 1 {
                    let active_entities = self.entities.iter().filter(|e| e.active).count();
                    let avg_money = self.calculate_average_money();

                    // Calculate additional metrics for enhanced progress bar
                    let trades_this_step = self.trades_per_step.last().copied().unwrap_or(0);
                    let avg_price = self.market.get_average_price();

                    // Calculate Gini coefficient for wealth inequality
                    // Collect and sort money values
                    let mut money_values: Vec<f64> = self
                        .entities
                        .iter()
                        .filter(|e| e.active)
                        .map(|e| e.person_data.money)
                        .collect();
                    money_values
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    // Calculate sum once and pass to gini coefficient function
                    let sum: f64 = money_values.iter().sum();
                    let gini = if sum > 0.0 {
                        crate::result::calculate_gini_coefficient(&money_values, sum)
                    } else {
                        0.0
                    };

                    pb.set_message(format!(
                        "Active: {} | $̄: {:.1} | Trades: {} | P̄: {:.1} | Gini: {:.3}",
                        active_entities, avg_money, trades_this_step, avg_price, gini
                    ));
                }
            } else {
                // Fallback to old-style progress logging if no progress bar
                if step % (self.config.max_steps / 10).max(1) == 0
                    || step == self.config.max_steps - 1
                {
                    let active_entities = self.entities.iter().filter(|e| e.active).count();
                    debug!(
                        "Step {}/{}, Active persons: {}, Avg Money: {:.2}",
                        step + 1,
                        self.config.max_steps,
                        active_entities,
                        self.calculate_average_money()
                    );
                }
            }
        }

        // Finish progress bar
        if let Some(pb) = progress_bar {
            pb.finish_with_message("Simulation complete");
        }

        let total_duration = start_time.elapsed();

        let mut final_money_distribution: Vec<f64> =
            self.entities.iter().filter(|e| e.active).map(|e| e.person_data.money).collect();
        final_money_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut final_reputation_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.reputation)
            .collect();
        final_reputation_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut final_savings_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.savings)
            .collect();
        final_savings_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate money statistics using centralized function, then override incremental values
        let money_stats = if !final_money_distribution.is_empty() {
            let mut stats = crate::result::calculate_money_stats(&final_money_distribution);

            // Override with incrementally tracked values for better performance
            // (these were tracked during simulation with O(1) updates)
            stats.average = self.money_incremental_stats.mean();
            stats.std_dev = self.money_incremental_stats.std_dev();
            stats.min_money = self.min_money;
            stats.max_money = self.max_money;

            stats
        } else {
            crate::result::calculate_money_stats(&[])
        };

        let reputation_stats = if !final_reputation_distribution.is_empty() {
            let sum: f64 = final_reputation_distribution.iter().sum();
            let count = final_reputation_distribution.len() as f64;
            let average = sum / count;
            let median = if count > 0.0 {
                if count as usize % 2 == 1 {
                    final_reputation_distribution[count as usize / 2]
                } else {
                    (final_reputation_distribution[count as usize / 2 - 1]
                        + final_reputation_distribution[count as usize / 2])
                        / 2.0
                }
            } else {
                1.0
            };
            let variance = final_reputation_distribution
                .iter()
                .map(|value| {
                    let diff = average - value;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            let std_dev = variance.sqrt();

            crate::result::ReputationStats {
                average,
                median,
                std_dev,
                min_reputation: *final_reputation_distribution.first().unwrap_or(&1.0),
                max_reputation: *final_reputation_distribution.last().unwrap_or(&1.0),
            }
        } else {
            crate::result::ReputationStats {
                average: 1.0,
                median: 1.0,
                std_dev: 0.0,
                min_reputation: 1.0,
                max_reputation: 1.0,
            }
        };

        let savings_stats = if !final_savings_distribution.is_empty() {
            let sum: f64 = final_savings_distribution.iter().sum();
            let count = final_savings_distribution.len() as f64;
            let average = sum / count;
            let median = if count > 0.0 {
                if count as usize % 2 == 1 {
                    final_savings_distribution[count as usize / 2]
                } else {
                    (final_savings_distribution[count as usize / 2 - 1]
                        + final_savings_distribution[count as usize / 2])
                        / 2.0
                }
            } else {
                0.0
            };

            crate::result::SavingsStats {
                total_savings: sum,
                average_savings: average,
                median_savings: median,
                min_savings: *final_savings_distribution.first().unwrap_or(&0.0),
                max_savings: *final_savings_distribution.last().unwrap_or(&0.0),
            }
        } else {
            crate::result::SavingsStats {
                total_savings: 0.0,
                average_savings: 0.0,
                median_savings: 0.0,
                min_savings: 0.0,
                max_savings: 0.0,
            }
        };

        let final_skill_prices_map = self.market.get_all_skill_prices();
        let mut final_skill_prices_vec: Vec<crate::result::SkillPriceInfo> = final_skill_prices_map
            .into_iter()
            .map(|(id, price)| crate::result::SkillPriceInfo { id, price })
            .collect();

        final_skill_prices_vec
            .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));

        let most_valuable_skill = final_skill_prices_vec.first().cloned();
        let least_valuable_skill = final_skill_prices_vec.last().cloned();

        // Calculate trade volume statistics
        let total_trades: usize = self.trades_per_step.iter().sum();
        let total_volume: f64 = self.volume_per_step.iter().sum();
        let steps_with_data = self.trades_per_step.len() as f64;

        let trade_volume_statistics = if steps_with_data > 0.0 {
            let avg_trades_per_step = total_trades as f64 / steps_with_data;
            let avg_volume_per_step = total_volume / steps_with_data;
            let avg_transaction_value = if total_trades > 0 {
                total_volume / total_trades as f64
            } else {
                0.0
            };
            let min_trades_per_step = *self.trades_per_step.iter().min().unwrap_or(&0);
            let max_trades_per_step = *self.trades_per_step.iter().max().unwrap_or(&0);

            // Calculate velocity of money: Total Transaction Volume / Total Money Supply
            // Total money supply is the sum of all money held by all persons
            let velocity_of_money = if !final_money_distribution.is_empty() {
                let total_money_supply = final_money_distribution.iter().sum::<f64>();
                if total_money_supply > 0.0 {
                    total_volume / total_money_supply
                } else {
                    0.0
                }
            } else {
                0.0 // No entities means no velocity
            };

            crate::result::TradeVolumeStats {
                total_trades,
                total_volume,
                avg_trades_per_step,
                avg_volume_per_step,
                avg_transaction_value,
                min_trades_per_step,
                max_trades_per_step,
                velocity_of_money,
            }
        } else {
            crate::result::TradeVolumeStats {
                total_trades: 0,
                total_volume: 0.0,
                avg_trades_per_step: 0.0,
                avg_volume_per_step: 0.0,
                avg_transaction_value: 0.0,
                min_trades_per_step: 0,
                max_trades_per_step: 0,
                velocity_of_money: 0.0,
            }
        };

        // Calculate loan statistics if loans are enabled
        let loan_statistics = if self.config.enable_loans {
            Some(crate::result::LoanStats {
                total_loans_issued: self.total_loans_issued,
                total_loans_repaid: self.total_loans_repaid,
                active_loans: self.loans.len(),
            })
        } else {
            None
        };

        // Calculate insurance statistics if insurance is enabled
        let insurance_statistics = if self.config.enable_insurance {
            let active_policies = self
                .insurances
                .values()
                .filter(|policy| policy.is_active && !policy.is_expired(self.current_step))
                .count();

            let loss_ratio = if self.total_premiums_collected > 0.0 {
                self.total_payouts_made / self.total_premiums_collected
            } else {
                0.0
            };

            Some(crate::result::InsuranceStats {
                total_policies_issued: self.total_insurance_policies_issued,
                active_policies,
                total_claims_paid: self.total_insurance_claims_paid,
                total_premiums_collected: self.total_premiums_collected,
                total_payouts_made: self.total_payouts_made,
                net_result: self.total_premiums_collected - self.total_payouts_made,
                loss_ratio,
            })
        } else {
            None
        };

        // Calculate technology breakthrough statistics if enabled
        let technology_breakthrough_statistics = if self.config.enable_technology_breakthroughs
            && !self.technology_breakthroughs.is_empty()
        {
            let total_breakthroughs = self.technology_breakthroughs.len();
            let unique_skills_affected = self
                .technology_breakthroughs
                .iter()
                .map(|b| &b.skill_id)
                .collect::<std::collections::HashSet<_>>()
                .len();

            let average_efficiency_boost =
                self.technology_breakthroughs.iter().map(|b| b.efficiency_boost).sum::<f64>()
                    / total_breakthroughs as f64;

            let min_efficiency_boost = self
                .technology_breakthroughs
                .iter()
                .map(|b| b.efficiency_boost)
                .fold(f64::INFINITY, f64::min);

            let max_efficiency_boost = self
                .technology_breakthroughs
                .iter()
                .map(|b| b.efficiency_boost)
                .fold(f64::NEG_INFINITY, f64::max);

            let breakthrough_events = self
                .technology_breakthroughs
                .iter()
                .map(|b| crate::result::BreakthroughEvent {
                    skill_id: b.skill_id.clone(),
                    efficiency_boost: b.efficiency_boost,
                    step: b.step_occurred,
                })
                .collect();

            Some(crate::result::TechnologyBreakthroughStats {
                total_breakthroughs,
                unique_skills_affected,
                average_efficiency_boost,
                min_efficiency_boost,
                max_efficiency_boost,
                breakthrough_events,
            })
        } else {
            None
        };

        // Calculate failed trade attempt statistics
        let failed_trade_statistics = {
            let total_attempts = total_trades + self.failed_trade_attempts;
            let failure_rate = if total_attempts > 0 {
                self.failed_trade_attempts as f64 / total_attempts as f64
            } else {
                0.0
            };

            let avg_failed_per_step = if steps_with_data > 0.0 {
                self.failed_trade_attempts as f64 / steps_with_data
            } else {
                0.0
            };

            let min_failed_per_step = *self.failed_attempts_per_step.iter().min().unwrap_or(&0);
            let max_failed_per_step = *self.failed_attempts_per_step.iter().max().unwrap_or(&0);

            crate::result::FailedTradeStats {
                total_failed_attempts: self.failed_trade_attempts,
                failure_rate,
                avg_failed_per_step,
                min_failed_per_step,
                max_failed_per_step,
            }
        };

        // Calculate per-skill trade statistics
        let mut per_skill_trade_stats: Vec<crate::result::SkillTradeStats> = self
            .per_skill_trades
            .iter()
            .map(|(skill_id, (trade_count, total_volume))| {
                let avg_price = if *trade_count > 0 {
                    total_volume / (*trade_count as f64)
                } else {
                    0.0
                };
                crate::result::SkillTradeStats {
                    skill_id: skill_id.clone(),
                    trade_count: *trade_count,
                    total_volume: *total_volume,
                    avg_price,
                }
            })
            .collect();

        // Sort by total volume (highest first)
        per_skill_trade_stats.sort_by(|a, b| {
            b.total_volume.partial_cmp(&a.total_volume).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate market concentration metrics for each skill
        let skill_market_concentration: Option<Vec<crate::result::SkillMarketConcentration>> =
            if !self.per_skill_seller_volumes.is_empty() {
                let mut concentrations: Vec<crate::result::SkillMarketConcentration> = self
                    .per_skill_seller_volumes
                    .iter()
                    .filter_map(|(skill_id, seller_volumes)| {
                        crate::result::calculate_skill_market_concentration(
                            skill_id.clone(),
                            seller_volumes,
                        )
                    })
                    .collect();

                // Sort by HHI (most concentrated first) for easier analysis
                concentrations.sort_by(|a, b| {
                    b.herfindahl_index
                        .partial_cmp(&a.herfindahl_index)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                Some(concentrations)
            } else {
                None
            };

        // Capture metadata for this simulation run
        let metadata = crate::result::SimulationMetadata::capture(
            self.config.seed,
            self.config.entity_count,
            self.config.max_steps,
        );

        let mut result = SimulationResult {
            metadata,
            total_steps: self.config.max_steps,
            total_duration: total_duration.as_secs_f64(),
            step_times,
            active_persons: self.entities.iter().filter(|e| e.active).count(),
            failed_steps: self.failed_steps,
            final_money_distribution,
            money_statistics: money_stats,
            final_reputation_distribution,
            reputation_statistics: reputation_stats,
            final_savings_distribution,
            savings_statistics: savings_stats,
            credit_score_statistics: if self.config.enable_credit_rating {
                self.calculate_credit_score_statistics()
            } else {
                None
            },
            final_skill_prices: final_skill_prices_vec,
            most_valuable_skill,
            least_valuable_skill,
            skill_price_history: self.market.skill_price_history.clone(),
            wealth_stats_history: self.wealth_stats_history.clone(),
            trade_volume_statistics,
            trades_per_step: self.trades_per_step.clone(),
            volume_per_step: self.volume_per_step.clone(),
            total_fees_collected: self.total_fees_collected,
            per_skill_trade_stats,
            skill_market_concentration,
            business_cycle_statistics: crate::result::detect_business_cycles(&self.volume_per_step),
            failed_trade_statistics,
            failed_attempts_per_step: self.failed_attempts_per_step.clone(),
            black_market_statistics: if self.config.enable_black_market {
                let total_black_market_trades: usize =
                    self.black_market_trades_per_step.iter().sum();
                let total_black_market_volume: f64 = self.black_market_volume_per_step.iter().sum();
                let total_trades: usize = self.trades_per_step.iter().sum();
                let total_volume: f64 = self.volume_per_step.iter().sum();
                let steps_with_data = self.black_market_trades_per_step.len() as f64;

                Some(crate::result::BlackMarketStats {
                    total_black_market_trades,
                    total_black_market_volume,
                    avg_black_market_trades_per_step: if steps_with_data > 0.0 {
                        total_black_market_trades as f64 / steps_with_data
                    } else {
                        0.0
                    },
                    avg_black_market_volume_per_step: if steps_with_data > 0.0 {
                        total_black_market_volume / steps_with_data
                    } else {
                        0.0
                    },
                    black_market_trade_percentage: if total_trades > 0 {
                        (total_black_market_trades as f64 / total_trades as f64) * 100.0
                    } else {
                        0.0
                    },
                    black_market_volume_percentage: if total_volume > 0.0 {
                        (total_black_market_volume / total_volume) * 100.0
                    } else {
                        0.0
                    },
                })
            } else {
                None
            },
            total_taxes_collected: if self.config.tax_rate > 0.0 {
                Some(self.total_taxes_collected)
            } else {
                None
            },
            total_taxes_redistributed: if self.config.enable_tax_redistribution
                && self.config.tax_rate > 0.0
            {
                Some(self.total_taxes_redistributed)
            } else {
                None
            },
            loan_statistics,
            investment_statistics: None, // Investment system not yet fully implemented
            contract_statistics: if self.config.enable_contracts {
                let active_contracts = self.contracts.values().filter(|c| c.is_active()).count();

                let completed_contracts: Vec<_> =
                    self.contracts.values().filter(|c| c.remaining_steps == 0).collect();

                let avg_contract_duration = if !completed_contracts.is_empty() {
                    completed_contracts.iter().map(|c| c.duration as f64).sum::<f64>()
                        / completed_contracts.len() as f64
                } else {
                    0.0
                };

                let total_contract_value: f64 =
                    self.contracts.values().map(|c| c.total_value_exchanged()).sum();

                Some(crate::result::ContractStats {
                    total_contracts_created: self.total_contracts_created,
                    total_contracts_completed: self.total_contracts_completed,
                    active_contracts,
                    avg_contract_duration,
                    total_contract_value,
                })
            } else {
                None
            },
            education_statistics: if self.config.enable_education {
                let total_skills_learned: usize = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .map(|e| e.person_data.learned_skills.len())
                    .sum();

                let active_persons = self.entities.iter().filter(|e| e.active).count();
                let avg_learned_skills_per_person = if active_persons > 0 {
                    total_skills_learned as f64 / active_persons as f64
                } else {
                    0.0
                };

                let max_learned_skills = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .map(|e| e.person_data.learned_skills.len())
                    .max()
                    .unwrap_or(0);

                // Calculate total education spending
                // Note: This is an approximation as we don't track historical learning costs
                // We estimate based on current learned skills and their current market prices
                let total_education_spending: f64 = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .flat_map(|e| &e.person_data.learned_skills)
                    .filter_map(|skill| self.market.skills.get(&skill.id))
                    .map(|skill| skill.current_price * self.config.learning_cost_multiplier)
                    .sum();

                Some(crate::result::EducationStats {
                    total_skills_learned,
                    avg_learned_skills_per_person,
                    max_learned_skills,
                    total_education_spending,
                })
            } else {
                None
            },
            mentorship_statistics: if self.config.enable_mentorship {
                Some(crate::result::MentorshipStats {
                    total_mentorships: self.total_mentorships_formed,
                    successful_mentored_learnings: self.successful_mentored_learnings,
                    total_cost_savings: self.total_mentorship_cost_savings,
                    unique_mentors: self.unique_mentors.len(),
                    unique_mentees: self.unique_mentees.len(),
                })
            } else {
                None
            },
            certification_statistics: if self.config.enable_certification {
                // Count active (non-expired) certifications
                let active_certifications = self
                    .market
                    .skills
                    .values()
                    .filter(|skill| {
                        if let Some(cert) = &skill.certification {
                            !cert.is_expired(self.current_step)
                        } else {
                            false
                        }
                    })
                    .count();

                Some(crate::result::CertificationStats {
                    total_issued: self.total_certifications_issued,
                    total_expired: self.total_certifications_expired,
                    active_certifications,
                    total_cost: self.total_certification_cost,
                })
            } else {
                None
            },
            environment_statistics: if self.config.enable_environment {
                if let Some(ref environment) = self.environment {
                    use crate::environment::Resource;

                    // Convert Resource enum keys to String keys for JSON serialization
                    let total_consumption: HashMap<String, f64> = environment
                        .total_consumption
                        .iter()
                        .map(|(resource, &amount)| (resource.name().to_string(), amount))
                        .collect();

                    let initial_reserves: HashMap<String, f64> = environment
                        .resource_reserves
                        .iter()
                        .map(|(resource, &amount)| (resource.name().to_string(), amount))
                        .collect();

                    let remaining_reserves: HashMap<String, f64> = Resource::all()
                        .iter()
                        .map(|resource| {
                            (resource.name().to_string(), environment.remaining_reserves(*resource))
                        })
                        .collect();

                    let sustainability_scores_raw = environment.sustainability_scores();
                    let sustainability_scores: HashMap<String, f64> = sustainability_scores_raw
                        .iter()
                        .map(|(resource, &score)| (resource.name().to_string(), score))
                        .collect();

                    let overall_sustainability_score = environment.overall_sustainability_score();
                    let is_sustainable = environment.is_sustainable();

                    Some(crate::result::EnvironmentStats {
                        total_consumption,
                        initial_reserves,
                        remaining_reserves,
                        sustainability_scores,
                        overall_sustainability_score,
                        is_sustainable,
                    })
                } else {
                    None
                }
            } else {
                None
            },
            friendship_statistics: if self.config.enable_friendships {
                // Calculate friendship statistics
                let total_friendships: usize = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .map(|e| e.person_data.friends.len())
                    .sum();
                // Divide by 2 because each friendship is counted twice (bidirectional)
                let total_friendships = total_friendships / 2;

                let active_persons = self.entities.iter().filter(|e| e.active).count();
                let avg_friends_per_person = if active_persons > 0 {
                    self.entities
                        .iter()
                        .filter(|e| e.active)
                        .map(|e| e.person_data.friends.len())
                        .sum::<usize>() as f64
                        / active_persons as f64
                } else {
                    0.0
                };

                let max_friends = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .map(|e| e.person_data.friends.len())
                    .max()
                    .unwrap_or(0);

                let min_friends = self
                    .entities
                    .iter()
                    .filter(|e| e.active)
                    .map(|e| e.person_data.friends.len())
                    .min()
                    .unwrap_or(0);

                // Calculate network density: actual friendships / possible friendships
                // Possible friendships = n * (n-1) / 2 where n is number of active persons
                let network_density = if active_persons > 1 {
                    let possible_friendships = (active_persons * (active_persons - 1)) / 2;
                    total_friendships as f64 / possible_friendships as f64
                } else {
                    0.0
                };

                Some(crate::result::FriendshipStats {
                    total_friendships,
                    avg_friends_per_person,
                    max_friends,
                    min_friends,
                    network_density,
                })
            } else {
                None
            },
            trust_network_statistics: self
                .trust_network
                .as_ref()
                .map(|trust_network| trust_network.get_statistics()),
            trade_agreement_statistics: if self.config.enable_trade_agreements {
                // Calculate trade agreement statistics
                let active_agreements =
                    self.trade_agreements.iter().filter(|a| a.is_active(self.current_step)).count();

                let total_agreement_trades: usize =
                    self.trade_agreements.iter().map(|a| a.trade_count).sum();

                let total_agreement_trade_value: f64 =
                    self.trade_agreements.iter().map(|a| a.total_trade_value).sum();

                let bilateral_agreements =
                    self.trade_agreements.iter().filter(|a| a.partner_count() == 2).count();

                let multilateral_agreements =
                    self.trade_agreements.iter().filter(|a| a.partner_count() > 2).count();

                let average_discount_rate = if !self.trade_agreements.is_empty() {
                    self.trade_agreements.iter().map(|a| a.discount_rate).sum::<f64>()
                        / self.trade_agreements.len() as f64
                } else {
                    0.0
                };

                let average_duration = if !self.trade_agreements.is_empty() {
                    self.trade_agreements.iter().map(|a| a.duration).sum::<usize>() as f64
                        / self.trade_agreements.len() as f64
                } else {
                    0.0
                };

                let average_trades_per_agreement = if !self.trade_agreements.is_empty() {
                    total_agreement_trades as f64 / self.trade_agreements.len() as f64
                } else {
                    0.0
                };

                Some(crate::trade_agreement::TradeAgreementStatistics {
                    total_agreements_formed: self.total_trade_agreements_formed,
                    active_agreements,
                    expired_agreements: self.total_trade_agreements_expired,
                    bilateral_agreements,
                    multilateral_agreements,
                    total_agreement_trades,
                    total_agreement_trade_value,
                    average_discount_rate,
                    average_duration,
                    average_trades_per_agreement,
                })
            } else {
                None
            },
            insurance_statistics,
            technology_breakthrough_statistics,
            group_statistics: if let Some(num_groups) = self.config.num_groups {
                // Calculate group statistics
                let mut group_data: HashMap<usize, Vec<&Entity>> = HashMap::new();

                // Group entities by group_id
                for entity in self.entities.iter().filter(|e| e.active) {
                    if let Some(group_id) = entity.person_data.group_id {
                        group_data.entry(group_id).or_default().push(entity);
                    }
                }

                // Calculate stats for each group
                let mut groups_stats = Vec::new();
                for group_id in 0..num_groups {
                    if let Some(members) = group_data.get(&group_id) {
                        let member_count = members.len();
                        let total_money: f64 = members.iter().map(|e| e.person_data.money).sum();
                        let avg_money = if member_count > 0 {
                            total_money / member_count as f64
                        } else {
                            0.0
                        };
                        let avg_reputation: f64 = if member_count > 0 {
                            members.iter().map(|e| e.person_data.reputation).sum::<f64>()
                                / member_count as f64
                        } else {
                            0.0
                        };

                        // Include resource pool data if pools are enabled
                        let (pool_balance, total_contributions, total_withdrawals) =
                            if self.config.enable_resource_pools {
                                self.resource_pools
                                    .get(&group_id)
                                    .map(|(balance, contrib, withdr)| {
                                        (Some(*balance), Some(*contrib), Some(*withdr))
                                    })
                                    .unwrap_or((Some(0.0), Some(0.0), Some(0.0)))
                            } else {
                                (None, None, None)
                            };

                        groups_stats.push(crate::result::SingleGroupStats {
                            group_id,
                            member_count,
                            avg_money,
                            total_money,
                            avg_reputation,
                            pool_balance,
                            total_contributions,
                            total_withdrawals,
                        });
                    } else {
                        // Empty group
                        let (pool_balance, total_contributions, total_withdrawals) =
                            if self.config.enable_resource_pools {
                                self.resource_pools
                                    .get(&group_id)
                                    .map(|(balance, contrib, withdr)| {
                                        (Some(*balance), Some(*contrib), Some(*withdr))
                                    })
                                    .unwrap_or((Some(0.0), Some(0.0), Some(0.0)))
                            } else {
                                (None, None, None)
                            };

                        groups_stats.push(crate::result::SingleGroupStats {
                            group_id,
                            member_count: 0,
                            avg_money: 0.0,
                            total_money: 0.0,
                            avg_reputation: 0.0,
                            pool_balance,
                            total_contributions,
                            total_withdrawals,
                        });
                    }
                }

                let group_sizes: Vec<usize> = groups_stats.iter().map(|g| g.member_count).collect();
                let min_group_size = group_sizes.iter().min().copied().unwrap_or(0);
                let max_group_size = group_sizes.iter().max().copied().unwrap_or(0);
                let avg_group_size = if num_groups > 0 {
                    group_sizes.iter().sum::<usize>() as f64 / num_groups as f64
                } else {
                    0.0
                };

                // Calculate aggregate pool statistics if pools are enabled
                let (total_pool_balance, total_contributions, total_withdrawals) =
                    if self.config.enable_resource_pools {
                        let mut total_balance = 0.0;
                        let mut total_contrib = 0.0;
                        let mut total_withdr = 0.0;
                        for (balance, contrib, withdr) in self.resource_pools.values() {
                            total_balance += balance;
                            total_contrib += contrib;
                            total_withdr += withdr;
                        }
                        (Some(total_balance), Some(total_contrib), Some(total_withdr))
                    } else {
                        (None, None, None)
                    };

                Some(crate::result::GroupStats {
                    total_groups: num_groups,
                    avg_group_size,
                    min_group_size,
                    max_group_size,
                    groups: groups_stats,
                    total_pool_balance,
                    total_contributions,
                    total_withdrawals,
                })
            } else {
                None
            },
            trading_partner_statistics: crate::result::calculate_trading_partner_statistics(
                &self.entities,
            ),
            centrality_analysis: {
                // Calculate centrality analysis from trading network
                // Note: Trading statistics are recalculated here for centrality analysis.
                // This is a minor redundancy but avoids complex refactoring of the result construction.
                let trading_stats =
                    crate::result::calculate_trading_partner_statistics(&self.entities);

                // Build network nodes and edges manually (similar to export_trading_network)
                let nodes: Vec<crate::result::NetworkNode> = trading_stats
                    .per_person
                    .iter()
                    .map(|person_stats| {
                        let person_id = person_stats.person_id;
                        // Get money and reputation directly from entities
                        let entity = &self.entities[person_id];
                        let money = entity.person_data.money;
                        let reputation = entity.person_data.reputation;
                        let trade_count = person_stats.total_trades_as_buyer
                            + person_stats.total_trades_as_seller;

                        crate::result::NetworkNode {
                            id: format!("Person{}", person_id),
                            money,
                            reputation,
                            trade_count,
                            unique_partners: person_stats.unique_partners,
                        }
                    })
                    .collect();

                // Build edges from trading relationships
                let mut edge_map: HashMap<(usize, usize), (usize, f64)> = HashMap::new();
                for person_stats in &trading_stats.per_person {
                    let person_id = person_stats.person_id;
                    for partner in &person_stats.top_partners {
                        let partner_id = partner.partner_id;
                        let edge_key = if person_id < partner_id {
                            (person_id, partner_id)
                        } else {
                            (partner_id, person_id)
                        };
                        let entry = edge_map.entry(edge_key).or_insert((0, 0.0));
                        entry.0 += partner.trade_count;
                        entry.1 += partner.total_value;
                    }
                }

                let edges: Vec<crate::result::NetworkEdge> = edge_map
                    .into_iter()
                    .map(|((source_id, target_id), (weight, total_value))| {
                        crate::result::NetworkEdge {
                            source: format!("Person{}", source_id),
                            target: format!("Person{}", target_id),
                            weight,
                            total_value,
                        }
                    })
                    .collect();

                // Only calculate centrality if there are nodes (avoid empty network)
                if !nodes.is_empty() {
                    Some(crate::centrality::calculate_centrality(&nodes, &edges))
                } else {
                    None
                }
            },
            mobility_statistics: crate::result::calculate_mobility_statistics(
                &self.mobility_quintiles,
            ),
            quality_statistics: if self.config.enable_quality {
                // Collect all quality ratings from all persons
                let mut all_qualities: Vec<f64> = Vec::new();
                for entity in self.entities.iter().filter(|e| e.active) {
                    for quality in entity.person_data.skill_qualities.values() {
                        // Quality values should always be valid (0.0-5.0), but filter out NaN just in case
                        if !quality.is_nan() {
                            all_qualities.push(*quality);
                        }
                    }
                }

                if !all_qualities.is_empty() {
                    // Safe to use unwrap_or(Equal) here as we've filtered out NaN values
                    all_qualities
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                    let sum: f64 = all_qualities.iter().sum();
                    let count = all_qualities.len();
                    let average = sum / count as f64;

                    let median = if count % 2 == 1 {
                        all_qualities[count / 2]
                    } else {
                        (all_qualities[count / 2 - 1] + all_qualities[count / 2]) / 2.0
                    };

                    let variance = all_qualities
                        .iter()
                        .map(|q| {
                            let diff = average - q;
                            diff * diff
                        })
                        .sum::<f64>()
                        / count as f64;
                    let std_dev = variance.sqrt();

                    // Safe to unwrap as we've already checked !is_empty()
                    let min_quality = *all_qualities.first().unwrap();
                    let max_quality = *all_qualities.last().unwrap();

                    let skills_at_max_quality = all_qualities.iter().filter(|&&q| q >= 5.0).count();
                    let skills_at_min_quality = all_qualities.iter().filter(|&&q| q <= 0.0).count();

                    Some(crate::result::QualityStats {
                        average_quality: average,
                        median_quality: median,
                        std_dev_quality: std_dev,
                        min_quality,
                        max_quality,
                        skills_at_max_quality,
                        skills_at_min_quality,
                    })
                } else {
                    None
                }
            } else {
                None
            },
            externality_statistics: if self.config.enable_externalities {
                // Finalize externality statistics to calculate averages
                let mut stats = self.externality_stats.clone();
                stats.finalize();
                Some(stats)
            } else {
                None
            },
            events: if self.event_bus.is_enabled() {
                Some(self.event_bus.events().to_vec())
            } else {
                None
            },
            final_persons_data: self.entities.clone(),
        };

        // Notify plugins that simulation has ended, allowing them to modify the result
        let persons: Vec<_> = self.entities.iter().map(|e| e.person_data.clone()).collect();
        let end_context = PluginContext {
            config: &self.config,
            current_step: self.current_step,
            total_steps: self.config.max_steps,
            persons: &persons,
        };
        self.plugin_registry.on_simulation_end(&end_context, &mut result);

        result
    }

    /// Attempts production for all active persons in the simulation.
    ///
    /// Each person has a chance (based on config.production_probability) to try producing
    /// a new skill by combining two skills they already have. If they have the required inputs
    /// and can afford the production cost, a new skill is learned.
    ///
    /// # Returns
    /// The number of successful productions this step
    fn attempt_production(&mut self) -> usize {
        if !self.config.enable_production {
            return 0;
        }

        // Use cached recipes (already validated to exist when enable_production is true)
        let recipes = self
            .production_recipes
            .as_ref()
            .expect("production_recipes should be initialized when enable_production is true");
        let mut productions_count = 0;

        // Collect entity indices to avoid borrow checker issues
        let entity_indices: Vec<usize> = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.active)
            .map(|(i, _)| i)
            .collect();

        for idx in entity_indices {
            // Check if this person attempts production this step
            if self.rng.random_range(0.0..1.0) > self.config.production_probability {
                continue;
            }

            // Get person's available skill IDs
            let available_skill_ids: Vec<SkillId> = self.entities[idx]
                .person_data
                .all_skills()
                .iter()
                .map(|s| s.id.clone())
                .collect();

            // Find recipes that can be crafted with available skills
            let craftable_recipes: Vec<&crate::production::Recipe> =
                recipes.iter().filter(|recipe| recipe.can_craft(&available_skill_ids)).collect();

            if craftable_recipes.is_empty() {
                continue;
            }

            // Pick a random craftable recipe
            if let Some(recipe) = craftable_recipes.choose(&mut self.rng) {
                // Check if person already has the output skill
                if self.entities[idx].person_data.has_skill(&recipe.output_skill) {
                    continue;
                }

                // Calculate production cost based on current market prices
                let input1_price = self
                    .market
                    .skills
                    .get(&recipe.input_skill_1)
                    .map(|s| s.current_price)
                    .unwrap_or(self.config.base_skill_price);
                let input2_price = self
                    .market
                    .skills
                    .get(&recipe.input_skill_2)
                    .map(|s| s.current_price)
                    .unwrap_or(self.config.base_skill_price);

                let production_cost = recipe.calculate_cost(input1_price, input2_price);

                // Check if person can afford production
                if !self.entities[idx].person_data.can_afford(production_cost) {
                    continue;
                }

                // Deduct production cost
                self.entities[idx].person_data.money -= production_cost;

                // Create new skill at higher price (reflects value added)
                let output_price = (input1_price + input2_price) * recipe.cost_multiplier;
                let new_skill = Skill::new(recipe.output_skill.clone(), output_price);

                // Add skill to person's learned skills
                self.entities[idx].person_data.learned_skills.push(new_skill.clone());

                // Add skill to market if it doesn't exist yet
                if !self.market.skills.contains_key(&new_skill.id) {
                    self.market.skills.insert(new_skill.id.clone(), new_skill.clone());
                    self.all_skill_ids.push(new_skill.id.clone());

                    // Also add to black market if enabled
                    if let Some(ref mut bm) = self.black_market {
                        let bm_skill = Skill::new(
                            new_skill.id.clone(),
                            new_skill.current_price * self.config.black_market_price_multiplier,
                        );
                        bm.skills.insert(bm_skill.id.clone(), bm_skill);
                    }
                }

                debug!(
                    "Person {} produced {} from {} + {} for ${:.2}",
                    self.entities[idx].id,
                    recipe.output_skill,
                    recipe.input_skill_1,
                    recipe.input_skill_2,
                    production_cost
                );

                productions_count += 1;
            }
        }

        productions_count
    }

    pub fn step(&mut self) {
        self.market.reset_demand_counts();
        for entity in self.entities.iter_mut() {
            if entity.active {
                entity.person_data.needed_skills.clear();
                entity.person_data.satisfied_needs_current_step.clear();
            }
        }

        // Try to form new trade agreements and remove expired ones
        self.try_form_trade_agreements();

        // Pre-calculate seasonal factors for all skills to avoid borrowing issues
        let seasonal_enabled = self.config.seasonal_amplitude > 0.0;
        let seasonal_factors: HashMap<SkillId, f64> = if seasonal_enabled {
            self.all_skill_ids
                .iter()
                .map(|skill_id| (skill_id.clone(), self.calculate_seasonal_factor(skill_id)))
                .collect()
        } else {
            HashMap::new()
        };

        for entity in self.entities.iter_mut() {
            if !entity.active {
                continue;
            }

            // Generate base number of needs using configured demand strategy
            let base_num_needs = self.demand_generator.generate_demand_count(
                entity.id,
                self.current_step,
                &mut self.rng,
            );

            // Apply seasonal modulation to the number of needs
            // Use the average seasonal factor across all owned skills
            let num_needs = if seasonal_enabled {
                let avg_seasonal_factor: f64 = entity
                    .person_data
                    .own_skills
                    .iter()
                    .map(|skill| seasonal_factors.get(&skill.id).copied().unwrap_or(1.0))
                    .sum::<f64>()
                    / entity.person_data.own_skills.len() as f64;
                // Modulate the number of needs, clamping between 1 and 5
                ((base_num_needs as f64 * avg_seasonal_factor).round() as usize).clamp(1, 5)
            } else {
                base_num_needs
            };

            // Filter out skills the person already has (either as own_skills or learned_skills)
            let mut potential_needs: Vec<SkillId> = self
                .all_skill_ids
                .iter()
                .filter(|&id| !entity.person_data.has_skill(id))
                .cloned()
                .collect();

            potential_needs.shuffle(&mut self.rng);

            for _ in 0..num_needs {
                if let Some(needed_skill_id) = potential_needs.pop() {
                    if !entity
                        .person_data
                        .needed_skills
                        .iter()
                        .any(|item| item.id == needed_skill_id)
                    {
                        let urgency = self.rng.random_range(1..=3);
                        entity.person_data.needed_skills.push(crate::person::NeededSkillItem {
                            id: needed_skill_id.clone(),
                            urgency,
                        });
                        self.market.increment_demand(&needed_skill_id);
                    }
                } else {
                    break;
                }
            }
        }

        // Capture prices before update for event emission and action recording
        // Use Vec instead of HashMap for better performance with small to medium number of skills
        let should_track_prices = self.event_bus.is_enabled() || self.action_log.is_some();
        let prices_before: Vec<(SkillId, f64)> = if should_track_prices {
            self.market
                .skills
                .iter()
                .map(|(id, skill)| (id.clone(), skill.current_price))
                .collect()
        } else {
            Vec::new()
        };

        self.market.update_prices(&mut self.rng);

        // Emit price update events and record actions for changed prices
        if should_track_prices {
            // Use a tolerance appropriate for currency comparisons (0.01 = 1 cent)
            const PRICE_CHANGE_TOLERANCE: f64 = 0.01;

            for (skill_id, old_price) in prices_before {
                if let Some(skill) = self.market.skills.get(&skill_id) {
                    if (old_price - skill.current_price).abs() > PRICE_CHANGE_TOLERANCE {
                        // Emit event if enabled
                        if self.event_bus.is_enabled() {
                            self.event_bus.emit_price_update(
                                self.current_step,
                                skill_id.clone(),
                                old_price,
                                skill.current_price,
                            );
                        }
                        // Record action if enabled
                        if let Some(ref mut action_log) = self.action_log {
                            action_log.record(crate::replay::SimulationAction::PriceUpdate {
                                step: self.current_step,
                                skill_id: skill_id.clone(),
                                old_price,
                                new_price: skill.current_price,
                            });
                        }
                    }
                }
            }
        }

        // Check for and trigger crisis events (if enabled)
        self.check_and_trigger_crisis();

        // Try to purchase insurance policies
        self.try_purchase_insurance();

        // Process insurance claims for income protection
        self.process_income_insurance_payouts();

        // Process insurance claims for credit defaults
        self.process_credit_insurance_payouts();

        /// Helper struct to hold priority information for purchase decisions.
        /// Combines multiple factors (urgency, affordability, efficiency, reputation)
        /// into a single priority score for sorting purchase options.
        #[derive(Debug, Clone)]
        struct PurchaseOption {
            needed_item: crate::person::NeededSkillItem,
            priority_score: f64,
        }

        // Constants for priority score normalization
        const EFFICIENCY_SCALE_FACTOR: f64 = 10.0; // Scales efficiency (typically 1.0-1.1) to 0.0-1.0 range
        const REPUTATION_OFFSET: f64 = 0.5; // Offset to center reputation (neutral = 1.0) at 0.5
        const REPUTATION_SCALE_FACTOR: f64 = 1.5; // Scales reputation (0.0-2.0) to 0.0-1.0 range

        // Build a map of skill providers
        // Since multiple persons can now provide the same skill, we use Vec<usize>
        let mut skill_providers: HashMap<SkillId, Vec<usize>> = HashMap::new();
        for entity_idx in 0..self.entities.len() {
            if self.entities[entity_idx].active {
                // Include both own_skills and learned_skills
                for skill in &self.entities[entity_idx].person_data.own_skills {
                    skill_providers
                        .entry(skill.id.clone())
                        .or_default()
                        .push(self.entities[entity_idx].id);
                }
                for skill in &self.entities[entity_idx].person_data.learned_skills {
                    skill_providers
                        .entry(skill.id.clone())
                        .or_default()
                        .push(self.entities[entity_idx].id);
                }
            }
        }

        let mut trades_to_execute: Vec<(usize, usize, SkillId, f64)> = Vec::new();
        let mut failed_attempts_this_step = 0usize;

        for buyer_idx in 0..self.entities.len() {
            if !self.entities[buyer_idx].active {
                continue;
            }

            // Calculate priority scores for all needed skills
            let buyer_money = self.entities[buyer_idx].person_data.money;
            let mut purchase_options: Vec<PurchaseOption> = Vec::new();

            for needed_item in &self.entities[buyer_idx].person_data.needed_skills {
                let needed_skill_id = &needed_item.id;

                // Skip if already satisfied in this step
                if self.entities[buyer_idx]
                    .person_data
                    .satisfied_needs_current_step
                    .contains(needed_skill_id)
                {
                    continue;
                }

                // Get price information for priority calculation
                if let Some(skill_price) = self.market.get_price(needed_skill_id) {
                    let efficiency = self.market.get_skill_efficiency(needed_skill_id);
                    let efficiency_adjusted_price = skill_price / efficiency;

                    // Get seller reputation if available
                    let seller_reputation = skill_providers
                        .get(needed_skill_id)
                        .and_then(|providers| providers.first().copied())
                        .map(|seller_id| self.entities[seller_id].person_data.reputation)
                        .unwrap_or(1.0); // Neutral reputation if no seller

                    // Calculate priority score components (normalized to 0.0-1.0 range)

                    // 1. Urgency component (urgency is 1-3, normalize to 0.0-1.0)
                    let urgency_score = (needed_item.urgency as f64 - 1.0) / 2.0;

                    // 2. Affordability component (inverse of price-to-money ratio, capped at 1.0)
                    // Higher score = more affordable (cheaper relative to available money)
                    let affordability_score = if buyer_money > 0.0 {
                        (1.0 - (efficiency_adjusted_price / buyer_money)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };

                    // 3. Efficiency component (efficiency typically > 1.0 due to tech progress)
                    // Normalize to 0.0-1.0 range, where 1.0 = neutral, > 1.0 = better
                    let efficiency_score =
                        ((efficiency - 1.0) * EFFICIENCY_SCALE_FACTOR).clamp(0.0, 1.0);

                    // 4. Reputation component (reputation 0.0-2.0, normalize to 0.0-1.0)
                    // Higher reputation = better, centered at 1.0 (neutral)
                    let reputation_score = ((seller_reputation - REPUTATION_OFFSET)
                        / REPUTATION_SCALE_FACTOR)
                        .clamp(0.0, 1.0);

                    // Weighted priority score
                    let priority_score = self.config.priority_urgency_weight * urgency_score
                        + self.config.priority_affordability_weight * affordability_score
                        + self.config.priority_efficiency_weight * efficiency_score
                        + self.config.priority_reputation_weight * reputation_score;

                    trace!(
                        "Person {} - Skill {:?} priority: {:.3} (urgency: {:.2}, affordability: {:.2}, efficiency: {:.2}, reputation: {:.2})",
                        self.entities[buyer_idx].id,
                        needed_skill_id,
                        priority_score,
                        urgency_score,
                        affordability_score,
                        efficiency_score,
                        reputation_score
                    );

                    purchase_options
                        .push(PurchaseOption { needed_item: needed_item.clone(), priority_score });
                }
            }

            // Sort by priority score (highest first)
            purchase_options.sort_by(|a, b| {
                b.priority_score
                    .partial_cmp(&a.priority_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for option in purchase_options {
                let needed_item = option.needed_item;
                let needed_skill_id = &needed_item.id;
                if self.entities[buyer_idx]
                    .person_data
                    .satisfied_needs_current_step
                    .contains(needed_skill_id)
                {
                    trace!(
                        "Person {} already satisfied need for skill {:?} in this step",
                        self.entities[buyer_idx].id,
                        needed_skill_id
                    );
                    continue;
                }

                if let Some(skill_price) = self.market.get_price(needed_skill_id) {
                    // Apply efficiency multiplier - higher efficiency reduces effective price
                    let efficiency = self.market.get_skill_efficiency(needed_skill_id);
                    let efficiency_adjusted_price = skill_price / efficiency;

                    // Find a provider for this skill - select the first available one
                    // (Could be enhanced to select based on reputation or other criteria)
                    let seller_id_opt = skill_providers
                        .get(needed_skill_id)
                        .and_then(|providers| providers.first().copied());

                    // Apply reputation-based price multiplier for the seller
                    let mut final_price = if let Some(seller_id) = seller_id_opt {
                        let seller_reputation_multiplier =
                            self.entities[seller_id].person_data.reputation_price_multiplier();
                        efficiency_adjusted_price * seller_reputation_multiplier
                    } else {
                        efficiency_adjusted_price
                    };

                    // Apply friendship discount if enabled and buyer-seller are friends
                    if self.config.enable_friendships {
                        if let Some(seller_id) = seller_id_opt {
                            let buyer_person_id = self.entities[buyer_idx].id;
                            let seller_person_id = self.entities[seller_id].id;

                            if self.entities[buyer_idx].person_data.is_friend_with(seller_person_id)
                            {
                                // Direct friendship discount
                                let friendship_multiplier = 1.0 - self.config.friendship_discount;
                                final_price *= friendship_multiplier;
                                trace!(
                                    "Friendship discount applied: Person {} and Person {} are friends, price reduced by {:.1}%",
                                    buyer_person_id,
                                    seller_person_id,
                                    self.config.friendship_discount * 100.0
                                );
                            } else if self.config.enable_trust_networks {
                                // Check for trust network discount (indirect trust)
                                if let Some(ref mut trust_network) = self.trust_network {
                                    let trust_level = trust_network
                                        .get_trust_level(buyer_person_id, seller_person_id);
                                    let trust_multiplier = trust_level.discount_multiplier();

                                    if trust_multiplier > 0.0 {
                                        // Apply partial friendship discount based on trust level
                                        let trust_discount =
                                            self.config.friendship_discount * trust_multiplier;
                                        let trust_price_multiplier = 1.0 - trust_discount;
                                        final_price *= trust_price_multiplier;
                                        trace!(
                                            "Trust network discount applied: Person {} trusts Person {} at level {:?}, price reduced by {:.1}%",
                                            buyer_person_id,
                                            seller_person_id,
                                            trust_level,
                                            trust_discount * 100.0
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // Apply trade agreement discount if enabled and buyer-seller have an agreement
                    if self.config.enable_trade_agreements {
                        if let Some(seller_id) = seller_id_opt {
                            let buyer_id = self.entities[buyer_idx].id;
                            let seller_person_id = self.entities[seller_id].id;

                            // Check if there's an active agreement between buyer and seller
                            let buyer_agreements =
                                self.entities[buyer_idx].person_data.trade_agreement_ids.clone();

                            for agreement_id in buyer_agreements {
                                if let Some(agreement) = self.trade_agreements.iter().find(|a| {
                                    a.id == agreement_id && a.is_active(self.current_step)
                                }) {
                                    if agreement.includes_both(buyer_id, seller_person_id) {
                                        let agreement_multiplier = 1.0 - agreement.discount_rate;
                                        final_price *= agreement_multiplier;
                                        trace!(
                                            "Trade agreement discount applied: Person {} and Person {} have agreement {}, price reduced by {:.1}%",
                                            buyer_id,
                                            seller_person_id,
                                            agreement_id,
                                            agreement.discount_rate * 100.0
                                        );
                                        break; // Only one agreement discount per trade
                                    }
                                }
                            }
                        }
                    }

                    // Apply quality-based price adjustment if enabled
                    if self.config.enable_quality {
                        if let Some(seller_id) = seller_id_opt {
                            if let Some(&quality) = self.entities[seller_id]
                                .person_data
                                .skill_qualities
                                .get(needed_skill_id)
                            {
                                // Apply specialization strategy quality bonus if enabled
                                let effective_quality = if self.config.enable_specialization {
                                    let specialization_bonus = self.entities[seller_id]
                                        .person_data
                                        .specialization_strategy
                                        .quality_bonus();
                                    (quality + specialization_bonus).min(5.0) // Cap at max quality
                                } else {
                                    quality
                                };

                                // Quality ranges from 0.0-5.0, with 3.0 as average
                                // Price adjustment: +10% per quality point above/below 3.0
                                // Quality 5.0 -> +20% price, Quality 3.0 -> base price, Quality 1.0 -> -20% price
                                let quality_multiplier = 1.0 + (effective_quality - 3.0) * 0.1;
                                final_price *= quality_multiplier;
                                trace!(
                                    "Quality price adjustment: Person {} skill '{}' quality {:.2} (effective {:.2}), price adjusted by {:.1}% to ${:.2}",
                                    self.entities[seller_id].id,
                                    needed_skill_id,
                                    quality,
                                    effective_quality,
                                    (quality_multiplier - 1.0) * 100.0,
                                    final_price
                                );
                            }

                            // Apply specialization strategy price multiplier if enabled
                            if self.config.enable_specialization {
                                let spec_price_multiplier = self.entities[seller_id]
                                    .person_data
                                    .specialization_strategy
                                    .price_multiplier();
                                final_price *= spec_price_multiplier;
                                if spec_price_multiplier != 1.0 {
                                    trace!(
                                        "Specialization price multiplier: Person {} strategy {:?}, price adjusted by {:.1}% to ${:.2}",
                                        self.entities[seller_id].id,
                                        self.entities[seller_id].person_data.specialization_strategy,
                                        (spec_price_multiplier - 1.0) * 100.0,
                                        final_price
                                    );
                                }
                            }
                        }
                    }

                    // Apply certification-based price premium if enabled
                    if self.config.enable_certification {
                        if let Some(skill) = self.market.skills.get(needed_skill_id) {
                            if let Some(cert) = &skill.certification {
                                if !cert.is_expired(self.current_step) {
                                    let cert_multiplier = cert.price_multiplier();
                                    final_price *= cert_multiplier;
                                    trace!(
                                        "Certification premium applied: Skill '{}' level {} certification, price increased by {:.1}% to ${:.2}",
                                        needed_skill_id,
                                        cert.level,
                                        (cert_multiplier - 1.0) * 100.0,
                                        final_price
                                    );
                                }
                            }
                        }
                    }

                    // Apply distance-based cost multiplier if enabled
                    if self.config.distance_cost_factor > 0.0 {
                        if let Some(seller_id) = seller_id_opt {
                            let buyer_location = &self.entities[buyer_idx].person_data.location;
                            let seller_location = &self.entities[seller_id].person_data.location;
                            let distance = buyer_location.distance_to(seller_location);
                            let distance_multiplier =
                                1.0 + (distance * self.config.distance_cost_factor);
                            final_price *= distance_multiplier;
                            trace!(
                                "Distance cost applied: Person {} to Person {} distance {:.2}, price increased by {:.1}% to ${:.2}",
                                self.entities[buyer_idx].id,
                                self.entities[seller_id].id,
                                distance,
                                (distance_multiplier - 1.0) * 100.0,
                                final_price
                            );
                        }
                    }

                    if self.entities[buyer_idx].person_data.can_afford_with_strategy(final_price) {
                        if let Some(seller_id) = seller_id_opt {
                            let seller_idx = seller_id;

                            if buyer_idx == seller_idx {
                                trace!(
                                    "Person {} cannot buy their own skill {:?}",
                                    self.entities[buyer_idx].id,
                                    needed_skill_id
                                );
                                continue;
                            }
                            if !self.entities[seller_idx].active {
                                trace!(
                                    "Seller {} for skill {:?} is inactive",
                                    seller_id,
                                    needed_skill_id
                                );
                                continue;
                            }

                            debug!(
                                "Trade scheduled: Person {} buying skill {:?} from Person {} for ${:.2} (urgency: {}, priority: {:.3})",
                                self.entities[buyer_idx].id,
                                needed_skill_id,
                                seller_id,
                                final_price,
                                needed_item.urgency,
                                option.priority_score
                            );

                            trades_to_execute.push((
                                buyer_idx,
                                seller_idx,
                                needed_skill_id.clone(),
                                final_price,
                            ));
                            self.entities[buyer_idx]
                                .person_data
                                .satisfied_needs_current_step
                                .push(needed_skill_id.clone());
                        }
                    } else {
                        // Trade failed due to insufficient funds - track this
                        failed_attempts_this_step += 1;
                        self.failed_trade_attempts += 1;

                        trace!(
                            "Person {} cannot afford skill {:?} at ${:.2} (has ${:.2}, strategy allows ${:.2})",
                            self.entities[buyer_idx].id,
                            needed_skill_id,
                            final_price,
                            self.entities[buyer_idx].person_data.money,
                            self.entities[buyer_idx].person_data.money * self.entities[buyer_idx].person_data.strategy.spending_multiplier()
                        );

                        // Record failed trade action for replay (if enabled and seller exists)
                        if let Some(ref mut action_log) = self.action_log {
                            if let Some(seller_id) = seller_id_opt {
                                action_log.record(crate::replay::SimulationAction::FailedTrade {
                                    step: self.current_step,
                                    buyer_id: self.entities[buyer_idx].id,
                                    seller_id,
                                    skill_id: needed_skill_id.clone(),
                                    price: final_price,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Track trade volume for this step
        let trades_count = trades_to_execute.len();
        let total_volume: f64 = trades_to_execute.iter().map(|(_, _, _, price)| price).sum();
        let step_taxes_collected_start = self.total_taxes_collected;

        // Track failed trade attempts for this step
        self.failed_attempts_per_step.push(failed_attempts_this_step);

        // Determine which trades go to black market (if enabled)
        let mut black_market_trade_indices: Vec<usize> = Vec::new();
        let mut black_market_volume = 0.0;

        if self.config.enable_black_market && self.black_market.is_some() {
            let num_black_market_trades = (trades_to_execute.len() as f64
                * self.config.black_market_participation_rate)
                .round() as usize;

            if num_black_market_trades > 0 {
                // Randomly select trades to route to black market
                let mut all_indices: Vec<usize> = (0..trades_to_execute.len()).collect();
                all_indices.shuffle(&mut self.rng);
                black_market_trade_indices =
                    all_indices.into_iter().take(num_black_market_trades).collect();

                debug!(
                    "Routing {} out of {} trades to black market ({}% participation rate)",
                    num_black_market_trades,
                    trades_to_execute.len(),
                    (self.config.black_market_participation_rate * 100.0)
                );

                // Apply black market price multiplier directly to selected trades
                for &trade_idx in &black_market_trade_indices {
                    let (_buyer_idx, _seller_idx, _skill_id, regular_price) =
                        &mut trades_to_execute[trade_idx];
                    let black_market_price =
                        *regular_price * self.config.black_market_price_multiplier;
                    trace!(
                        "Trade {} uses black market: ${:.2} -> ${:.2} ({}x multiplier)",
                        trade_idx,
                        *regular_price,
                        black_market_price,
                        self.config.black_market_price_multiplier
                    );
                    *regular_price = black_market_price;
                    black_market_volume += black_market_price;
                }
            }
        }

        // Execute all trades (prices already adjusted for black market trades)
        // If parallel trades are enabled, use the parallel execution path.
        // Otherwise, execute sequentially.
        if self.config.enable_parallel_trades {
            self.execute_trades_parallel(trades_to_execute);
        } else {
            // Sequential execution (original logic)
            self.execute_trades_sequential(trades_to_execute);
        }

        // Common post-trade processing continues below...
        self.trades_per_step.push(trades_count);
        self.volume_per_step.push(total_volume);
        self.black_market_trades_per_step.push(black_market_trade_indices.len());
        self.black_market_volume_per_step.push(black_market_volume);

        // Apply reputation decay for all active entities
        for entity in &mut self.entities {
            if entity.active {
                entity.person_data.apply_reputation_decay();
            }
        }

        // Adapt strategies based on performance (if enabled)
        if self.config.enable_adaptive_strategies {
            for entity in &mut self.entities {
                if entity.active {
                    let adapted = entity.person_data.adapt_strategy(
                        self.config.adaptation_rate,
                        &mut self.rng,
                        self.config.exploration_rate,
                    );
                    if adapted {
                        trace!(
                            "Person {} adapted strategy: adjustment_factor={:.3}, effective_multiplier={:.3}",
                            entity.id,
                            entity.person_data.strategy_params.adjustment_factor,
                            entity.person_data.get_effective_spending_multiplier()
                        );
                    }
                }
            }
        }

        // Update credit scores and history if credit rating is enabled
        if self.config.enable_credit_rating {
            for entity in &mut self.entities {
                if entity.active {
                    // Increment credit history for persons with credit history
                    entity.person_data.credit_score.increment_credit_history();

                    // Calculate current debt level
                    let total_debt: f64 = entity
                        .person_data
                        .borrowed_loans
                        .iter()
                        .filter_map(|loan_id| self.loans.get(loan_id))
                        .map(|loan| loan.remaining_principal)
                        .sum();

                    // Update credit score based on current financial state
                    entity.person_data.credit_score.calculate_score(
                        total_debt,
                        entity.person_data.money,
                        self.current_step,
                    );
                }
            }
        }

        // Apply savings - persons save a portion of their money
        if self.config.savings_rate > 0.0 {
            for entity in &mut self.entities {
                if entity.active {
                    let money_before = entity.person_data.money;
                    entity.person_data.apply_savings(self.config.savings_rate);
                    let saved_amount = money_before - entity.person_data.money;
                    if saved_amount > 0.0 {
                        trace!(
                            "Person {} saved ${:.2} (rate: {:.1}%), balance: ${:.2} -> ${:.2}",
                            entity.id,
                            saved_amount,
                            self.config.savings_rate * 100.0,
                            money_before,
                            entity.person_data.money
                        );
                    }
                }
            }
        }

        // Resource pool contributions and withdrawals (if enabled)
        if self.config.enable_resource_pools && self.config.num_groups.is_some() {
            // Step 1: Collect contributions from all group members
            for entity in &mut self.entities {
                if !entity.active {
                    continue;
                }

                if let Some(group_id) = entity.person_data.group_id {
                    // Only contribute if person has positive money
                    if entity.person_data.money > 0.0 {
                        let contribution =
                            entity.person_data.money * self.config.pool_contribution_rate;
                        entity.person_data.money -= contribution;

                        // Update pool balance and total contributions
                        if let Some(pool) = self.resource_pools.get_mut(&group_id) {
                            pool.0 += contribution; // balance
                            pool.1 += contribution; // total_contributions
                            trace!(
                                "Person {} contributed ${:.2} to group {} pool (balance: ${:.2})",
                                entity.id,
                                contribution,
                                group_id,
                                pool.0
                            );
                        }
                    }
                }
            }

            // Step 2: Distribute pool resources to needy members
            // First, identify needy members by group
            let mut needy_by_group: HashMap<usize, Vec<usize>> = HashMap::new();
            for entity in &self.entities {
                if !entity.active {
                    continue;
                }

                if let Some(group_id) = entity.person_data.group_id {
                    if entity.person_data.money < self.config.pool_withdrawal_threshold {
                        needy_by_group.entry(group_id).or_default().push(entity.id);
                    }
                }
            }

            // Distribute pool funds equally among needy members
            for (group_id, needy_ids) in needy_by_group {
                if needy_ids.is_empty() {
                    continue;
                }

                if let Some(pool) = self.resource_pools.get_mut(&group_id) {
                    if pool.0 > 0.0 {
                        // Equal distribution among needy members
                        let distribution_per_person = pool.0 / needy_ids.len() as f64;

                        for person_id in needy_ids {
                            if let Some(entity) =
                                self.entities.iter_mut().find(|e| e.id == person_id)
                            {
                                entity.person_data.money += distribution_per_person;
                                pool.0 -= distribution_per_person; // balance
                                pool.2 += distribution_per_person; // total_withdrawals
                                trace!(
                                    "Person {} received ${:.2} from group {} pool (new balance: ${:.2})",
                                    person_id,
                                    distribution_per_person,
                                    group_id,
                                    entity.person_data.money
                                );
                            }
                        }
                    }
                }
            }
        }

        // Education system - persons can learn new skills (with optional mentorship support)
        if self.config.enable_education && self.config.learning_probability > 0.0 {
            for i in 0..self.entities.len() {
                if !self.entities[i].active {
                    continue;
                }

                // Check if this person attempts to learn a skill this step
                let attempt_learning: f64 = self.rng.random();
                if attempt_learning >= self.config.learning_probability {
                    continue;
                }

                let learner_id = self.entities[i].id;

                // Find skills this person doesn't have yet
                let potential_skills: Vec<Skill> = self
                    .market
                    .skills
                    .values()
                    .filter(|skill| !self.entities[i].person_data.has_skill(&skill.id))
                    .cloned()
                    .collect();

                if potential_skills.is_empty() {
                    continue;
                }

                // Randomly select a skill to learn
                if let Some(skill_to_learn) =
                    potential_skills.as_slice().choose(&mut self.rng).cloned()
                {
                    let base_learning_cost =
                        skill_to_learn.current_price * self.config.learning_cost_multiplier;

                    // Try to find a mentor if mentorship is enabled
                    let mentor_info = if self.config.enable_mentorship {
                        self.find_mentor_for_skill(&skill_to_learn.id, learner_id)
                    } else {
                        None
                    };

                    // Calculate final cost (reduced if mentored)
                    let (final_cost, mentor_id) =
                        if let Some((mentor_id, _mentor_quality)) = mentor_info {
                            let reduced_cost =
                                base_learning_cost * self.config.mentorship_cost_reduction;
                            (reduced_cost, Some(mentor_id))
                        } else {
                            (base_learning_cost, None)
                        };

                    // Attempt to learn the skill
                    if self.entities[i].person_data.learn_skill(skill_to_learn.clone(), final_cost)
                    {
                        if let Some(mid) = mentor_id {
                            // Successful mentored learning
                            let cost_savings = base_learning_cost - final_cost;
                            self.successful_mentored_learnings += 1;
                            self.total_mentorship_cost_savings += cost_savings;
                            self.unique_mentees.insert(learner_id);
                            self.unique_mentors.insert(mid);

                            // Award reputation bonus to mentor
                            if let Some(mentor_entity) =
                                self.entities.iter_mut().find(|e| e.id == mid)
                            {
                                mentor_entity.person_data.reputation +=
                                    self.config.mentor_reputation_bonus;
                                debug!(
                                    "Person {} learned skill '{}' from mentor {} for ${:.2} (saved ${:.2}, mentor gained +{:.3} reputation)",
                                    learner_id,
                                    skill_to_learn.id,
                                    mid,
                                    final_cost,
                                    cost_savings,
                                    self.config.mentor_reputation_bonus
                                );
                            }

                            // Record the mentorship relationship
                            let mentorship = crate::person::Mentorship::new(
                                mid,
                                learner_id,
                                skill_to_learn.id.clone(),
                                self.current_step,
                            );
                            self.mentorships.push(mentorship);
                            self.total_mentorships_formed += 1;
                        } else {
                            debug!(
                                "Person {} learned skill '{}' for ${:.2} (market price: ${:.2}, no mentor)",
                                learner_id,
                                skill_to_learn.id,
                                final_cost,
                                skill_to_learn.current_price
                            );
                        }
                    }
                }
            }
        }

        // Certification system - persons can get their skills certified
        if self.config.enable_certification && self.config.certification_probability > 0.0 {
            for i in 0..self.entities.len() {
                if !self.entities[i].active {
                    continue;
                }

                // Check if this person attempts to certify a skill this step
                let attempt_cert: f64 = self.rng.random();
                if attempt_cert >= self.config.certification_probability {
                    continue;
                }

                let person_id = self.entities[i].id;

                // Find skills this person has that aren't certified yet
                let person_skills = self.entities[i].person_data.all_skills();
                let uncertified_skills: Vec<SkillId> = person_skills
                    .into_iter()
                    .map(|skill| skill.id.clone())
                    .filter(|skill_id| {
                        if let Some(skill) = self.market.skills.get(skill_id) {
                            // Check if skill is not certified or certification expired
                            if let Some(cert) = &skill.certification {
                                cert.is_expired(self.current_step)
                            } else {
                                true // No certification exists
                            }
                        } else {
                            false
                        }
                    })
                    .collect();

                if uncertified_skills.is_empty() {
                    continue;
                }

                // Randomly select a skill to certify
                if let Some(skill_id) = uncertified_skills.as_slice().choose(&mut self.rng).cloned()
                {
                    if let Some(skill) = self.market.skills.get(&skill_id) {
                        // Determine certification level (1-5, higher quality gives better chance at higher level)
                        let level: u8 = if self.config.enable_quality {
                            // Level based on quality: quality 5.0 -> level 5, quality 0.0 -> level 1
                            let quality = self.entities[i]
                                .person_data
                                .skill_qualities
                                .get(&skill_id)
                                .copied()
                                .unwrap_or(self.config.initial_quality);
                            ((quality / 5.0 * 4.0) + 1.0).round() as u8
                        } else {
                            // Random level 1-5 if quality not enabled
                            (self.rng.random::<u8>() % 5) + 1
                        };

                        // Calculate certification cost: base_price * cost_multiplier * level
                        let cert_cost = skill.current_price
                            * self.config.certification_cost_multiplier
                            * (level as f64);

                        // Check if person can afford certification
                        if self.entities[i].person_data.money >= cert_cost {
                            // Deduct cost
                            self.entities[i].person_data.money -= cert_cost;
                            self.total_certification_cost += cert_cost;

                            // Calculate expiration step
                            let expiration_step = self
                                .config
                                .certification_duration
                                .map(|duration| self.current_step + duration);

                            // Issue certification
                            let cert = crate::skill::Certification::new(
                                "CentralAuthority".to_string(),
                                level,
                                expiration_step,
                            );

                            // Update the skill in the market
                            if let Some(market_skill) = self.market.skills.get_mut(&skill_id) {
                                market_skill.certification = Some(cert.clone());
                                self.total_certifications_issued += 1;

                                debug!(
                                    "Person {} certified skill '{}' at level {} for ${:.2} (expires at step: {:?})",
                                    person_id, skill_id, level, cert_cost, expiration_step
                                );
                            }
                        }
                    }
                }
            }

            // Check for expired certifications and remove them
            for skill in self.market.skills.values_mut() {
                if let Some(cert) = &skill.certification {
                    if cert.is_expired(self.current_step) {
                        skill.certification = None;
                        self.total_certifications_expired += 1;
                        trace!(
                            "Certification expired for skill '{}' at step {}",
                            skill.id,
                            self.current_step
                        );
                    }
                }
            }
        }

        // Process loan payments - borrowers pay back loans
        if self.config.enable_loans {
            self.process_loan_payments();
        }

        // Apply technological progress - increase skill efficiency
        if self.config.tech_growth_rate > 0.0 {
            for skill in self.market.skills.values_mut() {
                skill.efficiency_multiplier *= 1.0 + self.config.tech_growth_rate;
            }
            // Apply same technological progress to black market
            if let Some(ref mut bm) = self.black_market {
                for skill in bm.skills.values_mut() {
                    skill.efficiency_multiplier *= 1.0 + self.config.tech_growth_rate;
                }
            }
        }

        // Apply technology breakthroughs - sudden positive innovations
        if self.config.enable_technology_breakthroughs {
            // Check if a breakthrough occurs this step
            if self.rng.random_range(0.0..1.0) < self.config.tech_breakthrough_probability {
                // Randomly select a skill to receive the breakthrough
                if !self.all_skill_ids.is_empty() {
                    let skill_index = self.rng.random_range(0..self.all_skill_ids.len());
                    let breakthrough_skill_id = &self.all_skill_ids[skill_index];

                    // Calculate the efficiency boost
                    let boost_range = self.config.tech_breakthrough_max_effect
                        - self.config.tech_breakthrough_min_effect;
                    let efficiency_boost = self.config.tech_breakthrough_min_effect
                        + (self.rng.random_range(0.0..1.0) * boost_range);

                    // Apply breakthrough to main market
                    if let Some(skill) = self.market.skills.get_mut(breakthrough_skill_id) {
                        let old_efficiency = skill.efficiency_multiplier;
                        skill.efficiency_multiplier *= efficiency_boost;
                        info!(
                            "Technology breakthrough for skill '{}': efficiency {:.2}x -> {:.2}x ({:.0}% boost)",
                            skill.id,
                            old_efficiency,
                            skill.efficiency_multiplier,
                            (efficiency_boost - 1.0) * 100.0
                        );

                        // Record the breakthrough event
                        self.technology_breakthroughs.push(TechnologyBreakthrough {
                            skill_id: breakthrough_skill_id.clone(),
                            efficiency_boost,
                            step_occurred: self.current_step,
                        });
                    }

                    // Apply same breakthrough to black market if enabled
                    if let Some(ref mut bm) = self.black_market {
                        if let Some(skill) = bm.skills.get_mut(breakthrough_skill_id) {
                            skill.efficiency_multiplier *= efficiency_boost;
                        }
                    }
                }
            }
        }

        // Attempt production - persons combine skills to create new ones
        if self.config.enable_production {
            let _productions_count = self.attempt_production();
            debug!("Production: {} persons successfully produced new skills", _productions_count);
        }

        // Tax redistribution - distribute collected taxes equally among all persons
        if self.config.enable_tax_redistribution && self.config.tax_rate > 0.0 {
            let active_count = self.entities.iter().filter(|e| e.active).count();
            if active_count > 0 {
                // Calculate actual taxes collected this step
                let step_taxes = self.total_taxes_collected - step_taxes_collected_start;

                if step_taxes > 0.0 {
                    let redistribution_per_person = step_taxes / active_count as f64;

                    debug!(
                        "Redistributing ${:.2} in taxes (${:.2} per person) to {} active persons",
                        step_taxes, redistribution_per_person, active_count
                    );

                    for entity in &mut self.entities {
                        if entity.active {
                            entity.person_data.money += redistribution_per_person;
                        }
                    }

                    self.total_taxes_redistributed += step_taxes;
                }
            }
        }

        // Write step data to streaming output if enabled
        if let Some(writer) = &mut self.stream_writer {
            use crate::result::SkillPriceInfo;

            // Calculate current step statistics
            let money_values: Vec<f64> =
                self.entities.iter().filter(|e| e.active).map(|e| e.person_data.money).collect();

            let avg_money = if !money_values.is_empty() {
                money_values.iter().sum::<f64>() / money_values.len() as f64
            } else {
                0.0
            };

            let gini = {
                let sum: f64 = money_values.iter().sum();
                crate::result::calculate_gini_coefficient(&money_values, sum)
            };

            let reputation_values: Vec<f64> = self
                .entities
                .iter()
                .filter(|e| e.active)
                .map(|e| e.person_data.reputation)
                .collect();

            let avg_reputation = if !reputation_values.is_empty() {
                reputation_values.iter().sum::<f64>() / reputation_values.len() as f64
            } else {
                1.0
            };

            // Get top 5 skill prices
            let mut skill_prices: Vec<SkillPriceInfo> = self
                .market
                .skills
                .iter()
                .map(|(id, skill)| SkillPriceInfo { id: id.clone(), price: skill.current_price })
                .collect();
            skill_prices
                .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
            skill_prices.truncate(5);

            let step_data = StepData {
                step: self.current_step,
                trades: trades_count,
                volume: total_volume,
                avg_money,
                gini_coefficient: gini,
                avg_reputation,
                top_skill_prices: skill_prices,
            };

            // Write to stream, but don't fail the simulation if streaming fails
            if let Err(e) = write_step_to_stream(writer, &step_data) {
                warn!("Failed to write step {} to streaming output: {}", self.current_step, e);
            }
        }

        // Collect wealth distribution statistics for this step
        // This enables time-series analysis of how wealth inequality evolves
        let money_values: Vec<f64> =
            self.entities.iter().filter(|e| e.active).map(|e| e.person_data.money).collect();

        if !money_values.is_empty() {
            let mut sorted_money = money_values.clone();
            sorted_money.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let sum: f64 = sorted_money.iter().sum();
            let count = sorted_money.len() as f64;
            let average = sum / count;

            let median = if sorted_money.len() % 2 == 1 {
                sorted_money[sorted_money.len() / 2]
            } else {
                (sorted_money[sorted_money.len() / 2 - 1] + sorted_money[sorted_money.len() / 2])
                    / 2.0
            };

            let variance = sorted_money
                .iter()
                .map(|value| {
                    let diff = average - value;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            let std_dev = variance.sqrt();

            let gini_coefficient = crate::result::calculate_gini_coefficient(&sorted_money, sum);
            let herfindahl_index = crate::result::calculate_herfindahl_index(&sorted_money);
            let (top_10_percent_share, top_1_percent_share, bottom_50_percent_share) =
                crate::result::calculate_wealth_concentration(&sorted_money, sum);

            let snapshot = crate::result::WealthStatsSnapshot {
                step: self.current_step,
                average,
                median,
                std_dev,
                min_money: *sorted_money.first().unwrap_or(&0.0),
                max_money: *sorted_money.last().unwrap_or(&0.0),
                gini_coefficient,
                herfindahl_index,
                top_10_percent_share,
                top_1_percent_share,
                bottom_50_percent_share,
            };

            self.wealth_stats_history.push(snapshot);

            // Track social mobility: assign each person to a quintile (0-4)
            // Create a sorted list of (money, entity_index) to handle ties properly
            let mut money_with_indices: Vec<(f64, usize)> = self
                .entities
                .iter()
                .enumerate()
                .map(|(idx, entity)| (entity.get_money(), idx))
                .collect();
            money_with_indices
                .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            // Assign quintiles based on sorted position (with proper handling of remainders)
            let total_persons = money_with_indices.len();
            for (sorted_position, (_money, entity_idx)) in money_with_indices.iter().enumerate() {
                // Map position to quintile (0-4) using integer division
                // This ensures more balanced quintiles when total_persons is not divisible by 5
                let quintile = ((sorted_position * 5) / total_persons).min(4);

                self.mobility_quintiles.entry(*entity_idx).or_default().push(quintile);
            }
        }

        // Update environment step counter (if enabled)
        if let Some(ref mut environment) = self.environment {
            environment.step();
        }

        // Apply quality decay for unused skills (if quality system enabled)
        if self.config.enable_quality {
            self.apply_quality_decay();
        }

        // Emit step completed event
        let trades_this_step = *self.trades_per_step.last().unwrap_or(&0);
        let volume_this_step = *self.volume_per_step.last().unwrap_or(&0.0);
        self.event_bus
            .emit_step_completed(self.current_step, trades_this_step, volume_this_step);

        // Update incremental money statistics
        self.update_money_statistics();

        self.current_step += 1;
    }

    /// Execute trades sequentially (original logic).
    ///
    /// This method executes all trades one by one in the order they were collected.
    /// It's the default execution mode and ensures deterministic results.
    fn execute_trades_sequential(&mut self, trades_to_execute: Vec<(usize, usize, SkillId, f64)>) {
        for (buyer_idx, seller_idx, skill_id, price) in trades_to_execute {
            self.execute_single_trade(buyer_idx, seller_idx, skill_id, price);
        }
    }

    /// Execute trades in parallel using conflict-free batching.
    ///
    /// Currently executes trades sequentially in their original order to maintain
    /// deterministic results. The infrastructure for conflict detection and batching
    /// is prepared for future true parallel execution.
    ///
    /// # Algorithm
    ///
    /// Currently: Sequential execution in original order
    /// Future: Conflict-free batching with true parallel execution
    ///
    /// # Performance
    ///
    /// Currently: No performance difference from sequential (determinism priority)
    /// Future: Expected 10-60% speedup when parallelization is enabled
    fn execute_trades_parallel(&mut self, trades_to_execute: Vec<(usize, usize, SkillId, f64)>) {
        // Currently executes sequentially to maintain deterministic results
        // The RNG state would differ if trade order changes, leading to different
        // simulation outcomes (friendship formation, etc.)

        // Execute all trades in their original order
        for (buyer_idx, seller_idx, skill_id, price) in trades_to_execute {
            self.execute_single_trade(buyer_idx, seller_idx, skill_id, price);
        }
    }

    /// Execute a single trade between a buyer and seller.
    ///
    /// This method contains the core trade execution logic that was originally
    /// in the main trading loop. It handles all aspects of a trade including:
    /// - Money transfer and tax collection
    /// - Transaction fees
    /// - Reputation updates
    /// - Quality improvements
    /// - Friendship formation
    /// - Event emission
    /// - Statistics tracking
    fn execute_single_trade(
        &mut self,
        buyer_idx: usize,
        seller_idx: usize,
        skill_id: SkillId,
        price: f64,
    ) {
        let seller_entity_id = self.entities[seller_idx].id;
        let buyer_entity_id = self.entities[buyer_idx].id;

        // Calculate transaction fee (deducted from seller's proceeds)
        let fee = price * self.config.transaction_fee;
        let seller_proceeds = price - fee;

        debug!(
            "Executing trade: Buyer {} pays ${:.2}, Seller {} receives ${:.2} (fee: ${:.2})",
            buyer_entity_id, price, seller_entity_id, seller_proceeds, fee
        );

        // Buyer pays full price
        // Note: This may result in negative balance (debt) for Aggressive strategy agents,
        // which is intentional behavior to simulate risk-taking. The simulation supports
        // negative money as reflected in Gini coefficient calculations.
        let buyer_balance_before = self.entities[buyer_idx].person_data.money;
        self.entities[buyer_idx].person_data.money -= price;
        trace!(
            "Person {} balance: ${:.2} -> ${:.2} (spent ${:.2})",
            buyer_entity_id,
            buyer_balance_before,
            self.entities[buyer_idx].person_data.money,
            price
        );
        self.entities[buyer_idx].person_data.record_transaction(
            self.current_step,
            skill_id.clone(),
            crate::person::TransactionType::Buy,
            price,
            Some(seller_entity_id),
        );
        // Increase buyer reputation for completing a purchase
        let buyer_rep_before = self.entities[buyer_idx].person_data.reputation;
        self.entities[buyer_idx].person_data.increase_reputation_as_buyer();
        debug!(
            "Person {} reputation increased as buyer: {:.3} -> {:.3}",
            buyer_entity_id, buyer_rep_before, self.entities[buyer_idx].person_data.reputation
        );
        // Emit reputation change event for buyer
        self.event_bus.emit_reputation_change(
            self.current_step,
            buyer_entity_id,
            buyer_rep_before,
            self.entities[buyer_idx].person_data.reputation,
        );
        // Track successful trade for adaptive strategies
        if self.config.enable_adaptive_strategies {
            self.entities[buyer_idx].person_data.strategy_params.record_successful_buy();
        }

        // Seller receives price minus fee
        let seller_balance_before = self.entities[seller_idx].person_data.money;
        self.entities[seller_idx].person_data.money += seller_proceeds;

        // Calculate and collect tax on seller's proceeds (after transaction fee)
        let tax = seller_proceeds * self.config.tax_rate;
        self.entities[seller_idx].person_data.money -= tax;

        if tax > 0.0 {
            trace!(
                "Person {} paid tax: ${:.2} on proceeds ${:.2}",
                seller_entity_id,
                tax,
                seller_proceeds
            );
        }

        self.total_taxes_collected += tax;

        // Track externality if enabled
        if self.config.enable_externalities {
            // Get externality rate for this skill (per-skill rate overrides default)
            let externality_rate = self
                .config
                .externality_rates_per_skill
                .get(&skill_id)
                .copied()
                .unwrap_or(self.config.externality_rate);

            if externality_rate != 0.0 {
                let externality = crate::externality::Externality::new(
                    skill_id.clone(),
                    self.current_step,
                    price,
                    externality_rate,
                );

                debug!(
                    "Externality recorded for skill {}: private ${:.2}, external ${:.2}, social ${:.2}",
                    skill_id, externality.private_value, externality.external_value, externality.social_value
                );

                self.externality_stats.record(&externality);
            }
        }

        trace!(
            "Person {} balance: ${:.2} -> ${:.2} (received ${:.2}, tax ${:.2})",
            seller_entity_id,
            seller_balance_before,
            self.entities[seller_idx].person_data.money,
            seller_proceeds,
            tax
        );

        self.entities[seller_idx].person_data.record_transaction(
            self.current_step,
            skill_id.clone(),
            crate::person::TransactionType::Sell,
            price,
            Some(buyer_entity_id),
        );
        // Increase seller reputation for completing a sale
        let seller_rep_before = self.entities[seller_idx].person_data.reputation;
        self.entities[seller_idx].person_data.increase_reputation_as_seller();
        debug!(
            "Person {} reputation increased as seller: {:.3} -> {:.3}",
            seller_entity_id, seller_rep_before, self.entities[seller_idx].person_data.reputation
        );
        // Emit reputation change event for seller
        self.event_bus.emit_reputation_change(
            self.current_step,
            seller_entity_id,
            seller_rep_before,
            self.entities[seller_idx].person_data.reputation,
        );
        // Track successful trade for adaptive strategies
        if self.config.enable_adaptive_strategies {
            self.entities[seller_idx].person_data.strategy_params.record_successful_sell();
        }

        // Emit trade executed event
        self.event_bus.emit_trade(
            self.current_step,
            buyer_entity_id,
            seller_entity_id,
            skill_id.clone(),
            price,
        );

        // Record trade action for replay (if enabled)
        if let Some(ref mut action_log) = self.action_log {
            action_log.record(crate::replay::SimulationAction::Trade {
                step: self.current_step,
                buyer_id: buyer_entity_id,
                seller_id: seller_entity_id,
                skill_id: skill_id.clone(),
                price,
            });
        }

        // Improve skill quality for seller (if quality system enabled)
        if self.config.enable_quality {
            if let Some(current_quality) =
                self.entities[seller_idx].person_data.skill_qualities.get_mut(&skill_id)
            {
                let old_quality = *current_quality;
                // Increase quality by improvement rate, capped at 5.0
                *current_quality =
                    (*current_quality + self.config.quality_improvement_rate).min(5.0);
                trace!(
                    "Person {} skill '{}' quality improved: {:.2} -> {:.2}",
                    seller_entity_id,
                    skill_id,
                    old_quality,
                    *current_quality
                );
            }
        }

        // Track total fees collected
        self.total_fees_collected += fee;

        // Friendship formation (if enabled)
        // After a successful trade, there's a chance the buyer and seller become friends
        if self.config.enable_friendships {
            let buyer_id = self.entities[buyer_idx].id;
            let seller_id = self.entities[seller_idx].id;

            // Check if they're not already friends
            if !self.entities[buyer_idx].person_data.is_friend_with(seller_id) {
                // Roll for friendship formation
                let friendship_roll: f64 = self.rng.random();
                if friendship_roll < self.config.friendship_probability {
                    // Form bidirectional friendship
                    self.entities[buyer_idx].person_data.add_friend(seller_id);
                    self.entities[seller_idx].person_data.add_friend(buyer_id);

                    // Update trust network if enabled
                    if let Some(ref mut trust_network) = self.trust_network {
                        trust_network.add_friendship(buyer_id, seller_id);
                    }

                    debug!(
                        "Friendship formed: Person {} and Person {} are now friends after successful trade",
                        buyer_id, seller_id
                    );
                }
            }
        }

        // Record trade in agreement if applicable
        let buyer_id = self.entities[buyer_idx].id;
        let seller_id = self.entities[seller_idx].id;
        self.record_trade_in_agreement(buyer_id, seller_id, price);

        // Track environmental resource consumption (if enabled)
        if let Some(ref mut environment) = self.environment {
            use crate::environment::Resource;
            use std::collections::HashMap as StdHashMap;

            // Calculate resource consumption based on transaction value
            let base_consumption = price * self.config.resource_cost_per_transaction;

            // Distribute consumption evenly across resource types
            let mut resource_costs = StdHashMap::new();
            let num_resources = Resource::all().len() as f64;
            for resource in Resource::all() {
                resource_costs.insert(resource, base_consumption / num_resources);
            }

            environment.consume_resources(&resource_costs);

            trace!(
                "Transaction consumed resources: {:.2} units (price: ${:.2}, multiplier: {:.2})",
                base_consumption,
                price,
                self.config.resource_cost_per_transaction
            );
        }

        // Update per-skill trade statistics
        let skill_stats = self.per_skill_trades.entry(skill_id.clone()).or_insert((0, 0.0));
        skill_stats.0 += 1; // Increment trade count
        skill_stats.1 += price; // Add to total volume

        // Track per-seller, per-skill volumes for market concentration analysis
        let seller_volumes = self.per_skill_seller_volumes.entry(skill_id.clone()).or_default();
        *seller_volumes.entry(seller_idx).or_insert(0.0) += price;

        *self.market.sales_this_step.entry(skill_id.clone()).or_insert(0) += 1;
    }

    /// Update incremental money statistics for efficient retrieval.
    ///
    /// This method recalculates incremental statistics from scratch each step
    /// by iterating through all active entities. The key benefit is enabling
    /// O(1) retrieval of mean, variance, std_dev, min, and max values, which
    /// is especially useful in interactive mode and for frequent statistics queries.
    ///
    /// # Performance Trade-offs
    ///
    /// - **Per-step cost:** O(n) to iterate all entities (unavoidable anyway for other step operations)
    /// - **Statistics retrieval:** O(1) instead of O(n) recalculation on demand
    /// - **Total simulation cost:** O(n × steps) for updates, but avoids O(n) cost per statistics query
    ///
    /// This approach was chosen over tracking individual entity money changes because:
    /// - Simpler implementation with lower bug risk
    /// - Money changes occur in many places throughout the codebase
    /// - The O(n) iteration per step is acceptable since we already iterate entities for other operations
    /// - The primary benefit is O(1) retrieval, not reducing per-step cost
    ///
    /// # Notes
    ///
    /// Median, Gini coefficient, and concentration metrics still require sorting
    /// and remain as post-processing operations, as they cannot be calculated incrementally.
    fn update_money_statistics(&mut self) {
        // Reset incremental stats
        self.money_incremental_stats = crate::result::IncrementalStats::new();
        self.min_money = f64::INFINITY;
        self.max_money = f64::NEG_INFINITY;

        // Update with current money values
        for entity in self.entities.iter().filter(|e| e.active) {
            let money = entity.person_data.money;
            self.money_incremental_stats.update(money);
            self.min_money = self.min_money.min(money);
            self.max_money = self.max_money.max(money);
        }

        // Handle edge case: no active entities
        if self.money_incremental_stats.count() == 0 {
            self.min_money = 0.0;
            self.max_money = 0.0;
        }
    }

    /// Apply quality decay for unused skills.
    ///
    /// For each person, checks which skills they didn't sell this step
    /// and reduces their quality by the configured decay rate.
    /// Quality is floored at 0.0 (minimum quality).
    ///
    /// This simulates the need for ongoing practice and "skills rust" over time.
    fn apply_quality_decay(&mut self) {
        for entity in self.entities.iter_mut() {
            if !entity.active {
                continue;
            }

            // Track which skills were sold this step (satisfied needs)
            let sold_skills: HashSet<SkillId> = entity
                .person_data
                .transaction_history
                .iter()
                .filter(|tx| {
                    tx.step == self.current_step
                        && matches!(tx.transaction_type, crate::person::TransactionType::Sell)
                })
                .map(|tx| tx.skill_id.clone())
                .collect();

            // Apply decay to skills that weren't sold
            for skill_id in entity.person_data.skill_qualities.keys().cloned().collect::<Vec<_>>() {
                if !sold_skills.contains(&skill_id) {
                    if let Some(quality) = entity.person_data.skill_qualities.get_mut(&skill_id) {
                        let old_quality = *quality;
                        // Decrease quality by decay rate, floored at 0.0
                        *quality = (*quality - self.config.quality_decay_rate).max(0.0);
                        if old_quality != *quality {
                            trace!(
                                "Person {} skill '{}' quality decayed (unused): {:.2} -> {:.2}",
                                entity.id,
                                skill_id,
                                old_quality,
                                *quality
                            );
                        }
                    }
                }
            }
        }
    }

    /// Processes loan payments for the current step.
    /// Borrowers make scheduled payments to lenders.
    fn process_loan_payments(&mut self) {
        let mut completed_loans = Vec::new();

        for (loan_id, loan) in self.loans.iter_mut() {
            if loan.is_repaid {
                continue;
            }

            let borrower_idx = loan.borrower_id;
            let lender_idx = loan.lender_id;

            // Check if borrower can afford the payment
            if self.entities[borrower_idx].person_data.money >= loan.payment_per_step {
                // Make the payment
                let payment_amount = loan.make_payment();

                // Transfer money
                self.entities[borrower_idx].person_data.money -= payment_amount;
                self.entities[lender_idx].person_data.money += payment_amount;

                // Update credit score if credit rating is enabled
                if self.config.enable_credit_rating {
                    self.entities[borrower_idx]
                        .person_data
                        .credit_score
                        .record_successful_payment();
                }

                debug!(
                    "Loan payment: Person {} paid ${:.2} to Person {} (remaining: ${:.2})",
                    self.entities[borrower_idx].id,
                    payment_amount,
                    self.entities[lender_idx].id,
                    loan.remaining_principal
                );

                // Check if loan is now fully repaid
                if loan.is_repaid {
                    completed_loans.push(*loan_id);
                    debug!(
                        "Loan {} fully repaid: Person {} to Person {}",
                        loan_id, self.entities[borrower_idx].id, self.entities[lender_idx].id
                    );
                }
            } else {
                // Borrower cannot afford the payment - record as missed payment
                if self.config.enable_credit_rating {
                    self.entities[borrower_idx].person_data.credit_score.record_missed_payment();
                    debug!(
                        "Person {} missed loan payment, credit score affected",
                        self.entities[borrower_idx].id
                    );
                }

                trace!(
                    "Person {} cannot afford loan payment of ${:.2} (has ${:.2})",
                    self.entities[borrower_idx].id,
                    loan.payment_per_step,
                    self.entities[borrower_idx].person_data.money
                );
            }
            // Note: If borrower can't afford payment, they skip it (could add penalties later)
        }

        // Remove completed loans and update statistics
        for loan_id in completed_loans {
            let loan = self.loans.remove(&loan_id).unwrap();

            // Remove loan from person tracking
            self.entities[loan.borrower_id]
                .person_data
                .borrowed_loans
                .retain(|&id| id != loan_id);
            self.entities[loan.lender_id].person_data.lent_loans.retain(|&id| id != loan_id);

            self.total_loans_repaid += 1;
        }
    }

    /// Attempts to sell insurance policies to persons based on configuration.
    ///
    /// Persons have a probability (insurance_purchase_probability) of attempting to
    /// purchase insurance each step if they can afford it and don't already have active coverage.
    /// Supports all three insurance types: Crisis, Income, and Credit.
    fn try_purchase_insurance(&mut self) {
        if !self.config.enable_insurance {
            return;
        }

        // Try to sell all three types of insurance
        let insurance_types = crate::insurance::InsuranceType::all_types();

        for entity_idx in 0..self.entities.len() {
            if !self.entities[entity_idx].active {
                continue;
            }

            // Random chance to attempt purchase
            if !self.rng.random_bool(self.config.insurance_purchase_probability) {
                continue;
            }

            // Randomly select an insurance type to purchase (if not already owned)
            let available_types: Vec<_> = insurance_types
                .iter()
                .filter(|&&insurance_type| {
                    // Check if person already has this type of active insurance
                    !self.entities[entity_idx].person_data.insurance_policies.iter().any(
                        |&policy_id| {
                            if let Some(policy) = self.insurances.get(&policy_id) {
                                policy.insurance_type == insurance_type
                                    && policy.is_active
                                    && !policy.is_expired(self.current_step)
                            } else {
                                false
                            }
                        },
                    )
                })
                .copied()
                .collect();

            if available_types.is_empty() {
                continue; // Already has all types of insurance
            }

            // Choose a random available type
            let insurance_type = *available_types.choose(&mut self.rng).unwrap();

            // Calculate premium based on coverage and reputation
            let base_premium = crate::insurance::Insurance::calculate_base_premium(
                self.config.insurance_coverage_amount,
                self.config.insurance_premium_rate,
            );
            let premium = crate::insurance::Insurance::apply_reputation_discount(
                base_premium,
                self.entities[entity_idx].person_data.reputation,
            );

            // Check if person can afford the premium
            if self.entities[entity_idx].person_data.money < premium {
                continue;
            }

            // Create insurance policy
            let insurance_id = self.insurance_counter;
            self.insurance_counter += 1;

            let insurance = crate::insurance::Insurance::new(
                insurance_id,
                entity_idx,
                insurance_type,
                premium,
                self.config.insurance_coverage_amount,
                self.config.insurance_duration,
                self.current_step,
            );

            // Deduct premium from person's money
            self.entities[entity_idx].person_data.money -= premium;

            // Track insurance
            self.entities[entity_idx].person_data.insurance_policies.push(insurance_id);
            self.insurances.insert(insurance_id, insurance);

            // Update statistics
            self.total_insurance_policies_issued += 1;
            self.total_premiums_collected += premium;

            debug!(
                "Person {} purchased {} insurance (ID: {}) - Premium: ${:.2}, Coverage: ${:.2}",
                entity_idx,
                insurance_type.name(),
                insurance_id,
                premium,
                self.config.insurance_coverage_amount
            );
        }
    }

    /// Processes insurance payouts for crisis events.
    ///
    /// When a crisis occurs, persons with active Crisis insurance receive payouts
    /// to compensate for their losses. The payout amount is calculated based on
    /// the crisis severity and the policy coverage.
    fn process_crisis_insurance_payouts(&mut self, crisis_severity: f64) {
        if !self.config.enable_insurance {
            return;
        }

        // Calculate damage amount based on crisis severity
        // Higher severity = more damage = higher payout claim
        let damage_per_person = self.config.insurance_coverage_amount * crisis_severity;

        let mut policies_to_claim: Vec<(crate::insurance::InsuranceId, usize)> = Vec::new();

        // Collect policies that should receive payouts
        for entity in &self.entities {
            if !entity.active {
                continue;
            }

            for &policy_id in &entity.person_data.insurance_policies {
                if let Some(policy) = self.insurances.get(&policy_id) {
                    if policy.insurance_type == crate::insurance::InsuranceType::Crisis
                        && policy.is_active
                        && !policy.is_expired(self.current_step)
                        && !policy.has_claimed
                    {
                        policies_to_claim.push((policy_id, entity.id));
                    }
                }
            }
        }

        // Process claims
        for (policy_id, owner_id) in policies_to_claim {
            if let Some(policy) = self.insurances.get_mut(&policy_id) {
                let payout = policy.file_claim(damage_per_person, self.current_step);

                if payout > 0.0 {
                    // Add payout to person's money
                    self.entities[owner_id].person_data.money += payout;

                    // Update statistics
                    self.total_insurance_claims_paid += 1;
                    self.total_payouts_made += payout;

                    info!(
                        "💰 Insurance payout: Person {} received ${:.2} from Crisis insurance (Policy ID: {})",
                        owner_id, payout, policy_id
                    );
                }
            }
        }
    }

    /// Processes income insurance payouts for persons with low trade income.
    ///
    /// When a person's trade income (successful sales) falls below a threshold,
    /// their income insurance pays out to help maintain minimum living standards.
    fn process_income_insurance_payouts(&mut self) {
        if !self.config.enable_insurance {
            return;
        }

        // Define low income threshold (50% of base skill price as minimum income expectation)
        let low_income_threshold = self.config.base_skill_price * 0.5;

        let mut policies_to_claim: Vec<(crate::insurance::InsuranceId, usize, f64)> = Vec::new();

        // Check each person's income from this step
        for entity in &self.entities {
            if !entity.active {
                continue;
            }

            // Calculate income from successful sales this step
            let step_income: f64 = entity
                .person_data
                .transaction_history
                .iter()
                .filter(|t| {
                    t.step == self.current_step
                        && matches!(t.transaction_type, crate::person::TransactionType::Sell)
                })
                .map(|t| t.amount)
                .sum();

            // If income is below threshold, check for income insurance
            if step_income < low_income_threshold {
                let income_shortfall = low_income_threshold - step_income;

                for &policy_id in &entity.person_data.insurance_policies {
                    if let Some(policy) = self.insurances.get(&policy_id) {
                        if policy.insurance_type == crate::insurance::InsuranceType::Income
                            && policy.is_active
                            && !policy.is_expired(self.current_step)
                            && !policy.has_claimed
                        {
                            policies_to_claim.push((policy_id, entity.id, income_shortfall));
                            break; // Only claim once per person per step
                        }
                    }
                }
            }
        }

        // Process claims
        for (policy_id, owner_id, claim_amount) in policies_to_claim {
            if let Some(policy) = self.insurances.get_mut(&policy_id) {
                let payout = policy.file_claim(claim_amount, self.current_step);

                if payout > 0.0 {
                    // Add payout to person's money
                    self.entities[owner_id].person_data.money += payout;

                    // Update statistics
                    self.total_insurance_claims_paid += 1;
                    self.total_payouts_made += payout;

                    debug!(
                        "💰 Income insurance payout: Person {} received ${:.2} (Policy ID: {})",
                        owner_id, payout, policy_id
                    );
                }
            }
        }
    }

    /// Processes credit insurance payouts when loan defaults occur.
    ///
    /// When a borrower faces financial distress (low money relative to outstanding debt),
    /// their credit insurance provides funds to help pay off loans, reducing default risk.
    /// This protects borrowers from defaulting and indirectly protects lenders from losses.
    fn process_credit_insurance_payouts(&mut self) {
        if !self.config.enable_insurance || !self.config.enable_loans {
            return;
        }

        // Check for loan defaults (persons with active loans who can't pay)
        // This is a simplified version - in practice, you'd track specific loan defaults

        let mut policies_to_claim: Vec<(crate::insurance::InsuranceId, usize, f64)> = Vec::new();

        // Identify borrowers with loans who are in financial distress (low money)
        let distress_threshold = self.config.base_skill_price * 0.5;

        for entity in &self.entities {
            if !entity.active {
                continue;
            }

            // Check if person has borrowed loans and very low money (potential default)
            let has_borrowed_loans = !entity.person_data.borrowed_loans.is_empty();

            if has_borrowed_loans && entity.person_data.money < distress_threshold {
                // Person is at risk of defaulting - check for credit insurance
                // Calculate potential loss as remaining debt
                let total_debt: f64 = entity
                    .person_data
                    .borrowed_loans
                    .iter()
                    .filter_map(|&loan_id| self.loans.get(&loan_id))
                    .map(|loan| loan.remaining_principal)
                    .sum();

                if total_debt > 0.0 {
                    for &policy_id in &entity.person_data.insurance_policies {
                        if let Some(policy) = self.insurances.get(&policy_id) {
                            if policy.insurance_type == crate::insurance::InsuranceType::Credit
                                && policy.is_active
                                && !policy.is_expired(self.current_step)
                                && !policy.has_claimed
                            {
                                policies_to_claim.push((policy_id, entity.id, total_debt));
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Process claims
        for (policy_id, owner_id, claim_amount) in policies_to_claim {
            if let Some(policy) = self.insurances.get_mut(&policy_id) {
                let payout = policy.file_claim(claim_amount, self.current_step);

                if payout > 0.0 {
                    // Add payout to person's money (helps pay off debt)
                    self.entities[owner_id].person_data.money += payout;

                    // Update statistics
                    self.total_insurance_claims_paid += 1;
                    self.total_payouts_made += payout;

                    debug!(
                        "💰 Credit insurance payout: Person {} received ${:.2} (Policy ID: {})",
                        owner_id, payout, policy_id
                    );
                }
            }
        }
    }

    /// Finds a suitable mentor for a given skill.
    ///
    /// A suitable mentor is someone who:
    /// - Has the skill (either as own_skill or learned)
    /// - Has sufficient quality in that skill (>= min_mentor_quality)
    /// - Is not the learner themselves
    ///
    /// Returns Some((mentor_id, mentor_quality)) if a mentor is found, None otherwise.
    fn find_mentor_for_skill(&self, skill_id: &SkillId, learner_id: usize) -> Option<(usize, f64)> {
        let mut potential_mentors = Vec::new();

        for entity in &self.entities {
            if !entity.active || entity.id == learner_id {
                continue;
            }

            // Check if this person has the skill
            let has_skill = entity.person_data.own_skills.iter().any(|s| &s.id == skill_id)
                || entity.person_data.learned_skills.iter().any(|s| &s.id == skill_id);

            if !has_skill {
                continue;
            }

            // Check quality if quality system is enabled
            if self.config.enable_quality {
                if let Some(&quality) = entity.person_data.skill_qualities.get(skill_id) {
                    if quality >= self.config.min_mentor_quality {
                        potential_mentors.push((entity.id, quality));
                    }
                }
            } else {
                // If quality system is disabled, any person with the skill can mentor
                // Use a default "quality" for sorting purposes
                potential_mentors.push((entity.id, 3.0));
            }
        }

        // Select the best mentor from those eligible (highest quality)
        if !potential_mentors.is_empty() {
            // Sort by quality descending to prefer better mentors
            potential_mentors
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            // Return the best mentor (highest quality)
            potential_mentors.first().copied()
        } else {
            None
        }
    }

    /// Calculates credit score statistics for all persons.
    /// Only called when credit rating system is enabled.
    fn calculate_credit_score_statistics(&self) -> Option<crate::result::CreditScoreStats> {
        let active_persons: Vec<_> = self.entities.iter().filter(|e| e.active).collect();

        if active_persons.is_empty() {
            return None;
        }

        let credit_scores: Vec<u16> =
            active_persons.iter().map(|e| e.person_data.credit_score.score).collect();

        let mut sorted_scores = credit_scores.clone();
        sorted_scores.sort_unstable();

        let total: u64 = sorted_scores.iter().map(|&s| s as u64).sum();
        let average_score = total as f64 / sorted_scores.len() as f64;

        let median_score = if sorted_scores.len().is_multiple_of(2) {
            let mid = sorted_scores.len() / 2;
            (sorted_scores[mid - 1] + sorted_scores[mid]) as f64 / 2.0
        } else {
            sorted_scores[sorted_scores.len() / 2] as f64
        };

        // Calculate standard deviation
        let variance: f64 = sorted_scores
            .iter()
            .map(|&s| {
                let diff = s as f64 - average_score;
                diff * diff
            })
            .sum::<f64>()
            / sorted_scores.len() as f64;
        let std_dev_score = variance.sqrt();

        let min_score = *sorted_scores.first().unwrap_or(&DEFAULT_CREDIT_SCORE);
        let max_score = *sorted_scores.last().unwrap_or(&DEFAULT_CREDIT_SCORE);

        // Count by rating category
        let excellent_count = credit_scores.iter().filter(|&&s| s >= 800).count();
        let very_good_count = credit_scores.iter().filter(|&&s| (740..800).contains(&s)).count();
        let good_count = credit_scores.iter().filter(|&&s| (670..740).contains(&s)).count();
        let fair_count = credit_scores.iter().filter(|&&s| (580..670).contains(&s)).count();
        let poor_count = credit_scores.iter().filter(|&&s| s < 580).count();

        // Sum up payment statistics
        let total_successful_payments: usize = active_persons
            .iter()
            .map(|e| e.person_data.credit_score.successful_payments)
            .sum();

        let total_missed_payments: usize =
            active_persons.iter().map(|e| e.person_data.credit_score.missed_payments).sum();

        Some(crate::result::CreditScoreStats {
            average_score,
            median_score,
            std_dev_score,
            min_score,
            max_score,
            excellent_count,
            very_good_count,
            good_count,
            fair_count,
            poor_count,
            total_successful_payments,
            total_missed_payments,
        })
    }

    fn calculate_average_money(&self) -> f64 {
        if self.entities.is_empty() {
            return 0.0;
        }
        let total_money: f64 =
            self.entities.iter().filter(|e| e.active).map(|e| e.person_data.money).sum();
        let active_count = self.entities.iter().filter(|e| e.active).count();
        if active_count == 0 {
            return 0.0;
        }
        total_money / active_count as f64
    }

    pub fn get_active_entity_count(&self) -> usize {
        self.entities.iter().filter(|e| e.active).count()
    }

    /// Get the current simulation step number
    pub fn get_current_step(&self) -> usize {
        self.current_step
    }

    /// Get the maximum number of steps configured for this simulation
    pub fn get_max_steps(&self) -> usize {
        self.config.max_steps
    }

    /// Get the current scenario being used
    pub fn get_scenario(&self) -> &crate::scenario::Scenario {
        &self.config.scenario
    }

    /// Get the number of active persons in the simulation
    pub fn get_active_persons(&self) -> usize {
        self.get_active_entity_count()
    }

    /// Get read-only access to all entities in the simulation.
    /// Useful for inspecting person state in interactive mode.
    pub fn get_entities(&self) -> &Vec<crate::entity::Entity> {
        &self.entities
    }

    /// Get read-only access to the market.
    /// Useful for inspecting market state and prices in interactive mode.
    pub fn get_market(&self) -> &crate::market::Market {
        &self.market
    }

    /// Get the current simulation result snapshot
    /// This creates a simplified SimulationResult for display in interactive mode
    /// Note: Some complex statistics are omitted for simplicity
    pub fn get_current_result(&self) -> SimulationResult {
        // Collect money distribution
        let mut final_money_distribution: Vec<f64> =
            self.entities.iter().filter(|e| e.active).map(|e| e.person_data.money).collect();
        final_money_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate money statistics using centralized function, then override incremental values
        let money_stats = if !final_money_distribution.is_empty() {
            let mut stats = crate::result::calculate_money_stats(&final_money_distribution);

            // Override with incrementally tracked values for better performance
            stats.average = self.money_incremental_stats.mean();
            stats.std_dev = self.money_incremental_stats.std_dev();
            stats.min_money = self.min_money;
            stats.max_money = self.max_money;

            // Simplified version: skip wealth concentration for intermediate snapshots
            // Note: These are computed by calculate_money_stats but immediately discarded here.
            // The overhead is minimal (O(n) after sorting) and avoids API complexity.
            stats.top_10_percent_share = 0.0;
            stats.top_1_percent_share = 0.0;
            stats.bottom_50_percent_share = 0.0;

            stats
        } else {
            crate::result::calculate_money_stats(&[])
        };

        // Collect reputation distribution
        let mut final_reputation_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.reputation)
            .collect();
        final_reputation_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let reputation_stats = if !final_reputation_distribution.is_empty() {
            let sum: f64 = final_reputation_distribution.iter().sum();
            let count = final_reputation_distribution.len() as f64;
            let average = sum / count;
            let median = if count as usize > 0 {
                if count as usize % 2 == 1 {
                    final_reputation_distribution[count as usize / 2]
                } else {
                    (final_reputation_distribution[count as usize / 2 - 1]
                        + final_reputation_distribution[count as usize / 2])
                        / 2.0
                }
            } else {
                1.0
            };
            crate::result::ReputationStats {
                average,
                median,
                std_dev: 0.0, // Simplified
                min_reputation: *final_reputation_distribution.first().unwrap_or(&1.0),
                max_reputation: *final_reputation_distribution.last().unwrap_or(&1.0),
            }
        } else {
            crate::result::ReputationStats {
                average: 1.0,
                median: 1.0,
                std_dev: 0.0,
                min_reputation: 1.0,
                max_reputation: 1.0,
            }
        };

        // Collect savings distribution
        let mut final_savings_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.savings)
            .collect();
        final_savings_distribution
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let savings_stats = if !final_savings_distribution.is_empty() {
            let sum: f64 = final_savings_distribution.iter().sum();
            let count = final_savings_distribution.len();
            let median = if count > 0 {
                if count % 2 == 1 {
                    final_savings_distribution[count / 2]
                } else {
                    (final_savings_distribution[count / 2 - 1]
                        + final_savings_distribution[count / 2])
                        / 2.0
                }
            } else {
                0.0
            };
            crate::result::SavingsStats {
                total_savings: sum,
                average_savings: if count > 0 { sum / count as f64 } else { 0.0 },
                median_savings: median,
                min_savings: *final_savings_distribution.first().unwrap_or(&0.0),
                max_savings: *final_savings_distribution.last().unwrap_or(&0.0),
            }
        } else {
            crate::result::SavingsStats {
                total_savings: 0.0,
                average_savings: 0.0,
                median_savings: 0.0,
                min_savings: 0.0,
                max_savings: 0.0,
            }
        };

        // Get skill prices from market
        let skill_prices_map = self.market.get_all_skill_prices();
        let final_skill_prices: Vec<crate::result::SkillPriceInfo> = skill_prices_map
            .into_iter()
            .map(|(id, price)| crate::result::SkillPriceInfo { id, price })
            .collect();

        let most_valuable_skill = final_skill_prices
            .iter()
            .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))
            .map(|info| crate::result::SkillPriceInfo { id: info.id.clone(), price: info.price });

        let least_valuable_skill = final_skill_prices
            .iter()
            .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))
            .map(|info| crate::result::SkillPriceInfo { id: info.id.clone(), price: info.price });

        // Capture metadata for this snapshot result
        let metadata = crate::result::SimulationMetadata::capture(
            self.config.seed,
            self.config.entity_count,
            self.config.max_steps,
        );

        SimulationResult {
            metadata,
            total_steps: self.current_step,
            total_duration: 0.0, // Not meaningful in interactive mode
            step_times: vec![],  // Not tracked in interactive mode
            active_persons: self.entities.iter().filter(|e| e.active).count(),
            failed_steps: self.failed_steps,
            final_money_distribution,
            money_statistics: money_stats,
            final_reputation_distribution,
            reputation_statistics: reputation_stats,
            final_savings_distribution,
            savings_statistics: savings_stats,
            credit_score_statistics: if self.config.enable_credit_rating {
                self.calculate_credit_score_statistics()
            } else {
                None
            },
            final_skill_prices,
            most_valuable_skill,
            least_valuable_skill,
            skill_price_history: self.market.skill_price_history.clone(),
            wealth_stats_history: self.wealth_stats_history.clone(),
            trade_volume_statistics: crate::result::TradeVolumeStats {
                total_trades: 0,
                total_volume: 0.0,
                avg_trades_per_step: 0.0,
                avg_volume_per_step: 0.0,
                avg_transaction_value: 0.0,
                min_trades_per_step: 0,
                max_trades_per_step: 0,
                velocity_of_money: 0.0, // Simplified version doesn't calculate
            },
            trades_per_step: self.trades_per_step.clone(),
            volume_per_step: self.volume_per_step.clone(),
            total_fees_collected: self.total_fees_collected,
            per_skill_trade_stats: vec![],    // Simplified
            skill_market_concentration: None, // Not calculated for current result
            business_cycle_statistics: crate::result::detect_business_cycles(&self.volume_per_step),
            failed_trade_statistics: crate::result::FailedTradeStats {
                total_failed_attempts: 0,
                failure_rate: 0.0,
                avg_failed_per_step: 0.0,
                min_failed_per_step: 0,
                max_failed_per_step: 0,
            },
            failed_attempts_per_step: self.failed_attempts_per_step.clone(),
            black_market_statistics: None,
            total_taxes_collected: if self.config.tax_rate > 0.0 {
                Some(self.total_taxes_collected)
            } else {
                None
            },
            total_taxes_redistributed: if self.config.enable_tax_redistribution
                && self.config.tax_rate > 0.0
            {
                Some(self.total_taxes_redistributed)
            } else {
                None
            },
            loan_statistics: None,       // Simplified
            investment_statistics: None, // Simplified for interactive mode
            contract_statistics: None,
            education_statistics: None,
            mentorship_statistics: None,
            certification_statistics: None,
            environment_statistics: None, // Simplified for interactive mode
            friendship_statistics: None,  // Simplified
            trust_network_statistics: None, // Simplified
            trade_agreement_statistics: None, // Simplified
            insurance_statistics: None,   // Simplified
            technology_breakthrough_statistics: None, // Simplified
            group_statistics: None,
            trading_partner_statistics: crate::result::TradingPartnerStats {
                per_person: vec![],
                network_metrics: crate::result::NetworkMetrics {
                    avg_unique_partners: 0.0,
                    network_density: 0.0,
                    most_active_pair: None,
                },
            },
            centrality_analysis: None, // Simplified for interactive mode
            mobility_statistics: crate::result::calculate_mobility_statistics(
                &self.mobility_quintiles,
            ),
            quality_statistics: None,     // Simplified for interactive mode
            externality_statistics: None, // Simplified for interactive mode
            events: if self.event_bus.is_enabled() {
                Some(self.event_bus.events().to_vec())
            } else {
                None
            },
            final_persons_data: self.entities.clone(),
        }
    }

    /// Saves the current simulation state to a checkpoint file.
    ///
    /// The checkpoint includes all stateful information needed to resume the simulation
    /// at the current step. The RNG state is not saved; when resuming, the RNG will be
    /// reseeded based on the current step for reproducibility.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the checkpoint file to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the checkpoint was saved successfully, or an error otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use simulation_framework::{SimulationConfig, SimulationEngine};
    ///
    /// let config = SimulationConfig::default();
    /// let mut engine = SimulationEngine::new(config);
    ///
    /// // Run some steps
    /// for _ in 0..100 {
    ///     engine.step();
    /// }
    ///
    /// // Save checkpoint
    /// engine.save_checkpoint("checkpoint.json").expect("Failed to save checkpoint");
    /// ```
    pub fn save_checkpoint<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        info!("Saving checkpoint at step {} to {:?}", self.current_step, path.as_ref());

        let checkpoint = SimulationCheckpoint {
            config: self.config.clone(),
            entities: self.entities.clone(),
            market: self.market.clone(),
            black_market: self.black_market.clone(),
            current_step: self.current_step,
            all_skill_ids: self.all_skill_ids.clone(),
            trades_per_step: self.trades_per_step.clone(),
            volume_per_step: self.volume_per_step.clone(),
            black_market_trades_per_step: self.black_market_trades_per_step.clone(),
            black_market_volume_per_step: self.black_market_volume_per_step.clone(),
            total_fees_collected: self.total_fees_collected,
            failed_steps: self.failed_steps,
            failed_trade_attempts: self.failed_trade_attempts,
            failed_attempts_per_step: self.failed_attempts_per_step.clone(),
            loans: self.loans.clone(),
            total_loans_issued: self.total_loans_issued,
            total_loans_repaid: self.total_loans_repaid,
            total_taxes_collected: self.total_taxes_collected,
            total_taxes_redistributed: self.total_taxes_redistributed,
            per_skill_trades: self.per_skill_trades.clone(),
            per_skill_seller_volumes: self.per_skill_seller_volumes.clone(),
            contracts: self.contracts.clone(),
            total_contracts_created: self.total_contracts_created,
            total_contracts_completed: self.total_contracts_completed,
            wealth_stats_history: self.wealth_stats_history.clone(),
            money_incremental_stats: self.money_incremental_stats.clone(),
            min_money: self.min_money,
            max_money: self.max_money,
            mobility_quintiles: self.mobility_quintiles.clone(),
            environment: self.environment.clone(),
            voting_system: self.voting_system.clone(),
            total_certifications_issued: self.total_certifications_issued,
            total_certifications_expired: self.total_certifications_expired,
            total_certification_cost: self.total_certification_cost,
            resource_pools: self.resource_pools.clone(),
            trade_agreements: self.trade_agreements.clone(),
            total_trade_agreements_formed: self.total_trade_agreements_formed,
            total_trade_agreements_expired: self.total_trade_agreements_expired,
            trade_agreement_counter: self.trade_agreement_counter,
            trust_network: self.trust_network.clone(),
            insurances: self.insurances.clone(),
            insurance_counter: self.insurance_counter,
            total_insurance_policies_issued: self.total_insurance_policies_issued,
            total_insurance_claims_paid: self.total_insurance_claims_paid,
            total_premiums_collected: self.total_premiums_collected,
            total_payouts_made: self.total_payouts_made,
            technology_breakthroughs: self.technology_breakthroughs.clone(),
            action_log: self.action_log.clone(),
            externality_stats: self.externality_stats.clone(),
        };

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &checkpoint)?;

        debug!("Checkpoint saved successfully");
        Ok(())
    }

    /// Loads a simulation state from a checkpoint file.
    ///
    /// This function creates a new SimulationEngine with the state restored from
    /// the checkpoint. The RNG is reseeded based on the checkpoint's current step
    /// to ensure reproducible behavior.
    ///
    /// **Note:** Plugins are not persisted in checkpoints. After loading a checkpoint,
    /// you must re-register any plugins that were previously registered before
    /// continuing the simulation. This ensures plugin state is properly initialized.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the checkpoint file to load
    ///
    /// # Returns
    ///
    /// Returns a new `SimulationEngine` with the restored state, or an error if
    /// the checkpoint file cannot be read or parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use simulation_framework::SimulationEngine;
    ///
    /// // Load from checkpoint
    /// let mut engine = SimulationEngine::load_checkpoint("checkpoint.json")
    ///     .expect("Failed to load checkpoint");
    ///
    /// // Continue simulation
    /// let result = engine.run();
    /// ```
    pub fn load_checkpoint<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        info!("Loading checkpoint from {:?}", path.as_ref());

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let checkpoint: SimulationCheckpoint = serde_json::from_reader(reader)?;

        // Reseed RNG based on the checkpoint's current step to ensure reproducibility
        // We combine the original seed with the current step to get a deterministic but
        // step-dependent seed
        let seed = checkpoint.config.seed.wrapping_add(checkpoint.current_step as u64);
        let rng = StdRng::seed_from_u64(seed);

        info!(
            "Checkpoint loaded: resuming from step {} with {} entities",
            checkpoint.current_step,
            checkpoint.entities.len()
        );

        // Re-initialize streaming output writer if path is provided
        let stream_writer = if let Some(path) = &checkpoint.config.stream_output_path {
            match File::create(path) {
                Ok(file) => Some(BufWriter::new(file)),
                Err(e) => {
                    warn!(
                        "Failed to create streaming output file: {}. Continuing without streaming.",
                        e
                    );
                    None
                },
            }
        } else {
            None
        };

        // Re-create demand generator from config
        let demand_generator = DemandGenerator::from(checkpoint.config.demand_strategy.clone());

        // Cache production recipes if production is enabled
        let production_recipes = if checkpoint.config.enable_production {
            Some(crate::production::generate_default_recipes())
        } else {
            None
        };

        // Initialize event bus (events from previous run are not preserved)
        let event_bus = EventBus::new(checkpoint.config.enable_events);

        Ok(Self {
            config: checkpoint.config,
            entities: checkpoint.entities,
            market: checkpoint.market,
            black_market: checkpoint.black_market,
            current_step: checkpoint.current_step,
            rng,
            all_skill_ids: checkpoint.all_skill_ids,
            demand_generator,
            trades_per_step: checkpoint.trades_per_step,
            volume_per_step: checkpoint.volume_per_step,
            black_market_trades_per_step: checkpoint.black_market_trades_per_step,
            black_market_volume_per_step: checkpoint.black_market_volume_per_step,
            total_fees_collected: checkpoint.total_fees_collected,
            failed_steps: checkpoint.failed_steps,
            failed_trade_attempts: checkpoint.failed_trade_attempts,
            failed_attempts_per_step: checkpoint.failed_attempts_per_step,
            loans: checkpoint.loans,
            total_loans_issued: checkpoint.total_loans_issued,
            total_loans_repaid: checkpoint.total_loans_repaid,
            total_taxes_collected: checkpoint.total_taxes_collected,
            total_taxes_redistributed: checkpoint.total_taxes_redistributed,
            per_skill_trades: checkpoint.per_skill_trades,
            per_skill_seller_volumes: checkpoint.per_skill_seller_volumes,
            stream_writer,
            contracts: checkpoint.contracts,
            total_contracts_created: checkpoint.total_contracts_created,
            total_contracts_completed: checkpoint.total_contracts_completed,
            mentorships: Vec::new(), // Reset mentorship data on resume
            total_mentorships_formed: 0,
            successful_mentored_learnings: 0,
            total_mentorship_cost_savings: 0.0,
            unique_mentors: HashSet::new(),
            unique_mentees: HashSet::new(),
            total_certifications_issued: checkpoint.total_certifications_issued,
            total_certifications_expired: checkpoint.total_certifications_expired,
            total_certification_cost: checkpoint.total_certification_cost,
            wealth_stats_history: checkpoint.wealth_stats_history,
            money_incremental_stats: checkpoint.money_incremental_stats,
            min_money: checkpoint.min_money,
            max_money: checkpoint.max_money,
            mobility_quintiles: checkpoint.mobility_quintiles,
            plugin_registry: PluginRegistry::new(),
            resource_pools: checkpoint.resource_pools,
            production_recipes,
            environment: checkpoint.environment,
            voting_system: checkpoint.voting_system,
            event_bus,
            trade_agreements: checkpoint.trade_agreements,
            total_trade_agreements_formed: checkpoint.total_trade_agreements_formed,
            total_trade_agreements_expired: checkpoint.total_trade_agreements_expired,
            trade_agreement_counter: checkpoint.trade_agreement_counter,
            trust_network: checkpoint.trust_network,
            insurances: checkpoint.insurances,
            insurance_counter: checkpoint.insurance_counter,
            total_insurance_policies_issued: checkpoint.total_insurance_policies_issued,
            total_insurance_claims_paid: checkpoint.total_insurance_claims_paid,
            total_premiums_collected: checkpoint.total_premiums_collected,
            total_payouts_made: checkpoint.total_payouts_made,
            technology_breakthroughs: checkpoint.technology_breakthroughs,
            action_log: checkpoint.action_log,
            externality_stats: checkpoint.externality_stats,
        })
    }

    /// Attempts to form trade agreements between persons who are friends.
    /// Called once per simulation step to create new agreements based on configured probability.
    fn try_form_trade_agreements(&mut self) {
        if !self.config.enable_trade_agreements {
            return;
        }

        // Remove expired agreements first
        self.trade_agreements.retain(|agreement| {
            let active = agreement.is_active(self.current_step);
            if !active {
                self.total_trade_agreements_expired += 1;
                // Remove agreement IDs from persons
                for entity in &mut self.entities {
                    entity.person_data.trade_agreement_ids.retain(|id| *id != agreement.id);
                }
                trace!("Trade agreement {} expired at step {}", agreement.id, self.current_step);
            }
            active
        });

        // Try to form new agreements
        for i in 0..self.entities.len() {
            if !self.entities[i].active {
                continue;
            }

            // Check probability of forming an agreement
            if self.rng.random_range(0.0..1.0) > self.config.trade_agreement_probability {
                continue;
            }

            let person_id = self.entities[i].id;
            let friends: Vec<PersonId> =
                self.entities[i].person_data.friends.iter().copied().collect();

            // Need at least one friend to form an agreement
            if friends.is_empty() {
                continue;
            }

            // Pick a random friend to form agreement with
            // Safe to unwrap here because we just checked friends is not empty above
            let friend_id = match friends.choose(&mut self.rng) {
                Some(&id) => id,
                None => continue, // Should never happen due to empty check, but handle safely
            };

            // Check if an agreement already exists between these two
            let already_has_agreement = self
                .trade_agreements
                .iter()
                .any(|agreement| agreement.includes_both(person_id, friend_id));

            if already_has_agreement {
                continue;
            }

            // Create new bilateral trade agreement
            let agreement = crate::trade_agreement::TradeAgreement::new_bilateral(
                self.trade_agreement_counter,
                person_id,
                friend_id,
                self.config.trade_agreement_discount,
                self.current_step,
                self.config.trade_agreement_duration,
            );

            trace!(
                "Trade agreement {} formed between persons {} and {} at step {}",
                agreement.id,
                person_id,
                friend_id,
                self.current_step
            );

            // Add agreement ID to both persons
            for entity in &mut self.entities {
                if entity.id == person_id || entity.id == friend_id {
                    entity.person_data.trade_agreement_ids.push(agreement.id);
                }
            }

            self.trade_agreements.push(agreement);
            self.trade_agreement_counter += 1;
            self.total_trade_agreements_formed += 1;
        }
    }

    /// Records a trade under a trade agreement, if applicable.
    fn record_trade_in_agreement(
        &mut self,
        buyer_id: PersonId,
        seller_id: PersonId,
        trade_value: f64,
    ) {
        if !self.config.enable_trade_agreements {
            return;
        }

        // Find if there's an active agreement between these two persons
        let buyer_agreements: Vec<usize> = self
            .entities
            .iter()
            .find(|e| e.id == buyer_id)
            .map(|e| e.person_data.trade_agreement_ids.clone())
            .unwrap_or_default();

        for agreement_id in buyer_agreements {
            if let Some(agreement) = self
                .trade_agreements
                .iter_mut()
                .find(|a| a.id == agreement_id && a.is_active(self.current_step))
            {
                if agreement.includes_both(buyer_id, seller_id) {
                    agreement.record_trade(trade_value);
                    trace!(
                        "Trade recorded in agreement {}: buyer={}, seller={}, value={}",
                        agreement_id,
                        buyer_id,
                        seller_id,
                        trade_value
                    );
                    break;
                }
            }
        }
    }
}
