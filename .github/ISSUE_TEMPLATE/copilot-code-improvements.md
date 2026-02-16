---
name: Copilot Code Improvements (Autonomous Analysis)
about: GitHub Copilot autonomously analyzes codebase and implements improvements
title: '[CODE-IMPROVEMENT] Autonomous analysis and improvement'
labels: ['refactoring', 'copilot', 'code-quality', 'auto-implementation']
assignees: ''
---

## Autonomous Code Improvement and Test Enhancement

**GitHub Copilot: This issue template is designed for autonomous code analysis and improvement implementation.**

### Instructions for Copilot

You are tasked with autonomously analyzing the codebase, identifying areas for improvement, and implementing ONE code improvement or test enhancement. Follow these steps:

1. **Codebase Analysis and Improvement Selection:**
   - **Analyze the codebase** to identify areas that need improvement
   - Use tools like `grep`, `glob`, and code exploration to find:
     a) Code duplication or similar patterns that could be refactored
     b) Complex functions that could be simplified
     c) Missing or inadequate error handling
     d) Performance bottlenecks (based on code analysis or profiling)
     e) Missing tests or low test coverage areas
     f) Poor code organization or structure
     g) Inconsistent patterns or style issues
     h) Missing documentation for public APIs
   - **Select ONE improvement** to implement based on impact and feasibility
   - Document your analysis and selection clearly in the PR description

2. **Before Implementation:**
   - State which improvement you identified and why it's important
   - Explain what you analyzed to identify this improvement
   - Show examples of the current problematic code (if applicable)
   - Outline your implementation plan as a checklist
   - Estimate the expected benefits (e.g., reduced duplication, improved performance, better test coverage)

3. **Implementation Requirements:**
   - Follow all guidelines in `copilot-instructions.md`
   - Make minimal, surgical changes that improve code quality
   - **CRITICAL:** Do not change external behavior or break existing functionality
   - Write or enhance tests for the improved code
   - Ensure all existing tests still pass
   - Add or update documentation (inline doc comments)
   - Run benchmarks if the improvement affects performance

4. **After Implementation:**
   - Update the PR description with implementation details
   - Include performance metrics if applicable (before/after comparisons)
   - Document any trade-offs or limitations
   - Show concrete examples of what was improved (e.g., before/after code snippets, coverage metrics)

### Success Criteria

The code improvement is complete when:

- [ ] Codebase analyzed and improvement identified
- [ ] Analysis documented in PR (what was analyzed, why this improvement was chosen)
- [ ] Code compiles without errors: `cargo build --verbose`
- [ ] All tests pass: `cargo test --verbose`
- [ ] All doctests pass: `cargo test --doc --verbose`
- [ ] Code formatted: `cargo fmt`
- [ ] Code linted: `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass without errors)
- [ ] **Code coverage:** Maintains or improves coverage (goal: 100%, minimum: 56%)
- [ ] Improvement tested manually (if applicable)
- [ ] Documentation updated (doc comments for affected code)
- [ ] No regressions in existing functionality
- [ ] Performance metrics documented (if applicable)
- [ ] Concrete improvements shown (before/after examples, metrics)
- [ ] Code review requested and feedback addressed

### Build, Test, and Validation Commands

```bash
# Build
cargo build --verbose

# Build release (for performance testing)
cargo build --release

# Test (unit and integration tests)
cargo test --verbose

# Test (doctests)
cargo test --doc --verbose

# Format
cargo fmt --all

# Lint (REQUIRED - must pass before completing development)
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# Code Coverage (goal: 100%, minimum: 56%)
cargo tarpaulin --verbose --all-features --workspace --timeout 300

# Run benchmarks (if performance-related)
cargo bench

