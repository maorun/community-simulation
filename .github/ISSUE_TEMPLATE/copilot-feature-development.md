---
name: Copilot Feature Development (Auto-Select)
about: GitHub Copilot autonomously selects and implements a feature from features.md
title: '[FEATURE] Auto-implement from features.md'
labels: ['enhancement', 'copilot', 'auto-implementation']
assignees: ''
---

## Autonomous Feature Implementation

**GitHub Copilot: This issue template is designed for autonomous feature implementation.**

### Instructions for Copilot

You are tasked with autonomously selecting and implementing a feature from the features.md file. Follow these steps:

1. **Feature Selection:**
   - Review `/home/runner/work/community-simulation/community-simulation/features.md`
   - Select ONE feature to implement based on the following criteria (in priority order):
     a) Features marked as "Hohe Priorität (Quick Wins)" - prioritize these
     b) Features that are simpler and can be implemented with minimal changes
     c) Features that don't require external dependencies or major architectural changes
     d) Features from the "Neue Features" section before "Code-Verbesserungen"
   - Document your selection clearly in the PR description

2. **Before Implementation:**
   - State which feature you selected (Feature ID and Name)
   - Explain why you selected this feature
   - Outline your implementation plan as a checklist

3. **Implementation Requirements:**
   - Follow all guidelines in `copilot-instructions.md`
   - Make minimal, surgical changes
   - Write tests for the new feature
   - Ensure all existing tests still pass
   - Add documentation (inline doc comments)

4. **After Implementation:**
   - **IMPORTANT:** Remove the implemented feature from `features.md`
   - If the feature is user-facing, add a brief mention in `README.md` under a "Recent Features" or "Features" section
   - Update the PR description with implementation details
   - Include example usage in the PR description

### Success Criteria

The feature implementation is complete when:

- [ ] Feature selected and documented in PR
- [ ] Code compiles without errors: `cargo build --verbose`
- [ ] All tests pass: `cargo test --verbose`
- [ ] Code formatted: `cargo fmt`
- [ ] Code linted: `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass without errors)
- [ ] Feature tested manually with example run
- [ ] Documentation added (doc comments for public APIs)
- [ ] Feature **removed** from `features.md`
- [ ] If user-facing: Feature mentioned in `README.md`
- [ ] No regressions in existing functionality

### Build and Test Commands

```bash
# Build
cargo build --verbose

# Test
cargo test --verbose

# Format
cargo fmt

# Lint (REQUIRED - must pass before completing development)
cargo clippy --all-targets --all-features -- -D warnings -A deprecated

# Run simulation with feature
./target/debug/simulation-framework -s 100 -p 10 -o /tmp/test.json
```

### Feature Selection Guidelines

**Prioritize features in this order:**

1. **High Priority (Quick Wins) from features.md:**
   - Logging-System implementieren
   - Erweiterte Tests schreiben
   - Dokumentation vervollständigen
   - CLI mit Progress Bar verbessern
   - YAML/TOML Konfiguration

2. **Simple features that add value without complexity:**
   - Single-field additions to existing structs
   - New configuration options
   - Analysis/statistics features
   - Documentation improvements

3. **Avoid initially:**
   - Features requiring new external dependencies
   - Major architectural changes (Plugin-System, Event-System)
   - Features requiring complex algorithms
   - Features with unclear specifications

### Implementation Workflow

1. **Explore & Plan** (use `report_progress` to share your plan)
   - Read features.md and select ONE feature
   - Review existing code architecture
   - Create implementation checklist
   
2. **Core Implementation**
   - Add data structures (if needed)
   - Implement core logic
   - Integrate with existing code
   - Add configuration (if needed)

3. **Testing**
   - Write unit tests
   - Write integration tests  
   - Run all tests
   - Test manually

4. **Quality & Documentation**
   - Run `cargo fmt`
   - Run `cargo clippy --all-targets --all-features -- -D warnings -A deprecated` (must pass)
   - Add doc comments
   - Update README.md (if user-facing)
   - **Remove feature from features.md**

5. **Validation & Review**
   - Build release: `cargo build --release`
   - Final manual test
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
```

Even minor changes require full validation to ensure no regressions are introduced.

### Copilot-Specific Instructions

**Read these carefully before starting:**

1. **Minimal Changes:** Make the smallest possible changes to achieve the feature. Don't refactor unrelated code.

2. **Existing Patterns:** Study and follow existing code patterns:
   - Price scenarios: `src/scenario.rs`
   - Data structures: `src/person.rs`, `src/skill.rs`, `src/market.rs`
   - Simulation logic: `src/engine.rs`
   - Configuration: `src/config.rs`
   - Results: `src/result.rs`

3. **Testing:** Add tests using `#[cfg(test)] mod tests { ... }` pattern. Use fixed seeds for deterministic tests.

4. **Configuration:** If adding CLI args, update `Args` struct in `main.rs` and `SimulationConfig` in `config.rs`.

5. **JSON Output:** If feature produces output, add fields to `SimulationResult` with proper `serde` serialization.

6. **Progress Reporting:** Use `report_progress` tool frequently to commit changes.

7. **Feature Removal:** After successful implementation, remove the feature section from `features.md` and mention removal in commit message.

8. **README Update:** If feature is user-facing (new CLI args, new output, changed behavior), add a brief note in README.md.

### Reference Documentation

- **Project Architecture:** See `.github/copilot-instructions.md`
- **Build/Test Commands:** See `.github/copilot-instructions.md`
- **Feature List:** See `features.md` (select ONE to implement)
- **Example Implementation:** See `.github/ISSUE_TEMPLATE/EXAMPLE.md`

### Example Feature Selection

**Good Selection Example:**
```markdown
Selected Feature: 3.2 Reputation und Vertrauen
- Feature ID: 3.2
- Category: Soziale Netzwerke und Beziehungen
- Why: Simple field addition to Person struct, minimal changes required
- Implementation: Add `reputation: f64` field, update it based on successful trades
```

**Avoid This Type:**
```markdown
Selected Feature: 4.4 Geografische Komponente
- Why NOT: Requires major changes (Location struct, distance calculations, trade cost modifications)
- Better to start with simpler features first
```

---

**Note:** This template enables autonomous feature implementation by Copilot. No manual feature specification is required - Copilot will select and implement a feature from features.md automatically.
