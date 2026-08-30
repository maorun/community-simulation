#[cfg(test)]
mod tests {
    use crate::skill::{Certification, Skill};
    use crate::tests::test_helpers::test_config;
    use crate::SimulationEngine;

    #[test]
    fn test_information_asymmetry_config_validation() {
        let mut config = test_config().build();
        config.enable_information_asymmetry = true;
        config.inspection_cost = 2.0;
        config.certification_cost = 10.0;

        assert!(config.validate().is_ok());

        // Negative inspection_cost is invalid
        config.inspection_cost = -1.0;
        assert!(config.validate().is_err());

        // Negative certification_cost is invalid
        config.inspection_cost = 2.0;
        config.certification_cost = -5.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_skill_qualities_and_inspection() {
        let mut skill = Skill::with_qualities("Programming".to_string(), 50.0, 4.5, 3.0);

        assert_eq!(skill.true_quality, 4.5);
        assert_eq!(skill.perceived_quality, 3.0);
        assert_eq!(skill.effective_perceived_quality(0), 3.0);

        // Certifying the skill signals true quality
        skill.certification = Some(Certification::new("Authority".to_string(), 3, Some(100)));
        assert_eq!(skill.effective_perceived_quality(50), 4.5);

        // Inspection reveals true quality
        let mut uncertified_skill = Skill::with_qualities("Design".to_string(), 30.0, 4.2, 2.5);
        assert_eq!(uncertified_skill.inspect(), 4.2);
        assert_eq!(uncertified_skill.perceived_quality, 4.2);
    }

    #[test]
    fn test_information_asymmetry_simulation_run() {
        let mut config = test_config().max_steps(30).entity_count(10).build();
        config.enable_information_asymmetry = true;
        config.inspection_cost = 1.0;
        config.certification_cost = 5.0;

        let mut engine = SimulationEngine::new(config);

        // Verify skills have true_quality initialized
        let skills = engine.get_market().get_all_skill_prices();
        assert!(!skills.is_empty());

        for (skill_id, _) in &skills {
            if let Some(market_skill) = engine.get_market().skills.get(skill_id) {
                assert!(market_skill.true_quality >= 1.0 && market_skill.true_quality <= 5.0);
            }
        }

        let result = engine.run();
        assert_eq!(result.total_steps, 30);
    }

    #[test]
    fn test_information_asymmetry_certification_signaling() {
        let mut config = test_config().max_steps(20).entity_count(10).build();
        config.enable_information_asymmetry = true;
        config.enable_certification = true;
        config.certification_probability = 0.5;
        config.certification_cost = 5.0;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 20);
        if let Some(cert_stats) = result.certification_statistics {
            let _ = cert_stats.total_issued;
        }
    }
}
