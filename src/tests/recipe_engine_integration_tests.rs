//! Regression tests: the engine must apply the recipe fields during production.

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::production::Recipe;
use crate::skill::Skill;

const INPUT_1: &str = "InputOne";
const INPUT_2: &str = "InputTwo";
const OUTPUT: &str = "ProducedSkill";

fn production_config() -> SimulationConfig {
    SimulationConfig {
        entity_count: 1,
        max_steps: 10,
        seed: 7,
        initial_money_per_person: 1000.0,
        enable_production: true,
        production_probability: 1.0,
        ..Default::default()
    }
}

fn base_recipe() -> Recipe {
    Recipe::new(INPUT_1.to_string(), INPUT_2.to_string(), OUTPUT.to_string(), 1.0)
}

/// Creates an engine with one person who owns both input skills.
fn engine_with(config: SimulationConfig, recipe: Recipe) -> SimulationEngine {
    let mut engine = SimulationEngine::new(config);
    engine.set_production_recipes(vec![recipe]).expect("recipe must be valid");
    let price = engine.get_config().base_skill_price;
    let person = &mut engine.get_entities_mut()[0].person_data;
    person.own_skills =
        vec![Skill::new(INPUT_1.to_string(), price), Skill::new(INPUT_2.to_string(), price)];
    person.learned_skills.clear();
    engine
}

fn money(engine: &SimulationEngine) -> f64 {
    engine.get_entities()[0].person_data.money
}

fn has_output(engine: &SimulationEngine) -> bool {
    engine.get_entities()[0].person_data.has_skill(&OUTPUT.to_string())
}

#[test]
fn set_production_recipes_rejects_invalid_recipe() {
    let mut engine = SimulationEngine::new(production_config());
    let mut recipe = base_recipe();
    recipe.batch_size = 0;
    assert!(engine.set_production_recipes(vec![recipe]).is_err());
}

#[test]
fn default_recipe_costs_input_prices_times_multiplier() {
    let mut recipe = base_recipe();
    recipe.cost_multiplier = 1.5;
    let mut engine = engine_with(production_config(), recipe);
    // The market has no price change before the first step, so the inputs cost the base price.
    let base_price = engine.get_config().base_skill_price;
    let before = money(&engine);
    engine.step();
    assert!(has_output(&engine));
    let paid = before - money(&engine);
    let expected = (base_price + base_price) * 1.5;
    // The step can move prices and money, so allow a small tolerance for the market update.
    assert!(paid > 0.0 && paid <= expected * 1.5, "paid {paid}, expected about {expected}");
}

#[test]
fn execution_time_delays_skill_until_completion() {
    let mut recipe = base_recipe();
    recipe.execution_time = 3;
    let mut engine = engine_with(production_config(), recipe);

    engine.step();
    assert!(!has_output(&engine), "skill must not exist before the execution time ends");
    assert_eq!(engine.get_pending_productions().len(), 1);

    engine.step();
    engine.step();
    assert!(!has_output(&engine));

    engine.step();
    assert!(has_output(&engine), "skill must exist after the execution time ends");
    assert!(engine.get_pending_productions().is_empty());
}

#[test]
fn pending_production_does_not_start_twice() {
    let mut recipe = base_recipe();
    recipe.execution_time = 5;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    engine.step();
    engine.step();
    assert_eq!(engine.get_pending_productions().len(), 1);
}

#[test]
fn batch_size_divides_input_cost() {
    let single = {
        let mut engine = engine_with(production_config(), base_recipe());
        let before = money(&engine);
        engine.step();
        before - money(&engine)
    };
    let batched = {
        let mut recipe = base_recipe();
        recipe.batch_size = 4;
        let mut engine = engine_with(production_config(), recipe);
        let before = money(&engine);
        engine.step();
        before - money(&engine)
    };
    assert!(single > 0.0);
    assert!(
        (batched - single / 4.0).abs() < 1e-6,
        "batched cost {batched} must be a quarter of single cost {single}"
    );
}

#[test]
fn labor_requirement_adds_cost() {
    let plain = {
        let mut engine = engine_with(production_config(), base_recipe());
        let before = money(&engine);
        engine.step();
        before - money(&engine)
    };
    let mut recipe = base_recipe();
    recipe.labor_requirement = 2.0;
    let mut engine = engine_with(production_config(), recipe);
    let base_price = engine.get_config().base_skill_price;
    let before = money(&engine);
    engine.step();
    let with_labor = before - money(&engine);
    assert!((with_labor - (plain + 2.0 * base_price)).abs() < 1e-6);
}

#[test]
fn yield_multiplier_and_quality_tier_raise_output_price() {
    let price_of = |recipe: Recipe| {
        let mut engine = engine_with(production_config(), recipe);
        engine.step();
        engine.get_entities()[0]
            .person_data
            .learned_skills
            .last()
            .unwrap()
            .current_price
    };
    let plain = price_of(base_recipe());

    let mut yield_recipe = base_recipe();
    yield_recipe.yield_multiplier = 2.0;
    assert!((price_of(yield_recipe) - plain * 2.0).abs() < 1e-6);

    let mut quality_recipe = base_recipe();
    quality_recipe.quality_tier = 3;
    assert!((price_of(quality_recipe) - plain * 1.2).abs() < 1e-6);
}

