//! # Voting System Module
//!
//! Implements a governance and collective decision-making system for the simulation.
//! Supports multiple voting methods to model different democratic mechanisms.
//!
//! ## Features
//!
//! - **Proposal System**: Create and track proposals for community decisions
//! - **Multiple Voting Methods**:
//!   - Simple Majority: One person, one vote
//!   - Weighted by Wealth: Voting power proportional to money
//!   - Quadratic Voting: Square root of wealth for balanced influence
//! - **Voting Statistics**: Track participation, outcomes, and voting patterns
//!
//! ## Example
//!
//! ```no_run
//! use simulation_framework::voting::{VotingSystem, VotingMethod, ProposalType};
//!
//! // Create voting system with quadratic voting
//! let mut voting_system = VotingSystem::new(VotingMethod::QuadraticVoting);
//!
//! // Create a proposal
//! let proposal_id = voting_system.create_proposal(
//!     ProposalType::TaxRateChange { new_rate: 0.15 },
//!     "Increase tax rate to 15%".to_string(),
//!     Some(10), // expires in 10 steps
//!     0, // current step
//! );
//!
//! // Cast votes
//! voting_system.cast_vote(proposal_id, 1, true, 100.0, 0); // person 1, yes, with 100 money, step 0
//! voting_system.cast_vote(proposal_id, 2, false, 50.0, 0); // person 2, no, with 50 money, step 0
//!
//! // Tally results
//! if let Some(result) = voting_system.tally_proposal(proposal_id, 1) {
//!     println!("Proposal passed: {}", result.passed);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a proposal
pub type ProposalId = usize;

/// Type of voting method to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VotingMethod {
    /// One person, one vote - pure democracy
    #[default]
    SimpleMajority,
    /// Voting power proportional to wealth - plutocracy
    WeightedByWealth,
    /// Square root of wealth for balanced influence - quadratic voting
    QuadraticVoting,
}

/// Type of proposal that can be voted on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change the tax rate
    TaxRateChange { new_rate: f64 },
    /// Change the base skill price
    BasePriceChange { new_price: f64 },
    /// Change the transaction fee
    TransactionFeeChange { new_fee: f64 },
    /// Generic proposal (no direct effect on simulation)
    Generic { description: String },
}

/// A single vote cast by a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// ID of the person who voted
    pub person_id: usize,
    /// Vote choice (true = yes/approve, false = no/reject)
    pub in_favor: bool,
    /// Voting power based on method and person's wealth
    pub voting_power: f64,
    /// Simulation step when vote was cast
    pub step: usize,
}

/// A proposal that can be voted on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier for this proposal
    pub id: ProposalId,
    /// Type of proposal
    pub proposal_type: ProposalType,
    /// Human-readable description
    pub description: String,
    /// Simulation step when proposal was created
    pub created_at: usize,
    /// Simulation step when voting closes (None = no expiration)
    pub expires_at: Option<usize>,
    /// All votes cast on this proposal
    pub votes: Vec<Vote>,
    /// Whether this proposal is still active
    pub active: bool,
}

/// Result of a completed vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// Proposal ID
    pub proposal_id: ProposalId,
    /// Total voting power in favor
    pub votes_in_favor: f64,
    /// Total voting power against
    pub votes_against: f64,
    /// Total number of unique voters
    pub total_voters: usize,
    /// Whether the proposal passed (favor > against)
    pub passed: bool,
    /// Step when vote was tallied
    pub tallied_at: usize,
}

/// Main voting system managing all proposals and votes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingSystem {
    /// Voting method to use for all proposals
    method: VotingMethod,
    /// All proposals, indexed by ID
    proposals: HashMap<ProposalId, Proposal>,
    /// Results of completed votes
    results: Vec<VotingResult>,
    /// Next proposal ID to assign
    next_proposal_id: ProposalId,
    /// Total number of votes cast across all proposals
    total_votes_cast: usize,
}

impl VotingSystem {
    /// Create a new voting system with the specified method
    pub fn new(method: VotingMethod) -> Self {
        Self {
            method,
            proposals: HashMap::new(),
            results: Vec::new(),
            next_proposal_id: 1,
            total_votes_cast: 0,
        }
    }

    /// Get the current voting method
    pub fn method(&self) -> VotingMethod {
        self.method
    }

