---
name: Copilot Feature Development
about: Template for GitHub Copilot to develop features from features.md
title: '[FEATURE] '
labels: ['enhancement', 'copilot']
assignees: ''
---

## Feature to Implement

<!-- Select a feature from features.md and describe it here -->

**Feature ID:** <!-- e.g., 1.1 Spar- und Investitionssystem -->

**Feature Name:** <!-- e.g., Savings and Investment System -->

**Category:** <!-- e.g., Erweiterte Wirtschaftsmechaniken / Advanced Economic Mechanics -->

**Priority:** <!-- High / Medium / Low (see Priorisierung section in features.md) -->

## Feature Description

<!-- Copy the description from features.md or provide your own detailed description -->

**Description (Beschreibung):**
<!-- What should this feature do? -->

**Benefit (Nutzen):**
<!-- Why is this feature valuable? -->

**Suggested Implementation (Implementierung):**
<!-- Initial implementation approach from features.md -->

## Implementation Requirements

### Files to Create/Modify

<!-- List the files that need to be created or modified -->
- [ ] 

### Core Components

<!-- Describe the main components to implement -->
- [ ] 

### Integration Points

<!-- Describe how this feature integrates with existing code -->
- [ ] 

## Technical Specifications

### Data Structures

<!-- Describe new structs, enums, or types needed -->
```rust
// Example:
// struct NewFeature {
//     field1: Type1,
//     field2: Type2,
// }
```

### API Changes

<!-- Describe new public APIs or changes to existing ones -->
- [ ] New functions:
- [ ] Modified functions:
- [ ] New traits:

### Configuration

<!-- Describe new configuration options needed -->
- [ ] CLI arguments:
- [ ] Config file parameters:
- [ ] Default values:

## Testing Requirements

### Unit Tests

- [ ] Test new data structures
- [ ] Test new functions
- [ ] Test edge cases

### Integration Tests

- [ ] Test feature integration with existing code
- [ ] Test with different scenarios
- [ ] Test with various parameter combinations

### Performance Tests

- [ ] Benchmark performance impact
- [ ] Test scalability with large simulations

## Documentation Requirements

- [ ] Update README.md if user-facing changes
- [ ] Add inline documentation (doc comments)
- [ ] Update features.md to mark feature as implemented
- [ ] Add usage examples

## Implementation Checklist

### Phase 1: Core Implementation
- [ ] Create new data structures
- [ ] Implement core logic
- [ ] Add to appropriate modules
- [ ] Ensure code compiles

### Phase 2: Integration
- [ ] Integrate with SimulationEngine
- [ ] Update configuration handling
- [ ] Add CLI arguments if needed
- [ ] Update serialization (JSON output)

### Phase 3: Testing
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Run all tests: `cargo test --verbose`
- [ ] Verify no regressions

### Phase 4: Code Quality
- [ ] Format code: `cargo fmt`
- [ ] Run linter: `cargo clippy`
- [ ] Fix all warnings related to new code
- [ ] Run security checks (CodeQL)

### Phase 5: Documentation
- [ ] Add doc comments to all public APIs
- [ ] Update relevant documentation files
- [ ] Add usage examples
- [ ] Update features.md status

### Phase 6: Validation
- [ ] Build release version: `cargo build --release`
- [ ] Test with sample scenarios
- [ ] Verify JSON output includes new data
- [ ] Test edge cases and error conditions

## Copilot Instructions

**For GitHub Copilot implementing this feature:**

1. **Read the Context:**
   - Review `/home/runner/work/community-simulation/community-simulation/features.md` for detailed feature description
   - Review copilot-instructions.md for build/test commands and project structure
   - Understand the existing codebase architecture before making changes

2. **Follow Minimal Change Principle:**
   - Make the smallest possible changes to achieve the feature
   - Don't refactor unrelated code
   - Maintain consistency with existing code style

3. **Implementation Order:**
   - Start with data structures (in appropriate files like `person.rs`, `market.rs`, etc.)
   - Implement core logic
   - Add configuration support
   - Update simulation engine integration
   - Add serialization support for JSON output
   - Write tests
   - Update documentation

4. **Build and Test Commands:**
   ```bash
   # Build
   cargo build --verbose
   
   # Test
   cargo test --verbose
   
   # Format
   cargo fmt
   
   # Lint
   cargo clippy
   
   # Run simulation
   ./target/debug/simulation-framework -s 100 -p 10 -o /tmp/test.json
   ```

5. **Code Style:**
   - Follow existing Rust conventions
   - Use descriptive variable names
   - Add doc comments for public APIs
   - Keep functions focused and small
   - Use existing patterns from the codebase

6. **Testing Strategy:**
   - Add tests in appropriate test modules
   - Use `#[cfg(test)] mod tests { ... }` pattern
   - Test both success and error cases
   - Ensure tests are deterministic (use fixed seeds)

7. **Configuration Integration:**
   - Add new fields to `SimulationConfig` if needed
   - Add CLI arguments to `Args` struct in `main.rs`
   - Provide sensible defaults
   - Document all new parameters

8. **JSON Output:**
   - Add new fields to `SimulationResult` if the feature produces output
   - Ensure fields are properly serialized with `serde`
   - Test that JSON output is valid

9. **Common Pitfalls to Avoid:**
   - Don't break existing tests
   - Don't remove or modify working code unnecessarily
   - Don't introduce new dependencies without checking security
   - Don't ignore compiler warnings in your new code
   - Don't skip documentation

10. **Report Progress:**
    - Use the `report_progress` tool frequently
    - Commit after each logical unit of work
    - Keep the implementation checklist updated

## Reference Documentation

- **Project Structure:** See `copilot-instructions.md` section "Project Layout and Architecture"
- **Build Commands:** See `copilot-instructions.md` section "Build, Test, and Validation Commands"
- **Features List:** See `features.md` for all available features
- **Existing Code Examples:**
  - Price scenarios: `src/scenario.rs`
  - Data structures: `src/person.rs`, `src/skill.rs`, `src/market.rs`
  - Simulation logic: `src/engine.rs`
  - Configuration: `src/config.rs`
  - Results: `src/result.rs`

## Success Criteria

This feature is considered successfully implemented when:

- [ ] All code compiles without errors
- [ ] All tests pass (including new tests for this feature)
- [ ] Code is formatted (`cargo fmt`) and linted (`cargo clippy`)
- [ ] Feature works as described in features.md
- [ ] JSON output includes relevant data for the feature (if applicable)
- [ ] Documentation is updated
- [ ] No regressions in existing functionality
- [ ] Feature can be demonstrated with a test run

## Additional Notes

<!-- Add any additional context, dependencies, or notes here -->

---

**Note for developers:** This template is designed to provide comprehensive guidance for GitHub Copilot when implementing features. If you're manually implementing this feature, you can use this template as a checklist and guide.