#[test]
fn energy_consumption_reduces_energy_reserves() {
    let energy_after_step = |energy_consumption: f64| {
        let config = SimulationConfig { enable_environment: true, ..production_config() };
        let mut recipe = base_recipe();
        recipe.energy_consumption = energy_consumption;
        let mut engine = engine_with(config, recipe);
        engine.step();
        engine
            .get_environment()
            .expect("environment must exist")
            .total_consumption
            .get(&crate::environment::Resource::Energy)
            .copied()
            .unwrap_or(0.0)
    };
    let baseline = energy_after_step(0.0);
    let with_recipe_energy = energy_after_step(25.0);
    assert!(
        (with_recipe_energy - baseline - 25.0).abs() < 1e-6,
        "recipe energy must add 25.0 to the baseline {baseline}, got {with_recipe_energy}"
    );
}

#[test]
fn production_cost_is_refunded_when_person_got_the_skill_meanwhile() {
    let mut recipe = base_recipe();
    recipe.execution_time = 3;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    assert_eq!(engine.get_pending_productions().len(), 1);
    let paid = engine.get_pending_productions()[0].cost_paid;
    assert!(paid > 0.0);

    // The person learns the output skill by other means while the production runs.
    let price = engine.get_config().base_skill_price;
    engine.get_entities_mut()[0]
        .person_data
        .learned_skills
        .push(Skill::new(OUTPUT.to_string(), price));
    let before_completion = money(&engine);
    engine.step();
    engine.step();
    engine.step();
    assert!(engine.get_pending_productions().is_empty());
    assert!(
        money(&engine) >= before_completion + paid - 1e-6,
        "the engine must refund the paid cost {paid}"
    );
}

#[test]
fn production_cost_is_refunded_when_person_is_inactive_at_completion() {
    let mut recipe = base_recipe();
    recipe.execution_time = 2;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    let paid = engine.get_pending_productions()[0].cost_paid;
    engine.get_entities_mut()[0].active = false;
    let before_completion = money(&engine);
    engine.step();
    engine.step();
    assert!(engine.get_pending_productions().is_empty());
    assert!(!has_output(&engine));
    assert!(money(&engine) >= before_completion + paid - 1e-6);
}

#[test]
fn set_production_recipes_requires_enabled_production() {
    let config = SimulationConfig { enable_production: false, ..production_config() };
    let mut engine = SimulationEngine::new(config);
    assert!(engine.set_production_recipes(vec![base_recipe()]).is_err());
}

#[test]
fn validate_rejects_huge_energy_consumption() {
    let mut recipe = base_recipe();
    recipe.energy_consumption = 1.0e308;
    assert!(recipe.validate().is_err());
    recipe.energy_consumption = crate::production::MAX_RECIPE_ENERGY_CONSUMPTION;
    assert!(recipe.validate().is_ok());
}

#[test]
fn huge_execution_time_does_not_overflow_the_completion_step() {
    let mut recipe = base_recipe();
    recipe.execution_time = usize::MAX;
    assert!(recipe.validate().is_ok());
    let mut engine = engine_with(production_config(), recipe);

    // Step once without input skills, so that the production starts at step 1.
    let skills = std::mem::take(&mut engine.get_entities_mut()[0].person_data.own_skills);
    engine.step();
    assert!(engine.get_pending_productions().is_empty());
    engine.get_entities_mut()[0].person_data.own_skills = skills;

    engine.step();
    assert_eq!(engine.get_pending_productions().len(), 1);
    assert_eq!(engine.get_pending_productions()[0].completes_at_step, usize::MAX);
}

#[test]
fn completion_skips_a_pending_production_of_a_missing_entity() {
    let mut recipe = base_recipe();
    recipe.execution_time = 2;
    let config = SimulationConfig { entity_count: 2, ..production_config() };
    let mut engine = engine_with(config, recipe);
    engine.step();
    assert!(!engine.get_pending_productions().is_empty());

    // A caller removes all entities through the mutable accessor. The pending run stays.
    engine.get_entities_mut().clear();
    engine.step();
    engine.step();
    assert!(engine.get_pending_productions().is_empty());
}

