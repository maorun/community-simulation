use crate::{
    scenario::{PriceUpdater, RngCore},
    skill::{Skill, SkillId},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Market {
    pub skills: HashMap<SkillId, Skill>,
    pub demand_counts: HashMap<SkillId, usize>,
    pub supply_counts: HashMap<SkillId, usize>,
    pub base_skill_price: f64,
    pub price_elasticity_factor: f64,
    pub volatility_percentage: f64,
    pub min_skill_price: f64,
    pub max_skill_price: f64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub skill_price_history: HashMap<SkillId, Vec<f64>>,
    #[serde(skip)]
    price_updater: Box<dyn PriceUpdater>,
    pub sales_this_step: HashMap<SkillId, usize>,
}

impl Market {
    pub fn new(base_skill_price: f64, price_updater: Box<dyn PriceUpdater>) -> Self {
        Market {
            skills: HashMap::new(),
            demand_counts: HashMap::new(),
            supply_counts: HashMap::new(),
            base_skill_price,
            price_elasticity_factor: 0.1,
            volatility_percentage: 0.02,
            min_skill_price: 1.0,
            max_skill_price: 1000.0,
            skill_price_history: HashMap::new(),
            price_updater,
            sales_this_step: HashMap::new(),
        }
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.supply_counts.insert(skill.id.clone(), 0);
        self.demand_counts.insert(skill.id.clone(), 0);
        self.skill_price_history.insert(skill.id.clone(), Vec::new());
        self.skills.insert(skill.id.clone(), skill);
    }

    pub fn increment_skill_supply(&mut self, skill_id: &SkillId) {
        *self.supply_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    pub fn reset_demand_counts(&mut self) {
        for demand_count in self.demand_counts.values_mut() {
            *demand_count = 0;
        }
        self.sales_this_step.clear();
    }

    pub fn increment_demand(&mut self, skill_id: &SkillId) {
        *self.demand_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    pub fn get_price(&self, skill_id: &SkillId) -> Option<f64> {
        self.skills.get(skill_id).map(|s| s.current_price)
    }

    pub fn update_prices(&mut self, rng: &mut dyn RngCore) {
        let updater = self.price_updater.clone();
        updater.update_prices(self, rng);
    }

    pub fn get_all_skill_prices(&self) -> HashMap<SkillId, f64> {
        self.skills
            .iter()
            .map(|(id, skill)| (id.clone(), skill.current_price))
            .collect()
    }
}
