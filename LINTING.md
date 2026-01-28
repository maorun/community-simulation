# Linting and Code Quality Guide

This document explains the linting configuration and code quality standards for the Economic Simulation Framework.

## Overview

The project uses two main tools for maintaining code quality:
- **Clippy**: Rust's linter that catches common mistakes and enforces best practices
- **rustfmt**: Code formatter that ensures consistent code style

Both tools are configured with custom settings to enforce additional complexity and style checks.

## Configuration Files

### `.clippy.toml`

Configures Clippy with stricter lints for code quality:

**Complexity Checks:**
- `cognitive-complexity-threshold = 25`: Functions with cognitive complexity > 25 trigger warnings
- `too-many-arguments-threshold = 7`: Functions with more than 7 parameters trigger warnings
- `too-many-lines-threshold = 150`: Functions with more than 150 lines trigger warnings
- `type-complexity-threshold = 500`: Complex type definitions trigger warnings

**Performance Checks:**
- `array-size-threshold = 16384`: Large arrays (> 16KB) trigger warnings
- `enum-variant-size-threshold = 200`: Large enum variants (> 200 bytes) trigger warnings

**Style Checks:**
- `single-char-binding-names-threshold = 4`: Limits single-character variable names
- `max-suggested-slice-pattern-length = 3`: Recommends slice patterns for small arrays
- `doc-valid-idents`: Whitelisted technical terms (GitHub, SQLite, YAML, etc.)

### `.rustfmt.toml`

Configures code formatting with consistent style:

**Basic Settings:**
- `edition = "2021"`: Rust 2021 edition formatting
- `max_width = 100`: Maximum line width of 100 characters
- `tab_spaces = 4`: 4 spaces per indentation level

**Code Layout:**
- `fn_call_width = 80`: Function calls wrap at 80 characters
- `struct_lit_width = 80`: Struct literals wrap at 80 characters
- `chain_width = 80`: Method chains wrap at 80 characters
- `single_line_if_else_max_width = 50`: Small if-else expressions on one line

**Style Preferences:**
- `use_field_init_shorthand = true`: Use shorthand for field initialization
- `use_try_shorthand = true`: Use `?` operator instead of `try!`
- `remove_nested_parens = true`: Remove unnecessary parentheses
- `reorder_imports = true`: Automatically reorder imports alphabetically

## Running Linting Tools

### Check Code with Clippy

```bash
# Run clippy with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# Run clippy with warnings (for exploration)
cargo clippy --all-targets --all-features
```

The `-D warnings` flag treats all warnings as errors, ensuring high code quality.
The `-A deprecated` flag allows deprecation warnings (useful for dependency updates).

### Format Code with rustfmt

```bash
# Apply formatting to all code
cargo fmt --all

# Check if formatting is needed (without applying)
cargo fmt --all -- --check
```

**Important**: Always run `cargo fmt --all` before committing code.

## What Clippy Checks For

### Complexity Issues

**High Cognitive Complexity:**
```rust
// BAD: Complex nested conditions (cognitive complexity > 25)
fn process_data(data: Vec<i32>) {
    for item in data {
        if item > 0 {
            if item < 100 {
                if item % 2 == 0 {
                    // ... many more nested conditions
                }
            }
        }
    }
}

// GOOD: Refactored with early returns
fn process_data(data: Vec<i32>) {
    for item in data {
        if !is_valid_item(item) {
            continue;
        }
        process_valid_item(item);
    }
}
```

**Too Many Arguments:**
```rust
// BAD: 8 parameters (threshold is 7)
fn create_person(
    id: usize,
    money: f64,
    skill: Skill,
    needs: Vec<NeededSkillItem>,
    reputation: f64,
    location_x: f64,
    location_y: f64,
    strategy: Strategy,
) -> Person { ... }

// GOOD: Use a configuration struct
struct PersonConfig {
    id: usize,
    money: f64,
    skill: Skill,
    needs: Vec<NeededSkillItem>,
    reputation: f64,
    location: (f64, f64),
    strategy: Strategy,
}

fn create_person(config: PersonConfig) -> Person { ... }
```

**Too Many Lines:**
```rust
// BAD: Function with 200+ lines
fn giant_function() {
    // ... 200+ lines of code
}

// GOOD: Split into smaller functions
fn main_function() {
    prepare_data();
    process_step_1();
    process_step_2();
    finalize_results();
}

fn prepare_data() { ... }
fn process_step_1() { ... }
fn process_step_2() { ... }
fn finalize_results() { ... }
```

### Performance Issues

**Large Stack Arrays:**
```rust
// BAD: Large array on stack (> 16KB)
let big_array = [0u8; 20_000];

// GOOD: Use heap allocation
let big_array = vec![0u8; 20_000];
```

**Large Enum Variants:**
```rust
// BAD: One variant is much larger than others
enum Message {
    Small(u8),
    Huge([u8; 1000]),  // > 200 bytes
}

// GOOD: Box the large variant
enum Message {
    Small(u8),
    Huge(Box<[u8; 1000]>),
}
```

### Style Issues

**Single-Character Variable Names:**
```rust
// BAD: Too many single-char variables
fn calculate(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let x = a + b;
    let y = c + d;
    x * y
}

// GOOD: Descriptive names
fn calculate(
    price: f64,
    quantity: f64,
    tax_rate: f64,
    discount: f64
) -> f64 {
    let subtotal = price + quantity;
    let total_adjustments = tax_rate + discount;
    subtotal * total_adjustments
}
```

## Integration with CI/CD

The GitHub Actions workflow (`.github/workflows/rust.yml`) automatically runs:
1. `cargo fmt --all -- --check` to verify formatting
2. `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` to check for issues

All pull requests must pass these checks before merging.

## Best Practices

### Before Committing

Always run these commands before creating a commit:
```bash
# 1. Format code
cargo fmt --all

# 2. Check for linting issues
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# 3. Run tests
cargo test --verbose

# 4. Build release (optional, for major changes)
cargo build --release
```

### Handling Warnings

If Clippy reports a warning:
1. **Understand the issue**: Read the warning message and documentation
2. **Decide on action**:
   - Fix the code if the warning is valid
   - Refactor to simplify complexity
   - If the warning is a false positive, consider using `#[allow(...)]` with a comment explaining why
3. **Don't suppress warnings casually**: Each suppression should be justified

### Documenting Exceptions

If you need to suppress a lint warning, add a comment explaining why:
```rust
// Allow high complexity here because this is a state machine with many
// legitimate branches that would be harder to read if extracted
#[allow(clippy::cognitive_complexity)]
fn process_state_machine(state: State) -> Result<State> {
    // Complex but necessary logic
}
```

## Additional Resources

- [Clippy Lints Documentation](https://rust-lang.github.io/rust-clippy/master/)
- [rustfmt Configuration Options](https://rust-lang.github.io/rustfmt/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Summary

The linting configuration helps maintain:
- **Code Quality**: Catches common mistakes and anti-patterns
- **Readability**: Enforces consistent formatting and naming
- **Performance**: Warns about potential performance issues
- **Maintainability**: Encourages simple, modular code

By following these guidelines and using the provided tools, we ensure the codebase remains clean, efficient, and easy to maintain.
