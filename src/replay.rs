use crate::error::{Result, SimulationError};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// Represents a single action in the simulation that can be logged and replayed.
///
/// This enum captures the key decisions and events that occur during simulation,
/// allowing for reproducible execution and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationAction {
    /// A trade occurred between two entities
    Trade { step: usize, buyer_id: usize, seller_id: usize, skill_id: String, price: f64 },
    /// A trade was attempted but failed due to insufficient funds
    FailedTrade { step: usize, buyer_id: usize, seller_id: usize, skill_id: String, price: f64 },
    /// Prices were updated in the market
    PriceUpdate { step: usize, skill_id: String, old_price: f64, new_price: f64 },
    /// A crisis event occurred
    CrisisEvent { step: usize, event_type: String, severity: f64 },
}

/// Action log for recording simulation events.
///
/// This structure maintains a list of actions that occurred during simulation,
/// which can be saved to disk for later replay or analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLog {
    /// The seed used for the simulation's RNG
    pub seed: u64,
    /// Number of entities in the simulation
    pub entity_count: usize,
    /// Maximum steps in the simulation
    pub max_steps: usize,
    /// List of all actions that occurred
    pub actions: Vec<SimulationAction>,
}

impl ActionLog {
    /// Creates a new empty action log with the given simulation parameters.
    pub fn new(seed: u64, entity_count: usize, max_steps: usize) -> Self {
        ActionLog { seed, entity_count, max_steps, actions: Vec::new() }
    }

    /// Records a single action in the log.
    pub fn record(&mut self, action: SimulationAction) {
        self.actions.push(action);
    }

    /// Saves the action log to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path where the action log should be saved
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(SimulationError)` if file I/O fails
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path).map_err(SimulationError::ActionLogWrite)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)
            .map_err(SimulationError::ActionLogSerialize)?;
        // Explicitly flush to ensure all data is written to disk
        writer.flush().map_err(SimulationError::ActionLogWrite)?;
        Ok(())
    }

    /// Loads an action log from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the action log file
    ///
    /// # Returns
    /// * `Ok(ActionLog)` on success
    /// * `Err(SimulationError)` if file I/O or parsing fails
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path).map_err(SimulationError::ActionLogRead)?;
        // Use BufReader for efficient reading of potentially large action logs
        let reader = BufReader::new(file);
        let log = serde_json::from_reader(reader).map_err(SimulationError::ActionLogDeserialize)?;
        Ok(log)
    }

    /// Returns the total number of recorded actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns true if the log contains no actions.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_log_creation() {
        let log = ActionLog::new(42, 100, 500);
        assert_eq!(log.seed, 42);
        assert_eq!(log.entity_count, 100);
        assert_eq!(log.max_steps, 500);
        assert!(log.is_empty());
    }

    #[test]
    fn test_record_action() {
        let mut log = ActionLog::new(42, 100, 500);
        log.record(SimulationAction::Trade {
            step: 1,
            buyer_id: 0,
            seller_id: 1,
            skill_id: "Skill1".to_string(),
            price: 10.0,
        });
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let mut log = ActionLog::new(42, 100, 500);
        log.record(SimulationAction::Trade {
            step: 1,
            buyer_id: 0,
            seller_id: 1,
            skill_id: "Skill1".to_string(),
            price: 10.0,
        });
        log.record(SimulationAction::FailedTrade {
            step: 2,
            buyer_id: 2,
            seller_id: 3,
            skill_id: "Skill2".to_string(),
            price: 15.0,
        });

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save the log
        log.save_to_file(path).unwrap();

        // Load it back
        let loaded_log = ActionLog::load_from_file(path).unwrap();

        assert_eq!(loaded_log.seed, log.seed);
        assert_eq!(loaded_log.entity_count, log.entity_count);
        assert_eq!(loaded_log.max_steps, log.max_steps);
        assert_eq!(loaded_log.len(), log.len());
    }

    #[test]
    fn test_multiple_action_types() {
        let mut log = ActionLog::new(42, 100, 500);

        log.record(SimulationAction::Trade {
            step: 1,
            buyer_id: 0,
            seller_id: 1,
            skill_id: "Skill1".to_string(),
            price: 10.0,
        });

        log.record(SimulationAction::PriceUpdate {
            step: 1,
            skill_id: "Skill1".to_string(),
            old_price: 10.0,
            new_price: 11.0,
        });

        log.record(SimulationAction::CrisisEvent {
            step: 10,
            event_type: "MarketCrash".to_string(),
            severity: 0.5,
        });

        assert_eq!(log.len(), 3);
    }
}
