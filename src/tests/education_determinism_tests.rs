// Regression tests for deterministic skill choice in the education phase

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::result::SimulationResult;
use crate::tests::test_helpers::test_config;

fn education_config() -> SimulationConfig {
    let mut config = test_config().entity_count(40).max_steps(60).seed(9001).build();
    config.enable_education = true;
    config.learning_probability = 0.5;
    config.learning_cost_multiplier = 0.2;
    config
}

fn learned_skills(result: &SimulationResult) -> Vec<Vec<String>> {
    result
        .final_persons_data
        .iter()
        .map(|e| e.person_data.learned_skills.iter().map(|s| s.id.clone()).collect())
        .collect()
}

#[test]
fn test_education_skill_choice_is_deterministic_per_seed() {
    let first = SimulationEngine::new(education_config()).run();
    let total_learned: usize = learned_skills(&first).iter().map(Vec::len).sum();
    assert!(total_learned > 0, "the scenario must learn at least one skill");

    // HashMap iteration order changes between map instances, so repeat the run.
    for _ in 0..5 {
        let next = SimulationEngine::new(education_config()).run();
        assert_eq!(learned_skills(&first), learned_skills(&next));
        let money = |r: &SimulationResult| -> Vec<f64> {
            r.final_persons_data.iter().map(|e| e.person_data.money).collect()
        };
        assert_eq!(money(&first), money(&next));
    }
}

#[test]
fn test_mentored_education_is_deterministic_per_seed() {
    let mentored = || {
        let mut config = education_config();
        config.enable_quality = true;
        config.enable_mentorship = true;
        config
    };
    let first = SimulationEngine::new(mentored()).run();
    let mentorship =
        first.mentorship_statistics.as_ref().expect("mentorship statistics must exist");
    assert!(mentorship.successful_mentored_learnings > 0, "the scenario must use mentors");

    for _ in 0..5 {
        let next = SimulationEngine::new(mentored()).run();
        assert_eq!(learned_skills(&first), learned_skills(&next));
        let next_mentorship = next.mentorship_statistics.as_ref().expect("mentorship statistics");
        assert_eq!(
            mentorship.successful_mentored_learnings,
            next_mentorship.successful_mentored_learnings
        );
        assert_eq!(mentorship.unique_mentors, next_mentorship.unique_mentors);
        assert_eq!(mentorship.total_cost_savings, next_mentorship.total_cost_savings);
    }
}
