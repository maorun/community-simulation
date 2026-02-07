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
            },
            SimulationError::YamlParse(msg) => {
                write!(f, "Failed to parse YAML configuration: {}", msg)
            },
            SimulationError::TomlParse(msg) => {
                write!(f, "Failed to parse TOML configuration: {}", msg)
            },
            SimulationError::UnsupportedConfigFormat(ext) => {
                write!(
                    f,
                    "Unsupported configuration file format: '{}'. Use .yaml, .yml, or .toml",
                    ext
                )
            },
            SimulationError::ValidationError(msg) => {
                write!(f, "Configuration validation failed: {}", msg)
            },
            SimulationError::IoError(e) => {
                write!(f, "I/O error: {}", e)
            },
            SimulationError::JsonSerialize(msg) => {
                write!(f, "Failed to serialize JSON: {}", msg)
            },
            SimulationError::ActionLogWrite(e) => {
                write!(f, "Failed to write action log file: {}", e)
            },
            SimulationError::ActionLogRead(e) => {
                write!(f, "Failed to read action log file: {}", e)
            },
            SimulationError::ActionLogSerialize(e) => {
                write!(f, "Failed to serialize action log: {}", e)
            },
            SimulationError::ActionLogDeserialize(e) => {
                write!(f, "Failed to deserialize action log: {}", e)
            },
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
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_config_file_read_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = SimulationError::ConfigFileRead(io_err);
        let display = format!("{}", err);
        assert!(display.contains("Failed to read configuration file"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_yaml_parse_error_display() {
        let err = SimulationError::YamlParse("invalid yaml".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to parse YAML configuration"));
        assert!(display.contains("invalid yaml"));
    }

    #[test]
    fn test_toml_parse_error_display() {
        let err = SimulationError::TomlParse("invalid toml".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to parse TOML configuration"));
        assert!(display.contains("invalid toml"));
    }

    #[test]
    fn test_unsupported_config_format_error_display() {
        let err = SimulationError::UnsupportedConfigFormat(".json".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Unsupported configuration file format"));
        assert!(display.contains(".json"));
        assert!(display.contains("yaml"));
        assert!(display.contains("toml"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = SimulationError::ValidationError("steps must be positive".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Configuration validation failed"));
        assert!(display.contains("steps must be positive"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err = SimulationError::IoError(io_err);
        let display = format!("{}", err);
        assert!(display.contains("I/O error"));
        assert!(display.contains("permission denied"));
    }

    #[test]
    fn test_json_serialize_error_display() {
        let err = SimulationError::JsonSerialize("invalid json".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to serialize JSON"));
        assert!(display.contains("invalid json"));
    }

    #[test]
    fn test_action_log_write_error_display() {
        let io_err = io::Error::other("write failed");
        let err = SimulationError::ActionLogWrite(io_err);
        let display = format!("{}", err);
        assert!(display.contains("Failed to write action log file"));
        assert!(display.contains("write failed"));
    }

    #[test]
    fn test_action_log_read_error_display() {
        let io_err = io::Error::other("read failed");
        let err = SimulationError::ActionLogRead(io_err);
        let display = format!("{}", err);
        assert!(display.contains("Failed to read action log file"));
        assert!(display.contains("read failed"));
    }

    #[test]
    fn test_action_log_serialize_error_display() {
        // Create a JSON error by trying to serialize something that will fail
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = SimulationError::ActionLogSerialize(json_err);
        let display = format!("{}", err);
        assert!(display.contains("Failed to serialize action log"));
    }

    #[test]
    fn test_action_log_deserialize_error_display() {
        // Create a JSON error by trying to deserialize something that will fail
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = SimulationError::ActionLogDeserialize(json_err);
        let display = format!("{}", err);
        assert!(display.contains("Failed to deserialize action log"));
    }

    #[test]
    fn test_error_source_config_file_read() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let err = SimulationError::ConfigFileRead(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_io_error() {
        let io_err = io::Error::other("error");
        let err = SimulationError::IoError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_action_log_write() {
        let io_err = io::Error::other("error");
        let err = SimulationError::ActionLogWrite(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_action_log_read() {
        let io_err = io::Error::other("error");
        let err = SimulationError::ActionLogRead(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_action_log_serialize() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = SimulationError::ActionLogSerialize(json_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_action_log_deserialize() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = SimulationError::ActionLogDeserialize(json_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_none_for_string_errors() {
        let err = SimulationError::YamlParse("error".to_string());
        assert!(err.source().is_none());

        let err = SimulationError::TomlParse("error".to_string());
        assert!(err.source().is_none());

        let err = SimulationError::UnsupportedConfigFormat("error".to_string());
        assert!(err.source().is_none());

        let err = SimulationError::ValidationError("error".to_string());
        assert!(err.source().is_none());

        let err = SimulationError::JsonSerialize("error".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::other("test error");
        let sim_err: SimulationError = io_err.into();
        match sim_err {
            SimulationError::IoError(_) => {},
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_debug_trait() {
        let err = SimulationError::ValidationError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ValidationError"));
    }
}
