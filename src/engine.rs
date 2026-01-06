use crate::{
    loan::{Loan, LoanId},
    scenario::PriceUpdater,
    Entity, Market, SimulationConfig, SimulationResult, Skill, SkillId,
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::panic;
use std::path::Path;
use std::time::Instant;

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
    /// Current simulation step
    pub current_step: usize,
    /// All skill IDs in the market
    pub all_skill_ids: Vec<SkillId>,
    /// Trade volume per step history
    pub trades_per_step: Vec<usize>,
    /// Money volume per step history
    pub volume_per_step: Vec<f64>,
    /// Total transaction fees collected
    pub total_fees_collected: f64,
    /// Number of failed steps (recovered from panics)
    pub failed_steps: usize,
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
}

pub struct SimulationEngine {
    config: SimulationConfig,
    entities: Vec<Entity>,
    market: Market,
    pub current_step: usize,
    rng: StdRng,
    all_skill_ids: Vec<SkillId>,
    // Trade volume tracking
    trades_per_step: Vec<usize>,
    volume_per_step: Vec<f64>,
    // Transaction fees tracking
    total_fees_collected: f64,
    // Panic recovery tracking
    failed_steps: usize,
    // Loan system tracking
    loans: HashMap<LoanId, Loan>,
    total_loans_issued: usize,
    total_loans_repaid: usize,
    // Tax system tracking
    total_taxes_collected: f64,
    total_taxes_redistributed: f64,
}

