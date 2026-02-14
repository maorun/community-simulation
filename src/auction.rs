//! Auction mechanisms for alternative price discovery.
//!
//! This module provides simple auction functionality as an alternative to bilateral trading.
//! Currently implements English (ascending-price) auctions where buyers submit bids and the
//! highest bidder wins.
//!
//! # Examples
//!
//! ```
//! use community_simulation::auction::{Auction, AuctionType};
//!
//! let mut auction = Auction::new("Programming".to_string(), AuctionType::English);
//! auction.add_bid(1, 50.0);
//! auction.add_bid(2, 75.0);
//! auction.add_bid(3, 60.0);
//!
//! let winner = auction.resolve();
//! assert_eq!(winner, Some((2, 75.0))); // Bidder 2 wins with 75.0
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of auction mechanisms available.
///
/// Currently only English auctions are implemented. This enum allows for
/// future expansion to other auction types (Dutch, Vickrey, etc.).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuctionType {
    /// English auction: ascending-price, highest bidder wins
    English,
}

/// Represents a single auction for a specific skill.
///
/// An auction collects bids from multiple buyers and determines the winner
/// based on the auction type. For English auctions, the highest bidder wins.
///
/// # Examples
///
/// ```
/// use community_simulation::auction::{Auction, AuctionType};
///
/// let mut auction = Auction::new("Cooking".to_string(), AuctionType::English);
///
/// // Add bids from different persons
/// auction.add_bid(10, 20.0);
/// auction.add_bid(11, 25.0);
/// auction.add_bid(12, 22.0);
///
/// // Resolve auction - person 11 wins with bid of 25.0
/// let result = auction.resolve();
/// assert_eq!(result, Some((11, 25.0)));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    /// The skill being auctioned
    pub skill_id: String,

    /// Type of auction mechanism
    pub auction_type: AuctionType,

    /// Bids submitted: PersonId -> bid amount
    pub bids: HashMap<usize, f64>,
}

impl Auction {
    /// Creates a new auction for a specific skill.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The skill being auctioned
    /// * `auction_type` - The type of auction mechanism to use
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::auction::{Auction, AuctionType};
    ///
    /// let auction = Auction::new("Gardening".to_string(), AuctionType::English);
    /// ```
    pub fn new(skill_id: String, auction_type: AuctionType) -> Self {
        Self { skill_id, auction_type, bids: HashMap::new() }
    }

    /// Adds a bid to the auction.
    ///
    /// If the person has already bid, this updates their bid to the new amount.
    /// Only accepts finite bid amounts (rejects NaN and infinity).
    ///
    /// # Arguments
    ///
    /// * `person_id` - The ID of the person submitting the bid
    /// * `amount` - The bid amount (must be finite, non-NaN, non-infinite)
    ///
    /// # Panics
    ///
    /// Panics if the bid amount is NaN or infinite.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::auction::{Auction, AuctionType};
    ///
    /// let mut auction = Auction::new("Plumbing".to_string(), AuctionType::English);
    /// auction.add_bid(5, 30.0);
    /// auction.add_bid(5, 35.0); // Update bid
    /// ```
    pub fn add_bid(&mut self, person_id: usize, amount: f64) {
        assert!(
            amount.is_finite(),
            "Bid amount must be finite (not NaN or infinite), got: {}",
            amount
        );
        self.bids.insert(person_id, amount);
    }

