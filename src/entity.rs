//! Entity wrapper for persons in the simulation framework.
//!
//! This module defines the [`Entity`] struct, which wraps a [`Person`] for compatibility
//! with the simulation engine architecture. The entity abstraction allows the engine
//! to manage persons uniformly while encapsulating person-specific economic behavior.

use crate::person::{Person, PersonId as InnerPersonId, Strategy};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};

/// Type alias for entity identifiers.
///
/// Each entity has a unique ID within the simulation, which corresponds to
/// the person's ID for simplicity.
pub type EntityId = usize;

/// Represents a simulated agent in the economic system.
///
/// An entity wraps a [`Person`] and adds simulation-level state management,
/// such as whether the entity is active in the current simulation.
///
/// # Examples
///
/// ```
/// use simulation_framework::{Entity, Skill, Strategy};
///
/// let skill = Skill::new("Programming".to_string(), 50.0);
/// let entity = Entity::new(0, 100.0, vec![skill], Strategy::Balanced);
///
/// assert_eq!(entity.id, 0);
/// assert_eq!(entity.get_money(), 100.0);
/// assert!(entity.active);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier for this entity in the simulation
    pub id: EntityId,

    /// The person data, including money, skills, transactions, and reputation
    pub person_data: Person,

    /// Whether this entity is currently active in the simulation
    ///
    /// Inactive entities are not processed during simulation steps.
    pub active: bool,
}

impl Entity {
    /// Creates a new entity with the specified initial conditions.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the entity
    /// * `initial_money` - Starting money amount for the person
    /// * `own_skills` - The skills this person can provide to others
    /// * `strategy` - Behavioral strategy for spending decisions
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::{Entity, Skill, Strategy};
    ///
    /// let skill = Skill::new("Accounting".to_string(), 45.0);
    /// let entity = Entity::new(1, 200.0, vec![skill], Strategy::Balanced);
    ///
    /// assert_eq!(entity.id, 1);
    /// assert_eq!(entity.person_data.money, 200.0);
    /// ```
    pub fn new(
        id: EntityId,
        initial_money: f64,
        own_skills: Vec<Skill>,
        strategy: Strategy,
    ) -> Self {
        let person = Person::new(id as InnerPersonId, initial_money, own_skills, strategy);
        Self {
            id,
            person_data: person,
            active: true,
        }
    }

    /// Gets the current money amount for this entity's person.
    ///
    /// # Returns
    ///
    /// The amount of money the person currently has
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::{Entity, Skill, Strategy};
    ///
    /// let skill = Skill::new("Writing".to_string(), 30.0);
    /// let entity = Entity::new(0, 150.0, vec![skill], Strategy::Balanced);
    ///
    /// assert_eq!(entity.get_money(), 150.0);
    /// ```
    pub fn get_money(&self) -> f64 {
        self.person_data.money
    }
}
