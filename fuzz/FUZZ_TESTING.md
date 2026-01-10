# Fuzz Testing

This directory contains fuzz targets for testing the robustness and security of the simulation framework using `cargo-fuzz`.

## What is Fuzz Testing?

Fuzz testing (fuzzing) is an automated testing technique that provides invalid, unexpected, or random data as inputs to a program. The goal is to find crashes, assertion failures, memory leaks, and other bugs that might not be caught by traditional testing.

## Prerequisites

Fuzz testing requires the Rust nightly toolchain. Install it with:

```bash
rustup toolchain install nightly
```

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

## Available Fuzz Targets

### 1. `fuzz_config_yaml`
Tests the robustness of YAML configuration file parsing.

**What it tests:**
- Parsing malformed YAML input
- Handling edge cases in configuration structure
- Resilience to invalid or unexpected YAML syntax

**Run:**
```bash
cargo +nightly fuzz run fuzz_config_yaml
```

### 2. `fuzz_config_toml`
Tests the robustness of TOML configuration file parsing.

**What it tests:**
- Parsing malformed TOML input
- Handling edge cases in configuration structure
- Resilience to invalid or unexpected TOML syntax

**Run:**
```bash
cargo +nightly fuzz run fuzz_config_toml
```

### 3. `fuzz_simulation_init`
Tests the initialization of the simulation engine with arbitrary numeric inputs.

**What it tests:**
- Configuration validation logic
- Handling of extreme or invalid numeric values (NaN, infinity, negative values, etc.)
- Robustness of simulation engine initialization
- Edge cases in parameter validation

**Run:**
```bash
cargo +nightly fuzz run fuzz_simulation_init
```

## Running Fuzz Tests

### Quick Test (5 seconds)
```bash
cargo +nightly fuzz run <target_name> -- -max_total_time=5
```

### Standard Run (60 seconds)
```bash
cargo +nightly fuzz run <target_name> -- -max_total_time=60
```

### Extended Run (recommended for thorough testing)
```bash
cargo +nightly fuzz run <target_name> -- -max_total_time=600
```

### Continuous Fuzzing (run until crash found or manually stopped)
```bash
cargo +nightly fuzz run <target_name>
```

## Understanding Results

### Success
If the fuzzer runs without finding crashes, you'll see output like:
```
Done <N> runs in <time> second(s)
```

This indicates the fuzzer tested N different inputs without finding any crashes or panics.

### Crash Found
If the fuzzer finds an issue, it will:
1. Save the crashing input to `fuzz/artifacts/<target_name>/crash-<hash>`
2. Display a stack trace showing where the crash occurred
3. Exit with an error

To reproduce a crash:
```bash
cargo +nightly fuzz run <target_name> fuzz/artifacts/<target_name>/crash-<hash>
```

## Useful Fuzzing Options

### Limit Execution Time
```bash
cargo +nightly fuzz run <target> -- -max_total_time=<seconds>
```

### Limit Number of Runs
```bash
cargo +nightly fuzz run <target> -- -runs=<number>
```

### Use Multiple Workers (Parallel Fuzzing)
```bash
cargo +nightly fuzz run <target> -- -workers=<number>
```

### Minimize a Crashing Input
```bash
cargo +nightly fuzz cmin <target>
```

This reduces the corpus to a minimal set of inputs that achieve the same code coverage.

## Corpus Management

The fuzzer maintains a corpus of interesting inputs in `fuzz/corpus/<target_name>/`. These inputs are automatically:
- Generated during fuzzing
- Minimized to remove unnecessary bytes
- Reused in future fuzzing runs for better coverage

To reset a corpus (start from scratch):
```bash
rm -rf fuzz/corpus/<target_name>
```

## Best Practices

1. **Run fuzzing regularly**: Integrate fuzzing into your CI/CD pipeline or run it periodically
2. **Run for extended periods**: Longer fuzzing sessions find more bugs
3. **Check all targets**: Each target tests different code paths
4. **Keep corpus**: The corpus improves over time, providing better coverage
5. **Fix bugs promptly**: Address any crashes found immediately

## Integration with CI/CD

For automated fuzzing in CI (with timeout):
```bash
#!/bin/bash
for target in fuzz_config_yaml fuzz_config_toml fuzz_simulation_init; do
    echo "Fuzzing $target for 60 seconds..."
    cargo +nightly fuzz run $target -- -max_total_time=60 || exit 1
done
```

## Interpreting Coverage

To see code coverage achieved by fuzzing:
```bash
cargo +nightly fuzz coverage <target>
```

This generates a coverage report showing which code paths were exercised during fuzzing.

## Troubleshooting

### "error: no such command: `fuzz`"
Install cargo-fuzz: `cargo install cargo-fuzz`

### "error: the option `Z` is only accepted on the nightly compiler"
Use the nightly toolchain: `cargo +nightly fuzz run <target>`

### "error: failed to build fuzz script"
Ensure nightly toolchain is installed: `rustup toolchain install nightly`

### Out of Memory
Reduce the memory limit:
```bash
cargo +nightly fuzz run <target> -- -rss_limit_mb=2048
```

## Additional Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)

## Contributing New Fuzz Targets

To add a new fuzz target:

1. Create a new file in `fuzz/fuzz_targets/` (e.g., `fuzz_my_feature.rs`)
2. Add the target to `fuzz/Cargo.toml`:
   ```toml
   [[bin]]
   name = "fuzz_my_feature"
   path = "fuzz_targets/fuzz_my_feature.rs"
   test = false
   doc = false
   bench = false
   ```
3. Implement the fuzz target following the existing patterns
4. Test it: `cargo +nightly fuzz run fuzz_my_feature -- -max_total_time=5`
5. Update this documentation

## Security Note

Fuzzing is an important security testing technique. Any crashes or hangs found should be:
1. Investigated thoroughly
2. Fixed promptly
3. Added to the test suite to prevent regressions
4. Considered for security implications (could this be exploited?)

Always run fuzzing before releasing new versions to catch potential security vulnerabilities early.
