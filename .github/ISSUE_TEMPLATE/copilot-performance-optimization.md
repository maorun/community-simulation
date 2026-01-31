---
name: Copilot Performance Optimization (Autonomous Analysis)
about: GitHub Copilot autonomously analyzes performance bottlenecks and implements optimizations
title: '[PERFORMANCE] Autonomous performance analysis and optimization'
labels: ['performance', 'copilot', 'optimization', 'auto-implementation']
assignees: ''
---

## Autonomous Performance Optimization

**GitHub Copilot: This issue template is designed for autonomous performance analysis and optimization implementation.**

### Instructions for Copilot

You are tasked with autonomously analyzing the codebase for performance bottlenecks, identifying optimization opportunities, and implementing ONE performance optimization. Follow these steps:

1. **Performance Analysis and Optimization Selection:**
   - **Analyze the codebase** for performance bottlenecks and optimization opportunities
   - Use tools like `grep`, `glob`, and code exploration to find:
     a) Inefficient algorithms (O(n²) that could be O(n log n) or O(n))
     b) Unnecessary allocations, clones, or copies
     c) Sequential operations that could be parallelized
     d) Missing caching opportunities for repeated computations
     e) Inefficient data structures (Vec when HashMap would be better, etc.)
     f) Hot loops with optimization potential
     g) Redundant or duplicate computations
     h) I/O operations that could be batched or buffered
   - **Run existing benchmarks** to establish baseline performance
   - **Select ONE optimization** to implement based on:
     - Measured impact potential (profile or analyze code)
     - Implementation feasibility
     - Risk of introducing bugs
   - Document your analysis and selection clearly in the PR description

2. **Before Implementation:**
   - State which bottleneck you identified and why it's impactful
   - Explain what you analyzed to identify this bottleneck (code review, algorithmic analysis, or benchmark results)
   - Show the current implementation and explain why it's inefficient
   - Outline your implementation plan as a checklist
   - Estimate the expected performance improvement (e.g., "Reduce single step time by ~30%")

3. **Implementation Requirements:**
   - Follow all guidelines in `copilot-instructions.md`
   - Make minimal, surgical changes focused on performance
   - **CRITICAL:** Do not change external behavior or break existing functionality
   - Measure performance BEFORE and AFTER with benchmarks
   - Write or enhance tests to ensure correctness is maintained
   - Ensure all existing tests still pass
   - Add comments explaining non-obvious optimizations
   - Document any trade-offs (e.g., memory vs speed)

4. **After Implementation:**
   - Update the PR description with implementation details
   - **REQUIRED:** Include concrete performance measurements (before/after comparisons)
   - Document the optimization technique used
   - Show benchmark results demonstrating the improvement
   - Document any trade-offs or limitations
   - Explain when the optimization helps most (e.g., "Most effective with >100 persons")

### Success Criteria

The performance optimization is complete when:

- [ ] Performance analysis completed and bottleneck identified
- [ ] Analysis documented in PR (what was analyzed, why this optimization was chosen)
- [ ] Baseline performance measured with benchmarks
- [ ] Code compiles without errors: `cargo build --verbose`
- [ ] All tests pass: `cargo test --verbose`
- [ ] All doctests pass: `cargo test --doc --verbose`
- [ ] Code formatted: `cargo fmt`
- [ ] Code linted: `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass without errors)
- [ ] Optimization tested manually with various simulation sizes
- [ ] Performance improvement measured with benchmarks
- [ ] Benchmark results show measurable improvement (include numbers)
- [ ] No regressions in existing functionality
- [ ] Documentation updated (doc comments explaining optimization)
- [ ] Trade-offs documented (if any)
- [ ] Code review requested and feedback addressed

### Build, Test, and Validation Commands

```bash
# Build
cargo build --verbose

# Build release (REQUIRED for performance testing)
cargo build --release

# Test (unit and integration tests)
cargo test --verbose

# Test (doctests)
cargo test --doc --verbose

# Format
cargo fmt --all

# Lint (REQUIRED - must pass before completing development)
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# Run benchmarks (CRITICAL for performance work)
cargo bench

# Save baseline (before optimization)
cargo bench -- --save-baseline before

# Compare with baseline (after optimization)
cargo bench -- --baseline before

# Manual performance test (small)
time ./target/release/simulation-framework -s 100 -p 10 -o /tmp/small.json

# Manual performance test (medium)
time ./target/release/simulation-framework -s 500 -p 100 -o /tmp/medium.json

