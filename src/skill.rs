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

/// Represents a certification for a skill.
///
/// Certifications validate the quality and authenticity of a skill, allowing holders to
/// charge higher prices. Certifications can expire and must be renewed to maintain their benefits.
///
/// # Examples
///
/// ```
/// use simulation_framework::skill::Certification;
///
/// let cert = Certification::new("CentralAuthority".to_string(), 2, Some(100));
/// assert_eq!(cert.issuer, "CentralAuthority");
/// assert_eq!(cert.level, 2);
/// assert!(!cert.is_expired(50));
/// assert!(cert.is_expired(150));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Certification {
    /// The entity or authority that issued this certification.
    pub issuer: String,

    /// The level or tier of the certification (1-5, higher is better).
    /// Higher levels typically command higher price premiums.
    pub level: u8,

    /// The simulation step at which this certification expires.
    /// If None, the certification never expires.
    pub expiration_step: Option<usize>,
}

impl Certification {
    /// Creates a new certification with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `issuer` - Name of the certifying authority
    /// * `level` - Certification level (1-5)
    /// * `expiration_step` - Optional step at which the certification expires
    pub fn new(issuer: String, level: u8, expiration_step: Option<usize>) -> Self {
        Self {
            issuer,
            level: level.clamp(1, 5), // Ensure level is in valid range
            expiration_step,
        }
    }

    /// Checks if the certification has expired at the given simulation step.
    ///
    /// # Arguments
    ///
    /// * `current_step` - The current simulation step to check against
    ///
    /// # Returns
    ///
    /// `true` if the certification has expired, `false` otherwise
    pub fn is_expired(&self, current_step: usize) -> bool {
        self.expiration_step.map(|exp| current_step >= exp).unwrap_or(false)
    }

    /// Calculates the price multiplier for this certification.
    ///
    /// The multiplier increases with certification level:
    /// - Level 1: +5% price
    /// - Level 2: +10% price
    /// - Level 3: +15% price
    /// - Level 4: +20% price
    /// - Level 5: +25% price
    ///
    /// # Returns
    ///
    /// A multiplier value (e.g., 1.10 for 10% price increase)
    pub fn price_multiplier(&self) -> f64 {
        1.0 + (self.level as f64 * 0.05)
    }
}

/// Represents a tradeable skill in the economy.
///
/// Each person in the simulation has one skill they can provide to others,
/// and needs various skills from other people. Skills have dynamic prices
/// that adjust based on supply, demand, and market conditions.
///
/// Skills can be certified to increase their value and trustworthiness in the market.
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
    /// by reducing their effective cost. For example, an efficiency of 2.0 means
    /// the skill provides twice as much value, effectively halving the price paid
    /// by buyers (from 10.0 to 5.0).
    pub efficiency_multiplier: f64,

    /// Optional certification for this skill.
    ///
    /// When present, certifications increase trust in the skill and allow the provider
    /// to charge higher prices. Certifications can expire and need renewal.
    pub certification: Option<Certification>,
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
        Self { id, current_price: base_price, efficiency_multiplier: 1.0, certification: None }
    }

    /// Returns the effective price for this skill, taking into account certification.
    ///
    /// If the skill has a valid (non-expired) certification, the price is increased
    /// based on the certification level.
    ///
    /// # Arguments
    ///
    /// * `current_step` - The current simulation step (used to check certification expiration)
    ///
    /// # Returns
    ///
    /// The effective price including any certification premium
    pub fn effective_price(&self, current_step: usize) -> f64 {
        if let Some(cert) = &self.certification {
            if !cert.is_expired(current_step) {
                return self.current_price * cert.price_multiplier();
            }
        }
        self.current_price
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
    fn test_certification_new() {
        let cert = Certification::new("CentralAuthority".to_string(), 3, Some(100));
        assert_eq!(cert.issuer, "CentralAuthority");
        assert_eq!(cert.level, 3);
        assert_eq!(cert.expiration_step, Some(100));
    }

    #[test]
    fn test_certification_level_clamping() {
        let cert_too_high = Certification::new("Authority".to_string(), 10, None);
        assert_eq!(cert_too_high.level, 5); // Should be clamped to max of 5

        let cert_too_low = Certification::new("Authority".to_string(), 0, None);
        assert_eq!(cert_too_low.level, 1); // Should be clamped to min of 1
    }

    #[test]
    fn test_certification_expiration() {
        let cert = Certification::new("Authority".to_string(), 2, Some(100));
        assert!(!cert.is_expired(50)); // Not expired at step 50
        assert!(!cert.is_expired(99)); // Not expired at step 99
        assert!(cert.is_expired(100)); // Expired at expiration step
        assert!(cert.is_expired(150)); // Expired after expiration step

        // Test never-expiring certification
        let perm_cert = Certification::new("Authority".to_string(), 2, None);
        assert!(!perm_cert.is_expired(0));
        assert!(!perm_cert.is_expired(1000));
        assert!(!perm_cert.is_expired(usize::MAX));
    }

    #[test]
    fn test_certification_price_multiplier() {
        let cert1 = Certification::new("Authority".to_string(), 1, None);
        assert!((cert1.price_multiplier() - 1.05).abs() < 0.001);

        let cert2 = Certification::new("Authority".to_string(), 2, None);
        assert!((cert2.price_multiplier() - 1.10).abs() < 0.001);

        let cert5 = Certification::new("Authority".to_string(), 5, None);
        assert!((cert5.price_multiplier() - 1.25).abs() < 0.001);
    }

    #[test]
    fn test_skill_new() {
        let skill = Skill::new("Programming".to_string(), 50.0);

        assert_eq!(skill.id, "Programming");
        assert_eq!(skill.current_price, 50.0);
        assert!(skill.certification.is_none());
    }

    #[test]
    fn test_skill_effective_price_without_certification() {
        let skill = Skill::new("Programming".to_string(), 50.0);
        assert_eq!(skill.effective_price(0), 50.0);
        assert_eq!(skill.effective_price(100), 50.0);
    }

    #[test]
    fn test_skill_effective_price_with_valid_certification() {
        let mut skill = Skill::new("Programming".to_string(), 50.0);
        skill.certification = Some(Certification::new("Authority".to_string(), 2, Some(100)));

        // Before expiration, should have 10% premium (level 2)
        assert!((skill.effective_price(50) - 55.0).abs() < 0.001);
        assert!((skill.effective_price(99) - 55.0).abs() < 0.001);
    }

    #[test]
    fn test_skill_effective_price_with_expired_certification() {
        let mut skill = Skill::new("Programming".to_string(), 50.0);
        skill.certification = Some(Certification::new("Authority".to_string(), 2, Some(100)));

        // After expiration, should return base price
        assert_eq!(skill.effective_price(100), 50.0);
        assert_eq!(skill.effective_price(200), 50.0);
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