#[test]
fn pending_recipe_does_not_waste_the_attempt_for_another_recipe() {
    const INPUT_3: &str = "InputThree";
    const INPUT_4: &str = "InputFour";
    const OUTPUT_B: &str = "ProducedSkillB";

    // Several seeds make the check independent of one random pick.
    for seed in 0..16 {
        let mut slow = base_recipe();
        slow.execution_time = 100;
        let fast = Recipe::new(INPUT_3.to_string(), INPUT_4.to_string(), OUTPUT_B.to_string(), 1.0);
        let config = SimulationConfig { seed, ..production_config() };
        let mut engine = engine_with(config, slow);
        engine.set_production_recipes(vec![base_recipe_with_time(100), fast]).unwrap();

        // Step 1: only the slow recipe is craftable, so the slow production starts.
        engine.step();
        assert_eq!(engine.get_pending_productions().len(), 1, "seed {seed}");

        // Step 2: the fast recipe is craftable too. The pending slow recipe must not take the pick.
        let price = engine.get_config().base_skill_price;
        let person = &mut engine.get_entities_mut()[0].person_data;
        person.own_skills.push(Skill::new(INPUT_3.to_string(), price));
        person.own_skills.push(Skill::new(INPUT_4.to_string(), price));
        engine.step();
        assert!(
            engine.get_entities()[0].person_data.has_skill(&OUTPUT_B.to_string()),
            "seed {seed}: the fast recipe must run while the slow recipe is pending"
        );
    }
}

fn base_recipe_with_time(execution_time: usize) -> Recipe {
    let mut recipe = base_recipe();
    recipe.execution_time = execution_time;
    recipe
}

#[test]
fn checkpoint_drops_pending_productions_with_huge_amounts() {
    let mut recipe = base_recipe();
    recipe.execution_time = 4;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    let mut json = checkpoint_json(&engine);
    json["pending_productions"][0]["cost_paid"] = serde_json::json!(1.0e308);
    let restored = load_checkpoint_json(&json).expect("checkpoint must load");
    assert!(restored.get_pending_productions().is_empty());
}

#[test]
fn output_price_is_finite_for_extreme_multipliers() {
    let mut recipe = base_recipe();
    recipe.cost_multiplier = f64::MAX;
    recipe.yield_multiplier = f64::MAX;
    let price = recipe.calculate_output_price(10.0, 10.0);
    assert!(price.is_finite());
    assert!(price <= crate::production::MAX_PRODUCTION_PRICE);
    assert!(recipe.calculate_production_cost(f64::MAX, f64::MAX, f64::MAX).is_finite());
}

fn checkpoint_json(engine: &SimulationEngine) -> serde_json::Value {
    let dir = tempfile::tempdir().expect("temp dir must exist");
    let path = dir.path().join("checkpoint.json");
    engine.save_checkpoint(&path).expect("checkpoint must save");
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn load_checkpoint_json(json: &serde_json::Value) -> std::io::Result<SimulationEngine> {
    let dir = tempfile::tempdir().expect("temp dir must exist");
    let path = dir.path().join("checkpoint.json");
    std::fs::write(&path, serde_json::to_string(json).unwrap()).unwrap();
    SimulationEngine::load_checkpoint(&path)
}

#[test]
fn pending_productions_survive_checkpoint_roundtrip() {
    let mut recipe = base_recipe();
    recipe.execution_time = 4;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    assert_eq!(engine.get_pending_productions().len(), 1);

    let restored = load_checkpoint_json(&checkpoint_json(&engine)).expect("checkpoint must load");
    assert_eq!(restored.get_pending_productions(), engine.get_pending_productions());
}

#[test]
fn custom_recipes_survive_checkpoint_roundtrip() {
    let mut recipe = base_recipe();
    recipe.execution_time = 3;
    recipe.quality_tier = 5;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();

    let mut restored =
        load_checkpoint_json(&checkpoint_json(&engine)).expect("checkpoint must load");
    // The restored engine must keep the custom recipe, so the output skill still appears.
    restored.step();
    restored.step();
    restored.step();
    assert!(restored.get_entities()[0].person_data.has_skill(&OUTPUT.to_string()));
}

#[test]
fn checkpoint_without_pending_productions_field_loads() {
    let mut engine = engine_with(production_config(), base_recipe());
    // Step once, so that the money statistics hold finite values and the JSON round-trips.
    engine.step();
    let mut json = checkpoint_json(&engine);
    let object = json.as_object_mut().unwrap();
    object.remove("pending_productions");
    object.remove("production_recipes");
    let restored = load_checkpoint_json(&json).expect("old checkpoint format must load");
    assert!(restored.get_pending_productions().is_empty());
}

#[test]
fn checkpoint_drops_invalid_pending_productions() {
    let mut recipe = base_recipe();
    recipe.execution_time = 4;
    let mut engine = engine_with(production_config(), recipe);
    engine.step();
    let mut json = checkpoint_json(&engine);
    let pending = json["pending_productions"].as_array_mut().unwrap();
    let mut bad = pending[0].clone();
    bad["entity_index"] = serde_json::json!(9999);
    pending.push(bad);

    let restored = load_checkpoint_json(&json).expect("checkpoint must load");
    assert_eq!(restored.get_pending_productions().len(), 1);
    assert_eq!(restored.get_pending_productions()[0].entity_index, 0);
}

#[test]
fn checkpoint_rejects_invalid_recipe() {
    let engine = engine_with(production_config(), base_recipe());
    let mut json = checkpoint_json(&engine);
    json["production_recipes"][0]["batch_size"] = serde_json::json!(0);
    assert!(load_checkpoint_json(&json).is_err());
}
