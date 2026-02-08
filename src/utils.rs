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
}
