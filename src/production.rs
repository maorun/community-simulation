//! Production system for combining skills to create new skills.
//!
//! This module implements a simple production/crafting system where persons can combine
//! two skills they own to produce a new, more valuable skill. This simulates supply chains,
//! skill composition, and economic specialization.

use crate::error::{Result, SimulationError};
use crate::skill::SkillId;
use serde::{Deserialize, Serialize};

/// Default execution time in steps for a recipe
fn default_execution_time() -> usize {
    1
}

/// Default batch size for a recipe
fn default_batch_size() -> usize {
    1
}

/// Default yield multiplier for a recipe
fn default_yield_multiplier() -> f64 {
    1.0
}

/// Default energy consumption for a recipe
fn default_energy_consumption() -> f64 {
    0.0
}

/// Default quality tier for a recipe
fn default_quality_tier() -> usize {
    1
}

/// Default labor requirement for a recipe
fn default_labor_requirement() -> f64 {
    0.0
}

/// Represents a production recipe that combines two input skills into one output skill.
///
/// Recipes define how skills can be combined in the economy, enabling supply chains
/// and skill composition. For example, "Programming" + "DataAnalysis" might produce
/// "MachineLearning".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recipe {
    /// First input skill required for production
    pub input_skill_1: SkillId,
    /// Second input skill required for production
    pub input_skill_2: SkillId,
    /// Output skill produced by combining the inputs
    pub output_skill: SkillId,
    /// Cost multiplier for production (1.0 = sum of input prices, 1.5 = 1.5x sum, etc.)
    pub cost_multiplier: f64,
    /// Execution time in steps required to complete production
    #[serde(default = "default_execution_time")]
    pub execution_time: usize,
    /// Batch size produced by the recipe
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Yield multiplier produced by the recipe
    #[serde(default = "default_yield_multiplier")]
    pub yield_multiplier: f64,
    /// Energy consumption required for production
    #[serde(default = "default_energy_consumption")]
    pub energy_consumption: f64,
    /// Quality tier produced by the recipe
    #[serde(default = "default_quality_tier")]
    pub quality_tier: usize,
    /// Labor requirement for production
    #[serde(default = "default_labor_requirement")]
    pub labor_requirement: f64,
}

