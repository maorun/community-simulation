# Features und Codeverbesserungen

Dieses Dokument enthält eine Sammlung möglicher Features und Verbesserungen für das Community-Simulation Framework.

## 🚀 Neue Features

### 1. Erweiterte Marktmechanismen

#### 1.1 Arbitrage-Handel
**Beschreibung:** Personen können Preisunterschiede zwischen Marktsegmenten oder geografischen Regionen ausnutzen, indem sie Skills günstig kaufen und teuer weiterverkaufen.

**Nutzen:** Modellierung von Zwischenhändlern, Markteffizienz durch Preisangleichung, realistische Handelsströme.

**Implementierung:** Personen mit "Arbitrageur"-Strategie scannen alle verfügbaren Märkte und führen profitable Arbitrage-Transaktionen durch. Risiko: Transport-/Transaktionskosten könnten Gewinn auffressen.

#### 1.2 Futures-Märkte & Absicherung
**Beschreibung:** Handel mit Termingeschäften - Vereinbarungen zum Kauf/Verkauf von Skills zu festgelegten Preisen in der Zukunft.

**Nutzen:** Risikomanagement gegen Preisschwankungen, Spekulation auf zukünftige Preisentwicklungen, Forward Guidance für Märkte.

**Implementierung:** Neue `FuturesContract`-Struktur mit Fälligkeitsdatum, vereinbartem Preis und zugrunde liegender Skill. Personen können Long/Short-Positionen eingehen.

#### 1.3 Informationsasymmetrie & Signaling
**Beschreibung:** Unterschiedliche Informationsstände zwischen Käufern und Verkäufern. Verkäufer kennen wahre Qualität, Käufer nur durchschnittliche Marktqualität (Lemons Problem).

**Nutzen:** Adverse Selection, Signaling durch Zertifikate/Garantien, Informationskosten und -beschaffung.

**Implementierung:** Versteckte Qualitätsattribute, Signaling-Mechanismen (teure aber vertrauenswürdige Signale), Screening durch Käufer.

#### 1.4 Marktmacht & Kartelle
**Beschreibung:** Personen mit dominanter Marktposition können Preise strategisch setzen. Mehrere Verkäufer können Kartelle bilden, um Preise künstlich hochzuhalten.

**Nutzen:** Monopolmacht-Effekte, Kartellbildung und -zusammenbruch, Regulierungsbedarf, Wohlfahrtsverluste.

**Implementierung:** Marktanteil-Berechnung, Kartell-Koordinationsmechanismus mit Anreizen zum Betrug, Anti-Trust-Interventionen.

#### 1.5 Liquiditätsengpässe & Bankruns
**Beschreibung:** Plötzlicher Vertrauensverlust führt zu massenhaften Abhebungen von Ersparnissen oder Panikverkäufen von Assets.

**Nutzen:** Finanzkrisen-Dynamik, Bank-Runs, Selbstverstärkende Liquiditätskrisen, Rolle von Einlagensicherungen.

**Implementierung:** Vertrauens-Schocks auslösen Massenaktion, Liquiditätsbeschränkungen bei Banken/Pools, Circuit Breakers.

### 2. Soziale Netzwerke und Beziehungen

#### 2.1 Nepotismus & Vetternwirtschaft ✅ (Implementiert)
**Beschreibung:** Bevorzugung von Freunden/Familienmitgliedern bei Jobvergabe, Krediten oder Handelspartnern, auch wenn andere qualifizierter/günstiger sind.

**Nutzen:** Ineffizienzen durch soziale Präferenzen, Korruption, Netzwerk-basierte Vorteile vs. Meritokratie.

