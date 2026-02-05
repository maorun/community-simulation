# GitHub Issue Templates

This directory contains issue templates for the Community Simulation project.

## Available Templates

### 1. Copilot Feature Development (`copilot-feature-development.md`)

**Purpose:** This template is specifically designed for GitHub Copilot to systematically develop features from the `features.md` file.

**When to use:**
- When you want GitHub Copilot to implement a feature from the features list
- When you need a structured approach to feature implementation
- When you want comprehensive guidance for feature development

**How to use:**
1. Create a new issue using this template
2. Select a feature from `features.md`
3. Fill in the feature details (ID, name, category, priority)
4. Describe implementation requirements
5. Assign to GitHub Copilot or use Copilot to implement the feature
6. Follow the implementation checklist step-by-step

**Key sections:**
- Feature selection and description
- Implementation requirements
- Technical specifications
- Testing requirements
- Documentation requirements
- Detailed implementation checklist
- Copilot-specific instructions
- Success criteria

### 2. Copilot Code Improvements (`copilot-code-improvements.md`)

**Purpose:** This template is specifically designed for GitHub Copilot to autonomously analyze the codebase and perform code improvements and test enhancements.

**When to use:**
- When you want GitHub Copilot to autonomously identify and improve code quality or performance
- When you need test coverage improvements
- When you want to refactor code without changing behavior
- When you want Copilot to find and fix code duplication, error handling issues, or other quality problems

**How to use:**
1. Create a new issue using this template
2. Copilot will autonomously analyze the codebase using grep, glob, and code exploration
3. Copilot will identify ONE specific improvement opportunity
4. Copilot will implement the improvement with minimal changes
5. Copilot will ensure no behavior changes and all tests pass

**Key sections:**
- Autonomous codebase analysis instructions
- Analysis tools and techniques (grep, glob, clippy)
- Implementation requirements (no behavior changes)
- Testing and validation requirements
- Performance measurement (if applicable)
- Code quality checks
- Documentation updates
- Success criteria

**Focus areas:**
- Code duplication reduction
- Error handling improvements (replace unwrap/expect)
- Performance optimizations
- Test coverage gaps
- Documentation improvements
- Architecture and design enhancements

### 3. Bug Report (`bug_report.md`)

**Purpose:** Report bugs, errors, or unexpected behavior in the simulation framework.

**When to use:**
- When you encounter crashes or errors
- When the simulation produces incorrect results
- When something doesn't work as documented

### 4. General Issue / Question (`general_issue.md`)

**Purpose:** Ask questions, start discussions, or raise topics that don't fit other categories.

**When to use:**
- When you have questions about the project
- When you want to discuss ideas or improvements
- When you need clarification on features or usage

## Template Configuration (`config.yml`)

The `config.yml` file provides:
- Quick links to important documentation (features.md, README.md, copilot-instructions.md)
- Configuration for blank issue creation
- Easy navigation to project resources

## Creating Issues on GitHub

When you create a new issue on GitHub:

