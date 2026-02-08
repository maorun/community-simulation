//! Event system for tracking simulation events
//!
//! This module provides a simple event-based architecture for tracking important
//! occurrences during the simulation. Events can be collected and analyzed to understand
//! simulation dynamics, debug issues, or export detailed timelines.
//!
//! # Overview
//!
//! The event system is designed to be:
//! - **Lightweight**: Minimal performance overhead when disabled
//! - **Optional**: Event collection is opt-in via configuration
//! - **Extensible**: New event types can be added easily
//! - **Non-invasive**: Doesn't require major refactoring of existing code
//!
//! # Event Types
//!
//! The system tracks several types of simulation events:
//! - **TradeExecuted**: When a successful trade occurs between two persons
//! - **PriceUpdated**: When a skill's price changes in the market
//! - **ReputationChanged**: When a person's reputation changes
//! - **StepCompleted**: When a simulation step finishes
//!
//! # Usage
//!
//! ```ignore
//! use community_simulation::{SimulationConfig, SimulationEngine};
//!
//! // Create configuration with event tracking enabled
//! let config = SimulationConfig {
//!     max_steps: 100,
//!     entity_count: 10,
//!     enable_events: true,
//!     ..Default::default()
//! };
//!
//! // Run simulation
//! let mut engine = SimulationEngine::new(config);
//! let result = engine.run();
//!
//! // Access collected events (once implemented)
//! // Events will be accessible via the simulation result
//! // for detailed analysis and debugging
//! ```

use crate::{PersonId, SkillId};
use serde::{Deserialize, Serialize};

/// A simulation event that occurred during execution
///
/// Events are timestamped with the simulation step number and contain
/// specific data about what happened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationEvent {
    /// The simulation step when this event occurred
    pub step: usize,
    /// The type and details of the event
    pub event_type: EventType,
}

/// Types of events that can occur during simulation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventType {
    /// A trade was executed between two persons
    TradeExecuted {
        /// Person ID of the buyer
        buyer_id: PersonId,
        /// Person ID of the seller
        seller_id: PersonId,
        /// Skill that was traded
        skill_id: SkillId,
        /// Price paid for the skill
        price: f64,
    },
    /// A skill's price was updated in the market
    PriceUpdated {
        /// Skill whose price changed
        skill_id: SkillId,
        /// Old price before update
        old_price: f64,
        /// New price after update
        new_price: f64,
    },
    /// A person's reputation changed
    ReputationChanged {
        /// Person whose reputation changed
        person_id: PersonId,
        /// Old reputation value
        old_reputation: f64,
        /// New reputation value
        new_reputation: f64,
    },
    /// A simulation step completed
    StepCompleted {
        /// Step number that completed
        step_number: usize,
        /// Number of trades in this step
        trades_count: usize,
        /// Total trade volume in this step
        trade_volume: f64,
    },
}

/// Event bus for collecting and managing simulation events
///
/// This struct collects events during simulation execution and provides
/// methods for querying and analyzing them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventBus {
    /// Collection of all events
    events: Vec<SimulationEvent>,
    /// Whether event collection is enabled
    enabled: bool,
}

impl EventBus {
    /// Create a new event bus
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to actually collect events. If false, all emit
    ///   operations are no-ops for zero performance overhead.
    pub fn new(enabled: bool) -> Self {
        Self { events: Vec::new(), enabled }
    }

    /// Emit a trade executed event
    ///
    /// Records that a trade occurred between two persons.
    ///
    /// # Arguments
    ///
    /// * `step` - Current simulation step
    /// * `buyer_id` - ID of the buyer
    /// * `seller_id` - ID of the seller
    /// * `skill_id` - Skill that was traded
    /// * `price` - Price paid
    pub fn emit_trade(
        &mut self,
        step: usize,
        buyer_id: PersonId,
        seller_id: PersonId,
        skill_id: SkillId,
        price: f64,
    ) {
        if !self.enabled {
            return;
        }
        self.events.push(SimulationEvent {
            step,
            event_type: EventType::TradeExecuted { buyer_id, seller_id, skill_id, price },
        });
    }

