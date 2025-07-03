use crate::{Entity, SimulationConfig, SimulationResult, Market, Skill, skill, SkillId};
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand::rngs::StdRng;
use rayon::prelude::*;
use std::time::Instant;
use std::collections::{HashSet, HashMap};

pub struct SimulationEngine {
    config: SimulationConfig,
    entities: Vec<Entity>, // These are Persons wrapped in Entity
    market: Market,
    current_step: usize,
    rng: StdRng, // RNG for use throughout the simulation
    all_skill_ids: Vec<SkillId>, // Cache all available skill IDs
}

impl SimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let mut market = Market::new(config.base_skill_price);

        let entities = Self::initialize_entities(&config, &mut rng, &mut market);

        let all_skill_ids = market.skills.keys().cloned().collect::<Vec<SkillId>>();

        Self {
            config,
            entities,
            market,
            current_step: 0,
            rng,
            all_skill_ids,
        }
    }

    fn initialize_entities(
        config: &SimulationConfig,
        rng: &mut StdRng,
        market: &mut Market,
    ) -> Vec<Entity> {
        let mut available_skills_for_market = Vec::new();
        for i in 0..config.entity_count {
            let skill_name = format!("Skill{}", i);
            let skill = Skill::new(skill_name.clone(), config.base_skill_price);
            available_skills_for_market.push(skill.clone());
            market.add_skill(skill);
        }

        let mut entities = Vec::with_capacity(config.entity_count);
        for i in 0..config.entity_count {
            let person_skill = available_skills_for_market.get(i)
                .expect("Not enough unique skills generated for persons")
                .clone();

            market.increment_skill_supply(&person_skill.id);

            let entity = Entity::new(
                i,
                config.initial_money_per_person,
                person_skill.clone(),
            );
            entities.push(entity);
        }
        entities
    }
                                                                                                                                                                                                                                                                                                                                                                                                                                            
    pub fn run(&mut self) -> SimulationResult {
        let start_time = Instant::now();
        let mut step_times = Vec::new();

        println!("Starting economic simulation...");

        for step in 0..self.config.max_steps {
            let step_start = Instant::now();
            self.step();
            let step_duration = step_start.elapsed();
            step_times.push(step_duration.as_secs_f64());
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            
            if step % (self.config.max_steps / 10).max(1) == 0 || step == self.config.max_steps - 1 {
                let active_entities = self.entities.iter().filter(|e| e.active).count();
                println!(
                    "Step {}/{}, Active persons: {}, Avg Money: {:.2}",
                    step + 1,
                    self.config.max_steps,
                    active_entities,
                    self.calculate_average_money()
                );
            }
        }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             
        let total_duration = start_time.elapsed();

        // Collect data for SimulationResult
        let mut final_money_distribution: Vec<f64> = self.entities.iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.money)
            .collect();
        final_money_distribution.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let money_stats = if !final_money_distribution.is_empty() {
            let sum: f64 = final_money_distribution.iter().sum();
            let count = final_money_distribution.len() as f64;
            let average = sum / count;
            let median = if count > 0.0 {
                if count as usize % 2 == 1 {
                    final_money_distribution[count as usize / 2]
                } else {
                    (final_money_distribution[count as usize / 2 - 1] + final_money_distribution[count as usize / 2]) / 2.0
                }
            } else { 0.0 };
            let variance = final_money_distribution.iter().map(|value| {
                let diff = average - value;
                diff * diff
            }).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            crate::result::MoneyStats {
                average,
                median,
                std_dev,
                min_money: *final_money_distribution.first().unwrap_or(&0.0),
                max_money: *final_money_distribution.last().unwrap_or(&0.0),
            }
        } else {
            crate::result::MoneyStats { average: 0.0, median: 0.0, std_dev: 0.0, min_money: 0.0, max_money: 0.0 }
        };

        let final_skill_prices_map = self.market.get_all_skill_prices();
        let mut final_skill_prices_vec: Vec<crate::result::SkillPriceInfo> = final_skill_prices_map
            .into_iter()
            .map(|(id, price)| crate::result::SkillPriceInfo { id, price })
            .collect();

        // Sort by price descending (most valuable first)
        final_skill_prices_vec.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));

        let most_valuable_skill = final_skill_prices_vec.first().cloned();
        let least_valuable_skill = final_skill_prices_vec.last().cloned();

        SimulationResult {
            total_steps: self.config.max_steps,
            total_duration: total_duration.as_secs_f64(),
            step_times,
            active_persons: self.entities.iter().filter(|e| e.active).count(),
            final_money_distribution,
            money_statistics: money_stats,
            final_skill_prices: final_skill_prices_vec,
            most_valuable_skill,
            least_valuable_skill,
            skill_price_history: self.market.skill_price_history.clone(), // Clone the history
            final_persons_data: self.entities.clone(),
        }
    }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 
    fn step(&mut self) {
        // 1. Reset market demand counts and person's satisfied needs for the new step
        self.market.reset_demand_counts();
        for entity in self.entities.iter_mut() {
            if entity.active {
                entity.person_data.needed_skills.clear();
                entity.person_data.satisfied_needs_current_step.clear();
            }
        }

        // 2. Generate demand for skills for each person
        // This can be parallelized if entity list is large, but for 100 entities, sequential is fine.
        // Making it parallel requires self.rng to be handled carefully (e.g. by creating local RNGs per thread).
        // For now, let's keep it sequential for simplicity with the shared rng.
        for entity in self.entities.iter_mut() {
            if !entity.active {
                continue;
            }

            let num_needs = self.rng.gen_range(2..=5);
            let own_skill_id = &entity.person_data.own_skill.id;

            // Create a list of potential skills to need (all skills except own)
            let mut potential_needs: Vec<SkillId> = self.all_skill_ids.iter()
                .filter(|&id| id != own_skill_id)
                .cloned()
                .collect();

            potential_needs.shuffle(&mut self.rng); // Shuffle to pick random ones

            for _ in 0..num_needs {
                if let Some(needed_skill_id) = potential_needs.pop() {
                    // Check if this skill_id is already in needed_skills to avoid duplicates from this round of demand gen
                    if !entity.person_data.needed_skills.iter().any(|item| item.id == needed_skill_id) {
                        let urgency = self.rng.gen_range(1..=3); // Assign random urgency: 1 (low), 2 (medium), 3 (high)
                        entity.person_data.needed_skills.push(crate::person::NeededSkillItem {
                            id: needed_skill_id.clone(),
                            urgency,
                        });
                        self.market.increment_demand(&needed_skill_id);
                    }
                } else {
                    break; // No more unique skills to need
                }
            }
        }

        // 3. Market: Update skill prices based on new demand/supply
        self.market.update_prices(&mut self.rng);

        // 4. Trading: Persons attempt to buy needed skills
        // Create a mapping from SkillId to provider EntityId for quick lookups.
        // This assumes each skill is unique to one person.
        let mut skill_providers: HashMap<SkillId, usize> = HashMap::new();
        for entity_idx in 0..self.entities.len() {
            if self.entities[entity_idx].active {
                let skill_id = self.entities[entity_idx].person_data.own_skill.id.clone();
                skill_providers.insert(skill_id, self.entities[entity_idx].id);
            }
        }

        // Iterate through persons (buyers) by index to allow mutable borrowing of buyer and seller.
        // We need to be careful with borrowing rules if we modify entities directly.
        // One way is to collect changes and apply them, or iterate by index.
        // Let's iterate by index for buyers, and then look up sellers.
        // We might need to clone some data or be careful with mutable access.

        // A temporary list of (buyer_idx, seller_idx, skill_id, price) to execute trades
        // This avoids issues with borrowing self.entities multiple times mutably.
        let mut trades_to_execute: Vec<(usize, usize, SkillId, f64)> = Vec::new();

        for buyer_idx in 0..self.entities.len() {
            if !self.entities[buyer_idx].active {
                continue;
            }

            // Clone needed_skills to iterate while potentially modifying person_data
            let mut current_needs = self.entities[buyer_idx].person_data.needed_skills.clone();

            // Sort needs by urgency, descending (higher urgency first)
            current_needs.sort_by(|a, b| b.urgency.cmp(&a.urgency));

            for needed_item in current_needs {
                let needed_skill_id = &needed_item.id;
                // Check saturation: if already satisfied this step (or bought this step), skip
                if self.entities[buyer_idx].person_data.satisfied_needs_current_step.contains(needed_skill_id) {
                    continue;
                }

                if let Some(skill_price) = self.market.get_price(needed_skill_id) {
                    if self.entities[buyer_idx].person_data.can_afford(skill_price) {
                        if let Some(&seller_id) = skill_providers.get(needed_skill_id) {
                            let seller_idx = seller_id; // Assuming id is the index

                            if buyer_idx == seller_idx { continue; }
                            if !self.entities[seller_idx].active { continue; }

                            trades_to_execute.push((buyer_idx, seller_idx, needed_skill_id.clone(), skill_price));
                            self.entities[buyer_idx].person_data.satisfied_needs_current_step.push(needed_skill_id.clone());
                        }
                    }
                }
            }
        }

        // Execute trades
        for (buyer_idx, seller_idx, skill_id, price) in trades_to_execute {
            // Buyer pays
            self.entities[buyer_idx].person_data.money -= price;
            self.entities[buyer_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(),
                crate::person::TransactionType::Buy,
                price,
                Some(self.entities[seller_idx].id),
            );

            // Seller receives money
            self.entities[seller_idx].person_data.money += price;
            self.entities[seller_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(), // The skill they sold
                crate::person::TransactionType::Sell,
                price,
                Some(self.entities[buyer_idx].id),
            );
        }

        // 5. Update transaction histories, money, etc. - Done as part of trade execution.

        self.current_step += 1;
    }

    fn calculate_average_money(&self) -> f64 {
        if self.entities.is_empty() {
            return 0.0;
        }
        let total_money: f64 = self.entities.iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.money)
            .sum();
        let active_count = self.entities.iter().filter(|e| e.active).count();
        if active_count == 0 { return 0.0; }
        total_money / active_count as f64
    }

    pub fn get_active_entity_count(&self) -> usize {
        self.entities.iter().filter(|e| e.active).count()
    }
}