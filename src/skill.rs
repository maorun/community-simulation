//! Skill definitions and generation for the economic simulation.
//!
//! This module defines the [`Skill`] type, which represents a tradeable skill in the economy.
//! Each skill has a unique identifier and a dynamically adjusting price based on market conditions.

use serde::{Deserialize, Serialize};

/// Type alias for skill identifiers.
///
/// Skills are identified by their name as a string, making them human-readable
/// and easy to debug. For example: "Programming", "Accounting", "Writing".
pub type SkillId = String;

/// Represents a tradeable skill in the economy.
///
/// Each person in the simulation has one skill they can provide to others,
/// and needs various skills from other people. Skills have dynamic prices
/// that adjust based on supply, demand, and market conditions.
///
/// # Examples
///
/// ```
/// use simulation_framework::Skill;
///
/// let skill = Skill::new("Programming".to_string(), 50.0);
/// assert_eq!(skill.id, "Programming");
/// assert_eq!(skill.current_price, 50.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    /// Unique identifier for the skill, also serves as the skill's name.
    pub id: SkillId,

    /// Current market price for this skill.
    ///
    /// This price is dynamically adjusted by the market based on supply and demand.
    /// The market ensures prices stay within configured min/max bounds.
    pub current_price: f64,

    /// Efficiency multiplier representing technological progress.
    ///
    /// This multiplier starts at 1.0 and increases over time based on the configured
    /// technology growth rate. Higher efficiency makes skills effectively cheaper
    /// by reducing their effective cost. For example, an efficiency of 1.1 means
    /// the skill provides 10% more value, effectively reducing the price by ~9%.
    pub efficiency_multiplier: f64,
    // Note: Supply is implicitly 1 per person offering it. Demand is calculated each step.
    // Price management is handled by the Market.
}

impl Skill {
    /// Creates a new skill with the given identifier and base price.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the skill (typically a descriptive name)
    /// * `base_price` - Initial price for the skill
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::Skill;
    ///
    /// let programming = Skill::new("Programming".to_string(), 50.0);
    /// let accounting = Skill::new("Accounting".to_string(), 45.0);
    /// ```
    pub fn new(id: SkillId, base_price: f64) -> Self {
        Self {
            id,
            current_price: base_price,
            efficiency_multiplier: 1.0,
        }
    }
}

/// Generates a list of unique skills for the simulation.
///
/// This function creates the specified number of unique skills, each with the same base price.
/// Skills are given predefined names (e.g., "Programming", "Accounting") when available,
/// and fall back to generated names (e.g., "Skill0", "Skill1") when the predefined list
/// is exhausted.
///
/// # Arguments
///
/// * `count` - Number of unique skills to generate
/// * `base_price` - Initial price to assign to all skills
///
/// # Returns
///
/// A vector of `Skill` instances, each with a unique identifier
///
/// # Examples
///
/// ```
/// use simulation_framework::skill::generate_unique_skills;
///
/// // Generate 5 skills with base price of 10.0
/// let skills = generate_unique_skills(5, 10.0);
/// assert_eq!(skills.len(), 5);
/// assert_eq!(skills[0].id, "Programming");
/// assert_eq!(skills[0].current_price, 10.0);
/// ```
pub fn generate_unique_skills(count: usize, base_price: f64) -> Vec<Skill> {
    let skill_names = [
        "Programming",
        "Accounting",
        "Writing",
        "GraphicDesign",
        "DataAnalysis",
        "Marketing",
        "Sales",
        "Engineering",
        "Consulting",
        "Teaching",
        "Plumbing",
        "Electrician",
        "Carpentry",
        "Chef",
        "Gardening",
        "Translation",
        "LegalAdvice",
        "Healthcare",
        "FitnessTraining",
        "MusicProduction",
        // Add more unique skill names to ensure we can cover `count`
        // For 100 persons, we need 100 unique skills.
        // This list is just an example, for a real scenario we might need a longer list or procedural generation.
    ];

    let mut skills = Vec::new();
    for i in 0..count {
        let name = format!("Skill{}", i); // Default naming if predefined list is short
        let skill_name = skill_names.get(i).unwrap_or(&name.as_str()).to_string();
        skills.push(Skill::new(skill_name, base_price));
    }
    skills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_new() {
        let skill = Skill::new("Programming".to_string(), 50.0);

        assert_eq!(skill.id, "Programming");
        assert_eq!(skill.current_price, 50.0);
    }

    #[test]
    fn test_skill_clone() {
        let skill = Skill::new("Writing".to_string(), 30.0);
        let cloned_skill = skill.clone();

        assert_eq!(skill.id, cloned_skill.id);
        assert_eq!(skill.current_price, cloned_skill.current_price);
    }

    #[test]
    fn test_generate_unique_skills_count() {
        let skills = generate_unique_skills(5, 10.0);

        assert_eq!(skills.len(), 5);
    }

    #[test]
    fn test_generate_unique_skills_base_price() {
        let base_price = 25.0;
        let skills = generate_unique_skills(3, base_price);

        for skill in skills {
            assert_eq!(skill.current_price, base_price);
        }
    }

    #[test]
    fn test_generate_unique_skills_unique_ids() {
        let skills = generate_unique_skills(10, 10.0);
        let mut ids = std::collections::HashSet::new();

        for skill in &skills {
            ids.insert(&skill.id);
        }

        // All IDs should be unique
        assert_eq!(ids.len(), skills.len());
    }

    #[test]
    fn test_generate_unique_skills_predefined_names() {
        let skills = generate_unique_skills(5, 10.0);

        // First few should use predefined names
        assert_eq!(skills[0].id, "Programming");
        assert_eq!(skills[1].id, "Accounting");
        assert_eq!(skills[2].id, "Writing");
    }

    #[test]
    fn test_generate_unique_skills_fallback_names() {
        // Test with count exceeding predefined list
        let skills = generate_unique_skills(25, 10.0);

        // Should have 25 skills even if predefined list is shorter
        assert_eq!(skills.len(), 25);

        // Later skills should use Skill{N} format
        assert!(skills[24].id.starts_with("Skill") || skills[24].id == "MusicProduction");
    }
}