impl SimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let price_updater = PriceUpdater::from(config.scenario.clone());
        let mut market = Market::new(config.base_skill_price, price_updater);

        // This is the version from feat/economic-simulation-model
        let entities = Self::initialize_entities(&config, &mut rng, &mut market);

        let all_skill_ids = market.skills.keys().cloned().collect::<Vec<SkillId>>();

        Self {
            config,
            entities,
            market,
            current_step: 0,
            rng,
            all_skill_ids,
            trades_per_step: Vec::new(),
            volume_per_step: Vec::new(),
            total_fees_collected: 0.0,
            failed_steps: 0,
            loans: HashMap::new(),
            total_loans_issued: 0,
            total_loans_repaid: 0,
            total_taxes_collected: 0.0,
            total_taxes_redistributed: 0.0,
        }
    }

    // This is the version from feat/economic-simulation-model
    fn initialize_entities(
        config: &SimulationConfig,
        _rng: &mut StdRng, // Currently unused but kept for potential future use
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

            let entity = Entity::new(i, config.initial_money_per_person, person_skills);
            entities.push(entity);
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

        info!(
            "Starting economic simulation with {} persons",
            self.entities.len()
        );
        debug!(
            "Simulation configuration: max_steps={}, scenario={:?}",
            self.config.max_steps, self.config.scenario
        );

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
                    warn!(
                        "Failed to save checkpoint at step {}: {}",
                        self.current_step, e
                    );
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
                    pb.set_message(format!(
                        "Step {}/{} | Active: {} | Avg Money: {:.2}",
                        step + 1,
                        self.config.max_steps,
                        active_entities,
                        avg_money
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

        let mut final_money_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.money)
            .collect();
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

        let money_stats = if !final_money_distribution.is_empty() {
            let sum: f64 = final_money_distribution.iter().sum();
            let count = final_money_distribution.len() as f64;
            let average = sum / count;
            let median = if count > 0.0 {
                if count as usize % 2 == 1 {
                    final_money_distribution[count as usize / 2]
                } else {
                    (final_money_distribution[count as usize / 2 - 1]
                        + final_money_distribution[count as usize / 2])
                        / 2.0
                }
            } else {
                0.0
            };
            let variance = final_money_distribution
                .iter()
                .map(|value| {
                    let diff = average - value;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            let std_dev = variance.sqrt();

            // Calculate Gini coefficient using the shared utility function
            let gini_coefficient =
                crate::result::calculate_gini_coefficient(&final_money_distribution, sum);

            // Calculate Herfindahl-Hirschman Index for wealth concentration
            let herfindahl_index =
                crate::result::calculate_herfindahl_index(&final_money_distribution);

            crate::result::MoneyStats {
                average,
                median,
                std_dev,
                min_money: *final_money_distribution.first().unwrap_or(&0.0),
                max_money: *final_money_distribution.last().unwrap_or(&0.0),
                gini_coefficient,
                herfindahl_index,
            }
        } else {
            crate::result::MoneyStats {
                average: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min_money: 0.0,
                max_money: 0.0,
                gini_coefficient: 0.0,
                herfindahl_index: 0.0,
            }
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

        final_skill_prices_vec.sort_by(|a, b| {
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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

            crate::result::TradeVolumeStats {
                total_trades,
                total_volume,
                avg_trades_per_step,
                avg_volume_per_step,
                avg_transaction_value,
                min_trades_per_step,
                max_trades_per_step,
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

        SimulationResult {
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
            final_skill_prices: final_skill_prices_vec,
            most_valuable_skill,
            least_valuable_skill,
            skill_price_history: self.market.skill_price_history.clone(),
            trade_volume_statistics,
            trades_per_step: self.trades_per_step.clone(),
            volume_per_step: self.volume_per_step.clone(),
            total_fees_collected: self.total_fees_collected,
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
            final_persons_data: self.entities.clone(),
        }
    }

    pub fn step(&mut self) {
        self.market.reset_demand_counts();
        for entity in self.entities.iter_mut() {
            if entity.active {
                entity.person_data.needed_skills.clear();
                entity.person_data.satisfied_needs_current_step.clear();
            }
        }

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

            // Calculate base number of needs (2-5)
            let base_num_needs = self.rng.gen_range(2..=5);

            // Apply seasonal modulation to the number of needs
            // Use the average seasonal factor across all owned skills
            let num_needs = if seasonal_enabled {
                let avg_seasonal_factor: f64 = entity
                    .person_data
                    .own_skills
                    .iter()
                    .map(|skill| {
                        seasonal_factors
                            .get(&skill.id)
                            .copied()
                            .unwrap_or(1.0)
                    })
                    .sum::<f64>()
                    / entity.person_data.own_skills.len() as f64;
                // Modulate the number of needs, clamping between 1 and 5
                ((base_num_needs as f64 * avg_seasonal_factor).round() as usize).clamp(1, 5)
            } else {
                base_num_needs
            };

            // Collect IDs of all skills this person owns
            let own_skill_ids: Vec<&SkillId> = entity
                .person_data
                .own_skills
                .iter()
                .map(|skill| &skill.id)
                .collect();

            // Filter out skills the person already owns
            let mut potential_needs: Vec<SkillId> = self
                .all_skill_ids
                .iter()
                .filter(|&id| !own_skill_ids.contains(&id))
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
                        let urgency = self.rng.gen_range(1..=3);
                        entity
                            .person_data
                            .needed_skills
                            .push(crate::person::NeededSkillItem {
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

        self.market.update_prices(&mut self.rng);

        // Build a map of skill providers
        // Since multiple persons can now provide the same skill, we use Vec<usize>
        let mut skill_providers: HashMap<SkillId, Vec<usize>> = HashMap::new();
        for entity_idx in 0..self.entities.len() {
            if self.entities[entity_idx].active {
                for skill in &self.entities[entity_idx].person_data.own_skills {
                    skill_providers
                        .entry(skill.id.clone())
                        .or_insert_with(Vec::new)
                        .push(self.entities[entity_idx].id);
                }
            }
        }

        let mut trades_to_execute: Vec<(usize, usize, SkillId, f64)> = Vec::new();

        for buyer_idx in 0..self.entities.len() {
            if !self.entities[buyer_idx].active {
                continue;
            }

            let mut current_needs = self.entities[buyer_idx].person_data.needed_skills.clone();
            current_needs.sort_by(|a, b| b.urgency.cmp(&a.urgency));

            for needed_item in current_needs {
                let needed_skill_id = &needed_item.id;
                if self.entities[buyer_idx]
                    .person_data
                    .satisfied_needs_current_step
                    .contains(needed_skill_id)
                {
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
                    let final_price = if let Some(seller_id) = seller_id_opt {
                        let seller_reputation_multiplier = self.entities[seller_id]
                            .person_data
                            .reputation_price_multiplier();
                        efficiency_adjusted_price * seller_reputation_multiplier
                    } else {
                        efficiency_adjusted_price
                    };

                    if self.entities[buyer_idx].person_data.can_afford(final_price) {
                        if let Some(seller_id) = seller_id_opt {
                            let seller_idx = seller_id;

                            if buyer_idx == seller_idx {
                                continue;
                            }
                            if !self.entities[seller_idx].active {
                                continue;
                            }

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
                    }
                }
            }
        }

        // Track trade volume for this step
        let trades_count = trades_to_execute.len();
        let total_volume: f64 = trades_to_execute.iter().map(|(_, _, _, price)| price).sum();
        let step_taxes_collected_start = self.total_taxes_collected;

        for (buyer_idx, seller_idx, skill_id, price) in trades_to_execute {
            let seller_entity_id = self.entities[seller_idx].id;
            let buyer_entity_id = self.entities[buyer_idx].id;

            // Calculate transaction fee (deducted from seller's proceeds)
            let fee = price * self.config.transaction_fee;
            let seller_proceeds = price - fee;

            // Buyer pays full price
            self.entities[buyer_idx].person_data.money -= price;
            self.entities[buyer_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(),
                crate::person::TransactionType::Buy,
                price,
                Some(seller_entity_id),
            );
            // Increase buyer reputation for completing a purchase
            self.entities[buyer_idx]
                .person_data
                .increase_reputation_as_buyer();

            // Seller receives price minus fee
            self.entities[seller_idx].person_data.money += seller_proceeds;

            // Calculate and collect tax on seller's proceeds (after transaction fee)
            let tax = seller_proceeds * self.config.tax_rate;
            self.entities[seller_idx].person_data.money -= tax;
            self.total_taxes_collected += tax;

            self.entities[seller_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(),
                crate::person::TransactionType::Sell,
                price,
                Some(buyer_entity_id),
            );
            // Increase seller reputation for completing a sale
            self.entities[seller_idx]
                .person_data
                .increase_reputation_as_seller();

            // Track total fees collected
            self.total_fees_collected += fee;

            *self
                .market
                .sales_this_step
                .entry(skill_id.clone())
                .or_insert(0) += 1;
        }

        // Record trade volume statistics for this step
        self.trades_per_step.push(trades_count);
        self.volume_per_step.push(total_volume);

        // Apply reputation decay for all active entities
        for entity in &mut self.entities {
            if entity.active {
                entity.person_data.apply_reputation_decay();
            }
        }

        // Apply savings - persons save a portion of their money
        if self.config.savings_rate > 0.0 {
            for entity in &mut self.entities {
                if entity.active {
                    entity.person_data.apply_savings(self.config.savings_rate);
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
        }

        // Tax redistribution - distribute collected taxes equally among all persons
        if self.config.enable_tax_redistribution && self.config.tax_rate > 0.0 {
            let active_count = self.entities.iter().filter(|e| e.active).count();
            if active_count > 0 {
                // Calculate actual taxes collected this step
                let step_taxes = self.total_taxes_collected - step_taxes_collected_start;

                if step_taxes > 0.0 {
                    let redistribution_per_person = step_taxes / active_count as f64;

                    for entity in &mut self.entities {
                        if entity.active {
                            entity.person_data.money += redistribution_per_person;
                        }
                    }

                    self.total_taxes_redistributed += step_taxes;
                }
            }
        }

        self.current_step += 1;
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

                // Check if loan is now fully repaid
                if loan.is_repaid {
                    completed_loans.push(*loan_id);
                }
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
            self.entities[loan.lender_id]
                .person_data
                .lent_loans
                .retain(|&id| id != loan_id);

            self.total_loans_repaid += 1;
        }
    }

    fn calculate_average_money(&self) -> f64 {
        if self.entities.is_empty() {
            return 0.0;
        }
        let total_money: f64 = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.money)
            .sum();
        let active_count = self.entities.iter().filter(|e| e.active).count();
        if active_count == 0 {
            return 0.0;
        }
        total_money / active_count as f64
    }

    pub fn get_active_entity_count(&self) -> usize {
        self.entities.iter().filter(|e| e.active).count()
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
        info!(
            "Saving checkpoint at step {} to {:?}",
            self.current_step,
            path.as_ref()
        );

        let checkpoint = SimulationCheckpoint {
            config: self.config.clone(),
            entities: self.entities.clone(),
            market: self.market.clone(),
            current_step: self.current_step,
            all_skill_ids: self.all_skill_ids.clone(),
            trades_per_step: self.trades_per_step.clone(),
            volume_per_step: self.volume_per_step.clone(),
            total_fees_collected: self.total_fees_collected,
            failed_steps: self.failed_steps,
            loans: self.loans.clone(),
            total_loans_issued: self.total_loans_issued,
            total_loans_repaid: self.total_loans_repaid,
            total_taxes_collected: self.total_taxes_collected,
            total_taxes_redistributed: self.total_taxes_redistributed,
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
        let seed = checkpoint
            .config
            .seed
            .wrapping_add(checkpoint.current_step as u64);
        let rng = StdRng::seed_from_u64(seed);

        info!(
            "Checkpoint loaded: resuming from step {} with {} entities",
            checkpoint.current_step,
            checkpoint.entities.len()
        );

        Ok(Self {
            config: checkpoint.config,
            entities: checkpoint.entities,
            market: checkpoint.market,
            current_step: checkpoint.current_step,
            rng,
            all_skill_ids: checkpoint.all_skill_ids,
            trades_per_step: checkpoint.trades_per_step,
            volume_per_step: checkpoint.volume_per_step,
            total_fees_collected: checkpoint.total_fees_collected,
            failed_steps: checkpoint.failed_steps,
            loans: checkpoint.loans,
            total_loans_issued: checkpoint.total_loans_issued,
            total_loans_repaid: checkpoint.total_loans_repaid,
            total_taxes_collected: checkpoint.total_taxes_collected,
            total_taxes_redistributed: checkpoint.total_taxes_redistributed,
        })
    }
}