**Implementierung:**
- Konfiguration: `enable_nepotism`, `nepotism_bias`, `nepotism_outgroup_penalty`, `nepotism_inner_circle_only`.
- Bevorzugte Partnerauswahl im Handel (`select_seller_with_nepotism`): Gewichtung von sozialer Nähe (`calculate_social_closeness`) und ökonomischem Angebot (`calculate_candidate_score`).
- Negativfaktor / Surcharge für Nicht-Freunde (`nepotism_outgroup_penalty`): Preisaufschlag für Transaktionen außerhalb des Freundschafts- bzw. Vertrauensnetzwerks.
- Inner-Circle-Exklusivität (`nepotism_inner_circle_only`): Beschränkung auf Kontakte aus dem sozialen Umfeld, wenn vorhanden.
- Kredittoleranz & Bevorzugung im P2P-Lending (`find_best_offer_with_nepotism`).
- Detailliertes Metrik-Tracking (`NepotismStats`): Nepotistische vs. meritokratische Transaktionen, Inner-Circle-Quote, angewandte Outgroup-Penalties, aggregierter Wohlfahrtsverlust (`total_welfare_loss`).

#### 2.2 Soziale Mobilität & Klassenbarrieren
**Beschreibung:** Explizite Klassensysteme (Unterschicht/Mittelschicht/Oberschicht) mit erschwerten Aufstiegschancen und ungleichem Zugang zu Bildung/Krediten.

**Nutzen:** Langfristige Ungleichheit, Generationenübergreifende Armut, Wirkung von Bildungsinvestitionen auf Mobilität.

**Implementierung:** Klassenattribut bei Geburt, Bildungskosten/Zugang klassenabhängig, Netzwerk-Effekte verstärken Klassentrennung.

#### 2.3 Kulturelle/Sprachliche Barrieren
**Beschreibung:** Personen aus verschiedenen "Kulturen" haben höhere Handelskosten, langsameren Vertrauensaufbau und bevorzugen Handel mit eigener Gruppe.

**Nutzen:** Segregation, Diversitäts-Effekte, Integration vs. Parallelgesellschaften, Mehrsprachigkeits-Vorteile.

**Implementierung:** Kultur-Tags, erhöhte Transaktionskosten für kulturübergreifenden Handel, Sprachwissen als Skill.

#### 2.4 Reputationssysteme mit Manipulation
**Beschreibung:** Personen können durch gefälschte Reviews, Sybil-Attacken oder koordinierte Bewertungen ihre Reputation künstlich erhöhen.

**Nutzen:** Vertrauensmissbrauch, Plattform-Ökonomie-Probleme, Notwendigkeit von Verifizierung.

**Implementierung:** Kosten für gefälschte Reputation, Entdeckungswahrscheinlichkeit, Strafen bei Aufdeckung.

#### 2.5 Sozialkapital-Erosion
**Beschreibung:** Freundschaften können durch negative Ereignisse (Kreditausfall, Vertragsbruch, Wettbewerb) zerbrechen.

**Nutzen:** Dynamische Netzwerke, Vertrauensverlust nach Enttäuschungen, Reparaturmechanismen.

**Implementierung:** Freundschafts-"Strength" die sinken kann, Freundschaft endet bei Unterschreitung eines Schwellwerts.

### 3. Erweiterte Szenarien

#### 3.1 Demografischer Wandel
**Beschreibung:** Alterung der Bevölkerung mit steigendem Altersabhängigkeitsverhältnis, Geburtenrückgang, unterschiedliche Produktivität nach Alter.

**Nutzen:** Rentensysteme unter Druck, Pflege-Ökonomie, generationenübergreifende Umverteilung.

**Implementierung:** Alter-Attribut, altersabhängige Produktivität/Skill-Decay, Renten-Zahlungen, Pflegebedarf.

#### 3.2 Technologische Singularität
**Beschreibung:** Exponentieller Anstieg der Automatisierung führt zu rapidem Skill-Obsoleszenz und massiver Arbeitslosigkeit in kurzer Zeit.

**Nutzen:** Zukunftsszenarien mit KI/Robotik, bedingungsloses Grundeinkommen als Lösung, Post-Scarcity-Ökonomie.

**Implementierung:** Beschleunigter Automation-Risk-Anstieg, plötzliche Skill-Entwertung, UBI-Experimente.

