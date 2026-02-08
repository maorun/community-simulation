// Example demonstrating the Causal Analysis Framework
//
// This example compares two scenarios:
// - Treatment: 20% savings rate
// - Control: No savings (0%)
//
// It runs multiple simulations with different random seeds for each scenario,
// then performs statistical analysis to determine if the savings rate has
// a significant causal effect on economic outcomes.

use community_simulation::{
    CausalAnalysisConfig, CausalAnalysisResult, SimulationConfig, SimulationEngine,
};

fn main() {
    println!("===========================================");
    println!("Causal Analysis Framework Example");
    println!("===========================================\n");

    // Configure treatment group: 20% savings rate
    println!("🔬 Running treatment group (20% savings rate)...");
    let treatment_config = SimulationConfig {
        max_steps: 100,
        entity_count: 20,
        savings_rate: 0.2,
        ..Default::default()
    };

    let mut treatment_results = vec![];
    for seed in 0..5 {
        let mut config = treatment_config.clone();
        config.seed = seed;
        let mut engine = SimulationEngine::new(config);
        treatment_results.push(engine.run());
    }
    println!("✓ Completed {} treatment runs\n", treatment_results.len());

    // Configure control group: No savings
    println!("🔬 Running control group (no savings)...");
    let control_config = SimulationConfig {
        max_steps: 100,
        entity_count: 20,
        savings_rate: 0.0,
        ..Default::default()
    };

    let mut control_results = vec![];
    for seed in 0..5 {
        let mut config = control_config.clone();
        config.seed = seed;
        let mut engine = SimulationEngine::new(config);
        control_results.push(engine.run());
    }
    println!("✓ Completed {} control runs\n", control_results.len());

    // Perform causal analysis
    println!("📊 Performing statistical causal analysis...\n");
    let config = CausalAnalysisConfig {
        treatment_name: "20% Savings Rate".to_string(),
        control_name: "No Savings".to_string(),
        confidence_level: 0.95,
        bootstrap_samples: 1000,
    };

    match CausalAnalysisResult::analyze(&treatment_results, &control_results, config) {
        Ok(analysis) => {
            analysis.print_summary();

            // Save to file
            let output_path = "/tmp/causal_analysis_result.json";
            if let Err(e) = analysis.save_to_file(output_path) {
                eprintln!("\n❌ Error saving results: {}", e);
            } else {
                println!("\n💾 Results saved to {}", output_path);
            }
        },
        Err(e) => {
            eprintln!("❌ Error performing causal analysis: {}", e);
            std::process::exit(1);
        },
    }

    println!("\n===========================================");
    println!("Example completed successfully!");
    println!("===========================================");
}