    /// Resolves the auction and determines the winner.
    ///
    /// For English auctions, returns the person with the highest bid and their bid amount.
    /// Returns `None` if there are no bids.
    ///
    /// Only considers finite bid amounts. Non-finite bids (if any bypass validation)
    /// are automatically excluded from consideration.
    ///
    /// # Returns
    ///
    /// * `Some((winner_id, winning_bid))` - The winning person ID and their bid
    /// * `None` - If no bids were submitted or all bids are non-finite
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::auction::{Auction, AuctionType};
    ///
    /// let mut auction = Auction::new("Carpentry".to_string(), AuctionType::English);
    /// auction.add_bid(7, 40.0);
    /// auction.add_bid(8, 45.0);
    /// auction.add_bid(9, 42.0);
    ///
    /// let winner = auction.resolve();
    /// assert_eq!(winner, Some((8, 45.0)));
    /// ```
    pub fn resolve(&self) -> Option<(usize, f64)> {
        match self.auction_type {
            AuctionType::English => {
                // Find the highest bidder among finite bids
                // Filter ensures we only compare valid (finite) bids
                self.bids
                    .iter()
                    .filter(|(_id, amount)| amount.is_finite())
                    .max_by(|a, b| {
                        // Safety: all values are finite due to filter, so partial_cmp always returns Some
                        a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(person_id, amount)| (*person_id, *amount))
            },
        }
    }

    /// Returns the number of bids submitted to this auction.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::auction::{Auction, AuctionType};
    ///
    /// let mut auction = Auction::new("Teaching".to_string(), AuctionType::English);
    /// assert_eq!(auction.bid_count(), 0);
    ///
    /// auction.add_bid(1, 50.0);
    /// auction.add_bid(2, 55.0);
    /// assert_eq!(auction.bid_count(), 2);
    /// ```
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    /// Clears all bids from the auction.
    ///
    /// Useful for resetting an auction for the next round.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::auction::{Auction, AuctionType};
    ///
    /// let mut auction = Auction::new("Writing".to_string(), AuctionType::English);
    /// auction.add_bid(3, 25.0);
    /// auction.add_bid(4, 30.0);
    /// assert_eq!(auction.bid_count(), 2);
    ///
    /// auction.clear_bids();
    /// assert_eq!(auction.bid_count(), 0);
    /// ```
    pub fn clear_bids(&mut self) {
        self.bids.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auction_creation() {
        let auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        assert_eq!(auction.skill_id, "TestSkill");
        assert_eq!(auction.auction_type, AuctionType::English);
        assert_eq!(auction.bid_count(), 0);
    }

    #[test]
    fn test_add_single_bid() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        assert_eq!(auction.bid_count(), 1);
        assert_eq!(auction.bids.get(&1), Some(&50.0));
    }

    #[test]
    fn test_add_multiple_bids() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        auction.add_bid(2, 75.0);
        auction.add_bid(3, 60.0);
        assert_eq!(auction.bid_count(), 3);
    }

    #[test]
    fn test_update_bid() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        auction.add_bid(1, 80.0); // Update bid
        assert_eq!(auction.bid_count(), 1);
        assert_eq!(auction.bids.get(&1), Some(&80.0));
    }

    #[test]
    fn test_resolve_english_auction_with_bids() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        auction.add_bid(2, 75.0);
        auction.add_bid(3, 60.0);

        let winner = auction.resolve();
        assert_eq!(winner, Some((2, 75.0)));
    }

    #[test]
    fn test_resolve_english_auction_no_bids() {
        let auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        let winner = auction.resolve();
        assert_eq!(winner, None);
    }

    #[test]
    fn test_resolve_english_auction_single_bid() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(5, 100.0);

        let winner = auction.resolve();
        assert_eq!(winner, Some((5, 100.0)));
    }

    #[test]
    fn test_resolve_english_auction_tie_breaker() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        auction.add_bid(2, 50.0);

        // One of them should win (either is acceptable)
        let winner = auction.resolve();
        assert!(winner.is_some());
        assert_eq!(winner.unwrap().1, 50.0);
    }

    #[test]
    fn test_clear_bids() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 50.0);
        auction.add_bid(2, 60.0);
        assert_eq!(auction.bid_count(), 2);

        auction.clear_bids();
        assert_eq!(auction.bid_count(), 0);
        assert_eq!(auction.resolve(), None);
    }

    #[test]
    fn test_bid_with_zero_amount() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, 0.0);
        auction.add_bid(2, 10.0);

        let winner = auction.resolve();
        assert_eq!(winner, Some((2, 10.0)));
    }

    #[test]
    fn test_bid_with_negative_amount() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, -5.0);
        auction.add_bid(2, 10.0);

        // Highest bid should still win (even if one is negative)
        let winner = auction.resolve();
        assert_eq!(winner, Some((2, 10.0)));
    }

    #[test]
    #[should_panic(expected = "Bid amount must be finite")]
    fn test_bid_with_nan_panics() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, f64::NAN); // Should panic
    }

    #[test]
    #[should_panic(expected = "Bid amount must be finite")]
    fn test_bid_with_infinity_panics() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, f64::INFINITY); // Should panic
    }

    #[test]
    #[should_panic(expected = "Bid amount must be finite")]
    fn test_bid_with_neg_infinity_panics() {
        let mut auction = Auction::new("TestSkill".to_string(), AuctionType::English);
        auction.add_bid(1, f64::NEG_INFINITY); // Should panic
    }
}
