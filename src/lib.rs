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
