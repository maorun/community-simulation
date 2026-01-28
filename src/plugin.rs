//! Plugin system for extending simulation functionality.
//!
//! This module provides a trait-based plugin system that allows extending
//! the simulation without modifying core code. Plugins can hook into various
//! points in the simulation lifecycle.
//!
//! # Example
//!
//! ```rust
//! use simulation_framework::plugin::{Plugin, PluginContext};
//! use std::any::Any;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str {
//!         "MyPlugin"
//!     }
//!
//!     fn on_simulation_start(&mut self, context: &PluginContext) {
//!         println!("Simulation starting with {} persons", context.config.entity_count);
//!     }
//!
//!     fn as_any(&self) -> &dyn Any {
//!         self
//!     }
//!
//!     fn as_any_mut(&mut self) -> &mut dyn Any {
//!         self
//!     }
//! }
//! ```

use crate::config::SimulationConfig;
use crate::person::Person;
use crate::result::SimulationResult;
use std::any::Any;

/// Context provided to plugins containing simulation state.
#[derive(Debug)]
pub struct PluginContext<'a> {
    /// The simulation configuration
    pub config: &'a SimulationConfig,
    /// Current simulation step
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Reference to all persons in the simulation
    pub persons: &'a [Person],
}

/// Trait that all plugins must implement.
///
/// Plugins can hook into various points in the simulation lifecycle
/// to extend functionality without modifying core code.
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Called once when the simulation is initialized.
    ///
    /// This is called before the first simulation step.
    fn on_simulation_start(&mut self, _context: &PluginContext) {}

    /// Called before each simulation step.
    ///
    /// This allows plugins to observe or modify state before trading occurs.
    fn on_step_start(&mut self, _context: &PluginContext) {}

    /// Called after each simulation step.
    ///
    /// This allows plugins to observe state changes after trading.
    fn on_step_end(&mut self, _context: &PluginContext) {}

    /// Called once when the simulation completes.
    ///
    /// This is called after all simulation steps have completed.
    fn on_simulation_end(&mut self, _context: &PluginContext, _result: &mut SimulationResult) {}

    /// Returns the plugin as Any for downcasting.
    ///
    /// This allows accessing plugin-specific methods after retrieval from registry.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the plugin as Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Registry for managing plugins.
///
/// The registry holds all registered plugins and provides methods
/// to invoke plugin hooks at appropriate points in the simulation.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Registers a plugin with the registry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use simulation_framework::plugin::{Plugin, PluginRegistry, PluginContext};
    ///
    /// struct MyPlugin;
    /// impl Plugin for MyPlugin {
    ///     fn name(&self) -> &str { "MyPlugin" }
    ///     fn as_any(&self) -> &dyn std::any::Any { self }
    ///     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    /// }
    ///
    /// let mut registry = PluginRegistry::new();
    /// registry.register(Box::new(MyPlugin));
    /// ```
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        log::info!("Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Returns the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns true if no plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Gets a reference to a plugin by name.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Gets a mutable reference to a plugin by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        for plugin in &mut self.plugins {
            if plugin.name() == name {
                return Some(plugin.as_mut());
            }
        }
        None
    }

    /// Invokes on_simulation_start for all plugins.
    pub fn on_simulation_start(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_simulation_start(context);
        }
    }

    /// Invokes on_step_start for all plugins.
    pub fn on_step_start(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_step_start(context);
        }
    }

    /// Invokes on_step_end for all plugins.
    pub fn on_step_end(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_step_end(context);
        }
    }

    /// Invokes on_simulation_end for all plugins.
    pub fn on_simulation_end(&mut self, context: &PluginContext, result: &mut SimulationResult) {
        for plugin in &mut self.plugins {
            plugin.on_simulation_end(context, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        start_called: bool,
        step_start_called: usize,
        step_end_called: usize,
        end_called: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                start_called: false,
                step_start_called: 0,
                step_end_called: 0,
                end_called: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn on_simulation_start(&mut self, _context: &PluginContext) {
            self.start_called = true;
        }

        fn on_step_start(&mut self, _context: &PluginContext) {
            self.step_start_called += 1;
        }

        fn on_step_end(&mut self, _context: &PluginContext) {
            self.step_end_called += 1;
        }

        fn on_simulation_end(&mut self, _context: &PluginContext, _result: &mut SimulationResult) {
            self.end_called = true;
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(TestPlugin::new("test1")));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        registry.register(Box::new(TestPlugin::new("test2")));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_plugin_registry_get() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin::new("test1")));
        registry.register(Box::new(TestPlugin::new("test2")));

        assert!(registry.get("test1").is_some());
        assert!(registry.get("test2").is_some());
        assert!(registry.get("test3").is_none());

        let plugin = registry.get("test1").unwrap();
        assert_eq!(plugin.name(), "test1");
    }

    #[test]
    fn test_plugin_lifecycle() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin::new("test")));

        let config = SimulationConfig {
            max_steps: 5,
            entity_count: 10,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            ..Default::default()
        };

        let context =
            PluginContext { config: &config, current_step: 0, total_steps: 5, persons: &[] };

        // Test lifecycle hooks
        registry.on_simulation_start(&context);

        for _ in 0..5 {
            registry.on_step_start(&context);
            registry.on_step_end(&context);
        }

        // Verify calls without needing SimulationResult
        let plugin = registry.get_mut("test").unwrap();
        let test_plugin = plugin.as_any_mut().downcast_mut::<TestPlugin>().unwrap();

        assert!(test_plugin.start_called);
        assert_eq!(test_plugin.step_start_called, 5);
        assert_eq!(test_plugin.step_end_called, 5);
    }
}