#### 3.3 Pandemie & Gesundheitskrisen
**Beschreibung:** Infektionskrankheit breitet sich über Netzwerk aus, betroffene Personen haben reduzierte Produktivität, erhöhte Kosten, können sterben.

**Nutzen:** Epidemiologische Modellierung, Wirtschaftseffekte von Lockdowns, Gesundheitssystem-Belastung.

**Implementierung:** Infektionsstatus, Ansteckung über Handelskontakte, Quarantäne-Maßnahmen, Behandlungskosten.

#### 3.4 Ressourcenknappheit & Peak Oil
**Beschreibung:** Endliche natürliche Ressourcen (Öl, seltene Erden) werden knapp, Extraktionskosten steigen exponentiell.

**Nutzen:** Ressourcenökonomie, Substitutionseffekte, Energiewende-Dynamik, Malthusianische Grenzen.

**Implementierung:** Ressourcen-Pools mit Abbauraten, steigende Kosten bei Erschöpfung, Alternative Energien mit höheren Anfangskosten.

#### 3.5 Migrationsdynamik
**Beschreibung:** Personen können zwischen Regionen migrieren basierend auf Wirtschaftschancen, Lebensqualität, Krisen.

**Nutzen:** Brain Drain vs. Brain Gain, Remittances-Effekte, Integration von Migranten, Urbanisierung.

**Implementierung:** Multi-Region-Simulation, Migrations-Entscheidungsmodell, Transaktionskosten für Migration, Heimatmarkt-Zugang.

### 4. Verschiedene Agentenstrategien

#### 4.1 Altruismus & Gemeinnützigkeit
**Beschreibung:** Einige Personen maximieren nicht nur eigenen Nutzen, sondern berücksichtigen Wohlergehen anderer (Utilitarismus, Effektiver Altruismus).

**Nutzen:** Charity-Sektoren, Spendenverhalten, Public Goods, soziale Präferenzen.

**Implementierung:** Utility-Funktion enthält gewichtetes Durchschnitts-Wohlergehen, freiwillige Transfers zu Ärmeren.

#### 4.2 Heuristiken & Bounded Rationality
**Beschreibung:** Statt optimaler Entscheidungen nutzen Personen Faustregeln: "Kaufe vom Billigsten", "Vertraue Freunden", "Folge der Mehrheit".

**Nutzen:** Realistische Entscheidungsfindung, schnelle aber suboptimale Strategien, Heuristik-Evaluation.

**Implementierung:** Verschiedene Heuristic-Strategien, Performance-Vergleich zu Optimization-Agents.

#### 4.3 Risikoliebende vs. Risikoaverse Profile
**Beschreibung:** Unterschiedliche Risikopräferenzen beeinflussen Investment-, Kredit- und Versicherungsentscheidungen.

**Nutzen:** Portfolio-Diversifikation, Versicherungsnachfrage, Entrepreneurship-Neigung.

**Implementierung:** Risiko-Parameter (risk_aversion coefficient), beeinflusst Utility-Berechnung bei unsicheren Outcomes.

#### 4.4 Emotionale & irrationale Strategien
**Beschreibung:** Panik-Verkäufe bei Krisen, FOMO (Fear of Missing Out) bei Booms, Rache-Verhalten nach schlechten Erfahrungen.

**Nutzen:** Blasenbildung und Crashes, herding behavior, emotional-getriebene Volatilität.

**Implementierung:** Emotionale Zustände (Fear, Greed, Anger) beeinflussen Entscheidungen, Feedback-Loops verstärken Emotionen.

## 🔧 Code-Verbesserungen

### 1. Architektur und Design

#### 1.1 Plugin-System für benutzerdefinierte Features
**Beschreibung:** Dynamisches Laden von benutzerdefinierten Features und Strategien zur Laufzeit über WASM oder dynamische Bibliotheken.

