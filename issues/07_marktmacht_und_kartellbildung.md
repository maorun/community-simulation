---
title: '[FEATURE] Marktmacht, Oligopole & Kartellbildung (Collusion & Market Power)'
labels: ['enhancement', 'market-dynamics', 'economic-policy']
assignees: ''
---

## 📌 Beschreibung
Erweiterung der Marktmechanismen um strategisches Verhalten bei hoher Marktkonzentration (HHI > 2500): Verkäufer mit hohem Marktanteil können Preise oberhalb des Grenzkostenniveaus setzen oder explizit/stillschweigend Kartelle zur Preisabsprache bilden.

## 🎯 Nutzen & Relevanz
- **Wettbewerbspolitik**: Erforschung von Wohlfahrtsverlusten (Deadweight Loss) durch Oligopole und Monopole.
- **Kartellstabilität**: Untersuchung von Anreizen zum Abweichen vom Kartellpreis (Defektion/Gefangenendilemma).
- **Regulierung**: Evaluierung von Kartellamts-Interventionen (Straffortzahlungen, Preiskappen).

## 📐 Technische Spezifikation & Implementierung
1. **Preissetzungsmacht**:
   - Nutzen der bereits berechneten Herfindahl-Hirschman-Index (HHI) und CR4-Kennzahlen (`skill_market_concentration`).
   - Preisaufschlag (Markup) als Funktion des Marktanteils $s_i$: $p_i = p_{base} \cdot (1 + \mu \cdot s_i)$.
2. **Kartell-Koordinationsmechanismus**:
   - Gruppe führender Anbieter vereinbart Mindestpreis für bestimmte Skills.
   - Anreiz zur Defektion: Heimliches Unterbieten zur Marktanteilsgewinnung.
3. **Kartellaufsicht / Regulatory Intervention**:
   - Zufällige Prüfwahrscheinlichkeit (`anti_trust_audit_probability`) und Bußgelder bei Entdeckung.

## 📋 Implementation Checklist
- [ ] Markup-Berechnung basierend auf HHI/Marktanteil in `src/market.rs` integrieren
- [ ] Kartellabsprachen-Logik und Defektionsentscheidung implementieren
- [ ] Kartellamts-Audits und Strafzahlungen in `src/engine.rs` einbauen
- [ ] Metriken zu Monopolrenten und Wohlfahrtsverlusten in `src/result.rs` ergänzen
- [ ] Tests zur Stabilität von Kartellen unter verschiedenen Marktstrukturen schreiben
