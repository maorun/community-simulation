//! # Economic Simulation Framework
//!
//! A configurable economic simulation framework written in Rust that models a small economy
//! of individuals (persons) with unique skills who engage in trade within a dynamic market.
//!
//! ## Overview
//!
//! This framework simulates an agent-based economic system where:
//! - Each person has a unique skill they can offer
//! - Persons have money and need skills from other persons
//! - A market mechanism adjusts prices based on supply and demand
//! - Transactions occur when buyers can afford skills they need
//! - Reputation affects pricing (better reputation = better prices)
//!
//! ## Quick Start
//!
//! ```no_run
//! use community_simulation::{SimulationConfig, SimulationEngine};
//!
//! // Create a simulation configuration
//! let config = SimulationConfig {
//!     max_steps: 500,
//!     entity_count: 100,
//!     seed: 42,
//!     initial_money_per_person: 100.0,
//!     base_skill_price: 10.0,
//!     ..Default::default()
//! };
//!
//! // Create and run the simulation
//! let mut engine = SimulationEngine::new(config);
//! let result = engine.run();
//!
//! // Access simulation results
//! println!("Final wealth distribution: {:?}", result.final_money_distribution);
//! println!("Gini coefficient: {:.2}", result.money_statistics.gini_coefficient);
//! ```
//!
//! ## Core Concepts
//!
//! ### Agents (Persons)
//! Each person in the simulation:
//! - Has a unique skill they can provide
//! - Needs other skills (randomly determined)
//! - Has money to purchase needed skills
//! - Has a reputation that affects prices
//! - Records all transaction history
//!
//! ### Market
//! The market coordinates trade:
//! - Tracks supply and demand for each skill
//! - Adjusts prices based on market dynamics
//! - Maintains historical price data
//! - Enforces price boundaries (min/max)
//!
//! ### Scenarios
//! Different pricing mechanisms can be configured:
//! - **Original**: Supply/demand-based pricing with volatility
//! - **DynamicPricing**: Sales-based pricing (increase if sold, decrease if not)
//!
//! ## Modules
//!
//! - [`asset`] - Asset system for long-term wealth building (property, equipment, stocks)
//! - [`causal_analysis`] - Causal inference framework for policy evaluation
//! - [`centrality`] - Network centrality analysis for trading networks
//! - [`config`] - Simulation configuration parameters
//! - [`contract`] - Contract system for long-term agreements
//! - [`credit_rating`] - Credit scoring system for evaluating creditworthiness
//! - [`database`] - SQLite database export functionality
//! - [`engine`] - Main simulation engine and execution loop
//! - [`entity`] - Entity wrapper around Person for simulation framework
//! - [`environment`] - Environmental resource tracking and sustainability metrics
//! - [`error`] - Custom error types for robust error handling
//! - [`event`] - Event system for tracking simulation events
//! - [`invariant`] - Invariant checking framework for simulation validation
//! - [`investment`] - Investment system for capital allocation and returns
//! - [`loan`] - Loan system for credit between persons
//! - [`market`] - Market mechanisms and price dynamics
//! - [`person`] - Person agents, transactions, and behavior
//! - [`plugin`] - Plugin system for extending simulation functionality
//! - [`pool`] - Memory pooling for reusing allocations and reducing overhead
//! - [`production`] - Production system for combining skills to create new skills
//! - [`replay`] - Action logging and simulation replay for debugging
//! - [`result`] - Simulation results, statistics, and output formatting
//! - [`scenario`] - Price update strategies for different simulation scenarios
//! - [`skill`] - Skill definitions and generation
//! - [`trade_agreement`] - Trade agreements between persons for preferential trading
//! - [`trust_network`] - Trust network system for transitive trust relationships
//! - [`voting`] - Voting system for governance and collective decision-making
//! - [`wizard`] - Interactive configuration wizard for guided setup

pub mod asset;
pub mod causal_analysis;
pub mod centrality;
pub mod completion;
pub mod config;
pub mod contract;
pub mod credit_rating;
pub mod crisis;
pub mod database;
pub mod engine;
pub mod entity; // Represents a Person in the simulation
pub mod environment;
pub mod error;
pub mod event;
pub mod externality;
pub mod insurance;
pub mod invariant;
pub mod investment;
pub mod list_commands;
pub mod loan;
pub mod market;
pub mod parameter_sweep;
pub mod person;
// pub mod physics; // Removed
pub mod plugin;
pub mod pool;
pub mod production;
pub mod replay;
pub mod result;
pub mod scenario;
pub mod scenario_comparison;
pub mod skill;
pub mod trade_agreement;
pub mod trust_network;
pub mod utils;
pub mod voting;
pub mod wizard;
pub mod wizard_helpers;

pub use asset::{Asset, AssetId, AssetType};
pub use causal_analysis::{CausalAnalysisConfig, CausalAnalysisResult, StatisticalTest};
pub use centrality::{calculate_centrality, CentralityAnalysis, NodeCentrality};
pub use config::{PresetName, SimulationConfig};
pub use contract::{Contract, ContractId};
pub use credit_rating::CreditScore;
pub use crisis::CrisisEvent;
pub use engine::{SimulationCheckpoint, SimulationEngine};
pub use entity::Entity; // This is our Person struct, wrapped for the engine
pub use environment::{Environment, Resource};
pub use error::{Result, SimulationError};
pub use event::{EventBus, EventType, SimulationEvent};
pub use externality::{Externality, ExternalityStats, SkillExternalityStats};
pub use insurance::{Insurance, InsuranceId, InsuranceType};
pub use invariant::{
    Invariant, InvariantChecker, InvariantViolation, MoneyConservationInvariant,
    NonNegativeWealthInvariant,
};
pub use investment::{Investment, InvestmentId, InvestmentType};
pub use loan::{Loan, LoanId};
pub use market::Market;
pub use parameter_sweep::{ParameterRange, ParameterSweepResult};
pub use person::{
    ClassChange, Location, Person, PersonId, SocialClass, Strategy, Transaction, TransactionType,
};
pub use plugin::{Plugin, PluginContext, PluginRegistry};
pub use pool::VecPool;
pub use production::{generate_default_recipes, Recipe};
pub use replay::{ActionLog, SimulationAction};
pub use result::{
    calculate_statistics, calculate_wealth_concentration, detect_business_cycles,
    write_step_to_stream, BusinessCycle, BusinessCycleStats, ContractStats, CyclePhase,
    IncrementalStats, MonteCarloResult, MonteCarloStats, SimulationMetadata, SimulationResult,
    SocialClassStats, StepData,
};
pub use scenario::{PriceUpdater, Scenario};
pub use scenario_comparison::ScenarioComparisonResult;
pub use skill::{Skill, SkillId};
pub use trade_agreement::{TradeAgreement, TradeAgreementStatistics};
pub use trust_network::{TrustLevel, TrustNetwork, TrustNetworkStats};
pub use voting::{
    Proposal, ProposalId, ProposalType, Vote, VotingMethod, VotingResult, VotingStatistics,
    VotingSystem,
};

#[cfg(test)]
mod tests;