impl Recipe {
    /// Creates a new production recipe with default execution time (1 step) and batch size (1).
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    ///
    /// # Examples
    /// ```
    /// use community_simulation::production::Recipe;
    ///
    /// let recipe = Recipe::new(
    ///     "Programming".to_string(),
    ///     "DataAnalysis".to_string(),
    ///     "MachineLearning".to_string(),
    ///     1.5
    /// );
    /// ```
    pub fn new(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size: 1,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified execution time.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `execution_time` - Number of simulation steps required for execution
    pub fn with_execution_time(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size: 1,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified batch size.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `batch_size` - Batch size produced by the recipe
    pub fn with_batch_size(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        batch_size: usize,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified yield multiplier.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `yield_multiplier` - Yield multiplier produced by the recipe
    pub fn with_yield_multiplier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        yield_multiplier: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size: 1,
            yield_multiplier,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified energy consumption.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `energy_consumption` - Energy consumption required for production
    pub fn with_energy_consumption(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        energy_consumption: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size: 1,
            yield_multiplier: 1.0,
            energy_consumption,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified quality tier.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `quality_tier` - Quality tier produced by the recipe
    pub fn with_quality_tier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        quality_tier: usize,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size: 1,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with a specified labor requirement.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `labor_requirement` - Labor requirement for production
    pub fn with_labor_requirement(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        labor_requirement: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size: 1,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement,
        }
    }

    /// Creates a new production recipe with specified execution time and batch size.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    /// * `execution_time` - Number of simulation steps required for execution
    /// * `batch_size` - Batch size produced by the recipe
    pub fn with_execution_time_and_batch_size(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        batch_size: usize,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size,
            yield_multiplier: 1.0,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified execution time and yield multiplier.
    pub fn with_execution_time_and_yield_multiplier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        yield_multiplier: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size: 1,
            yield_multiplier,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified batch size and yield multiplier.
    pub fn with_batch_size_and_yield_multiplier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        batch_size: usize,
        yield_multiplier: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time: 1,
            batch_size,
            yield_multiplier,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified execution time, batch size, and yield multiplier.
    pub fn with_execution_time_batch_size_and_yield_multiplier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        batch_size: usize,
        yield_multiplier: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size,
            yield_multiplier,
            energy_consumption: 0.0,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified execution time, batch size, yield multiplier, and energy consumption.
    #[allow(clippy::too_many_arguments)]
    pub fn with_execution_time_batch_size_yield_multiplier_and_energy_consumption(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        batch_size: usize,
        yield_multiplier: f64,
        energy_consumption: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size,
            yield_multiplier,
            energy_consumption,
            quality_tier: 1,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified execution time, batch size, yield multiplier, energy consumption, and quality tier.
    #[allow(clippy::too_many_arguments)]
    pub fn with_execution_time_batch_size_yield_multiplier_energy_consumption_and_quality_tier(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        batch_size: usize,
        yield_multiplier: f64,
        energy_consumption: f64,
        quality_tier: usize,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size,
            yield_multiplier,
            energy_consumption,
            quality_tier,
            labor_requirement: 0.0,
        }
    }

    /// Creates a new production recipe with specified execution time, batch size, yield multiplier, energy consumption, quality tier, and labor requirement.
    #[allow(clippy::too_many_arguments)]
    pub fn with_execution_time_batch_size_yield_multiplier_energy_consumption_quality_tier_and_labor_requirement(
        input_skill_1: SkillId,
        input_skill_2: SkillId,
        output_skill: SkillId,
        cost_multiplier: f64,
        execution_time: usize,
        batch_size: usize,
        yield_multiplier: f64,
        energy_consumption: f64,
        quality_tier: usize,
        labor_requirement: f64,
    ) -> Self {
        Recipe {
            input_skill_1,
            input_skill_2,
            output_skill,
            cost_multiplier,
            execution_time,
            batch_size,
            yield_multiplier,
            energy_consumption,
            quality_tier,
            labor_requirement,
        }
    }

    /// Validates the recipe parameters (e.g., positive execution time/duration, batch size, yield multiplier, energy consumption, quality tier, and labor requirement).
    pub fn validate(&self) -> Result<()> {
        if self.execution_time == 0 {
            return Err(SimulationError::ValidationError(
                "Recipe execution time (duration) must be greater than 0".to_string(),
            ));
        }
        if self.batch_size == 0 {
            return Err(SimulationError::ValidationError(
                "Recipe batch size must be greater than 0".to_string(),
            ));
        }
        if self.yield_multiplier <= 0.0 || !self.yield_multiplier.is_finite() {
            return Err(SimulationError::ValidationError(
                "Recipe yield multiplier must be greater than 0 and finite".to_string(),
            ));
        }
        if self.energy_consumption < 0.0
            || !self.energy_consumption.is_finite()
            || self.energy_consumption > MAX_RECIPE_ENERGY_CONSUMPTION
        {
            return Err(SimulationError::ValidationError(format!(
                "Recipe energy consumption must be non-negative, finite and at most {}",
                MAX_RECIPE_ENERGY_CONSUMPTION
            )));
        }
        if self.quality_tier == 0 {
            return Err(SimulationError::ValidationError(
                "Recipe quality tier must be greater than 0".to_string(),
            ));
        }
        if self.labor_requirement < 0.0 || !self.labor_requirement.is_finite() {
            return Err(SimulationError::ValidationError(
                "Recipe labor requirement must be non-negative and finite".to_string(),
            ));
        }
        Ok(())
    }

    /// Checks if the recipe can be crafted with the given skills.
    ///
    /// The recipe matches if the person has both required input skills,
    /// regardless of order.
    ///
    /// # Arguments
    /// * `available_skills` - List of skill IDs the person possesses
    ///
    /// # Returns
    /// `true` if both input skills are available, `false` otherwise
    pub fn can_craft(&self, available_skills: &[SkillId]) -> bool {
        let has_input_1 = available_skills.contains(&self.input_skill_1);
        let has_input_2 = available_skills.contains(&self.input_skill_2);
        has_input_1 && has_input_2
    }

    /// Calculates the production cost based on input skill prices and cost multiplier.
    ///
    /// # Arguments
    /// * `input_price_1` - Current market price of first input skill
    /// * `input_price_2` - Current market price of second input skill
    ///
    /// # Returns
    /// The total cost to produce the output skill
    pub fn calculate_cost(&self, input_price_1: f64, input_price_2: f64) -> f64 {
        (input_price_1 + input_price_2) * self.cost_multiplier
    }

    /// Calculates the cost that a person pays to start one production run.
    ///
    /// The batch size spreads the input cost over the produced batch, so the
    /// person pays `calculate_cost / batch_size` for the inputs. The labor
    /// requirement adds `labor_requirement * labor_unit_price` to the cost.
    ///
    /// # Arguments
    /// * `input_price_1` - Current market price of first input skill
    /// * `input_price_2` - Current market price of second input skill
    /// * `labor_unit_price` - Price of one labor unit (typically the base skill price)
    ///
    /// # Examples
    /// ```
    /// use community_simulation::production::Recipe;
    ///
    /// let mut recipe = Recipe::new("A".to_string(), "B".to_string(), "C".to_string(), 1.0);
    /// recipe.batch_size = 2;
    /// recipe.labor_requirement = 0.5;
    /// // (10 + 10) * 1.0 / 2 + 0.5 * 10 = 15
    /// assert_eq!(recipe.calculate_production_cost(10.0, 10.0, 10.0), 15.0);
    /// ```
    pub fn calculate_production_cost(
        &self,
        input_price_1: f64,
        input_price_2: f64,
        labor_unit_price: f64,
    ) -> f64 {
        let batch_size = self.batch_size.max(1) as f64;
        let input_cost = self.calculate_cost(input_price_1, input_price_2) / batch_size;
        let labor_cost = self.labor_requirement.max(0.0) * labor_unit_price;
        clamp_price(input_cost + labor_cost)
    }

    /// Calculates the market price of the output skill.
    ///
    /// The price is `calculate_cost * yield_multiplier * quality_price_factor`.
    ///
    /// # Arguments
    /// * `input_price_1` - Current market price of first input skill
    /// * `input_price_2` - Current market price of second input skill
    ///
    /// # Examples
    /// ```
    /// use community_simulation::production::Recipe;
    ///
    /// let mut recipe = Recipe::new("A".to_string(), "B".to_string(), "C".to_string(), 1.0);
    /// recipe.yield_multiplier = 2.0;
    /// recipe.quality_tier = 3;
    /// // (10 + 10) * 1.0 * 2.0 * 1.2 = 48
    /// assert!((recipe.calculate_output_price(10.0, 10.0) - 48.0).abs() < 1e-9);
    /// ```
    pub fn calculate_output_price(&self, input_price_1: f64, input_price_2: f64) -> f64 {
        clamp_price(
            self.calculate_cost(input_price_1, input_price_2)
                * self.yield_multiplier
                * self.quality_price_factor(),
        )
    }

    /// Returns the price factor for the quality tier.
    ///
    /// Tier 1 gives 1.0. Each higher tier adds [`QUALITY_TIER_PRICE_PREMIUM`].
    pub fn quality_price_factor(&self) -> f64 {
        let extra_tiers = self.quality_tier.saturating_sub(1) as f64;
        1.0 + extra_tiers * QUALITY_TIER_PRICE_PREMIUM
    }
}

/// Price premium per quality tier above tier 1 (0.1 = +10% per tier).
pub const QUALITY_TIER_PRICE_PREMIUM: f64 = 0.1;

/// Upper limit for the energy consumption of one recipe run.
///
/// The limit keeps the summed consumption of the environment finite.
pub const MAX_RECIPE_ENERGY_CONSUMPTION: f64 = 1.0e9;

/// Upper limit for a calculated production cost or output price.
///
/// The limit keeps each balance finite when a recipe has extreme multipliers.
pub const MAX_PRODUCTION_PRICE: f64 = 1.0e12;

/// Limits a price to the range `0.0..=MAX_PRODUCTION_PRICE`. A `NaN` value gives `0.0`.
fn clamp_price(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, MAX_PRODUCTION_PRICE)
    }
}

/// A production run that started but has not completed yet.
///
/// The engine creates a pending production when a recipe has an
/// `execution_time` greater than 1. The person gets the output skill
/// when the simulation reaches `completes_at_step`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingProduction {
    /// Index of the producing entity in the engine entity list
    pub entity_index: usize,
    /// Output skill that the person gets at completion
    pub output_skill: SkillId,
    /// Market price of the output skill at the start of production
    pub output_price: f64,
    /// Simulation step at which the production completes
    pub completes_at_step: usize,
    /// Cost that the person paid at the start. The engine refunds it if the production fails.
    #[serde(default)]
    pub cost_paid: f64,
}

/// Generates a predefined set of production recipes for the simulation.
///
/// This function creates recipes that combine basic skills into more advanced skills,
/// simulating technological progress and skill composition in the economy.
///
/// # Returns
/// A vector of production recipes
///
/// # Example Recipes
/// - Programming + DataAnalysis → MachineLearning
/// - Marketing + GraphicDesign → DigitalMarketing
/// - Engineering + Programming → SoftwareEngineering
/// - Writing + Marketing → ContentMarketing
/// - Accounting + DataAnalysis → FinancialAnalysis
///
/// # Examples
/// ```
/// use community_simulation::production::generate_default_recipes;
///
/// let recipes = generate_default_recipes();
/// assert!(!recipes.is_empty());
/// ```
pub fn generate_default_recipes() -> Vec<Recipe> {
    vec![
        // Tech/Data recipes
        Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        ),
        Recipe::new(
            "Programming".to_string(),
            "GraphicDesign".to_string(),
            "WebDevelopment".to_string(),
            1.3,
        ),
        Recipe::new(
            "Engineering".to_string(),
            "Programming".to_string(),
            "SoftwareEngineering".to_string(),
            1.4,
        ),
        // Business/Marketing recipes
        Recipe::new(
            "Marketing".to_string(),
            "GraphicDesign".to_string(),
            "DigitalMarketing".to_string(),
            1.3,
        ),
        Recipe::new(
            "Writing".to_string(),
            "Marketing".to_string(),
            "ContentMarketing".to_string(),
            1.2,
        ),
        Recipe::new(
            "Sales".to_string(),
            "Marketing".to_string(),
            "BusinessDevelopment".to_string(),
            1.3,
        ),
        // Finance/Analysis recipes
        Recipe::new(
            "Accounting".to_string(),
            "DataAnalysis".to_string(),
            "FinancialAnalysis".to_string(),
            1.4,
        ),
        Recipe::new(
            "LegalAdvice".to_string(),
            "Accounting".to_string(),
            "TaxConsulting".to_string(),
            1.5,
        ),
        // Service/Trade recipes
        Recipe::new(
            "Plumbing".to_string(),
            "Electrician".to_string(),
            "HomeRepair".to_string(),
            1.2,
        ),
        Recipe::new(
            "Chef".to_string(),
            "Marketing".to_string(),
            "RestaurantManagement".to_string(),
            1.3,
        ),
        Recipe::new(
            "FitnessTraining".to_string(),
            "Healthcare".to_string(),
            "PhysicalTherapy".to_string(),
            1.4,
        ),
        // Creative recipes
        Recipe::new(
            "MusicProduction".to_string(),
            "Marketing".to_string(),
            "MusicBusiness".to_string(),
            1.3,
        ),
        Recipe::new(
            "Writing".to_string(),
            "Translation".to_string(),
            "TechnicalWriting".to_string(),
            1.2,
        ),
        Recipe::new(
            "Teaching".to_string(),
            "DataAnalysis".to_string(),
            "EducationalResearch".to_string(),
            1.3,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_new() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        assert_eq!(recipe.input_skill_1, "Programming");
        assert_eq!(recipe.input_skill_2, "DataAnalysis");
        assert_eq!(recipe.output_skill, "MachineLearning");
        assert_eq!(recipe.cost_multiplier, 1.5);
    }

    #[test]
    fn test_recipe_can_craft_both_skills_available() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        let available_skills =
            vec!["Programming".to_string(), "DataAnalysis".to_string(), "Marketing".to_string()];

        assert!(recipe.can_craft(&available_skills));
    }

