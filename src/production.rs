//! Production system for combining skills to create new skills.
//!
//! This module implements a simple production/crafting system where persons can combine
//! two skills they own to produce a new, more valuable skill. This simulates supply chains,
//! skill composition, and economic specialization.

use crate::skill::SkillId;
use serde::{Deserialize, Serialize};

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
}

impl Recipe {
    /// Creates a new production recipe.
    ///
    /// # Arguments
    /// * `input_skill_1` - ID of the first input skill
    /// * `input_skill_2` - ID of the second input skill
    /// * `output_skill` - ID of the skill produced by this recipe
    /// * `cost_multiplier` - Cost multiplier (typically 1.0-2.0)
    ///
    /// # Examples
    /// ```
    /// use simulation_framework::production::Recipe;
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
        Recipe { input_skill_1, input_skill_2, output_skill, cost_multiplier }
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
/// use simulation_framework::production::generate_default_recipes;
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