# Run with example parameters
./target/debug/community-simulation -s 100 -p 10 -o /tmp/test.json
```

### Code Coverage Requirements

This project aims for **100% code coverage** with a progressive improvement approach:

- **Current baseline:** 56% (enforced in CI - will fail below this)
- **Target for improvements:** Maintain or increase coverage
- **Ultimate goal:** 100% coverage

When improving code:
- Maintain or improve test coverage for affected code
- Add tests for previously untested code paths
- Don't decrease overall project coverage
- See [COVERAGE.md](../../COVERAGE.md) for detailed guidance

### Code Improvement Categories

**Look for improvements in these categories:**

1. **Architecture and Design:**
   - Component-based architecture opportunities
   - Better separation of concerns
   - Reduced coupling between modules
   - Improved abstraction layers
   - Module organization improvements

2. **Performance Optimizations:**
   - Parallel processing opportunities (e.g., parallel trade matching)
   - Incremental statistics instead of recomputing
   - Memory usage optimization
   - Algorithmic improvements
   - Caching opportunities

3. **Code Quality:**
   - Reduced code duplication (look for similar patterns)
   - Improved error handling (check for unwrap(), expect() usage)
   - Better naming conventions
   - Simplified complex functions (high cyclomatic complexity)
   - Enhanced type safety

4. **Testing:**
   - Missing test coverage (use `cargo tarpaulin` or analyze test files)
   - Property-based testing opportunities
   - Integration test gaps
   - Fuzz testing improvements
   - Missing doctest examples for public APIs
   - Test helper functions to reduce duplication

5. **Data Management:**
   - Better serialization/deserialization
   - Improved data structures
   - More efficient data handling
   - Better configuration management

### Analysis Tools and Techniques

**Use these tools to identify improvements:**

1. **Code Search and Analysis:**
   - `grep` - Find patterns like `.unwrap()`, `.expect()`, duplicated code
   - `glob` - Find files by type to analyze structure
   - `view` - Examine individual files for complexity
   - Look for functions >50 lines or >10 parameters

2. **Linting and Quality:**
   - Run `cargo clippy` to find suggested improvements
   - Check for deprecated APIs or patterns
   - Look for TODO/FIXME comments

3. **Test Coverage:**
   - Analyze test files vs source files ratio
   - Look for modules without corresponding test modules
   - Check for untested error paths

4. **Performance:**
   - Review algorithmic complexity (O(n²) → O(n log n))
   - Look for unnecessary clones or allocations
   - Identify sequential operations that could be parallel

### Implementation Workflow

1. **Explore & Analyze** (use `report_progress` to share your plan)
   - Use `grep`, `glob`, and code exploration to analyze the codebase
   - Identify ONE specific improvement opportunity
   - Document what you found and why it needs improvement
   - Create implementation checklist
   
2. **Core Implementation**
   - Refactor code structure (if needed)
   - Implement performance improvements (if applicable)
   - Optimize algorithms or data structures
   - Improve error handling
   - Reduce code duplication

3. **Testing**
   - Write new tests for improved code
   - Update existing tests if behavior changed internally
   - Run all tests to ensure no regressions
   - Run doctests
   - Run benchmarks (for performance improvements)
   - Test manually to verify behavior

4. **Quality & Documentation**
   - Run `cargo fmt`
   - Run `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass)
   - Update doc comments
   - Document trade-offs or limitations

5. **Validation & Review**
   - Build release: `cargo build --release`
   - Final manual test
   - Compare performance metrics (if applicable)
   - Request code review using `code_review` tool
   - **After addressing code review feedback:**
     - Re-run `cargo fmt --all`
     - Re-run `cargo clippy --all-targets --all-features -- -D warnings -A deprecated`
     - Re-run `cargo build --verbose`
     - Re-run `cargo test --verbose`
   - Run security checks using `codeql_checker` tool

### Code Review Requirements

**CRITICAL:** After addressing ANY code review feedback, you MUST re-run the complete validation suite:

```bash
# 1. Format code
cargo fmt --all

# 2. Lint code (must pass without errors)
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# 3. Build project
cargo build --verbose

# 4. Run all tests
cargo test --verbose

# 5. Run doctests
cargo test --doc --verbose

# 6. Run benchmarks (if performance-related)
cargo bench
```

Even minor changes require full validation to ensure no regressions are introduced.

### Copilot-Specific Instructions

**Read these carefully before starting:**

1. **Autonomous Analysis Required:** You must analyze the codebase yourself to identify improvements. Do NOT rely on TODO.md or external suggestions. Use grep, glob, and code exploration tools.

2. **No Behavior Changes:** Code improvements should NOT change external behavior. Tests should pass without modification (except for internal implementation details).

3. **Minimal Scope:** Focus on one specific improvement. Don't try to refactor the entire codebase at once.