**Nutzen:** Erweiterbarkeit ohne Recompilierung, Community-Beiträge als Plugins, Sandbox-Sicherheit.

**Implementierung:** Plugin-API-Definition, WASM-Runtime-Integration, Plugin-Registry und Lifecycle-Management.

#### 1.2 Modular State Machine für Simulation-Lifecycle
**Beschreibung:** Klare Zustandsmaschine mit Pre-Step, Trading, Post-Trading, Update-Phasen mit Hooks für Erweiterungen.

**Nutzen:** Vorhersagbare Ausführungsreihenfolge, einfacheres Debugging, Plugin-Integration-Points.

**Implementierung:** State-Pattern mit definierten Übergangsregeln, Event-Hooks an jedem Übergang.

#### 1.3 Dependency Injection Container
**Beschreibung:** IoC-Container für Komponenten-Abhängigkeiten statt hartcodierter Verbindungen.

**Nutzen:** Testability durch Mock-Injektion, Konfigurierbarkeit, Lose Kopplung.

**Implementierung:** Trait-basierte Abstraktion, Container mit Lifetime-Management, Factory-Pattern.

#### 1.4 Command Pattern für Simulation-Befehle
**Beschreibung:** Alle Aktionen als Command-Objekte mit Undo/Redo-Fähigkeit.

**Nutzen:** Replay-Funktionalität, deterministische Reproduktion, Time-Travel-Debugging.

**Implementierung:** Command-Trait mit execute/undo, Command-Queue, Snapshot-basiertes Undo.

### 2. Performance-Optimierungen

#### 2.1 GPU-beschleunigte Berechnungen
**Beschreibung:** Parallelisierung von rechenintensiven Operationen (Netzwerk-Analysen, Matrix-Berechnungen) auf GPU.

**Nutzen:** Skalierung auf 10,000+ Agenten, schnellere Parameter-Sweeps, Echtzeit-Visualisierung.

**Implementierung:** CUDA/Vulkan-Compute-Integration für Bulk-Operationen, Device-Host-Memory-Transfer minimieren.

#### 2.2 Adaptive Sampling & Level-of-Detail
**Beschreibung:** Bei großen Simulationen werden unwichtige Agenten mit niedrigerer Frequenz oder vereinfachten Regeln simuliert.

**Nutzen:** Konstante Performance unabhängig von Agent-Anzahl, Fokus auf relevante Bereiche.

**Implementierung:** Wichtigkeits-Score für Agenten, Multi-Resolution-Update-Scheduling, Upsampling bei Bedarf.

#### 2.3 Inkrementelle Netzwerk-Updates
**Beschreibung:** Netzwerk-Metriken (Centrality, Clustering) nur bei Änderungen neu berechnen, nicht jeden Step.

**Nutzen:** Reduzierter Overhead bei stabilen Netzwerken, schnellere Ausführung.

**Implementierung:** Dirty-Flag-Tracking für Graph-Änderungen, Lazy-Evaluation von Metriken.

#### 2.4 Memory Pooling & Object Reuse
**Beschreibung:** Wiederverwendung von Objekten statt Allokation/Deallokation in jedem Step.

**Nutzen:** Reduzierte Allocator-Contention, weniger Fragmentierung, Cache-Freundlichkeit.

**Implementierung:** Pre-allocated Object Pools, Reset-Methode statt Drop, RAII-Guards.

### 3. Code-Qualität

#### 3.1 Formale Verifikation kritischer Invarianten
**Beschreibung:** Mathematische Beweise, dass wichtige Eigenschaften (z.B. Geld-Erhaltung) immer gelten.

**Nutzen:** Garantierte Korrektheit, Vertrauen in Simulation, Fehlerprävention.

**Implementierung:** Coq/Lean-Integration, Rustprover-Annotationen, Exhaustive-Property-Testing.

#### 3.2 Mutation Testing
**Beschreibung:** Automatisches Einführen von Bugs, um Test-Suite-Qualität zu messen.

