use crate::{
    scenario::PriceUpdater, Entity, Market, SimulationConfig, SimulationResult, Skill, SkillId,
};
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use std::collections::HashMap;
use std::time::Instant;

pub struct SimulationEngine {
    config: SimulationConfig,
    entities: Vec<Entity>,
    market: Market,
    pub current_step: usize,
    rng: StdRng,
    all_skill_ids: Vec<SkillId>,
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
        }
    }

    // This is the version from feat/economic-simulation-model
    fn initialize_entities(
        config: &SimulationConfig,
        _rng: &mut StdRng, // Prefixed with _ as it was marked unused after prior cleanup
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
            let person_skill = available_skills_for_market
                .get(i)
                .expect("Not enough unique skills generated for persons")
                .clone();

            market.increment_skill_supply(&person_skill.id);

            let entity = Entity::new(i, config.initial_money_per_person, person_skill.clone());
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

            if step % (self.config.max_steps / 10).max(1) == 0 || step == self.config.max_steps - 1
            {
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

        let mut final_money_distribution: Vec<f64> = self
            .entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.person_data.money)
            .collect();
        final_money_distribution
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

            crate::result::MoneyStats {
                average,
                median,
                std_dev,
                min_money: *final_money_distribution.first().unwrap_or(&0.0),
                max_money: *final_money_distribution.last().unwrap_or(&0.0),
            }
        } else {
            crate::result::MoneyStats {
                average: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min_money: 0.0,
                max_money: 0.0,
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
            skill_price_history: self.market.skill_price_history.clone(),
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

        for entity in self.entities.iter_mut() {
            if !entity.active {
                continue;
            }

            let num_needs = self.rng.gen_range(2..=5);
            let own_skill_id = &entity.person_data.own_skill.id;

            let mut potential_needs: Vec<SkillId> = self
                .all_skill_ids
                .iter()
                .filter(|&id| id != own_skill_id)
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

        let mut skill_providers: HashMap<SkillId, usize> = HashMap::new();
        for entity_idx in 0..self.entities.len() {
            if self.entities[entity_idx].active {
                let skill_id = self.entities[entity_idx].person_data.own_skill.id.clone();
                skill_providers.insert(skill_id, self.entities[entity_idx].id);
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
                    if self.entities[buyer_idx].person_data.can_afford(skill_price) {
                        if let Some(&seller_id) = skill_providers.get(needed_skill_id) {
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
                                skill_price,
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

        for (buyer_idx, seller_idx, skill_id, price) in trades_to_execute {
            let seller_entity_id = self.entities[seller_idx].id;
            let buyer_entity_id = self.entities[buyer_idx].id;

            self.entities[buyer_idx].person_data.money -= price;
            self.entities[buyer_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(),
                crate::person::TransactionType::Buy,
                price,
                Some(seller_entity_id),
            );

            self.entities[seller_idx].person_data.money += price;
            self.entities[seller_idx].person_data.record_transaction(
                self.current_step,
                skill_id.clone(),
                crate::person::TransactionType::Sell,
                price,
                Some(buyer_entity_id),
            );
            *self
                .market
                .sales_this_step
                .entry(skill_id.clone())
                .or_insert(0) += 1;
        }

        self.current_step += 1;
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
}
