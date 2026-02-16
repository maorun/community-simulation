//! Component-based architecture foundation for extensible person capabilities.
//!
//! This module provides a trait-based component system that enables modular extension
//! of person behavior without monolithic struct modifications. Components represent
//! specific capabilities or behaviors that can be attached to persons dynamically.
//!
//! # Design Philosophy
//!
//! The component system follows these principles:
//! - **Modularity**: Each component encapsulates a single concern or capability
//! - **Composability**: Multiple components can be combined for complex behaviors
//! - **Extensibility**: New components can be added without modifying existing code
//! - **Optional**: Components are opt-in and don't affect existing functionality
//!
//! # Architecture
//!
//! The system consists of:
//! - [`Component`] trait: Defines the interface for all components
//! - [`ComponentContainer`]: Stores and manages components for a person
//! - Concrete component implementations: Specific behavioral capabilities
//!
//! # Example
//!
//! ```
//! use community_simulation::component::{Component, ComponentContainer, TradingBehaviorComponent};
//!
//! // Create a component container
//! let mut container = ComponentContainer::new();
//!
//! // Add a component
//! let trading_component = TradingBehaviorComponent::new(0.8);
//! container.add_component(Box::new(trading_component));
//!
//! // Check if a component exists
//! assert!(container.has_component("TradingBehavior"));
//! assert_eq!(container.count(), 1);
//! ```
//!
//! # Future Extensions
//!
//! This foundation enables future components such as:
//! - Risk assessment components
//! - Learning and adaptation components
//! - Social behavior components
//! - Market strategy components
//! - Resource management components

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// Core trait for all components in the simulation.
///
/// Components are modular capabilities that can be attached to persons to extend
/// their behavior. Each component has a unique identifier and can maintain state
/// specific to its function.
///
/// # Implementation Guidelines
///
/// - Keep components focused on a single responsibility
/// - Use meaningful, descriptive identifiers
/// - Implement Clone when component state should be duplicable
/// - Document the purpose and interaction patterns clearly
///
/// # Examples
///
/// ```
/// use community_simulation::component::Component;
/// use std::any::Any;
///
/// #[derive(Clone)]
/// struct CustomComponent {
///     value: f64,
/// }
///
/// impl Component for CustomComponent {
///     fn identifier(&self) -> &str {
///         "Custom"
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn Any {
///         self
///     }
///
///     fn clone_box(&self) -> Box<dyn Component> {
///         Box::new(self.clone())
///     }
/// }
/// ```
pub trait Component: Send + Sync {
    /// Returns the unique identifier for this component type.
    ///
    /// The identifier should be:
    /// - Unique across all component types
    /// - Descriptive of the component's purpose
    /// - Consistent across instances of the same type
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{Component, TradingBehaviorComponent};
    ///
    /// let component = TradingBehaviorComponent::new(0.5);
    /// assert_eq!(component.identifier(), "TradingBehavior");
    /// ```
    fn identifier(&self) -> &str;

    /// Returns a reference to self as `Any` for downcasting.
    ///
    /// This enables type-safe downcasting to concrete component types
    /// when specific behavior or data access is needed.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to self as `Any` for downcasting.
    ///
    /// This enables type-safe mutable access to concrete component types.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Creates a boxed clone of this component.
    ///
    /// Required because trait objects don't automatically support Clone.
    /// Implementations should delegate to their concrete type's Clone impl.
    fn clone_box(&self) -> Box<dyn Component>;
}

/// Container for managing multiple components attached to a person.
///
/// The container provides efficient storage and retrieval of components by their
/// identifier. It supports adding, removing, and querying components at runtime.
///
/// # Thread Safety
///
/// ComponentContainer is designed to be serializable and can be safely shared
/// across threads when the contained components are Send + Sync.
///
/// # Examples
///
/// ```
/// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
///
/// let mut container = ComponentContainer::new();
///
/// // Add components
/// container.add_component(Box::new(TradingBehaviorComponent::new(0.7)));
///
/// // Query components
/// if let Some(trading) = container.get_component("TradingBehavior") {
///     // Downcast to concrete type if needed
///     if let Some(trading_concrete) = trading.as_any().downcast_ref::<TradingBehaviorComponent>() {
///         let risk = trading_concrete.risk_tolerance();
///         assert_eq!(risk, 0.7);
///     }
/// }
///
/// // Remove components
/// assert!(container.remove_component("TradingBehavior").is_some());
/// assert_eq!(container.count(), 0);
/// ```
#[derive(Default)]
pub struct ComponentContainer {
    components: HashMap<String, Box<dyn Component>>,
}

impl ComponentContainer {
    /// Creates a new empty component container.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::ComponentContainer;
    ///
    /// let container = ComponentContainer::new();
    /// assert_eq!(container.count(), 0);
    /// ```
    pub fn new() -> Self {
        Self { components: HashMap::new() }
    }

    /// Adds a component to the container.
    ///
    /// If a component with the same identifier already exists, it will be replaced
    /// and the old component will be returned.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to add
    ///
    /// # Returns
    ///
    /// The previously stored component with the same identifier, if any
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// let old = container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    /// assert!(old.is_none());
    ///
    /// let old = container.add_component(Box::new(TradingBehaviorComponent::new(0.8)));
    /// assert!(old.is_some());
    /// ```
    pub fn add_component(&mut self, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        let id = component.identifier().to_string();
        self.components.insert(id, component)
    }

    /// Retrieves a component by its identifier.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The unique identifier of the component to retrieve
    ///
    /// # Returns
    ///
    /// A reference to the component if found, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    ///
    /// assert!(container.get_component("TradingBehavior").is_some());
    /// assert!(container.get_component("NonExistent").is_none());
    /// ```
    pub fn get_component(&self, identifier: &str) -> Option<&dyn Component> {
        self.components.get(identifier).map(|c| c.as_ref())
    }

    /// Retrieves a mutable reference to a component by its identifier.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The unique identifier of the component to retrieve
    ///
    /// # Returns
    ///
    /// A mutable reference to the component if found, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    ///
    /// if let Some(component) = container.get_component_mut("TradingBehavior") {
    ///     // Can modify the component
    ///     if let Some(trading) = component.as_any_mut().downcast_mut::<TradingBehaviorComponent>() {
    ///         trading.set_risk_tolerance(0.9);
    ///     }
    /// }
    /// ```
    pub fn get_component_mut(&mut self, identifier: &str) -> Option<&mut (dyn Component + '_)> {
        match self.components.get_mut(identifier) {
            Some(boxed) => Some(boxed.as_mut()),
            None => None,
        }
    }

    /// Removes a component from the container.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The unique identifier of the component to remove
    ///
    /// # Returns
    ///
    /// The removed component if it existed, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    ///
    /// assert!(container.remove_component("TradingBehavior").is_some());
    /// assert!(container.remove_component("TradingBehavior").is_none());
    /// ```
    pub fn remove_component(&mut self, identifier: &str) -> Option<Box<dyn Component>> {
        self.components.remove(identifier)
    }

    /// Checks if a component with the given identifier exists.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The unique identifier to check
    ///
    /// # Returns
    ///
    /// `true` if a component with that identifier exists, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// assert!(!container.has_component("TradingBehavior"));
    ///
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    /// assert!(container.has_component("TradingBehavior"));
    /// ```
    pub fn has_component(&self, identifier: &str) -> bool {
        self.components.contains_key(identifier)
    }

    /// Returns the number of components in the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// assert_eq!(container.count(), 0);
    ///
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    /// assert_eq!(container.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// Returns an iterator over the component identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    ///
    /// let ids: Vec<&str> = container.identifiers().collect();
    /// assert_eq!(ids, vec!["TradingBehavior"]);
    /// ```
    pub fn identifiers(&self) -> impl Iterator<Item = &str> {
        self.components.keys().map(|s| s.as_str())
    }

    /// Clears all components from the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::{ComponentContainer, TradingBehaviorComponent};
    ///
    /// let mut container = ComponentContainer::new();
    /// container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
    /// assert_eq!(container.count(), 1);
    ///
    /// container.clear();
    /// assert_eq!(container.count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.components.clear();
    }
}

