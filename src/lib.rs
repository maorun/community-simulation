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
//! use simulation_framework::{SimulationConfig, SimulationEngine};
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
//! - [`config`] - Simulation configuration parameters
//! - [`engine`] - Main simulation engine and execution loop
//! - [`entity`] - Entity wrapper around Person for simulation framework
//! - [`market`] - Market mechanisms and price dynamics
//! - [`person`] - Person agents, transactions, and behavior
//! - [`result`] - Simulation results, statistics, and output formatting
//! - [`scenario`] - Price update strategies for different simulation scenarios
//! - [`skill`] - Skill definitions and generation

pub mod config;
pub mod engine;
pub mod entity; // Represents a Person in the simulation
pub mod market;
pub mod person;
// pub mod physics; // Removed
pub mod result;
pub mod scenario;
pub mod skill;

pub use config::SimulationConfig;
pub use engine::SimulationEngine;
pub use entity::Entity; // This is our Person struct, wrapped for the engine
pub use market::Market;
pub use person::{Person, PersonId, Transaction, TransactionType};
pub use result::SimulationResult;
pub use scenario::{PriceUpdater, Scenario};
pub use skill::{Skill, SkillId};

#[cfg(test)]
mod tests;