    /// Create a new proposal
    ///
    /// # Arguments
    ///
    /// * `proposal_type` - Type of proposal
    /// * `description` - Human-readable description
    /// * `duration_steps` - Number of steps until expiration (None for no expiration)
    /// * `current_step` - Current simulation step
    ///
    /// # Returns
    ///
    /// The ID of the newly created proposal
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        description: String,
        duration_steps: Option<usize>,
        current_step: usize,
    ) -> ProposalId {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let expires_at = duration_steps.map(|duration| current_step + duration);

        let proposal = Proposal {
            id,
            proposal_type,
            description,
            created_at: current_step,
            expires_at,
            votes: Vec::new(),
            active: true,
        };

        self.proposals.insert(id, proposal);
        id
    }

    /// Cast a vote on a proposal
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `person_id` - ID of the person casting the vote
    /// * `in_favor` - True for yes/approve, false for no/reject
    /// * `wealth` - Person's current wealth (used for weighted voting)
    /// * `current_step` - Current simulation step
    ///
    /// # Returns
    ///
    /// `true` if vote was successfully cast, `false` if proposal doesn't exist or is closed
    pub fn cast_vote(
        &mut self,
        proposal_id: ProposalId,
        person_id: usize,
        in_favor: bool,
        wealth: f64,
        current_step: usize,
    ) -> bool {
        // Calculate voting power before getting mutable reference
        let voting_power = self.calculate_voting_power(wealth);

        // Check if proposal exists and is active
        let proposal = match self.proposals.get_mut(&proposal_id) {
            Some(p) if p.active => p,
            _ => return false,
        };

        // Check if proposal has expired
        if let Some(expires_at) = proposal.expires_at {
            if current_step >= expires_at {
                proposal.active = false;
                return false;
            }
        }

        // Check if person has already voted
        if proposal.votes.iter().any(|v| v.person_id == person_id) {
            return false; // Already voted
        }

        let vote = Vote {
            person_id,
            in_favor,
            voting_power,
            step: current_step,
        };

        proposal.votes.push(vote);
        self.total_votes_cast += 1;
        true
    }

    /// Calculate voting power for a person based on their wealth
    fn calculate_voting_power(&self, wealth: f64) -> f64 {
        match self.method {
            VotingMethod::SimpleMajority => 1.0, // Everyone has equal vote
            VotingMethod::WeightedByWealth => wealth.max(0.0), // Direct wealth proportional
            VotingMethod::QuadraticVoting => wealth.max(0.0).sqrt(), // Square root for balance
        }
    }

    /// Tally the results of a proposal and mark it as closed
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - ID of the proposal to tally
    /// * `current_step` - Current simulation step
    ///
    /// # Returns
    ///
    /// `Some(VotingResult)` if proposal exists and was tallied, `None` otherwise
    pub fn tally_proposal(
        &mut self,
        proposal_id: ProposalId,
        current_step: usize,
    ) -> Option<VotingResult> {
        let proposal = self.proposals.get_mut(&proposal_id)?;

        if !proposal.active {
            return None; // Already tallied
        }

        let mut votes_in_favor = 0.0;
        let mut votes_against = 0.0;
        let total_voters = proposal.votes.len();

        for vote in &proposal.votes {
            if vote.in_favor {
                votes_in_favor += vote.voting_power;
            } else {
                votes_against += vote.voting_power;
            }
        }

        let passed = votes_in_favor > votes_against;
        proposal.active = false;

        let result = VotingResult {
            proposal_id,
            votes_in_favor,
            votes_against,
            total_voters,
            passed,
            tallied_at: current_step,
        };

        self.results.push(result.clone());
        Some(result)
    }

    /// Automatically tally expired proposals
    ///
    /// # Arguments
    ///
    /// * `current_step` - Current simulation step
    ///
    /// # Returns
    ///
    /// Vector of results from newly tallied proposals
    pub fn tally_expired_proposals(&mut self, current_step: usize) -> Vec<VotingResult> {
        let expired_ids: Vec<ProposalId> = self
            .proposals
            .values()
            .filter(|p| {
                p.active
                    && p.expires_at
                        .map(|expires| current_step >= expires)
                        .unwrap_or(false)
            })
            .map(|p| p.id)
            .collect();

        expired_ids
            .into_iter()
            .filter_map(|id| self.tally_proposal(id, current_step))
            .collect()
    }

    /// Get all active proposals
    pub fn active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().filter(|p| p.active).collect()
    }

    /// Get all completed results
    pub fn results(&self) -> &[VotingResult] {
        &self.results
    }

    /// Get a specific proposal by ID
    pub fn get_proposal(&self, proposal_id: ProposalId) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get statistics about the voting system
    pub fn statistics(&self) -> VotingStatistics {
        let total_proposals = self.proposals.len();
        let active_proposals = self.proposals.values().filter(|p| p.active).count();
        let completed_proposals = self.results.len();

        let passed_proposals = self.results.iter().filter(|r| r.passed).count();
        let failed_proposals = self.results.iter().filter(|r| !r.passed).count();

        let total_voters: usize = self.results.iter().map(|r| r.total_voters).sum();
        let avg_participation = if completed_proposals > 0 {
            total_voters as f64 / completed_proposals as f64
        } else {
            0.0
        };

        VotingStatistics {
            total_proposals,
            active_proposals,
            completed_proposals,
            passed_proposals,
            failed_proposals,
            total_votes_cast: self.total_votes_cast,
            avg_participation,
            voting_method: self.method,
        }
    }
}