    /// Emit a price updated event
    ///
    /// Records that a skill's price changed in the market.
    ///
    /// # Arguments
    ///
    /// * `step` - Current simulation step
    /// * `skill_id` - Skill whose price changed
    /// * `old_price` - Price before update
    /// * `new_price` - Price after update
    pub fn emit_price_update(
        &mut self,
        step: usize,
        skill_id: SkillId,
        old_price: f64,
        new_price: f64,
    ) {
        if !self.enabled {
            return;
        }
        // Only emit if price actually changed
        if (old_price - new_price).abs() > f64::EPSILON {
            self.events.push(SimulationEvent {
                step,
                event_type: EventType::PriceUpdated { skill_id, old_price, new_price },
            });
        }
    }

    /// Emit a reputation changed event
    ///
    /// Records that a person's reputation changed.
    ///
    /// # Arguments
    ///
    /// * `step` - Current simulation step
    /// * `person_id` - Person whose reputation changed
    /// * `old_reputation` - Reputation before change
    /// * `new_reputation` - Reputation after change
    pub fn emit_reputation_change(
        &mut self,
        step: usize,
        person_id: PersonId,
        old_reputation: f64,
        new_reputation: f64,
    ) {
        if !self.enabled {
            return;
        }
        // Only emit if reputation actually changed
        if (old_reputation - new_reputation).abs() > f64::EPSILON {
            self.events.push(SimulationEvent {
                step,
                event_type: EventType::ReputationChanged {
                    person_id,
                    old_reputation,
                    new_reputation,
                },
            });
        }
    }

    /// Emit a step completed event
    ///
    /// Records that a simulation step finished.
    ///
    /// # Arguments
    ///
    /// * `step_number` - Step that completed
    /// * `trades_count` - Number of trades in this step
    /// * `trade_volume` - Total money exchanged in this step
    pub fn emit_step_completed(
        &mut self,
        step_number: usize,
        trades_count: usize,
        trade_volume: f64,
    ) {
        if !self.enabled {
            return;
        }
        self.events.push(SimulationEvent {
            step: step_number,
            event_type: EventType::StepCompleted { step_number, trades_count, trade_volume },
        });
    }

    /// Get all collected events
    pub fn events(&self) -> &[SimulationEvent] {
        &self.events
    }

    /// Get the number of collected events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the event bus is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Check if event collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get event statistics by type
    ///
    /// Returns a tuple of (trade_count, price_update_count, reputation_change_count, step_completed_count)
    pub fn event_counts(&self) -> (usize, usize, usize, usize) {
        let mut trades = 0;
        let mut price_updates = 0;
        let mut reputation_changes = 0;
        let mut steps_completed = 0;

        for event in &self.events {
            match event.event_type {
                EventType::TradeExecuted { .. } => trades += 1,
                EventType::PriceUpdated { .. } => price_updates += 1,
                EventType::ReputationChanged { .. } => reputation_changes += 1,
                EventType::StepCompleted { .. } => steps_completed += 1,
            }
        }

        (trades, price_updates, reputation_changes, steps_completed)
    }

