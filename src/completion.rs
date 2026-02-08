//! Shell completion generation utilities
//!
//! This module provides functions for generating shell completion scripts
//! for various shells (bash, zsh, fish, powershell).

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Parse a shell name string into a Shell enum
///
/// Converts shell name strings (case-insensitive) into the corresponding
/// clap_complete::Shell enum variant.
///
/// # Arguments
///
/// * `shell_name` - The name of the shell (e.g., "bash", "zsh", "fish", "powershell")
///
/// # Returns
///
/// * `Some(Shell)` if the shell name is recognized
/// * `None` if the shell name is not supported
///
/// # Examples
///
/// ```
/// use community_simulation::completion::parse_shell_name;
/// use clap_complete::Shell;
///
/// assert_eq!(parse_shell_name("bash"), Some(Shell::Bash));
/// assert_eq!(parse_shell_name("BASH"), Some(Shell::Bash));
/// assert_eq!(parse_shell_name("zsh"), Some(Shell::Zsh));
/// assert_eq!(parse_shell_name("fish"), Some(Shell::Fish));
/// assert_eq!(parse_shell_name("powershell"), Some(Shell::PowerShell));
/// assert_eq!(parse_shell_name("pwsh"), Some(Shell::PowerShell));
/// assert_eq!(parse_shell_name("unknown"), None);
/// ```
pub fn parse_shell_name(shell_name: &str) -> Option<Shell> {
    match shell_name.to_lowercase().as_str() {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" | "pwsh" => Some(Shell::PowerShell),
        _ => None,
    }
}

/// Get a list of supported shell names
///
/// Returns a vector of all shell names that are supported for
/// completion script generation.
///
/// # Returns
///
/// A vector of supported shell name strings
///
/// # Examples
///
/// ```
/// use community_simulation::completion::get_supported_shells;
///
/// let shells = get_supported_shells();
/// assert!(shells.contains(&"bash"));
/// assert!(shells.contains(&"zsh"));
/// assert!(shells.contains(&"fish"));
/// assert!(shells.contains(&"powershell"));
/// ```
pub fn get_supported_shells() -> Vec<&'static str> {
    vec!["bash", "zsh", "fish", "powershell"]
}

/// Generate shell completion script
///
/// Generates a completion script for the specified shell and command.
/// The script is written to the provided writer.
///
/// # Type Parameters
///
/// * `T` - The command type that implements CommandFactory
///
/// # Arguments
///
/// * `shell` - The shell to generate completions for
/// * `bin_name` - The name of the binary/command
/// * `writer` - Where to write the completion script
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use clap_complete::Shell;
/// use community_simulation::completion::generate_completion;
///
/// #[derive(Parser)]
/// struct Cli {
///     // ... command definition
/// }
///
/// let mut output = Vec::new();
/// generate_completion::<Cli>(Shell::Bash, "my-app", &mut output);
/// ```
pub fn generate_completion<T: CommandFactory>(
    shell: Shell,
    bin_name: &str,
    writer: &mut dyn io::Write,
) {
    let mut cmd = T::command();
    generate(shell, &mut cmd, bin_name, writer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shell_name_bash() {
        assert_eq!(parse_shell_name("bash"), Some(Shell::Bash));
        assert_eq!(parse_shell_name("BASH"), Some(Shell::Bash));
        assert_eq!(parse_shell_name("Bash"), Some(Shell::Bash));
    }

    #[test]
    fn test_parse_shell_name_zsh() {
        assert_eq!(parse_shell_name("zsh"), Some(Shell::Zsh));
        assert_eq!(parse_shell_name("ZSH"), Some(Shell::Zsh));
    }

    #[test]
    fn test_parse_shell_name_fish() {
        assert_eq!(parse_shell_name("fish"), Some(Shell::Fish));
        assert_eq!(parse_shell_name("FISH"), Some(Shell::Fish));
    }

    #[test]
    fn test_parse_shell_name_powershell() {
        assert_eq!(parse_shell_name("powershell"), Some(Shell::PowerShell));
        assert_eq!(parse_shell_name("POWERSHELL"), Some(Shell::PowerShell));
        assert_eq!(parse_shell_name("pwsh"), Some(Shell::PowerShell));
        assert_eq!(parse_shell_name("PWSH"), Some(Shell::PowerShell));
    }

    #[test]
    fn test_parse_shell_name_unknown() {
        assert_eq!(parse_shell_name("unknown"), None);
        assert_eq!(parse_shell_name("cmd"), None);
        assert_eq!(parse_shell_name(""), None);
    }

    #[test]
    fn test_get_supported_shells() {
        let shells = get_supported_shells();
        assert_eq!(shells.len(), 4);
        assert!(shells.contains(&"bash"));
        assert!(shells.contains(&"zsh"));
        assert!(shells.contains(&"fish"));
        assert!(shells.contains(&"powershell"));
    }

    #[test]
    fn test_generate_completion_produces_output() {
        use clap::Parser;

        #[derive(Parser)]
        #[command(name = "test-app")]
        struct TestCli {
            #[arg(short, long)]
            option: Option<String>,
        }

        let mut output = Vec::new();
        generate_completion::<TestCli>(Shell::Bash, "test-app", &mut output);

        // Verify that completion script was generated
        assert!(!output.is_empty());
        let script = String::from_utf8(output).unwrap();

        // Check for common bash completion patterns
        assert!(script.contains("test-app") || script.contains("_test"));
    }

    #[test]
    fn test_generate_completion_different_shells() {
        use clap::Parser;

        #[derive(Parser)]
        #[command(name = "test-app")]
        struct TestCli {}

        // Test that all supported shells can generate completions without panicking
        for shell_str in get_supported_shells() {
            if let Some(shell) = parse_shell_name(shell_str) {
                let mut output = Vec::new();
                generate_completion::<TestCli>(shell, "test-app", &mut output);
                assert!(!output.is_empty(), "Shell {} should generate output", shell_str);
            }
        }
    }
}
