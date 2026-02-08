//! Utility functions for the simulation framework
//!
//! This module contains small helper functions that are used across the application.

/// Converts a certification duration argument to an Option
///
/// A value of 0 is interpreted as "no expiration" (None).
/// Any other value is returned as Some(duration).
///
/// # Arguments
///
/// * `duration` - The duration value from the command line argument
///
/// # Returns
///
/// * `None` if duration is 0 (infinite/no expiration)
/// * `Some(duration)` otherwise
///
/// # Examples
///
/// ```
/// use simulation_framework::utils::certification_duration_from_arg;
///
/// assert_eq!(certification_duration_from_arg(0), None);
/// assert_eq!(certification_duration_from_arg(100), Some(100));
/// assert_eq!(certification_duration_from_arg(200), Some(200));
/// ```
pub fn certification_duration_from_arg(duration: usize) -> Option<usize> {
    if duration == 0 {
        None
    } else {
        Some(duration)
    }
}

/// Get the binary name from command line arguments
///
/// Extracts the binary name from the first command line argument (argv[0]).
/// Falls back to a default name if the binary name cannot be determined.
///
/// # Arguments
///
/// * `default_name` - The default name to use if the binary name cannot be determined
///
/// # Returns
///
/// The binary name as a String
///
/// # Examples
///
/// ```
/// use simulation_framework::utils::get_binary_name;
///
/// let name = get_binary_name("my-default-app");
/// // Will return the actual binary name or "my-default-app"
/// assert!(!name.is_empty());
/// ```
pub fn get_binary_name(default_name: &str) -> String {
    std::env::args()
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| default_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certification_duration_zero_returns_none() {
        assert_eq!(certification_duration_from_arg(0), None);
    }

    #[test]
    fn test_certification_duration_nonzero_returns_some() {
        assert_eq!(certification_duration_from_arg(1), Some(1));
        assert_eq!(certification_duration_from_arg(50), Some(50));
        assert_eq!(certification_duration_from_arg(100), Some(100));
        assert_eq!(certification_duration_from_arg(200), Some(200));
        assert_eq!(certification_duration_from_arg(usize::MAX), Some(usize::MAX));
    }

    #[test]
    fn test_get_binary_name_returns_non_empty() {
        let name = get_binary_name("default-name");
        assert!(!name.is_empty());
        // The name should be either the actual binary name or the default
        assert!(!name.is_empty());
    }

    #[test]
    fn test_get_binary_name_with_different_defaults() {
        let name1 = get_binary_name("app1");
        let name2 = get_binary_name("app2");
        // Both should return valid names
        assert!(!name1.is_empty());
        assert!(!name2.is_empty());
    }
}
