pub mod config;
pub mod engine;
pub mod entity;
pub mod physics;
pub mod result;

pub use config::SimulationConfig;
pub use engine::SimulationEngine;
pub use entity::{Entity, EntityId, EntityState};
pub use result::SimulationResult;