// Manual Clone implementation since Box<dyn Component> doesn't auto-derive Clone
impl Clone for ComponentContainer {
    fn clone(&self) -> Self {
        let mut components = HashMap::new();
        for (id, component) in &self.components {
            components.insert(id.clone(), component.clone_box());
        }
        Self { components }
    }
}

// ComponentContainer is not directly serializable due to trait objects,
// but we can implement a workaround if needed in the future using type registry
impl std::fmt::Debug for ComponentContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentContainer")
            .field("component_count", &self.components.len())
            .field("identifiers", &self.components.keys().collect::<Vec<_>>())
            .finish()
    }
}

// Example component implementation
/// A simple trading behavior component that modifies trading decisions.
///
/// This component demonstrates how to implement the Component trait and
/// provides a foundation for more sophisticated trading strategies.
///
/// # Fields
///
/// * `risk_tolerance` - How willing the person is to take risks (0.0-1.0)
///   - 0.0: Very risk-averse, only buys essentials
///   - 0.5: Balanced risk approach
///   - 1.0: Risk-seeking, willing to spend aggressively
///
/// # Examples
///
/// ```
/// use community_simulation::component::{Component, TradingBehaviorComponent};
///
/// let mut component = TradingBehaviorComponent::new(0.7);
/// assert_eq!(component.risk_tolerance(), 0.7);
///
/// component.set_risk_tolerance(0.9);
/// assert_eq!(component.risk_tolerance(), 0.9);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingBehaviorComponent {
    /// Risk tolerance level (0.0 = risk-averse, 1.0 = risk-seeking)
    risk_tolerance: f64,
}

