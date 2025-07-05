// This file now defines the main simulated agent for our economic simulation.
// To maintain compatibility with parts of the existing framework (engine, results)
// that expect an `Entity`, we are defining `Entity` here as our `Person`.

use crate::person::{Person, PersonId as InnerPersonId}; // Import the Person struct
use crate::skill::Skill; // Required for Person initialization, SkillId removed
use serde::{Deserialize, Serialize};

pub type EntityId = usize; // Consistent with PersonId

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    // These fields should mirror what SimulationEngine and SimulationResult expect
    // from an "Entity", and also what's necessary for our economic sim.
    pub id: EntityId, // This is the simulation-wide entity ID
    pub person_data: Person, // Encapsulates Person data
    pub active: bool, // Still useful to mark if a person is active in the simulation
}

impl Entity {
    // Constructor that the engine will use.
    // It now needs enough info to create a Person.
    // The `initialize_entities` method in `SimulationEngine` will need to be updated
    // to call this constructor with appropriate Person data.
    pub fn new(id: EntityId, initial_money: f64, own_skill: Skill) -> Self {
        let person = Person::new(id as InnerPersonId, initial_money, own_skill);
        Self {
            id,
            person_data: person,
            active: true,
        }
    }

    // Example of how one might provide access to Person's properties or methods
    // For instance, if external parts of the simulation needed to check money:
    pub fn get_money(&self) -> f64 {
        self.person_data.money
    }

    // The old `update` method which took physics `forces` is no longer relevant here.
    // Entity state changes (money, skills) will be managed by the economic logic
    // in the SimulationEngine's `step` method.
}

// The old Vector3 and EntityState are removed as they are physics-specific.
// If any generic vector math or state representation is needed later,
// it can be added back in a more generic form.
