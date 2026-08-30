---
title: '[FEATURE] Demografischer Wandel & Alternde Bevölkerung (Demographic Dynamics)'
labels: ['enhancement', 'scenarios', 'economic-policy']
assignees: ''
---

## 📌 Beschreibung
Erweiterung des Simulation-Frameworks um demografische Dynamiken: Alterung der Agenten, Produktivitätsänderungen über die Lebensspanne, Generationenwechsel (Geburt/Vererbung/Ruhestand) und Rentensysteme.

## 🎯 Nutzen & Relevanz
- **Makroökonomische Forschung**: Untersuchung der Auswirkungen einer alternden Gesellschaft auf den Arbeitsmarkt und die Produktivität.
- **Renten- & Sozialpolitik**: Modellierung von Umlage- vs. Kapitaldeckungsverfahren für Rentensysteme unter demografischem Druck.
- **Generationsübergreifender Wohlstand**: Analyse der Vererbung von Vermögen und Fertigkeiten über Generationen hinweg.

## 📐 Technische Spezifikation & Implementierung
1. **Person-Attribute (`src/person.rs`)**:
   - `age: usize` (Erhöhung pro Simulationsschritt / Epoche).
   - `retirement_age: usize`.
   - `productivity_factor: f64` (Kurvenverlauf: Steigt in der Jugend, Plateau im mittleren Alter, Abfall im Alter).
2. **Generationswechsel & Vererbung**:
   - Nach Erreichen des Maximalalters (`max_age`) stirbt die Person; Erbe geht an Nachkommen oder den Staat.
   - Neue Agenten treten mit Grundkapital und teilweise vererbten Skills in den Markt ein.
3. **Rentensystem**:
   - Erhebung von Rentenbeiträgen arbeitender Agenten zur Auszahlung von Altersrenten an Agenten im Ruhestand.
4. **Preset Scenario**:
   - Neues Scenario / Preset: `demographic_transition` in `src/scenario.rs` und `config.example.yaml`.

## 📋 Implementation Checklist
- [ ] Alterungs- und Produktivitätslogik in `Person` implementieren
- [ ] Lifecycle-Management (Vererbung, Generierung neuer Agenten) in `SimulationEngine` einbauen
- [ ] Rentenkassen-Mechanik & Umverteilung umsetzen
- [ ] CLI-Flags (`--enable-demographics`, `--retirement-age`) & Config-Optionen hinzufügen
- [ ] Preset `demographic_transition` erstellen
- [ ] Tests zur Umlagerenten-Stabilität unter Alterungsdruck schreiben
