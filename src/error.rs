//! Error types for the simulation framework.
//!
//! This module provides custom error types that improve error handling throughout
//! the simulation. All errors implement the standard `Error` trait and provide
//! clear, descriptive error messages.
//!
//! # Examples
//!
//! ```
//! use community_simulation::{SimulationConfig, SimulationError};
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

/// Category of a simulation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCategory {
    /// Configuration related errors
    Configuration,
    /// Parsing related errors
    Parsing,
    /// Validation related errors
    Validation,
    /// I/O related errors
    Io,
    /// Serialization related errors
    Serialization,
    /// Storage related errors (action log)
    Storage,
    /// Export related errors (Parquet)
    Export,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Configuration => write!(f, "CONFIGURATION"),
            ErrorCategory::Parsing => write!(f, "PARSING"),
            ErrorCategory::Validation => write!(f, "VALIDATION"),
            ErrorCategory::Io => write!(f, "IO"),
            ErrorCategory::Serialization => write!(f, "SERIALIZATION"),
            ErrorCategory::Storage => write!(f, "STORAGE"),
            ErrorCategory::Export => write!(f, "EXPORT"),
        }
    }
}

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

    /// Error occurred while exporting to Parquet format
    ParquetExport(String),
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
            SimulationError::ParquetExport(msg) => {
                write!(f, "Failed to export Parquet file: {}", msg)
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

impl SimulationError {
    /// Returns the category of this error.
    pub fn category(&self) -> ErrorCategory {
        match self {
            SimulationError::ConfigFileRead(_) => ErrorCategory::Configuration,
            SimulationError::YamlParse(_) => ErrorCategory::Parsing,
            SimulationError::TomlParse(_) => ErrorCategory::Parsing,
            SimulationError::UnsupportedConfigFormat(_) => ErrorCategory::Configuration,
            SimulationError::ValidationError(_) => ErrorCategory::Validation,
            SimulationError::IoError(_) => ErrorCategory::Io,
            SimulationError::JsonSerialize(_) => ErrorCategory::Serialization,
            SimulationError::ActionLogWrite(_) => ErrorCategory::Storage,
            SimulationError::ActionLogRead(_) => ErrorCategory::Storage,
            SimulationError::ActionLogSerialize(_) => ErrorCategory::Serialization,
            SimulationError::ActionLogDeserialize(_) => ErrorCategory::Serialization,
            SimulationError::ParquetExport(_) => ErrorCategory::Export,
        }
    }

    /// Validates the error state.
    pub fn validate(&self) -> Result<()> {
        match self {
            SimulationError::YamlParse(msg)
            | SimulationError::TomlParse(msg)
            | SimulationError::UnsupportedConfigFormat(msg)
            | SimulationError::ValidationError(msg)
            | SimulationError::JsonSerialize(msg)
            | SimulationError::ParquetExport(msg)
                if msg.is_empty() =>
            {
                return Err(SimulationError::ValidationError(
                    "Error message cannot be empty".to_string(),
                ));
            },
            _ => {},
        }
        Ok(())
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

        let err = SimulationError::ParquetExport("error".to_string());
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
    fn test_parquet_export_error_display() {
        let err = SimulationError::ParquetExport("failed to write parquet".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to export Parquet file"));
        assert!(display.contains("failed to write parquet"));
    }

    #[test]
    fn test_error_debug_trait() {
        let err = SimulationError::ValidationError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ValidationError"));
    }

    #[test]
    fn test_error_categories() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        assert_eq!(
            SimulationError::ConfigFileRead(io_err).category(),
            ErrorCategory::Configuration
        );
        assert_eq!(
            SimulationError::YamlParse("bad".to_string()).category(),
            ErrorCategory::Parsing
        );
        assert_eq!(
            SimulationError::TomlParse("bad".to_string()).category(),
            ErrorCategory::Parsing
        );
        assert_eq!(
            SimulationError::UnsupportedConfigFormat("json".to_string()).category(),
            ErrorCategory::Configuration
        );
        assert_eq!(
            SimulationError::ValidationError("bad".to_string()).category(),
            ErrorCategory::Validation
        );
        assert_eq!(SimulationError::IoError(io::Error::other("io")).category(), ErrorCategory::Io);
        assert_eq!(
            SimulationError::JsonSerialize("bad".to_string()).category(),
            ErrorCategory::Serialization
        );
        assert_eq!(
            SimulationError::ActionLogWrite(io::Error::other("io")).category(),
            ErrorCategory::Storage
        );
        assert_eq!(
            SimulationError::ActionLogRead(io::Error::other("io")).category(),
            ErrorCategory::Storage
        );
        assert_eq!(
            SimulationError::ActionLogSerialize(
                serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()
            )
            .category(),
            ErrorCategory::Serialization
        );
        assert_eq!(
            SimulationError::ActionLogDeserialize(
                serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()
            )
            .category(),
            ErrorCategory::Serialization
        );
        assert_eq!(
            SimulationError::ParquetExport("bad".to_string()).category(),
            ErrorCategory::Export
        );
    }

    #[test]
    fn test_error_validation() {
        let err = SimulationError::ValidationError("invalid config".to_string());
        assert!(err.validate().is_ok());

        let empty_err = SimulationError::ValidationError("".to_string());
        assert!(empty_err.validate().is_err());
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(format!("{}", ErrorCategory::Configuration), "CONFIGURATION");
        assert_eq!(format!("{}", ErrorCategory::Parsing), "PARSING");
        assert_eq!(format!("{}", ErrorCategory::Validation), "VALIDATION");
        assert_eq!(format!("{}", ErrorCategory::Io), "IO");
        assert_eq!(format!("{}", ErrorCategory::Serialization), "SERIALIZATION");
        assert_eq!(format!("{}", ErrorCategory::Storage), "STORAGE");
        assert_eq!(format!("{}", ErrorCategory::Export), "EXPORT");
    }
}