impl TradingBehaviorComponent {
    /// Creates a new trading behavior component with the specified risk tolerance.
    ///
    /// # Arguments
    ///
    /// * `risk_tolerance` - Risk level between 0.0 (risk-averse) and 1.0 (risk-seeking)
    ///
    /// # Panics
    ///
    /// Panics if risk_tolerance is not in the range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::TradingBehaviorComponent;
    ///
    /// let component = TradingBehaviorComponent::new(0.5);
    /// assert_eq!(component.risk_tolerance(), 0.5);
    /// ```
    pub fn new(risk_tolerance: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&risk_tolerance),
            "Risk tolerance must be between 0.0 and 1.0"
        );
        Self { risk_tolerance }
    }

    /// Returns the current risk tolerance level.
    ///
    /// # Returns
    ///
    /// The risk tolerance value between 0.0 and 1.0
    pub fn risk_tolerance(&self) -> f64 {
        self.risk_tolerance
    }

    /// Updates the risk tolerance level.
    ///
    /// # Arguments
    ///
    /// * `new_tolerance` - New risk tolerance value between 0.0 and 1.0
    ///
    /// # Panics
    ///
    /// Panics if new_tolerance is not in the range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::TradingBehaviorComponent;
    ///
    /// let mut component = TradingBehaviorComponent::new(0.5);
    /// component.set_risk_tolerance(0.8);
    /// assert_eq!(component.risk_tolerance(), 0.8);
    /// ```
    pub fn set_risk_tolerance(&mut self, new_tolerance: f64) {
        assert!(
            (0.0..=1.0).contains(&new_tolerance),
            "Risk tolerance must be between 0.0 and 1.0"
        );
        self.risk_tolerance = new_tolerance;
    }

    /// Calculates a spending multiplier based on risk tolerance.
    ///
    /// This can be used to modify purchasing behavior:
    /// - Low risk tolerance (0.0) → 0.5x spending multiplier (conservative)
    /// - Medium risk tolerance (0.5) → 1.0x spending multiplier (neutral)
    /// - High risk tolerance (1.0) → 1.5x spending multiplier (aggressive)
    ///
    /// # Returns
    ///
    /// A multiplier between 0.5 and 1.5 based on risk tolerance
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::component::TradingBehaviorComponent;
    ///
    /// let conservative = TradingBehaviorComponent::new(0.0);
    /// assert_eq!(conservative.spending_multiplier(), 0.5);
    ///
    /// let balanced = TradingBehaviorComponent::new(0.5);
    /// assert_eq!(balanced.spending_multiplier(), 1.0);
    ///
    /// let aggressive = TradingBehaviorComponent::new(1.0);
    /// assert_eq!(aggressive.spending_multiplier(), 1.5);
    /// ```
    pub fn spending_multiplier(&self) -> f64 {
        // Map risk tolerance [0.0, 1.0] to spending multiplier [0.5, 1.5]
        0.5 + self.risk_tolerance
    }
}