**Nutzen:** Schwachstellen in Tests identifizieren, höhere Testabdeckung, Regression-Prävention.

**Implementierung:** Mutagen oder Stryker-Integration, CI-Pipeline-Integration, Mutation-Score-Tracking.

#### 3.3 Fuzz Testing für Konfigurationen
**Beschreibung:** Zufälliges Erzeugen von Konfigurationen und Prüfung auf Crashes/Panics.

**Nutzen:** Edge-Case-Erkennung, Robustheit gegen ungültige Eingaben, Sicherheit.

**Implementierung:** cargo-fuzz mit Custom Mutators, Property-based-Tests mit Arbitrary-Derive.

#### 3.4 Continuous Benchmarking
**Beschreibung:** Performance-Regression-Tracking durch automatische Benchmarks bei jedem Commit.

**Nutzen:** Frühes Erkennen von Performance-Problemen, Optimierungs-Validierung, Historische Trends.

**Implementierung:** Criterion-Integration in CI, Benchmark-Ergebnis-Visualisierung, Automatische Alerts.

### 4. Datenmanagement

#### 4.1 Time-Series-Datenbank-Integration
**Beschreibung:** Speicherung von Simulations-Zeitreihen in spezialisierter DB (InfluxDB, TimescaleDB) statt JSON.

**Nutzen:** Effiziente Range-Queries, Aggregationen, sehr lange Simulationen, Multi-Run-Vergleiche.

**Implementierung:** DB-Client-Integration, Streaming-Write während Simulation, Query-API für Analyse.

#### 4.2 Parquet-Export für Analytics
**Beschreibung:** Export der Ergebnisse im Parquet-Format für Big-Data-Analyse mit Apache Spark/Pandas.

**Nutzen:** Spaltenorientierte Kompression, Interoperabilität, Data-Science-Workflows.

**Implementierung:** arrow/parquet-Crate-Integration, Schema-Definition, Batch-Writing.

#### 4.3 Incremental Checkpointing
**Beschreibung:** Nur geänderte Daten im Checkpoint speichern, nicht kompletter State.

**Nutzen:** Schnelleres Checkpointing, weniger Speicherplatz, häufigere Snapshots möglich.

**Implementierung:** Diff-basierte Serialisierung, Change-Tracking pro Entity, Delta-Encoding.

#### 4.4 Verteilte Simulations-Datenbank
**Beschreibung:** Sharding der Simulations-Daten über mehrere Nodes für horizontale Skalierung.

**Nutzen:** Unbegrenzte Simulation-Größe, fault-tolerance, geografisch verteilte Simulationen.

**Implementierung:** Distributed Hash Table für Entities, Consensus-Protokoll, Eventual Consistency.

## 🛠️ Entwickler-Tools

### 1. Debugging-Tools

#### 1.1 Interaktiver Time-Travel-Debugger
**Beschreibung:** Schrittweises Vor- und Zurückspulen der Simulation mit Breakpoints und State-Inspektion.

**Nutzen:** Ursachenforschung bei unerwarteten Ergebnissen, visuelle Exploration von Entscheidungen.

**Implementierung:** Web-UI mit Step-Controls, State-Viewer, Conditional Breakpoints auf Agent-Events.

#### 1.2 Agent-Profiler
**Beschreibung:** Performance-Profiling pro Agent - welche Agenten verursachen die meiste Rechenzeit?

**Nutzen:** Performance-Hotspots identifizieren, Load-Balancing-Probleme, Strategie-Optimierung.

**Implementierung:** Per-Agent CPU-Time-Tracking, Flamegraph-Visualisierung, Sampling-Profiler.

#### 1.3 Causality Tracer
**Beschreibung:** Rückverfolgung von Effekten zu ihren Ursachen - "Warum hat Agent X Geld verloren?"

**Nutzen:** Komplexe Kausalitätsketten verstehen, What-If-Analysen, Explaining AI.

**Implementierung:** Event-Dependency-Graph, Provenance-Tracking, Counterfactual-Simulation.

