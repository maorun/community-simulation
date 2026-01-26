---
name: Copilot Code Improvements (Auto-Select)
about: GitHub Copilot autonomously performs code improvements and test enhancements
title: '[CODE-IMPROVEMENT] Auto-implement from features.md'
labels: ['refactoring', 'copilot', 'code-quality', 'auto-implementation']
assignees: ''
---

## Autonomous Code Improvement and Test Enhancement

**GitHub Copilot: This issue template is designed for autonomous code improvement implementation.**

### Instructions for Copilot

You are tasked with autonomously selecting and implementing a code improvement or test enhancement from the features.md file. Follow these steps:

1. **Improvement Selection:**
   - Review `/home/runner/work/community-simulation/community-simulation/features.md`
   - Select ONE code improvement to implement based on the following criteria (in priority order):
     a) Code improvements from the "🔧 Code-Verbesserungen" section
     b) Improvements that enhance code quality, maintainability, or performance
     c) Test improvements that increase code coverage or test quality
     d) Refactorings that simplify code without changing external behavior
     e) Improvements that don't require new external dependencies
     f) Focus on items listed under "Code-Verbesserungen (Kontinuierlich)"
   - Document your selection clearly in the PR description

2. **Before Implementation:**
   - State which improvement you selected (Section and Item Name)
   - Explain why you selected this improvement
   - Analyze the current code to understand what needs to be improved
   - Outline your implementation plan as a checklist

3. **Implementation Requirements:**
   - Follow all guidelines in `copilot-instructions.md`
   - Make minimal, surgical changes that improve code quality
   - **CRITICAL:** Do not change external behavior or break existing functionality
   - Write or enhance tests for the improved code
   - Ensure all existing tests still pass
   - Add or update documentation (inline doc comments)
   - Run benchmarks if the improvement affects performance

4. **After Implementation:**
   - **CRITICAL: COMPLETELY REMOVE the implemented improvement from `features.md`**
     - Do NOT comment it out with `<!-- -->` tags
     - Do NOT mark it as "IMPLEMENTED" 
     - DELETE the entire improvement section (title, description, benefits, implementation notes)
     - Renumber subsequent items if needed to maintain sequential numbering
   - Update the PR description with implementation details
   - Include performance metrics if applicable (before/after comparisons)
   - Document any trade-offs or limitations

### Success Criteria

The code improvement is complete when:

- [ ] Improvement selected and documented in PR
- [ ] Code compiles without errors: `cargo build --verbose`
- [ ] All tests pass: `cargo test --verbose`
- [ ] All doctests pass: `cargo test --doc --verbose`
- [ ] Code formatted: `cargo fmt`
- [ ] Code linted: `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass without errors)
- [ ] Improvement tested manually (if applicable)
- [ ] Documentation updated (doc comments for affected code)
- [ ] Improvement **completely removed** from `features.md` (not commented out, not marked as implemented - DELETED)
- [ ] No regressions in existing functionality
- [ ] Performance metrics documented (if applicable)
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

# Run benchmarks (if performance-related)
cargo bench

# Run with example parameters
./target/debug/simulation-framework -s 100 -p 10 -o /tmp/test.json
```

### Code Improvement Categories

**Select improvements from these categories:**

1. **Architecture and Design:**
   - Erweiterbare Agentenarchitektur (Component-based architecture)
   - Better separation of concerns
   - Reduced coupling between modules
   - Improved abstraction layers

2. **Performance Optimizations:**
   - Parallele Trade-Matching (Parallel trade matching)
   - Inkrementelle Statistiken (Incremental statistics)
   - Memory usage optimization
   - Algorithmic improvements

3. **Code Quality:**
   - Reduced code duplication
   - Improved error handling
   - Better naming conventions
   - Simplified complex functions
   - Enhanced type safety

4. **Testing:**
   - Integration-Tests (Integration tests)
   - Erweiterte Tests (Extended tests)
   - Property-based testing enhancements
   - Fuzz testing improvements
   - Doctest coverage
   - Test helper functions
   - Mock/stub improvements

5. **Data Management:**
   - Better serialization/deserialization
   - Improved data structures
   - More efficient data handling
   - Better configuration management

### Priority Code Improvements (from features.md)