4. **Existing Patterns:** Study and maintain existing code patterns:
   - Module organization: See `src/` structure
   - Error handling patterns: Check existing `Result<T, E>` usage
   - Testing patterns: See `#[cfg(test)] mod tests { ... }` blocks
   - Documentation style: Follow existing doc comment style

5. **Performance Validation:** If claiming performance improvements:
   - Run benchmarks BEFORE and AFTER changes
   - Document the performance gain with actual numbers
   - Test with different simulation sizes (small, medium, large)

6. **Testing Improvements:** When adding tests:
   - Follow existing test organization
   - Use deterministic seeds for reproducibility
   - Add doctests for public API examples
   - Test edge cases and error conditions

7. **Backward Compatibility:** Ensure all changes maintain backward compatibility:
   - JSON output format should remain compatible
   - CLI interface should remain compatible
   - Configuration file format should remain compatible

8. **Progress Reporting:** Use `report_progress` tool frequently to commit changes.

9. **Documentation:** Update relevant documentation if the improvement affects usage or understanding of the code.

### Performance Measurement

For performance-related improvements, use these benchmarks:

```bash
# Run existing benchmarks
cargo bench

# Manual performance test (small)
time ./target/release/community-simulation -s 100 -p 10 -o /tmp/small.json

# Manual performance test (medium)
time ./target/release/community-simulation -s 500 -p 100 -o /tmp/medium.json

# Manual performance test (large)
time ./target/release/community-simulation -s 1000 -p 500 -o /tmp/large.json
```

Document results in this format:
```
Before: X steps/second
After: Y steps/second
Improvement: Z% faster
```

### Reference Documentation

- **Project Architecture:** See `.github/copilot-instructions.md`
- **Build/Test Commands:** See `.github/copilot-instructions.md`
- **Example Implementation:** See `.github/ISSUE_TEMPLATE/EXAMPLE.md`

### Example Code Improvement Analysis

**Good Analysis Example 1:**
```markdown
Analysis: Identified code duplication in trade matching
- Tool used: grep -r "fn match_trade" src/
- Found: 3 similar implementations in engine.rs (lines 145-178, 234-267, 412-445)
- Issue: ~90 lines of duplicated logic with only minor variations
- Impact: Hard to maintain, bug fixes need to be applied in multiple places
- Solution: Extract common logic into a shared function with configuration parameters
- Expected benefit: Reduce code by ~70 lines, improve maintainability
```

**Good Analysis Example 2:**
```markdown
Analysis: Missing error handling in file operations
- Tool used: grep -n "\.unwrap()" src/
- Found: 15 instances of .unwrap() calls, 8 in file I/O operations
- Issue: Program panics instead of handling errors gracefully
- Files affected: src/result.rs (lines 87, 123, 256), src/config.rs (line 42)
- Impact: Poor user experience, crashes instead of helpful error messages
- Solution: Replace unwrap() with proper error handling and Result types
- Expected benefit: More robust error handling, better user experience
```

**Good Analysis Example 3:**
```markdown
Analysis: Missing test coverage for error paths
- Tool used: Analyzed test files vs source files
- Found: src/market.rs has 450 lines but only 120 lines of tests (~27% coverage)
- Issue: Error handling code paths are not tested (update_prices failure scenarios)
- Impact: Bugs in error handling might not be caught
- Solution: Add tests for error conditions and edge cases
- Expected benefit: Increase test coverage to >80%, catch bugs earlier
```

### Important Notes

1. **Autonomous Analysis Required:** This template requires you to analyze the codebase yourself. Do NOT look for predefined lists of improvements. Use code analysis tools and techniques.

2. **Refactoring vs. Features:** This template is for code improvements, not new features. The goal is to improve existing code, not add new functionality.

3. **Test-Driven:** Write or update tests FIRST to ensure you understand the expected behavior before refactoring.

4. **Incremental Changes:** Make small, incremental improvements. Large refactorings are harder to review and more likely to introduce bugs.

5. **Measure Impact:** For performance improvements, always measure the actual impact with benchmarks. Don't assume an optimization is faster without proof.

6. **Document Your Analysis:** Always explain what you analyzed and how you identified the improvement opportunity.

5. **Backward Compatibility:** Never break existing APIs or change output formats without explicit approval.

---

**Note:** This template enables autonomous code improvement implementation by Copilot. No manual specification is required - Copilot will analyze the codebase and identify improvements autonomously.