#### 1.4 Assertion-basierte Validierung
**Beschreibung:** Benutzer-definierte Invarianten, die während Simulation überprüft werden (z.B. "Gesamtgeld konstant").

**Nutzen:** Schnelles Erkennen von Bugs, Domain-spezifische Constraints, Test-Automation.

**Implementierung:** Assertion-DSL, Runtime-Checking mit Detail-Feedback, Assertion-Violation-Reports.

#### 1.5 Visual Scenario Builder
**Beschreibung:** Grafisches Interface zum Erstellen von Szenarien ohne YAML/Code zu schreiben.

**Nutzen:** Niedrige Einstiegshürde, schnelles Prototyping, A/B-Testing von Parametern.

**Implementierung:** Web-basierter Drag-and-Drop-Editor, Parameter-Widgets, Preview & Export zu YAML.

### 2. Visualisierung & Dashboards

#### 2.1 Echtzeit-Monitoring-Dashboard
**Beschreibung:** Live-Dashboards während Simulation-Ausführung mit Graphs, Metriken und Agent-Positionen.

**Nutzen:** Sofortiges Feedback, Erkennen von Problemen während Laufzeit, Demonstrationszwecke.

**Implementierung:** WebSocket-Streaming, React/D3.js-Frontend, Konfigurierbare Widget-Layouts.

#### 2.2 Network-Graph-Visualisierung
**Beschreibung:** Interaktive Darstellung des sozialen Netzwerks mit Friendship/Trust-Beziehungen.

**Nutzen:** Visuelle Netzwerk-Analyse, Cluster-Erkennung, Einflussträger identifizieren.

**Implementierung:** Force-Directed Layout (D3-Force), Zoom/Pan, Node-Detail on Hover, Community-Detection-Coloring.

#### 2.3 Gini-Koeffizient-Lorenz-Kurven
**Beschreibung:** Dynamische Lorenz-Kurven für Vermögen, Einkommen, Bildung über Zeit.

**Nutzen:** Ungleichheits-Tracking, Policy-Impact-Visualisierung, Research-Output.

**Implementierung:** SVG-Plotting, Animierte Kurven, Multi-Run-Overlay für Vergleiche.

#### 2.4 Heatmaps für geografische Aktivität
**Beschreibung:** 2D-Heatmap der Handelsaktivität, Wohlstandsverteilung nach geografischer Position.

**Nutzen:** Räumliche Ökonomie-Patterns, Urbanisierung, Transport-Hub-Identifikation.

**Implementierung:** Grid-basierte Aggregation, Color-Mapping, Time-Animation-Slider.

### 3. Testing & Quality Assurance

#### 3.1 Property-based Integration Tests
**Beschreibung:** Automatisch generierte Test-Szenarien mit invarianten-Überprüfung (z.B. proptest).

**Nutzen:** Breite Test-Coverage, Edge-Cases finden, Robustheit.

**Implementierung:** Proptest-Strategien für Config-Generation, Custom Shrinkers, Regression-Tests.

#### 3.2 Chaos Engineering für Simulation
**Beschreibung:** Zufälliges Injecten von Fehlern (Agent-Crashes, Netzwerk-Partitions) zur Resilienz-Prüfung.

**Nutzen:** Fehlertoleranz-Validierung, Graceful Degradation, Disaster-Recovery.

**Implementierung:** Fault-Injection-Framework, Failure-Scenarios, Recovery-Metrics.

#### 3.3 Differential Testing
**Beschreibung:** Vergleich von Simulation-Ergebnissen mit alternativen Implementierungen oder bekannten analytischen Lösungen.

**Nutzen:** Korrektheit-Validierung, Regressions-Erkennung, Cross-Referencing.

**Implementierung:** Reference-Implementierung in Python, Numerical-Solver für Gleichgewichte, Automated-Comparison.

### 4. Dokumentation & Education

