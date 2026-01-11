//! Error types for the simulation framework.
//!
//! This module provides custom error types that improve error handling throughout
//! the simulation. All errors implement the standard `Error` trait and provide
//! clear, descriptive error messages.
//!
//! # Examples
//!
//! ```
//! use simulation_framework::{SimulationConfig, SimulationError};
//!
//! // Attempting to load a non-existent config file returns a descriptive error
//! let result = SimulationConfig::from_file("nonexistent.yaml");
//! match result {
//!     Ok(_) => println!("Config loaded"),
//!     Err(SimulationError::ConfigFileRead(e)) => {
//!         println!("Failed to read config file: {}", e);
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```

use std::error::Error as StdError;
use std::fmt;
use std::io;

/// Custom error type for the simulation framework.
///
/// This enum represents all possible errors that can occur during
/// simulation configuration, execution, and result output.
#[derive(Debug)]
pub enum SimulationError {
    /// Error occurred while reading or parsing a configuration file
    ConfigFileRead(io::Error),

    /// Error occurred while parsing YAML configuration
    YamlParse(String),

    /// Error occurred while parsing TOML configuration
    TomlParse(String),

    /// Configuration file has an unsupported extension
    UnsupportedConfigFormat(String),

    /// Configuration validation failed
    ValidationError(String),

    /// Error occurred while writing output files
    IoError(io::Error),

    /// Error occurred while serializing JSON output
    JsonSerialize(String),

    /// Error occurred while writing action log file
    ActionLogWrite(io::Error),

    /// Error occurred while reading action log file
    ActionLogRead(io::Error),

    /// Error occurred while serializing action log
    ActionLogSerialize(serde_json::Error),

    /// Error occurred while deserializing action log
    ActionLogDeserialize(serde_json::Error),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimulationError::ConfigFileRead(e) => {
                write!(f, "Failed to read configuration file: {}", e)
            }
            SimulationError::YamlParse(msg) => {
                write!(f, "Failed to parse YAML configuration: {}", msg)
            }
            SimulationError::TomlParse(msg) => {
                write!(f, "Failed to parse TOML configuration: {}", msg)
            }
            SimulationError::UnsupportedConfigFormat(ext) => {
                write!(
                    f,
                    "Unsupported configuration file format: '{}'. Use .yaml, .yml, or .toml",
                    ext
                )
            }
            SimulationError::ValidationError(msg) => {
                write!(f, "Configuration validation failed: {}", msg)
            }
            SimulationError::IoError(e) => {
                write!(f, "I/O error: {}", e)
            }
            SimulationError::JsonSerialize(msg) => {
                write!(f, "Failed to serialize JSON: {}", msg)
            }
            SimulationError::ActionLogWrite(e) => {
                write!(f, "Failed to write action log file: {}", e)
            }
            SimulationError::ActionLogRead(e) => {
                write!(f, "Failed to read action log file: {}", e)
            }
            SimulationError::ActionLogSerialize(e) => {
                write!(f, "Failed to serialize action log: {}", e)
            }
            SimulationError::ActionLogDeserialize(e) => {
                write!(f, "Failed to deserialize action log: {}", e)
            }
        }
    }
}

impl StdError for SimulationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SimulationError::ConfigFileRead(e)
            | SimulationError::IoError(e)
            | SimulationError::ActionLogWrite(e)
            | SimulationError::ActionLogRead(e) => Some(e),
            SimulationError::ActionLogSerialize(e) | SimulationError::ActionLogDeserialize(e) => {
                Some(e)
            }
            _ => None,
        }
    }
}

impl From<io::Error> for SimulationError {
    fn from(err: io::Error) -> Self {
        SimulationError::IoError(err)
    }
}

/// Type alias for Result with SimulationError
pub type Result<T> = std::result::Result<T, SimulationError>;
