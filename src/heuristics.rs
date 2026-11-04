//! Rule-of-thumb agents (heuristics and bounded rationality).
//!
//! Heuristic agents do not optimize. They use one simple rule when they choose a seller.
//! The module assigns the rules to agents, selects sellers, and collects group statistics.

use crate::entity::Entity;
use crate::person::Heuristic;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Salt that separates the heuristic assignment RNG from the main simulation RNG.
const ASSIGNMENT_SEED_SALT: u64 = 0x4845_5552_4953_5449;

/// Assign heuristics to a share of the agents.
///
/// The function uses its own seeded RNG. It does not touch the main simulation RNG.
/// It spreads the three rules evenly over the selected agents.
/// The share is clamped to `0.0..=1.0`. A non-finite share selects no agent.
pub fn assign_heuristics(entities: &mut [Entity], share: f64, seed: u64) {
    let share = if share.is_finite() {
        share.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let selected = ((entities.len() as f64) * share).round() as usize;
    if selected == 0 {
        return;
    }
    let mut order: Vec<usize> = (0..entities.len()).collect();
    let mut rng = StdRng::seed_from_u64(seed ^ ASSIGNMENT_SEED_SALT);
    order.shuffle(&mut rng);
    let rules = Heuristic::all_variants();
    for (position, &idx) in order.iter().take(selected).enumerate() {
        entities[idx].person_data.heuristic = Some(rules[position % rules.len()]);
    }
}

/// Pick a seller index for a heuristic buyer.
///
/// Returns `None` when no valid provider exists. The caller then uses the default choice.
/// A valid provider is active and differs from the buyer.
///
/// * `CheapestSeller` picks the lowest price after the reputation adjustment.
/// * `TrustFriends` picks the first provider that is a friend of the buyer.
/// * `FollowMajority` picks the provider with the most sales in the previous step.
///
/// Ties go to the provider that appears first in the list.
pub fn select_seller(
    heuristic: Heuristic,
    buyer_idx: usize,
    providers: &[usize],
    entities: &[Entity],
    previous_sales: &HashMap<usize, usize>,
) -> Option<usize> {
    let valid = || {
        providers
            .iter()
            .copied()
            .filter(|&idx| idx != buyer_idx && entities.get(idx).is_some_and(|e| e.active))
    };
    match heuristic {
        Heuristic::CheapestSeller => {
            let mut best: Option<(usize, f64)> = None;
            for idx in valid() {
                let price = entities[idx].person_data.reputation_price_multiplier();
                if best.is_none_or(|(_, best_price)| price < best_price) {
                    best = Some((idx, price));
                }
            }
            best.map(|(idx, _)| idx)
        },
        Heuristic::TrustFriends => {
            let buyer = &entities[buyer_idx].person_data;
            valid().find(|&idx| buyer.is_friend_with(entities[idx].person_data.id))
        },
        Heuristic::FollowMajority => {
            let mut best: Option<(usize, usize)> = None;
            for idx in valid() {
                let sales = previous_sales.get(&idx).copied().unwrap_or(0);
                if best.is_none_or(|(_, best_sales)| sales > best_sales) {
                    best = Some((idx, sales));
                }
            }
            best.map(|(idx, _)| idx)
        },
    }
}

/// Statistics for one group of agents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeuristicGroupStats {
    /// Number of agents in the group.
    #[serde(default)]
    pub agent_count: usize,
    /// Average money of the agents in the group (0.0 for an empty group).
    #[serde(default)]
    pub average_wealth: f64,
}

/// Wealth comparison between heuristic agents and optimizing agents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeuristicStats {
    /// Agents without a heuristic. They keep the standard behavior.
    #[serde(default)]
    pub optimizing: HeuristicGroupStats,
    /// Agents that buy from the cheapest seller.
    #[serde(default)]
    pub cheapest_seller: HeuristicGroupStats,
    /// Agents that prefer friends as sellers.
    #[serde(default)]
    pub trust_friends: HeuristicGroupStats,
    /// Agents that follow the seller with the most previous sales.
    #[serde(default)]
    pub follow_majority: HeuristicGroupStats,
}

impl HeuristicStats {
    /// Build the group statistics from the final agent state.
    pub fn from_entities(entities: &[Entity]) -> Self {
        let mut sums = [(0usize, 0.0f64); 4];
        for entity in entities {
            let slot = match entity.person_data.heuristic {
                None => 0,
                Some(Heuristic::CheapestSeller) => 1,
                Some(Heuristic::TrustFriends) => 2,
                Some(Heuristic::FollowMajority) => 3,
            };
            let money = entity.person_data.money;
            sums[slot].0 += 1;
            if money.is_finite() {
                sums[slot].1 += money;
            }
        }
        let group = |(count, total): (usize, f64)| HeuristicGroupStats {
            agent_count: count,
            average_wealth: if count > 0 { total / count as f64 } else { 0.0 },
        };
        Self {
            optimizing: group(sums[0]),
            cheapest_seller: group(sums[1]),
            trust_friends: group(sums[2]),
            follow_majority: group(sums[3]),
        }
    }
}

/// Count one sale for a seller in the current step.
pub fn record_sale(current_sales: &mut HashMap<usize, usize>, seller_idx: usize) {
    *current_sales.entry(seller_idx).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::person::{Location, Strategy};

    fn entities(count: usize) -> Vec<Entity> {
        (0..count)
            .map(|i| {
                Entity::new(i, 100.0, vec![], Strategy::Balanced, Location::new(0.0, 0.0), 0.95)
            })
            .collect()
    }

    #[test]
    fn cheapest_seller_picks_lowest_price_and_skips_buyer() {
        let mut list = entities(4);
        list[0].person_data.reputation = 0.1; // buyer, must be skipped
        list[1].person_data.reputation = 1.0;
        list[2].person_data.reputation = 2.0; // best reputation = lowest price
        list[3].person_data.reputation = 1.0;
        let m = |i: usize| list[i].person_data.reputation_price_multiplier();
        assert!(m(2) < m(1));
        let picked =
            select_seller(Heuristic::CheapestSeller, 0, &[0, 1, 2, 3], &list, &HashMap::new());
        assert_eq!(picked, Some(2));
    }

    #[test]
    fn trust_friends_picks_friend_or_none() {
        let mut list = entities(3);
        let friend_id = list[2].person_data.id;
        list[0].person_data.add_friend(friend_id);
        let prev = HashMap::new();
        assert_eq!(select_seller(Heuristic::TrustFriends, 0, &[1, 2], &list, &prev), Some(2));
        assert_eq!(select_seller(Heuristic::TrustFriends, 0, &[1], &list, &prev), None);
    }

    #[test]
    fn follow_majority_picks_most_sales_and_ignores_inactive() {
        let mut list = entities(4);
        let prev = HashMap::from([(1usize, 2usize), (2, 5), (3, 9)]);
        list[3].active = false;
        assert_eq!(select_seller(Heuristic::FollowMajority, 0, &[1, 2, 3], &list, &prev), Some(2));
        assert_eq!(select_seller(Heuristic::FollowMajority, 0, &[0, 3], &list, &prev), None);
    }

    #[test]
    fn assignment_respects_share_and_ignores_nan() {
        let mut list = entities(30);
        assign_heuristics(&mut list, 0.5, 7);
        let assigned = list.iter().filter(|e| e.person_data.heuristic.is_some()).count();
        assert_eq!(assigned, 15);
        let mut none = entities(30);
        assign_heuristics(&mut none, f64::NAN, 7);
        assert!(none.iter().all(|e| e.person_data.heuristic.is_none()));
    }
}