    #[test]
    fn test_recipe_can_craft_only_one_skill_available() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        let available_skills = vec!["Programming".to_string(), "Marketing".to_string()];

        assert!(!recipe.can_craft(&available_skills));
    }

    #[test]
    fn test_recipe_can_craft_no_skills_available() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        let available_skills = vec!["Marketing".to_string(), "Sales".to_string()];

        assert!(!recipe.can_craft(&available_skills));
    }

    #[test]
    fn test_recipe_can_craft_reverse_order() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        // Skills in reverse order should still work
        let available_skills = vec!["DataAnalysis".to_string(), "Programming".to_string()];

        assert!(recipe.can_craft(&available_skills));
    }

    #[test]
    fn test_recipe_calculate_cost() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        let cost = recipe.calculate_cost(10.0, 15.0);
        assert_eq!(cost, (10.0 + 15.0) * 1.5); // (10 + 15) * 1.5 = 37.5
    }

    #[test]
    fn test_recipe_calculate_cost_with_different_multiplier() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            2.0,
        );

        let cost = recipe.calculate_cost(20.0, 30.0);
        assert_eq!(cost, (20.0 + 30.0) * 2.0); // (20 + 30) * 2.0 = 100.0
    }

    #[test]
    fn test_generate_default_recipes_not_empty() {
        let recipes = generate_default_recipes();
        assert!(!recipes.is_empty());
        assert!(recipes.len() >= 10);
    }

    #[test]
    fn test_generate_default_recipes_has_expected_recipes() {
        let recipes = generate_default_recipes();

        // Check for some expected recipes
        let has_ml_recipe = recipes.iter().any(|r| r.output_skill == "MachineLearning");
        let has_digital_marketing = recipes.iter().any(|r| r.output_skill == "DigitalMarketing");

        assert!(has_ml_recipe);
        assert!(has_digital_marketing);
    }

    #[test]
    fn test_recipe_cost_multipliers_are_positive() {
        let recipes = generate_default_recipes();

        for recipe in recipes {
            assert!(recipe.cost_multiplier > 0.0);
            assert!(recipe.cost_multiplier <= 2.0); // Reasonable range
        }
    }

    #[test]
    fn test_recipe_clone() {
        let recipe = Recipe::new(
            "Programming".to_string(),
            "DataAnalysis".to_string(),
            "MachineLearning".to_string(),
            1.5,
        );

        let cloned = recipe.clone();
        assert_eq!(recipe, cloned);
    }
}
