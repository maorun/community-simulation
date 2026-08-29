---
title: '[FEATURE] Futures-Märkte & Absicherungskontrakte (Hedging)'
labels: ['enhancement', 'financial-systems', 'risk-management']
assignees: ''
---

## 📌 Beschreibung
Einführung von Termingeschäften (Futures / Forward Contracts) für Skills und Rohstoffe. Agenten können Kontrakte abschließen, um Skills zu einem festgelegten zukünftigen Zeitpunkt und Preis zu kaufen oder zu verkaufen.

## 🎯 Nutzen & Relevanz
- **Risikomanagement**: Produzenten und Konsumenten können sich gegen Preisschwankungen und Volatilität absichern (Hedging).
- **Markteffizienz**: Futures-Preise bieten eine Preissignalfunktion für zukünftige Marktphasen (Forward Guidance).
- **Erweiterte Strategien**: Spekulanten können Long- und Short-Positionen eingehen.

## 📐 Technische Spezifikation & Implementierung
1. **Datenstruktur `FuturesContract`**:
   ```rust
   pub struct FuturesContract {
       pub id: usize,
       pub seller_id: usize,
       pub buyer_id: Option<usize>,
       pub skill_name: String,
       pub execution_step: usize,
       pub strike_price: f64,
       pub quantity: f64,
       pub margin_deposit: f64,
   }
   ```
2. **Orderbuch für Terminkontrakte**:
   - Neues Modul `src/futures.rs` oder Integration in `src/market.rs` zur Verwaltung offener Futures-Orders.
3. **Execution Engine Phase**:
   - Automatische Abwicklung der Kontrakte bei Erreichen des `execution_step` in `src/engine.rs`.
   - Nachschusspflicht (Margin Call) und Liquidation bei Unvermögen.
4. **Konfiguration**:
   - `enable_futures_market: bool`, `futures_margin_requirement: f64`.

## 📋 Implementation Checklist
- [ ] Modul `src/futures.rs` mit Kontrakt- und Orderbuch-Strukturen erstellen
- [ ] Fälligkeitsprüfung und Settlement in den Engine-Simulationsschritt integrieren
- [ ] Agentenentscheidungslogik für Hedging vs. Spekulation hinzufügen
- [ ] Ergebnisauswertung & Statistiken (offenes Interesse, Futures-Volumen) in `src/result.rs` ergänzen
- [ ] Systemtests für Nachschusspflicht und Fristenabwicklung hinzufügen
