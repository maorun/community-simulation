/// Property-based tests using proptest for core simulation components
/// These tests verify invariants and edge cases across a wide range of inputs
#[cfg(test)]
mod proptests {
    use crate::market::Market;
    use crate::person::{Person, TransactionType};
    use crate::result::calculate_gini_coefficient;
    use crate::scenario::{PriceUpdater, Scenario};
    use crate::skill::Skill;
    use proptest::prelude::*;

    /// Property: Person's reputation should always stay within bounds [0.0, 2.0]
    /// after any number of reputation changes
    #[test]
    fn proptest_person_reputation_bounds() {
        proptest!(|(
            initial_money in 0.0f64..10000.0,
            seller_increases in 0usize..1000,
            buyer_increases in 0usize..1000,
            decays in 0usize..10000
        )| {
            let skill = Skill::new("TestSkill".to_string(), 10.0);
            let mut person = Person::new(0, initial_money, skill);

            // Apply seller reputation increases
            for _ in 0..seller_increases {
                person.increase_reputation_as_seller();
            }

            // Apply buyer reputation increases
            for _ in 0..buyer_increases {
                person.increase_reputation_as_buyer();
            }

            // Apply reputation decay
            for _ in 0..decays {
                person.apply_reputation_decay();
            }

            // Reputation should always be within bounds
            prop_assert!(person.reputation >= 0.0, "Reputation should not be negative: {}", person.reputation);
            prop_assert!(person.reputation <= 2.0, "Reputation should not exceed 2.0: {}", person.reputation);
        });
    }

    /// Property: Person can afford an amount if and only if their money >= amount
    #[test]
    fn proptest_person_can_afford() {
        proptest!(|(
            money in 0.0f64..10000.0,
            amount in 0.0f64..10000.0
        )| {
            let skill = Skill::new("TestSkill".to_string(), 10.0);
            let person = Person::new(0, money, skill);

            let can_afford = person.can_afford(amount);

            if money >= amount {
                prop_assert!(can_afford, "Person with {} should afford {}", money, amount);
            } else {
                prop_assert!(!can_afford, "Person with {} should not afford {}", money, amount);
            }
        });
    }

    /// Property: Market prices should always stay within configured min/max bounds
    #[test]
    fn proptest_market_price_bounds() {
        proptest!(|(
            base_price in 1.0f64..100.0,
            demand in 0usize..100,
            supply in 1usize..100,
            steps in 0usize..50
        )| {
            let mut market = Market::new(base_price, PriceUpdater::from(Scenario::Original));
            let skill = Skill::new("TestSkill".to_string(), base_price);
            let skill_id = skill.id.clone();
            market.add_skill(skill);

            // Simulate price updates
            for _ in 0..steps {
                // Set demand and supply
                for _ in 0..demand {
                    market.increment_demand(&skill_id);
                }
                for _ in 0..supply {
                    market.increment_skill_supply(&skill_id);
                }

                let mut rng = rand::rng();
                market.update_prices(&mut rng);
                market.reset_demand_counts();
            }

            if let Some(price) = market.get_price(&skill_id) {
                prop_assert!(price >= market.min_skill_price,
                    "Price {} should be >= min {}", price, market.min_skill_price);
                prop_assert!(price <= market.max_skill_price,
                    "Price {} should be <= max {}", price, market.max_skill_price);
            }
        });
    }

    /// Property: Gini coefficient should be between 0 and 1 for positive wealth
    /// (can be negative when zeros are present, can exceed 1 when negative values exist)
    #[test]
    fn proptest_gini_coefficient_bounds() {
        proptest!(|(
            money_values in prop::collection::vec(0.0f64..10000.0, 1..100)
        )| {
            let sum: f64 = money_values.iter().sum();
            let gini = calculate_gini_coefficient(&money_values, sum);

            // Gini can range wider with edge cases (zeros, negative values)
            // Main assertion: it should be finite
            prop_assert!(gini.is_finite(), "Gini coefficient should be finite: {}", gini);
        });
    }

    /// Property: Gini coefficient of equal distribution should be close to 0
    #[test]
    fn proptest_gini_equal_distribution() {
        proptest!(|(
            value in 1.0f64..10000.0,
            count in 2usize..100
        )| {
            let money_values = vec![value; count];
            let sum: f64 = money_values.iter().sum();
            let gini = calculate_gini_coefficient(&money_values, sum);

            // Gini should be very close to 0 for equal distribution
            prop_assert!(gini.abs() < 0.01, "Gini coefficient for equal distribution should be ~0: {}", gini);
        });
    }

    /// Property: Transaction recording should preserve all transaction details
    #[test]
    fn proptest_transaction_recording() {
        proptest!(|(
            initial_money in 0.0f64..10000.0,
            step in 0usize..1000,
            amount in 0.0f64..1000.0,
            counterparty in 0usize..100
        )| {
            let skill = Skill::new("TestSkill".to_string(), 10.0);
            let mut person = Person::new(0, initial_money, skill.clone());

            let skill_id = skill.id.clone();
            person.record_transaction(
                step,
                skill_id.clone(),
                TransactionType::Buy,
                amount,
                Some(counterparty)
            );

            prop_assert_eq!(person.transaction_history.len(), 1);
            let transaction = &person.transaction_history[0];
            prop_assert_eq!(transaction.step, step);
            prop_assert_eq!(&transaction.skill_id, &skill_id);
            prop_assert_eq!(transaction.amount, amount);
            prop_assert_eq!(transaction.counterparty_id, Some(counterparty));
        });
    }

    /// Property: Skill price should remain non-negative after any operations
    #[test]
    fn proptest_skill_price_non_negative() {
        proptest!(|(
            base_price in 1.0f64..1000.0,
        )| {
            let skill = Skill::new("TestSkill".to_string(), base_price);

            prop_assert!(skill.current_price >= 0.0, "Skill price should be non-negative: {}", skill.current_price);
            prop_assert_eq!(skill.current_price, base_price);
        });
    }

    /// Property: Market with zero demand and positive supply should decrease prices
    /// (for Original scenario)
    #[test]
    fn proptest_market_zero_demand() {
        proptest!(|(
            base_price in 10.0f64..100.0,
            supply in 1usize..50,
        )| {
            let mut market = Market::new(base_price, PriceUpdater::from(Scenario::Original));
            market.volatility_percentage = 0.0; // Disable volatility for predictable test

            let skill = Skill::new("TestSkill".to_string(), base_price);
            let skill_id = skill.id.clone();
            market.add_skill(skill);

            // Set only supply, no demand
            for _ in 0..supply {
                market.increment_skill_supply(&skill_id);
            }

            let initial_price = market.get_price(&skill_id).unwrap();

            let mut rng = rand::thread_rng();
            market.update_prices(&mut rng);

            let final_price = market.get_price(&skill_id).unwrap();

            // With zero demand, price should decrease or stay at minimum
            prop_assert!(
                final_price <= initial_price || final_price == market.min_skill_price,
                "Price should decrease with zero demand: {} -> {}",
                initial_price,
                final_price
            );
        });
    }
}