# Manual performance test (large)
time ./target/release/simulation-framework -s 1000 -p 500 -o /tmp/large.json
```

### Performance Optimization Categories

**Look for optimization opportunities in these categories:**

1. **Algorithmic Improvements:**
   - Reduce time complexity (O(n²) → O(n log n) or O(n))
   - Use more efficient algorithms (binary search vs linear search)
   - Early termination conditions
   - Avoid redundant computations
   - Cache computed results

2. **Data Structure Optimizations:**
   - Choose appropriate data structures (HashMap vs Vec for lookups)
   - Reduce memory allocations
   - Use iterators instead of collecting into Vec
   - Pre-allocate collections with capacity
   - Use references instead of clones where possible

3. **Parallelization Opportunities:**
   - Identify independent operations that can run in parallel
   - Use Rayon for parallel iteration (already a dependency)
   - Parallel collection processing
   - Concurrent data structure updates (with proper synchronization)

4. **Memory Optimization:**
   - Reduce cloning and copying
   - Reuse allocations
   - Stack allocation instead of heap where possible
   - Smaller data structures (pack fields, use smaller types)
   - Avoid String allocations in hot paths

5. **Computation Optimization:**
   - Incremental computation instead of full recalculation
   - Lazy evaluation
   - Memoization/caching
   - Avoid floating-point operations where integers suffice
   - Batch operations

6. **I/O and Serialization:**
   - Batch file I/O operations
   - Buffer writes
   - Defer JSON serialization until needed
   - Stream large outputs instead of building in memory

### Performance Analysis Tools and Techniques

**Use these tools and techniques to identify bottlenecks:**

1. **Code Analysis:**
   - Look for nested loops (potential O(n²) or worse)
   - Find `.clone()` calls that might be unnecessary
   - Identify repeated identical computations
   - Check for Vec/HashMap usage patterns
   - Look for type conversions in hot paths

2. **Benchmark Analysis:**
   - Run `cargo bench` to measure current performance
   - Identify slowest benchmarks
   - Look at benchmark output for timing breakdowns
   - Compare scenarios (Original vs DynamicPricing)

3. **Code Search Patterns:**
   - `grep -n "\.clone()" src/` - Find cloning operations
   - `grep -n "for.*for" src/` - Find nested loops
   - `grep -n "unwrap()" src/` - Find potential panic points
   - `grep -n "collect()" src/` - Find collection allocations

4. **Profiling (Advanced):**
   - Use `cargo flamegraph` for visual profiling (if installed)
   - Use `perf` on Linux for detailed profiling
   - Use `cargo instruments` on macOS

5. **Manual Timing:**
   - Use `time` command for end-to-end measurements
   - Compare different simulation sizes to understand scaling
   - Test with different scenarios

### Implementation Workflow

1. **Analyze & Measure** (use `report_progress` to share your plan)
   - Run existing benchmarks to establish baseline: `cargo bench -- --save-baseline before`
   - Analyze code for potential bottlenecks
   - Identify ONE specific optimization opportunity
   - Document what you found, why it's slow, and expected improvement
   - Create implementation checklist
   
2. **Core Implementation**
   - Implement the optimization with minimal changes
   - Add comments explaining the optimization technique
   - Ensure code still compiles: `cargo build --release`
   - Verify correctness with tests: `cargo test --verbose`

3. **Performance Testing**
   - Run benchmarks: `cargo bench`
   - Compare with baseline: `cargo bench -- --baseline before`
   - Test manually with different simulation sizes
   - Document the performance improvement with specific numbers
   - Verify improvement is consistent across different scenarios

4. **Correctness Verification**
   - Run all tests to ensure no regressions: `cargo test --verbose`
   - Run doctests: `cargo test --doc --verbose`
   - Test edge cases manually
   - Verify output JSON remains correct
   - Test with different seeds for determinism

5. **Quality & Documentation**
   - Run `cargo fmt`
   - Run `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass)
   - Add doc comments explaining the optimization
   - Document trade-offs (e.g., "Uses 10% more memory for 2x speedup")
   - Update PR description with benchmark results

6. **Validation & Review**
   - Final benchmark run with all measurements
   - Create before/after comparison table
   - Request code review using `code_review` tool
   - **After addressing code review feedback:**
     - Re-run `cargo fmt --all`
     - Re-run `cargo clippy --all-targets --all-features -- -D warnings -A deprecated`
     - Re-run `cargo build --release`
     - Re-run `cargo test --verbose`
     - Re-run `cargo bench` to verify optimization still works
   - Run security checks using `codeql_checker` tool

### Code Review Requirements

**CRITICAL:** After addressing ANY code review feedback, you MUST re-run the complete validation suite:

```bash
# 1. Format code
cargo fmt --all

# 2. Lint code (must pass without errors)
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# 3. Build project (release for accurate performance)
cargo build --release

# 4. Run all tests
cargo test --verbose

# 5. Run doctests
cargo test --doc --verbose

# 6. Re-run benchmarks to verify optimization still works
cargo bench
```

Even minor changes require full validation to ensure no regressions are introduced.

### Copilot-Specific Instructions

**Read these carefully before starting:**

1. **Measure First:** ALWAYS establish baseline performance before making changes. Run `cargo bench -- --save-baseline before` at the start.

2. **Prove the Improvement:** Performance claims MUST be backed by actual benchmark measurements. No "should be faster" without proof.

3. **No Premature Optimization:** Only optimize code that is actually measured to be slow. Don't optimize for hypothetical bottlenecks.

4. **Minimal Scope:** Focus on one specific optimization. Don't try to optimize everything at once.

5. **Correctness First:** Performance means nothing if the code is wrong. All tests must pass.

6. **Existing Patterns:** Study existing optimization patterns:
   - Parallel processing: See Rayon usage in the codebase
   - Caching: Check how market prices are cached
   - Data structures: See how persons and markets are organized
   - Memory management: Follow existing patterns for borrowing vs cloning

7. **Benchmark Interpretation:**
   - Criterion provides statistical significance - look for "Change: +/- X%"
   - Small improvements (<5%) may be noise - aim for >10% improvements
   - Verify improvements are consistent across multiple runs
   - Test with different simulation parameters

8. **Trade-offs Documentation:** If optimization trades memory for speed (or vice versa):
   - Document the trade-off clearly
   - Explain when it's beneficial
   - Consider making it configurable if impact is significant

9. **Avoid Breaking Changes:** 
   - JSON output format must remain compatible
   - CLI interface must remain compatible
   - Public API must remain compatible
   - Simulation results should be deterministic (same seed = same results)

10. **Progress Reporting:** Use `report_progress` tool frequently to commit changes.

### Performance Measurement Requirements

**Before/After Format:**

Always present results in this format:

```
## Benchmark Results

### Baseline (Before Optimization)
- Engine initialization (100 persons): 450 µs ± 25 µs
- Single step (100 persons): 1.2 ms ± 0.05 ms
- Full simulation (100p, 100s): 125 ms ± 5 ms

### After Optimization
- Engine initialization (100 persons): 380 µs ± 20 µs  [**15.6% faster**]
- Single step (100 persons): 850 µs ± 0.04 ms         [**29.2% faster**]
- Full simulation (100p, 100s): 89 ms ± 4 ms          [**28.8% faster**]

### Impact
- Primary improvement: Single step performance
- Scaling: Improvement increases with more persons (>50)
- Trade-offs: +2% memory usage due to caching
```

**Manual Timing Format:**

```
## Manual Performance Tests

### Small (100 steps, 10 persons)
- Before: 0.05s (2000 steps/sec)
- After:  0.03s (3333 steps/sec)  [**66.7% faster**]

### Medium (500 steps, 100 persons)
- Before: 1.2s (417 steps/sec)
- After:  0.8s (625 steps/sec)   [**50% faster**]

### Large (1000 steps, 500 persons)
- Before: 8.5s (118 steps/sec)
- After:  5.1s (196 steps/sec)   [**66.1% faster**]
```

### Example Performance Analysis

**Good Analysis Example 1:**
```markdown
## Analysis: Redundant Price Lookups

### Investigation
- Tool used: Code review of src/engine.rs::step()
- Found: Market price lookup called 3 times per person per step
- Location: Lines 145, 178, 234 in engine.rs
- Issue: HashMap lookup (O(1) but with overhead) repeated unnecessarily

### Current Implementation
```rust
// Each person does this:
let price1 = market.get_price(&skill); // First lookup
// ... some logic ...
let price2 = market.get_price(&skill); // Second lookup (same skill!)
// ... more logic ...
let price3 = market.get_price(&skill); // Third lookup (same skill!)
```

### Impact
- With 100 persons: 300 redundant lookups per step
- With 500 steps: 150,000 unnecessary HashMap accesses
- Estimated overhead: 10-15% of step time based on profiling

### Solution
Cache the price lookup result in a local variable and reuse it.

### Expected Benefit
- Reduce single step time by ~10-15%
- More significant improvement with larger simulations (>100 persons)
```

