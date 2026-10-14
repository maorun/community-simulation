---
title: '[FEATURE] Arbitrage-Handel & Markt-Zwischenhändler (Intermediaries)'
labels: ['enhancement', 'market-mechanisms', 'trading-strategies']
assignees: ''
---

## 📌 Beschreibung
Einführung einer Arbitrageur- bzw. Zwischenhändler-Strategie. Agenten mit dieser Strategie identifizieren Preisunterschiede zwischen geografischen Regionen oder Marktsegmenten, kaufen Skills/Güter in günstigen Teilmärkten und verkaufen sie mit Marge in teuren Teilmärkten weiter.

## 🎯 Nutzen & Relevanz
- **Markteffizienz**: Preisangleichung über Regionen und Marktsegmente hinweg (Law of One Price).
- **Händlerrollen**: Simulation der Rolle von Groß- und Einzelhändlern, Logistik- und Handelsunternehmen.
- **Handelsströme**: Entstehung komplexer Versorgungs- und Handelsketten im Multi-Regionen-Modell.

## 📐 Technische Spezifikation & Implementierung
1. **Agentenstrategie `Arbitrageur`**:
   - Neue `AgentStrategy::Arbitrageur` in `src/plugin.rs` / `src/person.rs`.
   - Regelmäßiges Scannen der Marktpreise in verschiedenen Regionen (`location`) oder Segmenten (`MarketSegment`).
2. **Handelslogik**:
   - Berücksichtigung von Transport- und Transaktionskosten (`distance_cost_factor`, `transaction_fee`).
   - Marge muss Transportkosten übersteigen: `expected_profit = price_sell - price_buy - transport_cost - fee`.
3. **Konfiguration**:
   - `enable_arbitrage: bool`, `arbitrage_min_margin: f64`.

## 📋 Implementation Checklist
- [ ] `Arbitrageur`-Strategie in Agenten-Verhaltenslogik einbauen
- [ ] Regionalübergreifende Preisscans in `src/market.rs` ermöglichen
- [ ] Transaktions- und Transportkostenberechnung für Zwischenhändler anpassen
- [ ] Statistiken zu Händlermargen und Handelsvolumen in `src/result.rs` aufnehmen
- [ ] Integrationstests zur Preiskonvergenz durch Arbitrage schreiben