    /// Clear all collected events
    ///
    /// Useful for resetting the event bus between simulation runs.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let bus = EventBus::new(true);
        assert!(bus.is_enabled());
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());

        let bus_disabled = EventBus::new(false);
        assert!(!bus_disabled.is_enabled());
    }

    #[test]
    fn test_emit_trade_event() {
        let mut bus = EventBus::new(true);
        let buyer = 1; // PersonId is usize
        let seller = 2;
        let skill = "Skill5".to_string(); // SkillId is String

        bus.emit_trade(10, buyer, seller, skill.clone(), 15.0);

        assert_eq!(bus.len(), 1);
        let events = bus.events();
        assert_eq!(events[0].step, 10);
        match &events[0].event_type {
            EventType::TradeExecuted { buyer_id, seller_id, skill_id, price } => {
                assert_eq!(*buyer_id, buyer);
                assert_eq!(*seller_id, seller);
                assert_eq!(*skill_id, skill);
                assert_eq!(*price, 15.0);
            },
            _ => panic!("Expected TradeExecuted event"),
        }
    }

    #[test]
    fn test_emit_price_update_event() {
        let mut bus = EventBus::new(true);
        let skill = "Skill3".to_string(); // SkillId is String

        bus.emit_price_update(5, skill.clone(), 10.0, 12.0);

        assert_eq!(bus.len(), 1);
        let events = bus.events();
        match &events[0].event_type {
            EventType::PriceUpdated { skill_id, old_price, new_price } => {
                assert_eq!(*skill_id, skill);
                assert_eq!(*old_price, 10.0);
                assert_eq!(*new_price, 12.0);
            },
            _ => panic!("Expected PriceUpdated event"),
        }
    }

    #[test]
    fn test_emit_reputation_change_event() {
        let mut bus = EventBus::new(true);
        let person = 7; // PersonId is usize

        bus.emit_reputation_change(20, person, 1.0, 1.05);

        assert_eq!(bus.len(), 1);
        let events = bus.events();
        match &events[0].event_type {
            EventType::ReputationChanged { person_id, old_reputation, new_reputation } => {
                assert_eq!(*person_id, person);
                assert_eq!(*old_reputation, 1.0);
                assert_eq!(*new_reputation, 1.05);
            },
            _ => panic!("Expected ReputationChanged event"),
        }
    }

    #[test]
    fn test_emit_step_completed_event() {
        let mut bus = EventBus::new(true);

        bus.emit_step_completed(50, 25, 300.0);

        assert_eq!(bus.len(), 1);
        let events = bus.events();
        match &events[0].event_type {
            EventType::StepCompleted { step_number, trades_count, trade_volume } => {
                assert_eq!(*step_number, 50);
                assert_eq!(*trades_count, 25);
                assert_eq!(*trade_volume, 300.0);
            },
            _ => panic!("Expected StepCompleted event"),
        }
    }

    #[test]
    fn test_disabled_event_bus() {
        let mut bus = EventBus::new(false);

        // Emit various events - none should be collected
        bus.emit_trade(1, 1, 2, "Skill1".to_string(), 10.0);
        bus.emit_price_update(1, "Skill1".to_string(), 10.0, 11.0);
        bus.emit_reputation_change(1, 1, 1.0, 1.05);
        bus.emit_step_completed(1, 5, 50.0);

        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_event_counts() {
        let mut bus = EventBus::new(true);

        // Emit various events
        bus.emit_trade(1, 1, 2, "Skill1".to_string(), 10.0);
        bus.emit_trade(1, 3, 4, "Skill2".to_string(), 15.0);
        bus.emit_price_update(1, "Skill1".to_string(), 10.0, 11.0);
        bus.emit_reputation_change(1, 1, 1.0, 1.05);
        bus.emit_step_completed(1, 2, 25.0);

        let (trades, price_updates, reputation_changes, steps) = bus.event_counts();
        assert_eq!(trades, 2);
        assert_eq!(price_updates, 1);
        assert_eq!(reputation_changes, 1);
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_clear_events() {
        let mut bus = EventBus::new(true);

        bus.emit_trade(1, 1, 2, "Skill1".to_string(), 10.0);
        assert_eq!(bus.len(), 1);

        bus.clear();
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_no_emit_on_zero_change() {
        let mut bus = EventBus::new(true);

        // Price update with no actual change
        bus.emit_price_update(1, "Skill1".to_string(), 10.0, 10.0);
        assert_eq!(bus.len(), 0); // Should not emit event

        // Reputation change with no actual change
        bus.emit_reputation_change(1, 1, 1.0, 1.0);
        assert_eq!(bus.len(), 0); // Should not emit event
    }
}