**Good Analysis Example 2:**
```markdown
## Analysis: Sequential Trade Matching

### Investigation
- Benchmark analysis: single_step benchmark shows 80% of time in trade matching
- Tool used: cargo bench and code review
- Found: Trade matching in engine.rs processes persons sequentially
- Current: O(n²) nested loop for matching buyers and sellers

### Current Implementation
```rust
for buyer in persons.iter() {
    for seller in persons.iter() {
        if can_trade(buyer, seller) {
            execute_trade(buyer, seller);
        }
    }
}
```

### Impact
- With 100 persons: 10,000 iterations per step
- With 500 persons: 250,000 iterations per step (2500x worse!)
- Scaling: Quadratic - doubles when persons double

### Baseline Measurements
- 10 persons: 120 µs per step
- 50 persons: 1.8 ms per step (15x slower for 5x more persons)
- 100 persons: 6.5 ms per step (54x slower for 10x more persons)

### Solution
Use parallel iteration with Rayon + indexed matching to avoid duplicate pairs:

```rust
persons.par_iter().enumerate().for_each(|(i, buyer)| {
    persons[i+1..].iter().for_each(|seller| {
        if can_trade(buyer, seller) {
            execute_trade(buyer, seller);
        }
    });
});
```

### Expected Benefit
- Parallel execution: 4-8x speedup on multi-core CPUs
- Index optimization: 2x reduction in iterations (check each pair once)
- Combined: 8-16x faster for large simulations
```

**Good Analysis Example 3:**
```markdown
## Analysis: Unnecessary Cloning in Statistics

### Investigation
- Tool: grep -n "\.clone()" src/result.rs
- Found: 12 .clone() calls in calculate_statistics()
- Location: src/result.rs lines 145-210

### Current Implementation
Every statistic calculation clones the entire person vector:

```rust
pub fn calculate_statistics(&self, persons: Vec<Person>) -> Statistics {
    let monies: Vec<f64> = persons.clone().iter().map(|p| p.money).collect();
    let average = monies.clone().iter().sum::<f64>() / monies.len();
    // ... more clones ...
}
```

### Impact
- With 500 persons: ~5 clones of 500-element vectors per statistics call
- Each Person is ~200 bytes: 5 × 500 × 200 = 500KB of unnecessary copying
- Called every step: 500KB × 500 steps = 250MB extra allocations

### Solution
Use references and iterator chains without collecting:

```rust
pub fn calculate_statistics(&self, persons: &[Person]) -> Statistics {
    let average = persons.iter().map(|p| p.money).sum::<f64>() / persons.len() as f64;
    // Use iterators throughout, no cloning
}
```

### Expected Benefit
- Eliminate 250MB of allocations per simulation
- Reduce statistics time by ~60% (measured with micro-benchmark)
- Lower memory pressure = better cache behavior
```

### Important Notes

1. **Evidence-Based:** This template requires measured performance improvements. Speculation without measurements is not acceptable.

2. **Benchmark-Driven:** Use the existing benchmark infrastructure. Add new benchmarks if needed to measure specific optimizations.

3. **Trade-offs:** All optimizations have trade-offs. Document them clearly (memory vs speed, code complexity vs performance, etc.).

4. **Scalability:** Test and document how the optimization scales with different simulation parameters (persons, steps).

5. **Minimal Risk:** Prefer optimizations with low risk of introducing bugs. Algorithmic improvements are better than micro-optimizations.

6. **No Premature Optimization:** Only optimize hot paths. Use benchmarks to identify what actually matters.

7. **Backward Compatibility:** Never break existing APIs, output formats, or deterministic behavior for performance gains without explicit approval.

8. **Reproducibility:** All measurements should be reproducible. Use fixed seeds, warm-up runs, and multiple iterations.

### Reference Documentation

- **Project Architecture:** See `.github/copilot-instructions.md`
- **Build/Test Commands:** See `.github/copilot-instructions.md`
- **Existing Benchmarks:** See `benches/simulation_benchmarks.rs`
- **Example Implementation:** See `.github/ISSUE_TEMPLATE/EXAMPLE.md`

### Profiling Resources

If you need to do detailed profiling (optional, for complex optimizations):

```bash
# Install cargo-flamegraph (optional)
cargo install flamegraph

# Generate flamegraph (Linux)
cargo flamegraph --bench simulation_benchmarks

# Use perf for detailed profiling (Linux)
perf record --call-graph=dwarf cargo bench
perf report

# Use Instruments on macOS
cargo instruments --bench simulation_benchmarks
```

---

**Note:** This template enables autonomous performance optimization implementation by Copilot. No manual specification is required - Copilot will analyze performance bottlenecks and implement optimizations autonomously with measured results.
