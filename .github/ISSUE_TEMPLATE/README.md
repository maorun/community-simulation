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

### 2. Bug Report (`bug_report.md`)

**Purpose:** Report bugs, errors, or unexpected behavior in the simulation framework.

**When to use:**
- When you encounter crashes or errors
- When the simulation produces incorrect results
- When something doesn't work as documented

### 3. General Issue / Question (`general_issue.md`)

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
   - **Copilot Feature Development** - For implementing features from features.md
   - **Bug Report** - For reporting bugs
   - **General Issue / Question** - For questions or discussions
4. Fill in the template fields
5. Submit the issue

## For GitHub Copilot Users

The **Copilot Feature Development** template is optimized for GitHub Copilot workflows:

1. **Comprehensive Guidance:** Includes detailed instructions for Copilot
2. **Structured Approach:** Follows a phase-based implementation plan
3. **Quality Assurance:** Built-in checklists for testing and validation
4. **Context-Aware:** References project structure and existing patterns
5. **Best Practices:** Enforces minimal changes and proper testing

### Example Workflow with Copilot

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

### Hohe Priorität (High Priority - Quick Wins)
1. Logging-System implementieren
2. Erweiterte Tests schreiben
3. Dokumentation vervollständigen
4. CLI mit Progress Bar verbessern
5. YAML/TOML Konfiguration

### Mittlere Priorität (Medium Priority - Value Add)
1. Event-System einführen
2. Mehrere Fähigkeiten pro Person
3. Reputation-System
4. Checkpoint-System
5. REST API

### Niedrige Priorität (Low Priority - Long-term)
1. Geografische Komponente
2. Datenbank-Integration
3. Plugin-System
4. Produktionssimulation mit Rezepten
5. Politische Simulation

## Contributing

When implementing features:
- ✅ Maintain backward compatibility
- ✅ Write tests for new functionality
- ✅ Update documentation
- ✅ Consider performance implications
- ✅ Follow existing code style (run `cargo fmt`)
- ✅ Fix linting issues (run `cargo clippy`)

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
