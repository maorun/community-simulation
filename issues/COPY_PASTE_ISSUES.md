# 📋 GitHub Issues - Copy & Paste Vorlagen

Hier findest du 5 evaluierte Verbesserungsvorschläge und Erweiterungen für das **Economic Simulation Framework**. Jedes Issue ist so formatiert, dass du **Titel**, **Labels** und **Beschreibung** einzeln kopieren und direkt in GitHub als neues Issue einfügen kannst.

---

## 1. Informationsasymmetrie & Signaling (Lemons Market Mechanism)

### 📌 Title (Kopieren)
```text
[FEATURE] Informationsasymmetrie & Signaling (Lemons Market Mechanism)
```

### 🏷️ Labels (Kopieren)
```text
enhancement, economic-mechanics, market-dynamics
```

### 📝 Issue Body / Description (Kopieren)
```markdown
## 📌 Beschreibung
Implementierung von Informationsasymmetrie zwischen Käufern und Verkäufern auf dem Marktplatz (Akerlof's "Market for Lemons"). Verkäufer kennen die exakte Qualität ihrer angebotenen Skills/Güter (`true_quality`), während Käufer nur die durchschnittliche Marktqualität oder unvollständige Signale wahrnehmen (`perceived_quality`).

## 🎯 Nutzen & Relevanz
- **Ökonomische Forschung**: Simulation von Marktversagen durch Adverse Selektion (minderwertige Qualität verdrängt hochwertige Angeboten).
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
```

---

## 2. Futures-Märkte & Absicherungskontrakte (Hedging)

### 📌 Title (Kopieren)
```text
[FEATURE] Futures-Märkte & Absicherungskontrakte (Hedging)
```

### 🏷️ Labels (Kopieren)
```text
enhancement, financial-systems, risk-management
```

### 📝 Issue Body / Description (Kopieren)
```markdown
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
```

---

## 3. Echtzeit Web-Monitoring & Visualisierungs-Dashboard

### 📌 Title (Kopieren)
```text
[FEATURE] Echtzeit Web-Monitoring & Visualisierungs-Dashboard
```

### 🏷️ Labels (Kopieren)
```text
enhancement, developer-experience, visualization, web
```

### 📝 Issue Body / Description (Kopieren)
```markdown
## 📌 Beschreibung
Entwicklung eines leichtgewichtigen, interaktiven Web-Dashboards zur Live-Überwachung von Simulationsläufen. Die Simulations-Engine kann Metriken über WebSocket oder HTTP-SSE an ein Web-Frontend streamen.

## 🎯 Nutzen & Relevanz
- **Echtzeit-Feedback**: Sofortiges Verfolgen von Preisverläufen, Gini-Koeffizienten und Handelsvolumina während die Simulation läuft.
- **Interaktive Demonstration**: Perfekt für Präsentationen, Lehre und schnelles Prototyping.
- **Netzwerk-Visualisierung**: Live-Darstellung dynamischer Freundschafts- und Vertrauensnetzwerke.

## 📐 Technische Spezifikation & Implementierung
1. **WebSocket Server in Rust**:
   - Optionaler WebSocket-Server (z.B. via `tokio-tungstenite` oder `axum` hinter einem Feature-Gate `--features web-dashboard`).
   - Senden von JSON-Frames nach jedem Simulationsschritt (Preise, Transaktionszahlen, Ungleichheitsmetriken).
2. **Frontend App**:
   - Single-Page Application (HTML5/TypeScript/D3.js oder Chart.js) im Ordner `web/dashboard/`.
   - Widgets: Preisverlauf-Diagramm, Lorenz-Kurve, soziale Netzwerk-Graphen.
3. **CLI Integration**:
   - Flag `--serve-dashboard [PORT]` zum automatischen Starten des Webservers und Öffnen des Dashboards.

## 📋 Implementation Checklist
- [ ] Feature-Flag `web-dashboard` in `Cargo.toml` definieren
- [ ] Non-blocking WebSocket Broadcast Manager in `src/engine.rs` integrieren
- [ ] Embedded Dashboard Assets (via `rust-embed` oder Stativ-Hosting) einbinden
- [ ] Live charts (Gini, skill prices, transaction volume) auf dem Frontend umsetzen
- [ ] CLI Command `/ --serve-dashboard` Option erweitern
```

---

## 4. Modular Plugin System via WebAssembly (WASM) & Dynamic Traits

### 📌 Title (Kopieren)
```text
[ENHANCEMENT] Modular Plugin System via WebAssembly (WASM) & Dynamic Traits
```

### 🏷️ Labels (Kopieren)
```text
architecture, enhancement, extensibility, wasm
```

### 📝 Issue Body / Description (Kopieren)
```markdown
## 📌 Beschreibung
Erweiterung der bestehenden Trait-basierten Plugin-Architektur (`src/plugin.rs`) um die Fähigkeit, benutzerdefinierte Agenten-Strategien, Marktregeln und Steuer-Policies zur Laufzeit via WebAssembly (WASM) Module zu laden.

## 🎯 Nutzen & Relevanz
- **Erweiterbarkeit ohne Neukompilierung**: Entwickler und Forscher können neue Handelsstrategien in C, C++, Rust, Python oder Go schreiben und als `.wasm`-Plugin laden.
- **Sicherheit & Sandboxing**: Skripte von Drittanbietern laufen in einer isolierten WASM-Sandbox ohne Zugriff auf das Host-Dateisystem.
- **Moduläre Ökosystem-Erstellung**: Vereinfacht die Verteilung und das Sharing von Forschungsmodellen.

## 📐 Technische Spezifikation & Implementierung
1. **WASM Host Runtime**:
   - Integration von `wasmtime` oder `wasmer` als optionale Dependency (`--features wasm-plugins`).
2. **Plugin Interface**:
   - C-ABI kompatibles Interface oder `wit-bindgen` Definition für Callbacks:
     - `on_trade_decide(agent_state) -> trade_order`
     - `on_price_adjust(market_state) -> price_map`
     - `on_policy_apply(economy_state) -> tax_rates`
3. **Integration in PluginRegistry**:
   - `WasmPlugin` Struktur, die das `SimulationPlugin`-Trait in `src/plugin.rs` implementiert.

## 📋 Implementation Checklist
- [ ] WASM Runtime Crate (`wasmtime`) einbinden
- [ ] Host-Export-Funktionen und Sandbox-Limits definieren
- [ ] `WasmPlugin` Wrapper für `SimulationPlugin` Trait erstellen
- [ ] Example-Plugin in C/Rust zur Demonstration schreiben
- [ ] Unit-Tests für Plugin-Lifecycle und Security-Boundaries erstellen
```

---

## 5. Demografischer Wandel & Alternde Bevölkerung (Demographic Dynamics)

### 📌 Title (Kopieren)
```text
[FEATURE] Demografischer Wandel & Alternde Bevölkerung (Demographic Dynamics)
```

### 🏷️ Labels (Kopieren)
```text
enhancement, scenarios, economic-policy
```

### 📝 Issue Body / Description (Kopieren)
```markdown
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
```
