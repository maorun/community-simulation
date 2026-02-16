# Example: Using the Copilot Feature Development Template

This is an example of how to fill out the Copilot Feature Development template to implement a feature from TODO.md.

---

## Feature to Implement

**Feature ID:** 1.1

**Feature Name:** Spar- und Investitionssystem (Savings and Investment System)

**Category:** Erweiterte Wirtschaftsmechaniken / Advanced Economic Mechanics

**Priority:** Medium (Mittlere Priorität)

## Feature Description

**Description (Beschreibung):**
Personen können Geld sparen und in Fähigkeiten oder den Markt investieren.

**Benefit (Nutzen):**
Realistischere Vermögensbildung und wirtschaftliche Dynamiken. This allows persons to build wealth over time through savings and investments, creating more realistic economic behavior patterns.

**Suggested Implementation (Implementierung):**
Neue `Investment` und `Savings` Strukturen in `person.rs`

## Implementation Requirements

### Files to Create/Modify

- [ ] `src/person.rs` - Add Investment and Savings structures
- [ ] `src/engine.rs` - Add investment processing logic
- [ ] `src/config.rs` - Add configuration for interest rates and investment parameters
- [ ] `src/result.rs` - Add investment statistics to output
- [ ] `src/tests/mod.rs` - Add tests for savings and investment functionality

### Core Components

- [ ] `Savings` struct with fields: amount, interest_rate, accumulated_interest
- [ ] `Investment` struct with fields: amount, investment_type, expected_return, risk_level
- [ ] Investment processing logic in simulation step
- [ ] Interest calculation for savings accounts
- [ ] Investment return calculation with risk

### Integration Points

- [ ] Integrate with Person struct to track savings and investments
- [ ] Add investment decisions to person behavior logic
- [ ] Update transaction system to handle deposits and withdrawals
- [ ] Add investment results to JSON output

## Technical Specifications

### Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Savings {
    pub amount: f64,
    pub interest_rate: f64,
    pub accumulated_interest: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    pub amount: f64,
    pub investment_type: InvestmentType,
    pub expected_return: f64,
    pub risk_level: f64,
    pub steps_held: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvestmentType {
    SkillTraining(Skill),
    MarketInvestment,
}

// Add to Person struct:
pub struct Person {
    // ... existing fields ...
    pub savings: Savings,
    pub investments: Vec<Investment>,
}
```

### API Changes

- [ ] New functions:
  - `Person::save_money(amount: f64) -> Result<(), String>`
  - `Person::invest(amount: f64, investment_type: InvestmentType) -> Result<(), String>`
  - `Person::withdraw_savings(amount: f64) -> Result<f64, String>`
  - `Savings::apply_interest(&mut self)`
  - `Investment::calculate_return(&self) -> f64`
- [ ] Modified functions:
  - `SimulationEngine::step()` - add investment processing
  - `Person::new()` - initialize savings and investments
- [ ] New traits:
  - None required

### Configuration

- [ ] CLI arguments:
  - `--savings-interest-rate <RATE>` - Interest rate for savings (default: 0.02)
  - `--investment-enabled <BOOL>` - Enable investment system (default: true)
- [ ] Config file parameters:
  - `savings_interest_rate: f64`
  - `investment_enabled: bool`
  - `investment_risk_factor: f64`
- [ ] Default values:
  - savings_interest_rate: 0.02 (2%)
  - investment_enabled: true
  - investment_risk_factor: 0.1

## Testing Requirements

### Unit Tests

- [ ] Test Savings::apply_interest() with various rates
- [ ] Test Investment::calculate_return() with different risk levels
- [ ] Test Person::save_money() for valid and invalid amounts
- [ ] Test Person::invest() error handling for insufficient funds
- [ ] Test Person::withdraw_savings() boundary cases

### Integration Tests

- [ ] Test savings accumulation over multiple simulation steps
- [ ] Test investment returns affecting person wealth
- [ ] Test interaction between savings, investments, and trading
- [ ] Test with different interest rate configurations
- [ ] Test with investments disabled

### Performance Tests

- [ ] Benchmark impact on simulation step time
- [ ] Test memory usage with many persons having investments
- [ ] Test scalability with 1000+ persons with savings

## Documentation Requirements

- [ ] Update README.md to mention savings and investment system
- [ ] Add doc comments to all public functions
- [ ] Update TODO.md to mark 1.1 as implemented
- [ ] Add example showing how to run simulation with investments enabled

## Implementation Checklist

### Phase 1: Core Implementation
- [ ] Create Savings struct in person.rs
- [ ] Create Investment struct and InvestmentType enum in person.rs
- [ ] Add savings and investments fields to Person struct
- [ ] Implement Savings::apply_interest()
- [ ] Implement Investment::calculate_return()
- [ ] Ensure code compiles

### Phase 2: Integration
- [ ] Update Person::new() to initialize savings and investments
- [ ] Add investment logic to SimulationEngine::step()
- [ ] Update SimulationConfig with new parameters
- [ ] Add CLI arguments in main.rs
- [ ] Update JSON serialization in result.rs

### Phase 3: Testing
- [ ] Write unit tests for Savings
- [ ] Write unit tests for Investment
- [ ] Write integration tests for savings system
- [ ] Run all tests: `cargo test --verbose`
- [ ] Run doctests: `cargo test --doc --verbose`
- [ ] Verify no regressions

### Phase 4: Code Quality
- [ ] Format code: `cargo fmt`
- [ ] Run linter: `cargo clippy`
- [ ] Fix all warnings related to new code
- [ ] Run security checks (CodeQL)

### Phase 5: Documentation
- [ ] Add doc comments to Savings struct
- [ ] Add doc comments to Investment struct
- [ ] Update README.md
- [ ] Update TODO.md (mark 1.1 as done)

### Phase 6: Validation
- [ ] Build release version: `cargo build --release`
- [ ] Test with sample scenario: 100 persons, 500 steps
- [ ] Verify JSON output includes savings/investment data
- [ ] Test edge cases (zero money, high interest rates)

## Copilot Instructions

**For GitHub Copilot implementing this feature:**

Follow the standard Copilot instructions in the template, with these feature-specific notes:

1. **Start with data structures** - person.rs is the right place
2. **Keep it simple** - Start with basic savings before complex investment types
3. **Test incrementally** - Build and test after each phase
4. **Consider edge cases** - What if person has no money to save? What if interest rate is negative?
5. **Performance** - Interest calculation happens every step, keep it efficient

## Success Criteria

This feature is considered successfully implemented when:

- [ ] All code compiles without errors
- [ ] All tests pass (including new tests for savings/investments)
- [ ] All doctests pass
- [ ] Code is formatted and linted
- [ ] Persons can save money and earn interest
- [ ] Persons can make investments with configurable risk
- [ ] JSON output shows savings and investment data
- [ ] Documentation is updated
- [ ] No regressions in existing trading functionality
- [ ] Can run: `./target/release/community-simulation -s 100 -p 50 --savings-interest-rate 0.03 -o /tmp/test.json`

## Additional Notes

This is a medium-priority feature that will significantly enhance the economic realism of the simulation. It's a good starting point before implementing more complex features like credit systems (feature 1.2) since it establishes the foundation for financial instruments in the simulation.

---

**Note:** This is just an example. When creating an actual issue, customize the details based on the specific feature you want to implement.