#### 4.1 Interaktive Tutorials
**Beschreibung:** Step-by-Step-Guides mit ausführbaren Code-Snippets und Ergebnis-Visualisierung.

**Nutzen:** Onboarding neuer Nutzer, Best-Practices-Vermittlung, Feature-Discovery.

**Implementierung:** mdBook mit Rust-Playground-Integration, Embedded-Visualizations, Quizzes.

#### 4.2 Economics-Glossar
**Beschreibung:** In-App-Glossar mit Erklärungen ökonomischer Konzepte und wie sie in der Simulation umgesetzt sind.

**Nutzen:** Bildungszweck, Verständnis für Nicht-Ökonomen, Linking zu akademischen Ressourcen.

**Implementierung:** Markdown-basiertes Glossar, Kontext-sensitive Hilfe, Suchfunktion.

#### 4.3 Beispiel-Szenarien-Bibliothek
**Beschreibung:** Kuratierte Sammlung von Szenarien mit Beschreibungen und erwarteten Ergebnissen.

**Nutzen:** Inspiration, Reproduzierbare Forschung, Benchmarking.

**Implementierung:** Git-Submodule oder Registry, Metadata (Tags, Difficulty, Duration), Download-Metriken.

## 🎯 Priorisierung

### Hohe Priorität (Kurzfristig - hoher Forschungswert)

1. **Informationsasymmetrie & Signaling (1.3)** - Klassisches Marktversagen-Modell, gut erforscht, klare Implementierung
2. **Heuristiken & Bounded Rationality (4.2)** - Realistische Entscheidungsfindung, Integration mit bestehendem Reinforcement Learning
3. **Futures-Märkte (1.2)** - Ergänzt bestehende Finanzsysteme (Loans, Investments), ermöglicht Risikohedging
4. **Echtzeit-Monitoring-Dashboard (2.1)** - Großer UX-Gewinn, nützlich für alle Nutzer, Demo-Fähigkeit

### Mittlere Priorität (Mittelfristig - erweitert bestehende Features)

5. **Marktmacht & Kartelle (1.4)** - Baut auf bestehendem Black-Market-System auf, wichtig für Regulierungsforschung
6. **Sozialkapital-Erosion (2.5)** - Erweitert Friendship-System um Dynamik, realistischere Netzwerke
7. **Demografischer Wandel (3.1)** - Hochrelevant für Policy-Forschung, erfordert Generationen-Modell
8. **Risikoliebende vs. Risikoaverse Profile (4.3)** - Ergänzt Credit-Rating und Insurance, erklärt Portfolio-Choices
9. **Time-Series-Datenbank-Integration (4.1)** - Technische Infrastruktur für Langzeit-Simulationen
10. **Network-Graph-Visualisierung (2.2)** - Nutzt bestehendes Trust-Network, visuelles Debugging

### Niedrige Priorität (Langfristig - Spezialfälle)

11. **GPU-beschleunigte Berechnungen (2.1)** - Nur bei >10,000 Agenten nötig, komplexe Implementierung
12. **Technologische Singularität (3.2)** - Spekulatives Szenario, weniger akademische Relevanz
13. **Plugin-System (1.1)** - Hoher Architektur-Aufwand, erst bei großer Community sinnvoll
14. **Pandemie & Gesundheitskrisen (3.3)** - Spezifisches Szenario, weniger Harmonie mit anderen Features
15. **Verteilte Simulations-Datenbank (4.4)** - Nur für extreme Skalierung, sehr komplex
16. **Spezielle Anwendungsfälle** - Domänenspezifisch

## 🔄 Kontinuierliche Verbesserungen
- Integration-Tests für jedes neue Feature
- Refactoring zu komponenten-basierter Architektur
- Dokumentations-Updates
  - ✅ Comprehensive configuration file documentation completed (all 108 parameters documented in YAML and TOML formats with usage examples)
  - ✅ Insurance system example added (examples/insurance_demo.rs demonstrating all three insurance types with comparative analysis)

## 📝 Notizen