/// Statistics about voting activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStatistics {
    /// Total number of proposals created
    pub total_proposals: usize,
    /// Number of active (unclosed) proposals
    pub active_proposals: usize,
    /// Number of completed proposals
    pub completed_proposals: usize,
    /// Number of proposals that passed
    pub passed_proposals: usize,
    /// Number of proposals that failed
    pub failed_proposals: usize,
    /// Total number of votes cast across all proposals
    pub total_votes_cast: usize,
    /// Average number of voters per proposal
    pub avg_participation: f64,
    /// Voting method used
    pub voting_method: VotingMethod,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voting_system_creation() {
        let voting_system = VotingSystem::new(VotingMethod::SimpleMajority);
        assert_eq!(voting_system.method(), VotingMethod::SimpleMajority);
        assert_eq!(voting_system.active_proposals().len(), 0);
        assert_eq!(voting_system.results().len(), 0);
    }

    #[test]
    fn test_create_proposal() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);
        let proposal_id = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Test proposal".to_string(),
            },
            "Test description".to_string(),
            Some(10),
            0,
        );

        assert_eq!(proposal_id, 1);
        assert_eq!(voting_system.active_proposals().len(), 1);

        let proposal = voting_system.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.id, proposal_id);
        assert_eq!(proposal.created_at, 0);
        assert_eq!(proposal.expires_at, Some(10));
        assert!(proposal.active);
    }

    #[test]
    fn test_simple_majority_voting() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);
        let proposal_id = voting_system.create_proposal(
            ProposalType::TaxRateChange { new_rate: 0.15 },
            "Increase tax rate".to_string(),
            None,
            0,
        );

        // Cast 3 yes votes and 2 no votes
        assert!(voting_system.cast_vote(proposal_id, 1, true, 100.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 2, true, 50.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 3, true, 200.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 4, false, 150.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 5, false, 300.0, 0));

        let result = voting_system.tally_proposal(proposal_id, 1).unwrap();
        assert!(result.passed); // 3 yes > 2 no
        assert_eq!(result.votes_in_favor, 3.0);
        assert_eq!(result.votes_against, 2.0);
        assert_eq!(result.total_voters, 5);
    }

    #[test]
    fn test_weighted_by_wealth_voting() {
        let mut voting_system = VotingSystem::new(VotingMethod::WeightedByWealth);
        let proposal_id = voting_system.create_proposal(
            ProposalType::BasePriceChange { new_price: 12.0 },
            "Change base price".to_string(),
            None,
            0,
        );

        // Cast votes with different wealth levels
        assert!(voting_system.cast_vote(proposal_id, 1, true, 100.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 2, false, 150.0, 0));

        let result = voting_system.tally_proposal(proposal_id, 1).unwrap();
        assert!(!result.passed); // 100 yes < 150 no
        assert_eq!(result.votes_in_favor, 100.0);
        assert_eq!(result.votes_against, 150.0);
    }

    #[test]
    fn test_quadratic_voting() {
        let mut voting_system = VotingSystem::new(VotingMethod::QuadraticVoting);
        let proposal_id = voting_system.create_proposal(
            ProposalType::TransactionFeeChange { new_fee: 0.05 },
            "Change transaction fee".to_string(),
            None,
            0,
        );

        // Cast votes - quadratic voting uses square root of wealth
        // Person with 100 wealth: sqrt(100) = 10.0
        // Person with 400 wealth: sqrt(400) = 20.0
        assert!(voting_system.cast_vote(proposal_id, 1, true, 100.0, 0));
        assert!(voting_system.cast_vote(proposal_id, 2, false, 400.0, 0));

        let result = voting_system.tally_proposal(proposal_id, 1).unwrap();
        assert!(!result.passed); // 10.0 yes < 20.0 no
        assert_eq!(result.votes_in_favor, 10.0);
        assert_eq!(result.votes_against, 20.0);
    }

    #[test]
    fn test_duplicate_vote_prevention() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);
        let proposal_id = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Test".to_string(),
            },
            "Test".to_string(),
            None,
            0,
        );

        // First vote should succeed
        assert!(voting_system.cast_vote(proposal_id, 1, true, 100.0, 0));

        // Second vote from same person should fail
        assert!(!voting_system.cast_vote(proposal_id, 1, false, 100.0, 0));

        let proposal = voting_system.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes.len(), 1);
    }

    #[test]
    fn test_proposal_expiration() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);
        let proposal_id = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Test".to_string(),
            },
            "Test".to_string(),
            Some(5), // Expires after 5 steps
            0,
        );

        // Vote should succeed at step 4
        assert!(voting_system.cast_vote(proposal_id, 1, true, 100.0, 4));

        // Vote should fail at step 5 (expired)
        assert!(!voting_system.cast_vote(proposal_id, 2, true, 100.0, 5));
    }

    #[test]
    fn test_auto_tally_expired_proposals() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);

        // Create two proposals with different expiration times
        let proposal1 = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Proposal 1".to_string(),
            },
            "First".to_string(),
            Some(5),
            0,
        );

        let proposal2 = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Proposal 2".to_string(),
            },
            "Second".to_string(),
            Some(10),
            0,
        );

        // Cast some votes
        voting_system.cast_vote(proposal1, 1, true, 100.0, 0);
        voting_system.cast_vote(proposal2, 2, false, 100.0, 0);

        // Tally expired proposals at step 5
        let results = voting_system.tally_expired_proposals(5);
        assert_eq!(results.len(), 1); // Only proposal1 should be tallied
        assert_eq!(results[0].proposal_id, proposal1);

        // Tally again at step 10
        let results = voting_system.tally_expired_proposals(10);
        assert_eq!(results.len(), 1); // Only proposal2 should be tallied
        assert_eq!(results[0].proposal_id, proposal2);
    }

    #[test]
    fn test_voting_statistics() {
        let mut voting_system = VotingSystem::new(VotingMethod::SimpleMajority);

        // Create and vote on two proposals
        let proposal1 = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Proposal 1".to_string(),
            },
            "First".to_string(),
            None,
            0,
        );

        let proposal2 = voting_system.create_proposal(
            ProposalType::Generic {
                description: "Proposal 2".to_string(),
            },
            "Second".to_string(),
            None,
            0,
        );

        // Vote and tally first proposal (pass)
        voting_system.cast_vote(proposal1, 1, true, 100.0, 0);
        voting_system.cast_vote(proposal1, 2, true, 100.0, 0);
        voting_system.cast_vote(proposal1, 3, false, 100.0, 0);
        voting_system.tally_proposal(proposal1, 1);

        // Vote and tally second proposal (fail)
        voting_system.cast_vote(proposal2, 4, false, 100.0, 0);
        voting_system.cast_vote(proposal2, 5, false, 100.0, 0);
        voting_system.tally_proposal(proposal2, 2);

        let stats = voting_system.statistics();
        assert_eq!(stats.total_proposals, 2);
        assert_eq!(stats.active_proposals, 0);
        assert_eq!(stats.completed_proposals, 2);
        assert_eq!(stats.passed_proposals, 1);
        assert_eq!(stats.failed_proposals, 1);
        assert_eq!(stats.total_votes_cast, 5);
        assert_eq!(stats.avg_participation, 2.5);
    }
}
