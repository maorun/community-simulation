// Example demonstrating the Insurance System
//
// This example shows how the insurance system works in the simulation,
// demonstrating all three types of insurance coverage:
// - Crisis Insurance: Protection against economic shocks
// - Income Insurance: Safety net when trade income is low
// - Credit Insurance: Protection against loan defaults
//
// The example runs multiple scenarios to show insurance effectiveness.

use simulation_framework::{SimulationConfig, SimulationEngine};

fn main() {
    println!("===========================================");
    println!("Insurance System Demonstration");
    println!("===========================================\n");

    println!("This example demonstrates the three types of insurance:");
    println!("  1. Crisis Insurance - Protects against economic shocks");
    println!("  2. Income Insurance - Safety net for low income periods");
    println!("  3. Credit Insurance - Protection against loan defaults\n");

    // Scenario 1: Economy WITH insurance protection against crises
    println!("📊 Scenario 1: Economy WITH Insurance + Crisis Events");
    println!("-----------------------------------------------------");

    let base_config = SimulationConfig {
        max_steps: 200,
        entity_count: 30,
        seed: 42,

        // Enable crisis events
        enable_crisis_events: true,
        crisis_probability: 0.08, // 8% chance per step (high for demo)
        crisis_severity: 0.25,    // 25% severity

        // Enable loans
        enable_loans: true,
        loan_interest_rate: 0.02,  // 2% interest per step
        loan_repayment_period: 20, // 20 step repayment
        min_money_to_lend: 30.0,

        // Base parameters
        initial_money_per_person: 100.0,
        base_skill_price: 10.0,

        ..Default::default()
    };

    let insured_config = SimulationConfig {
        // Enable insurance system
        enable_insurance: true,
        insurance_premium_rate: 0.05, // 5% of coverage as premium
        insurance_duration: 100,      // Policies last 100 steps
        insurance_purchase_probability: 0.15, // 15% chance per step (high for demo)
        insurance_coverage_amount: 75.0, // Coverage of 75 units

        ..base_config.clone()
    };

    let mut insured_engine = SimulationEngine::new(insured_config);
    let insured_result = insured_engine.run();

    println!("\n✅ Results WITH Insurance:");
    println!("   Total Persons: {}", insured_result.active_persons);
    println!("   Final Average Wealth: ${:.2}", insured_result.money_statistics.average);
    println!("   Wealth Std Dev: ${:.2}", insured_result.money_statistics.std_dev);

    if let Some(ref insurance_stats) = insured_result.insurance_statistics {
        println!("\n   📋 Insurance Statistics:");
        println!("   - Total Policies Issued: {}", insurance_stats.total_policies_issued);
        println!("   - Active Policies: {}", insurance_stats.active_policies);
        println!("   - Total Claims Paid: {}", insurance_stats.total_claims_paid);
        println!(
            "   - Total Premiums Collected: ${:.2}",
            insurance_stats.total_premiums_collected
        );
        println!("   - Total Payouts Made: ${:.2}", insurance_stats.total_payouts_made);
        println!("   - Net Result (Premiums - Payouts): ${:.2}", insurance_stats.net_result);

        if insurance_stats.total_premiums_collected > 0.0 {
            println!(
                "   - Loss Ratio: {:.2}% (payouts/premiums)",
                insurance_stats.loss_ratio * 100.0
            );
        }
    }

    // Scenario 2: Same economy WITHOUT insurance for comparison
    println!("\n\n📊 Scenario 2: Economy WITHOUT Insurance (Comparison)");
    println!("--------------------------------------------------------");

    let uninsured_config = SimulationConfig {
        enable_insurance: false, // Only difference: no insurance
        ..base_config
    };

    let mut uninsured_engine = SimulationEngine::new(uninsured_config);
    let uninsured_result = uninsured_engine.run();

    println!("\n❌ Results WITHOUT Insurance:");
    println!("   Total Persons: {}", uninsured_result.active_persons);
    println!("   Final Average Wealth: ${:.2}", uninsured_result.money_statistics.average);
    println!("   Wealth Std Dev: ${:.2}", uninsured_result.money_statistics.std_dev);

    // Compare the two scenarios
    println!("\n\n📈 Comparative Analysis");
    println!("========================");

    let wealth_with_insurance = insured_result.money_statistics.average;
    let wealth_without_insurance = uninsured_result.money_statistics.average;
    let wealth_difference = wealth_with_insurance - wealth_without_insurance;
    let wealth_difference_pct = (wealth_difference / wealth_without_insurance) * 100.0;

    println!("Average Wealth Comparison:");
    println!("   WITH Insurance:    ${:.2}", wealth_with_insurance);
    println!("   WITHOUT Insurance: ${:.2}", wealth_without_insurance);
    println!(
        "   Difference:        ${:.2} ({:+.2}%)",
        wealth_difference, wealth_difference_pct
    );

    let volatility_with = insured_result.money_statistics.std_dev;
    let volatility_without = uninsured_result.money_statistics.std_dev;
    let volatility_reduction =
        ((volatility_without - volatility_with) / volatility_without) * 100.0;

    println!("\nWealth Volatility (Risk):");
    println!("   WITH Insurance:    ${:.2}", volatility_with);
    println!("   WITHOUT Insurance: ${:.2}", volatility_without);

    if volatility_reduction > 0.0 {
        println!("   Risk Reduction:    {:.2}%", volatility_reduction);
    } else {
        println!("   Risk Increase:     {:.2}%", volatility_reduction.abs());
    }

    // Key insights
    println!("\n\n💡 Key Insights");
    println!("================");

    if wealth_with_insurance > wealth_without_insurance {
        println!("✅ Insurance provided net economic benefit in this simulation.");
        println!("   The wealth protection from insurance payouts exceeded premium costs.");
    } else {
        println!("⚠️  Insurance had a net cost in this simulation.");
        println!("   Premiums exceeded payouts, but insurance still provided risk protection.");
    }

    if volatility_with < volatility_without {
        println!("✅ Insurance successfully reduced wealth volatility (economic stability).");
        println!("   This demonstrates insurance's core value: risk mitigation.");
    } else {
        println!("⚠️  Insurance did not reduce volatility in this particular run.");
        println!("   Different random seeds may show different outcomes.");
    }

    if let Some(ref insurance_stats) = insured_result.insurance_statistics {
        if insurance_stats.total_claims_paid > 0 {
            println!(
                "✅ Insurance claims were successfully paid ({} claims).",
                insurance_stats.total_claims_paid
            );
            println!("   This shows the insurance system actively protected persons from losses.");
        } else {
            println!("ℹ️  No insurance claims were paid in this simulation.");
            println!(
                "   This can happen if crises were mild or persons didn't qualify for payouts."
            );
        }
    }

    // Scenario 3: Focused demonstration with high crisis rate
    println!("\n\n📊 Scenario 3: High Crisis Rate (Insurance Value Demo)");
    println!("-------------------------------------------------------");
    println!("This scenario uses an extremely high crisis rate to clearly");
    println!("demonstrate insurance effectiveness.\n");

    let extreme_config = SimulationConfig {
        max_steps: 150,
        entity_count: 25,
        seed: 123,

        enable_insurance: true,
        insurance_premium_rate: 0.08,         // 8% premium
        insurance_duration: 0,                // 0 = indefinite coverage (never expires)
        insurance_purchase_probability: 0.25, // 25% chance (very high)
        insurance_coverage_amount: 100.0,

        enable_crisis_events: true,
        crisis_probability: 0.15, // 15% chance per step (extreme)
        crisis_severity: 0.25,    // 25% severity

        enable_loans: true,
        loan_interest_rate: 0.02,
        loan_repayment_period: 15,
        min_money_to_lend: 40.0,

        initial_money_per_person: 150.0,
        base_skill_price: 12.0,

        ..Default::default()
    };

    let mut extreme_engine = SimulationEngine::new(extreme_config);
    let extreme_result = extreme_engine.run();

    println!("🚨 High Crisis Scenario Results:");
    println!("   Simulation Steps: {}", extreme_result.total_steps);
    println!("   Persons Survived: {}", extreme_result.active_persons);
    println!("   Final Average Wealth: ${:.2}", extreme_result.money_statistics.average);

    if let Some(ref insurance_stats) = extreme_result.insurance_statistics {
        println!("\n   📋 Insurance Performance:");
        println!("   - Policies Issued: {}", insurance_stats.total_policies_issued);
        println!("   - Claims Paid: {}", insurance_stats.total_claims_paid);

        if extreme_result.active_persons > 0 {
            println!(
                "   - Coverage Ratio: {:.2}%",
                (insurance_stats.active_policies as f64 / extreme_result.active_persons as f64)
                    * 100.0
            );
        }

        if insurance_stats.total_premiums_collected > 0.0 {
            let value_ratio =
                insurance_stats.total_payouts_made / insurance_stats.total_premiums_collected;
            println!("   - Value Delivered: ${:.2} payout per $1.00 premium", value_ratio);
        }
    }

    println!("\n\n===========================================");
    println!("Example completed successfully!");
    println!("===========================================");
    println!("\n💾 Try running the simulation yourself:");
    println!("   cargo run --example insurance_demo");
    println!("\n📚 For more information, see the README.md file.");
}
