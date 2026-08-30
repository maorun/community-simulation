---
title: '[FEATURE] Informationsasymmetrie & Signaling (Lemons Market Mechanism)'
labels: ['enhancement', 'economic-mechanics', 'market-dynamics']
assignees: ''
---

## 📌 Beschreibung
Implementierung von Informationsasymmetrie zwischen Käufern und Verkäufern auf dem Marktplatz (Akerlof's "Market for Lemons"). Verkäufer kennen die exakte Qualität ihrer angebotenen Skills/Güter, während Käufer nur die durchschnittliche Marktqualität oder unvollständige Signale wahrnehmen.

## 🎯 Nutzen & Relevanz
- **Ökonomische Forschung**: Simulation von Marktversagen durch Adverse Selektion (Schlechte Qualität verdrängt gute Qualität).
- **Signaling-Mechanismen**: Untersuchung der Effizienz von Zertifikaten, Garantien und Reputation zur Überwindung von Informationsasymmetrie.
- **Realismus**: Erhöhung der Realitätstreue von Dienstleistungs- und Gütermärkten.

## 📐 Technische Spezifikation & Implementierung
1. **Versteckte Qualität**:
   - `Skill` oder Güter-Angebote erhalten ein attributives `true_quality: f64` (nur für Verkäufer einsehbar) und ein `perceived_quality: f64` (für den Markt einsehbar).
2. **Signaling-System**:
   - Einführung von Zertifikaten (`Zertifikat`-Struct) mit Anschaffungs- oder Prüfungskosten (`certification_cost`), um Käufern verifizierte Qualitäts-Signale zu senden.
3. **Screening durch Käufer**:
   - Käufer-Agenten können Inspektionskosten aufwenden, um die wahre Qualität vor dem Kauf zu enthüllen (`inspection_cost`).
4. **Konfiguration**:
   - Parameter in `SimulationConfig`: `enable_information_asymmetry: bool`, `inspection_cost: f64`, `certification_cost: f64`.

## 📋 Implementation Checklist
- [ ] `true_quality` und `perceived_quality` Datenfelder in Angeboten/Skills ergänzen
- [ ] Screening-Logik im Trade-Matching in `src/market.rs` und `src/engine.rs` integrieren
- [ ] Zertifizierungssystem in `src/skill.rs` / `src/person.rs` ausbauen
- [ ] CLI-Flags & Config-Felder in `src/config.rs` hinterlegen
- [ ] Unit- und Integrationstests für Adverse Selektion und Signaling-Effizienz schreiben
- [ ] Dokumentation in `FEATURES.md` ergänzen
