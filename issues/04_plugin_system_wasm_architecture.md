---
title: '[ENHANCEMENT] Modular Plugin System via WebAssembly (WASM) & Dynamic Traits'
labels: ['architecture', 'enhancement', 'extensibility', 'wasm']
assignees: ''
---

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
