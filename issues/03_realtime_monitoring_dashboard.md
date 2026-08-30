---
title: '[FEATURE] Echtzeit Web-Monitoring & Visualisierungs-Dashboard'
labels: ['enhancement', 'developer-experience', 'visualization', 'web']
assignees: ''
---

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