1. Go to the [Issues page](https://github.com/maorun/community-simulation/issues)
2. Click "New issue"
3. Select the appropriate template:
   - **Copilot Feature Development** - For implementing new features from features.md
   - **Copilot Code Improvements** - For code quality, performance, or test improvements
   - **Bug Report** - For reporting bugs
   - **General Issue / Question** - For questions or discussions
4. Fill in the template fields
5. Submit the issue

## For GitHub Copilot Users

Both the **Copilot Feature Development** and **Copilot Code Improvements** templates are optimized for GitHub Copilot workflows:

1. **Comprehensive Guidance:** Includes detailed instructions for Copilot
2. **Structured Approach:** Follows a phase-based implementation plan
3. **Quality Assurance:** Built-in checklists for testing and validation
4. **Context-Aware:** References project structure and existing patterns
5. **Best Practices:** Enforces minimal changes and proper testing

### Differences Between Templates

**Copilot Feature Development:**
- For implementing NEW features from features.md (🚀 Neue Features section)
- Adds new functionality and behavior
- May require new data structures and APIs
- Focus on value-add for users
- Uses predefined feature list from features.md

**Copilot Code Improvements:**
- For improving EXISTING code through autonomous analysis
- Analyzes codebase to identify improvement opportunities
- No external behavior changes
- Focus on performance, quality, and maintainability
- Uses grep, glob, and code analysis tools
- May improve tests and documentation
- Does NOT use features.md - finds improvements independently

### Example Workflow with Copilot - Feature Development

```bash
# 1. Create issue using the Copilot Feature Development template
# 2. Select feature (e.g., "1.1 Spar- und Investitionssystem")
# 3. GitHub Copilot reads the issue and features.md
# 4. Copilot implements following the checklist:
#    - Phase 1: Core Implementation
#    - Phase 2: Integration
#    - Phase 3: Testing
#    - Phase 4: Code Quality
#    - Phase 5: Documentation
#    - Phase 6: Validation
# 5. Each phase is committed and tested
# 6. Issue is closed when all success criteria are met
```

### Example Workflow with Copilot - Code Improvements

```bash
# 1. Create issue using the Copilot Code Improvements template
# 2. Copilot analyzes the codebase using grep, glob, and exploration tools
#    Example: grep -r "\.unwrap()" src/ to find error handling issues
#    Example: Look for code duplication patterns
#    Example: Analyze test coverage gaps
# 3. Copilot identifies ONE specific improvement opportunity
#    Example: "Found 15 .unwrap() calls in file I/O, should use proper error handling"
# 4. Copilot implements the improvement following the checklist:
#    - Phase 1: Analysis & Planning (document findings)
#    - Phase 2: Core Refactoring/Optimization
#    - Phase 3: Testing (ensure no behavior changes)
#    - Phase 4: Code Quality & Documentation
#    - Phase 5: Performance Validation (if applicable)
#    - Phase 6: Review & Finalization
# 5. Each phase is committed with validation
# 6. Issue is closed when all success criteria are met
```

## Features Reference

All features are documented in `features.md` at the repository root. The file contains:

- 🚀 **Neue Features** (New Features)
  - Erweiterte Wirtschaftsmechaniken (Advanced Economic Mechanics)
  - Erweiterte Marktmechanismen (Advanced Market Mechanisms)
  - Soziale Netzwerke und Beziehungen (Social Networks and Relationships)
  - Erweiterte Szenarien (Extended Scenarios)
  - Visualisierung und Analyse (Visualization and Analysis)
  - Verschiedene Agentenstrategien (Various Agent Strategies)

- 🔧 **Code-Verbesserungen** (Code Improvements)
  - Architektur und Design (Architecture and Design)
  - Performance-Optimierungen (Performance Optimizations)
  - Code-Qualität (Code Quality)
  - Konfiguration und Deployment (Configuration and Deployment)
  - Datenmanagement (Data Management)

- 📊 **Analyse und Forschung** (Analysis and Research)
- 🛠️ **Entwickler-Tools** (Developer Tools)
- 🌍 **Erweiterungen für spezifische Anwendungsfälle** (Extensions for Specific Use Cases)

## Priority Levels (from features.md)

### For New Features (Copilot Feature Development)

#### Hohe Priorität (High Priority - Quick Wins)
1. Logging-System implementieren
2. Erweiterte Tests schreiben
3. Dokumentation vervollständigen
4. CLI mit Progress Bar verbessern
5. YAML/TOML Konfiguration

#### Mittlere Priorität (Medium Priority - Value Add)
1. Event-System einführen
2. Mehrere Fähigkeiten pro Person
3. Reputation-System
4. Checkpoint-System
5. REST API

#### Niedrige Priorität (Low Priority - Long-term)
1. Geografische Komponente
2. Datenbank-Integration
3. Plugin-System
4. Produktionssimulation mit Rezepten
5. Politische Simulation

### For Code Improvements (Copilot Code Improvements)

#### High Priority (Continuous)
1. **Parallele Trade-Matching** - Performance for large simulations (>1000 persons)
2. **Inkrementelle Statistiken** - Scalability improvements
3. **Integration-Tests** - Quality assurance
4. **Erweiterbare Architektur** - Long-term maintainability

#### Quality Improvements
- Reduce code duplication
- Improve error handling
- Enhance documentation
- Simplify complex logic

#### Test Improvements
- Increase test coverage
- Add property-based tests
- Improve test organization
- Add integration tests
- Enhance doctest examples

## Contributing

When implementing features or improvements:
- ✅ Maintain backward compatibility
- ✅ Write tests for new functionality
- ✅ **Aim for 100% code coverage** (minimum: 56%, target for new code: 80%+)
- ✅ Update documentation
- ✅ Consider performance implications
- ✅ Follow existing code style (run `cargo fmt`)
- ✅ Fix linting issues (run `cargo clippy`)
- ✅ For code improvements: ensure no behavior changes

## Additional Resources

- **Project README:** [README.md](../../README.md)
- **Features List:** [features.md](../../features.md)
- **Copilot Instructions:** [copilot-instructions.md](../copilot-instructions.md)
- **CI/CD Workflow:** [.github/workflows/rust.yml](../workflows/rust.yml)

## Feedback

If you have suggestions for improving these templates, please:
1. Open an issue using the "General Issue / Question" template
2. Describe your suggested improvements
3. Explain the benefit of the change