**High Priority (Continuous):**
1. **Parallele Trade-Matching** - Performance for large simulations (>1000 persons)
2. **Inkrementelle Statistiken** - Scalability improvements
3. **Integration-Tests** - Quality assurance
4. **Erweiterbare Architektur** - Long-term maintainability

**Quality Improvements:**
- Reduce code duplication
- Improve error handling
- Enhance documentation
- Simplify complex logic

**Test Improvements:**
- Increase test coverage
- Add property-based tests
- Improve test organization
- Add integration tests
- Enhance doctest examples

### Implementation Workflow

1. **Explore & Analyze** (use `report_progress` to share your plan)
   - Read features.md and select ONE improvement
   - Analyze existing code to understand what needs improvement
   - Identify specific files and functions to modify
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
   - **COMPLETELY REMOVE improvement from features.md** (delete the entire section)

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

1. **No Behavior Changes:** Code improvements should NOT change external behavior. Tests should pass without modification (except for internal implementation details).

2. **Minimal Scope:** Focus on one specific improvement. Don't try to refactor the entire codebase at once.

3. **Existing Patterns:** Study and maintain existing code patterns:
   - Module organization: See `src/` structure
   - Error handling patterns: Check existing `Result<T, E>` usage
   - Testing patterns: See `#[cfg(test)] mod tests { ... }` blocks
   - Documentation style: Follow existing doc comment style

4. **Performance Validation:** If claiming performance improvements:
   - Run benchmarks BEFORE and AFTER changes
   - Document the performance gain with actual numbers
   - Test with different simulation sizes (small, medium, large)

5. **Testing Improvements:** When adding tests:
   - Follow existing test organization
   - Use deterministic seeds for reproducibility
   - Add doctests for public API examples
   - Test edge cases and error conditions

6. **Backward Compatibility:** Ensure all changes maintain backward compatibility:
   - JSON output format should remain compatible
   - CLI interface should remain compatible
   - Configuration file format should remain compatible

7. **Progress Reporting:** Use `report_progress` tool frequently to commit changes.

8. **Improvement Removal:** After successful implementation, **COMPLETELY DELETE** the improvement section from `features.md`. Do NOT comment it out or mark as "IMPLEMENTED". Mention the removal in your commit message.

9. **Documentation:** Update relevant documentation if the improvement affects usage or understanding of the code.

### Performance Measurement

For performance-related improvements, use these benchmarks:

```bash
# Run existing benchmarks
cargo bench

# Manual performance test (small)
time ./target/release/simulation-framework -s 100 -p 10 -o /tmp/small.json

# Manual performance test (medium)
time ./target/release/simulation-framework -s 500 -p 100 -o /tmp/medium.json

# Manual performance test (large)
time ./target/release/simulation-framework -s 1000 -p 500 -o /tmp/large.json
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
- **Code Improvements List:** See `features.md` (🔧 Code-Verbesserungen section)
- **Example Implementation:** See `.github/ISSUE_TEMPLATE/EXAMPLE.md`

### Example Code Improvement Selection

**Good Selection Example:**
```markdown
Selected Improvement: Parallele Trade-Matching
- Section: 2.1 Performance-Optimierungen
- Category: Performance
- Why: Current trade matching is sequential; parallel execution can significantly improve performance for large simulations
- Impact: Expected 2-4x speedup for simulations with >1000 persons
- Implementation: Use Rayon to parallelize conflict-free trades
```

**Another Good Example:**
```markdown
Selected Improvement: Integration-Tests
- Section: 3. Code-Qualität
- Category: Testing
- Why: Current tests are mostly unit tests; integration tests would catch more real-world issues
- Impact: Better quality assurance, fewer bugs in production
- Implementation: Add integration tests that run full simulation scenarios and validate output
```

### Important Notes

1. **Refactoring vs. Features:** This template is for code improvements, not new features. The goal is to improve existing code, not add new functionality.

2. **Test-Driven:** Write or update tests FIRST to ensure you understand the expected behavior before refactoring.

3. **Incremental Changes:** Make small, incremental improvements. Large refactorings are harder to review and more likely to introduce bugs.

4. **Measure Impact:** For performance improvements, always measure the actual impact with benchmarks. Don't assume an optimization is faster without proof.

5. **Backward Compatibility:** Never break existing APIs or change output formats without explicit approval.

---

**Note:** This template enables autonomous code improvement implementation by Copilot. No manual specification is required - Copilot will select and implement an improvement from features.md automatically.