impl Component for TradingBehaviorComponent {
    fn identifier(&self) -> &str {
        "TradingBehavior"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_container_basic() {
        let mut container = ComponentContainer::new();
        assert_eq!(container.count(), 0);

        let component = Box::new(TradingBehaviorComponent::new(0.5));
        container.add_component(component);

        assert_eq!(container.count(), 1);
        assert!(container.has_component("TradingBehavior"));
    }

    #[test]
    fn test_component_retrieval() {
        let mut container = ComponentContainer::new();
        let component = Box::new(TradingBehaviorComponent::new(0.7));
        container.add_component(component);

        let retrieved = container.get_component("TradingBehavior");
        assert!(retrieved.is_some());

        if let Some(component) = retrieved {
            let trading = component.as_any().downcast_ref::<TradingBehaviorComponent>();
            assert!(trading.is_some());
            assert_eq!(trading.unwrap().risk_tolerance(), 0.7);
        }
    }

    #[test]
    fn test_component_mutation() {
        let mut container = ComponentContainer::new();
        container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));

        if let Some(component) = container.get_component_mut("TradingBehavior") {
            if let Some(trading) = component.as_any_mut().downcast_mut::<TradingBehaviorComponent>()
            {
                trading.set_risk_tolerance(0.9);
            }
        }

        let retrieved = container.get_component("TradingBehavior").unwrap();
        let trading = retrieved.as_any().downcast_ref::<TradingBehaviorComponent>().unwrap();
        assert_eq!(trading.risk_tolerance(), 0.9);
    }

    #[test]
    fn test_component_removal() {
        let mut container = ComponentContainer::new();
        container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));

        assert!(container.has_component("TradingBehavior"));
        let removed = container.remove_component("TradingBehavior");
        assert!(removed.is_some());
        assert!(!container.has_component("TradingBehavior"));
        assert_eq!(container.count(), 0);
    }

    #[test]
    fn test_component_replacement() {
        let mut container = ComponentContainer::new();

        let old = container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
        assert!(old.is_none());

        let old = container.add_component(Box::new(TradingBehaviorComponent::new(0.8)));
        assert!(old.is_some());

        let current = container.get_component("TradingBehavior").unwrap();
        let trading = current.as_any().downcast_ref::<TradingBehaviorComponent>().unwrap();
        assert_eq!(trading.risk_tolerance(), 0.8);
    }

    #[test]
    fn test_trading_behavior_component() {
        let component = TradingBehaviorComponent::new(0.6);
        assert_eq!(component.risk_tolerance(), 0.6);
        assert_eq!(component.spending_multiplier(), 1.1);
    }

    #[test]
    fn test_spending_multiplier_bounds() {
        let conservative = TradingBehaviorComponent::new(0.0);
        assert_eq!(conservative.spending_multiplier(), 0.5);

        let balanced = TradingBehaviorComponent::new(0.5);
        assert_eq!(balanced.spending_multiplier(), 1.0);

        let aggressive = TradingBehaviorComponent::new(1.0);
        assert_eq!(aggressive.spending_multiplier(), 1.5);
    }

    #[test]
    #[should_panic(expected = "Risk tolerance must be between 0.0 and 1.0")]
    fn test_invalid_risk_tolerance_high() {
        TradingBehaviorComponent::new(1.5);
    }

    #[test]
    #[should_panic(expected = "Risk tolerance must be between 0.0 and 1.0")]
    fn test_invalid_risk_tolerance_low() {
        TradingBehaviorComponent::new(-0.1);
    }

    #[test]
    fn test_component_container_clone() {
        let mut container = ComponentContainer::new();
        container.add_component(Box::new(TradingBehaviorComponent::new(0.7)));

        let cloned = container.clone();
        assert_eq!(cloned.count(), 1);
        assert!(cloned.has_component("TradingBehavior"));

        let component = cloned.get_component("TradingBehavior").unwrap();
        let trading = component.as_any().downcast_ref::<TradingBehaviorComponent>().unwrap();
        assert_eq!(trading.risk_tolerance(), 0.7);
    }

    #[test]
    fn test_component_container_clear() {
        let mut container = ComponentContainer::new();
        container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));
        assert_eq!(container.count(), 1);

        container.clear();
        assert_eq!(container.count(), 0);
        assert!(!container.has_component("TradingBehavior"));
    }

    #[test]
    fn test_component_identifiers() {
        let mut container = ComponentContainer::new();
        container.add_component(Box::new(TradingBehaviorComponent::new(0.5)));

        let ids: Vec<&str> = container.identifiers().collect();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"TradingBehavior"));
    }
}
