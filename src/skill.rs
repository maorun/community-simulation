use serde::{Deserialize, Serialize};

pub type SkillId = String; // Using String for skill names as IDs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // Removed Eq and Hash
pub struct Skill {
    pub id: SkillId, // Name of the skill, also used as a unique identifier
    pub current_price: f64,
    // Supply is implicitly 1 per person offering it. Demand is calculated each step.
    // We can store a demand_count or supply_demand_ratio if needed for pricing.
    // For now, price will be managed by the Market.
}

impl Skill {
    pub fn new(id: SkillId, base_price: f64) -> Self {
        Self {
            id,
            current_price: base_price,
        }
    }
}

// Example function to generate a predefined list of unique skills
// This could be expanded or made more dynamic.
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