Diese Liste ist als lebendiges Dokument gedacht und sollte regelmäßig aktualisiert werden, wenn neue Ideen entstehen oder Features implementiert werden.

Bei der Implementierung neuer Features sollte immer darauf geachtet werden:
- **Rückwärtskompatibilität** zu wahren
- **Tests zu schreiben** (Unit + Integration)
- **Dokumentation zu aktualisieren** (README.md, Code-Kommentare)
- **Performance-Implikationen** zu bedenken
- **Feature-Toggles** zu nutzen (via Config oder Compile-Time Flags)
- **Harmonien zu maximieren** - Neue Features sollten mit existierenden synergieren

### Harmonien-Design-Prinzipien

Bei der Entwicklung neuer Features sollten folgende Prinzipien beachtet werden:

1. **Multiplexe Verknüpfung**: Features sollten mit mindestens 3-5 anderen Features interagieren
2. **Emergente Effekte**: Features sollten unerwartete Verhaltensweisen ermöglichen
3. **Konfigurierbare Stärke**: Interaktionseffekte sollten parametrisierbar sein
4. **Datensynergien**: Features sollten Daten produzieren, die andere Features nutzen können
5. **Mechanismus-Komposition**: Einfache Mechanismen kombiniert zu komplexem Verhalten

### Vorschläge zur Feature-Priorisierung

Verwende diese Kriterien für die Priorisierung zusätzlicher Features:

1. **Harmonie-Score** (0-10): Anzahl und Stärke der Verknüpfungen mit existierenden Features
2. **Implementierungs-Aufwand** (S/M/L/XL): Geschätzter Entwicklungsaufwand
3. **Forschungs-Relevanz** (0-10): Wie interessant ist das Feature für ökonomische Forschung?
4. **Praxis-Relevanz** (0-10): Wie relevant ist das Feature für reale Wirtschaftssysteme?
5. **Lehr-Eignung** (0-10): Wie gut eignet sich das Feature für Lehrzwecke?

**Beispiel-Scoring (Neue Features):**

| Feature | Harmonie | Aufwand | Forschung | Praxis | Lehre | Gesamt |
|---------|----------|---------|-----------|--------|-------|--------|
| Informationsasymmetrie | 9 | M | 10 | 9 | 10 | 38/50 |
| Futures-Märkte | 8 | M | 8 | 10 | 7 | 33/50 |
| Heuristiken | 9 | S | 8 | 8 | 9 | 34/50 |
| Marktmacht/Kartelle | 7 | L | 9 | 10 | 8 | 34/50 |
| Demografischer Wandel | 6 | XL | 9 | 10 | 7 | 32/50 |
| Echtzeit-Dashboard | 5 | M | 5 | 6 | 8 | 24/50 |
| GPU-Beschleunigung | 3 | XL | 4 | 5 | 3 | 15/50 |

**Legende:**
- **Harmonie-Score**: Wie viele existierende Features werden erweitert/genutzt?
- **Aufwand**: S (Small, 1-2 Wochen), M (Medium, 1 Monat), L (Large, 2-3 Monate), XL (Extra Large, >3 Monate)
- **Forschungs-Relevanz**: Gibt es etablierte ökonomische Theorien dazu?
- **Praxis-Relevanz**: Wie häufig kommt das in realen Märkten vor?
- **Lehr-Eignung**: Ist es intuitiv verständlich und lehrreich?

Contributions sind willkommen! Bitte öffnen Sie ein Issue oder Pull Request, um Diskussionen zu starten oder Änderungen vorzuschlagen.

### Wie man beiträgt

1. **Issue öffnen**: Beschreibe das Feature und seine Harmonien mit bestehenden Features
2. **Design diskutieren**: Community-Feedback zu Implementierungsdetails
3. **Pull Request**: Implementation mit Tests und Dokumentation
4. **Review**: Code-Review mit Fokus auf Harmonien und Qualität
5. **Integration**: Merge und Aktualisierung dieser Features-Liste
