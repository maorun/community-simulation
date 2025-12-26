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
