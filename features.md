# Features und Codeverbesserungen

Dieses Dokument enthält eine Sammlung möglicher Features und Verbesserungen für das Community-Simulation Framework.

## 🚀 Neue Features

### 1. Erweiterte Wirtschaftsmechaniken

#### 1.1 Versicherungssystem
**Beschreibung:** Ein Versicherungsmarkt, bei dem Personen Versicherungspolicen kaufen können, um sich gegen wirtschaftliche Risiken abzusichern. Versicherungen zahlen bei Eintritt bestimmter Ereignisse (z.B. Krisen, niedriges Einkommen, Kreditausfall).

**Harmonien:**
- **Reputation:** Personen mit höherer Reputation erhalten günstigere Versicherungsprämien
- **Kredit-System:** Versicherungen können Kreditausfälle abdecken, reduzieren das Risiko für Kreditgeber
- **Verträge:** Langfristige Versicherungsverträge mit garantierten Prämien
- **Krisen-Events:** Versicherungen werden besonders wertvoll während Wirtschaftskrisen
- **Ersparnisse:** Alternative zu reinen Ersparnissen zur Risikominimierung

**Nutzen:** Ermöglicht die Untersuchung von Risikoallokation, Versicherungsmärkten und der Stabilisierung von Wirtschaften durch Risikoteilung.

**Implementierung:** 
- Neue `Insurance` Struktur mit Typ (Kredit, Einkommen, Katastrophe), Prämie, Deckung
- Versicherungsanbieter als spezielle Rolle oder alle Personen als Pool
- Ereignisgesteuerte Auszahlungen basierend auf definierten Bedingungen
- Prämienkalkulation basierend auf Reputation und historischem Risiko

#### 1.3 Investitionssystem
**Beschreibung:** Personen können in Produktionskapazitäten, Bildung anderer Personen oder gemeinsame Projekte investieren und erwarten dafür zukünftige Renditen.

**Harmonien:**
- **Ersparnisse:** Überschüssige Ersparnisse können produktiv investiert werden
- **Kredit-System:** Investitionen können mit Krediten finanziert werden
- **Produktionssystem:** Investitionen in Produktionsrezepte erhöhen Output
- **Bildungssystem:** Investitionen in Bildung anderer schaffen zukünftige Handelsmöglichkeiten
- **Gruppen:** Gruppeninvestitionen in gemeinsame Projekte
- **Verträge:** Investitionsverträge mit garantierten Rückflüssen

**Nutzen:** Ermöglicht die Modellierung von Kapitalallokation, Risikobereitschaft und wirtschaftlichem Wachstum durch Investitionen.

**Implementierung:**
- `Investment` Struktur mit Investor, Investitionsziel, Betrag, erwartete Rendite, Laufzeit
- Verschiedene Investitionstypen: Produktionskapazität, Bildung, Infrastruktur
- ROI-Berechnung basierend auf Erfolg der Investition
- Portfolio-Tracking pro Person

### 2. Erweiterte Marktmechanismen

#### 2.1 Zertifizierungssystem
**Beschreibung:** Fähigkeiten können durch eine zentrale Autorität oder durch Peer-Review zertifiziert werden, was Vertrauen schafft und höhere Preise ermöglicht.

**Harmonien:**
- **Reputation:** Zertifizierte Fähigkeiten bauen schneller Reputation auf
- **Bildungssystem:** Bildungsabschlüsse werden automatisch zertifiziert
- **Qualitätsbewertung:** Zertifizierung garantiert Mindestqualität
- **Produktionssystem:** Zertifizierte Inputs führen zu höherwertigen Outputs
- **Verträge:** Verträge können Zertifizierung verlangen

**Nutzen:** Modelliert Berufsqualifikationen, Lizenzen und Qualitätssicherungsmechanismen in Märkten.

**Implementierung:**
- `Certification` Komponente mit Zertifizierungsstelle, Level, Ablaufdatum
- Kosten für Zertifizierung (Zeit und Geld)
- Zertifizierte Skills haben Preisaufschlag
- Zertifizierung kann ablaufen und muss erneuert werden

#### 2.2 Handelsabkommen zwischen Personen
**Beschreibung:** Zwei oder mehr Personen können bilaterale oder multilaterale Handelsabkommen schließen, die gegenseitige Präferenzen, Rabatte oder Exklusivität gewähren.

**Harmonien:**
- **Freundschaft:** Handelsabkommen entstehen natürlich zwischen Freunden
- **Verträge:** Langfristige Handelsabkommen sind formalisierte Verträge
- **Gruppen:** Gruppeninterne Handelsabkommen (Binnenmarkt)
- **Reputation:** Einhaltung von Abkommen stärkt Reputation
- **Geografie:** Regionale Handelsabkommen zwischen nahegelegenen Personen

**Nutzen:** Ermöglicht die Untersuchung von Handelspolitik, regionalen Wirtschaftsblöcken und präferentiellen Handelsbeziehungen.

**Implementierung:**
- `TradeAgreement` Struktur mit Partnern, Rabattsatz, Exklusivitätsklauseln, Dauer
- Verschiedene Typen: Bilateral, Regional, Multilateral
- Handelsvolumen-Boni für Abkommenspartner
- Strafen bei Bruch des Abkommens

### 3. Soziale Netzwerke und Beziehungen

#### 3.1 Vertrauensnetzwerke
**Beschreibung:** Ein mehrschichtiges Vertrauenssystem, bei dem Vertrauen transitiv ist (Freund eines Freundes erhält teilweises Vertrauen). Ermöglicht Handel mit reduzierten Risiken in erweiterten Netzwerken.

**Harmonien:**
- **Freundschaft:** Freundschaften bilden die Basis von Vertrauensnetzwerken
- **Reputation:** Netzwerkposition beeinflusst Reputation
- **Kredite:** Kredite innerhalb von Vertrauensnetzwerken haben niedrigere Zinsen
- **Verträge:** Verträge im Netzwerk sind zuverlässiger
- **Versicherung:** Gegenseitige Versicherung innerhalb von Vertrauensgruppen

**Nutzen:** Ermöglicht die Untersuchung von sozialem Kapital, Netzwerkeffekten und informellem Kreditwesen.

**Implementierung:**
- Graph-basiertes Vertrauensmodell mit Vertrauensstufen (direkt, 2. Grad, 3. Grad)
- Vertrauenswert nimmt mit Distanz ab (z.B. 100%, 50%, 25%)
- Vorteile skalieren mit Vertrauenslevel
- Vertrauensbruch propagiert im Netzwerk

#### 3.2 Gemeinschaftliche Ressourcenpools
**Beschreibung:** Gruppen können gemeinsame Ressourcenpools bilden (Geld, Fähigkeiten, Versicherung), auf die Mitglieder zugreifen können. Fördert Solidarität und kollektive Sicherheit.

**Harmonien:**
- **Gruppen:** Natürliche Erweiterung des Gruppensystems
- **Ersparnisse:** Kollektive Sparmodelle
- **Versicherung:** Gegenseitige Versicherungsvereine
- **Kredite:** Kreditgenossenschaften innerhalb der Gruppe
- **Steuern/Umverteilung:** Alternative zu zentraler Umverteilung

**Nutzen:** Modelliert Genossenschaften, Mikrofinanzsysteme und informelle Spargruppen (wie ROSCAs).

**Implementierung:**
- `ResourcePool` pro Gruppe mit Einzahlungen, Auszahlungen, Regeln
- Mitgliedsbeiträge (Prozentsatz oder fester Betrag)
- Zugangsregeln: Bedürftigkeit, Rotation, Abstimmung
- Transparenz und Rechenschaftspflicht-Mechanismen

### 4. Erweiterte Szenarien

#### 4.1 Technologieschocks
**Beschreibung:** Plötzliche technologische Durchbrüche, die bestimmte Fähigkeiten obsolet machen oder neue Fähigkeiten schaffen. Simuliert technologischen Wandel und Strukturwandel.

**Harmonien:**
- **Technologischer Fortschritt:** Beschleunigt bestehende Tech-Wachstumsrate
- **Bildungssystem:** Umschulungsbedarf steigt dramatisch
- **Produktionssystem:** Neue Produktionsrezepte werden verfügbar
- **Krisen-Events:** Kann kurzfristig Krisencharakter haben
- **Arbeitslosigkeit:** Personen mit veralteten Skills temporär arbeitslos

**Nutzen:** Ermöglicht die Untersuchung von Strukturwandel, technologischer Arbeitslosigkeit und Anpassungsfähigkeit.

**Implementierung:**
- `TechShock` Event mit betroffenen Fähigkeiten (obsolet/neu)
- Veraltete Fähigkeiten verlieren massiv an Wert
- Neue Fähigkeiten mit hohem Anfangswert erscheinen
- Umschulungskosten und -zeit für betroffene Personen

#### 4.2 Regulatorische Interventionen
**Beschreibung:** Externe Regulierungsbehörde kann Markteingriffe vornehmen: Preiskontrollen, Berufszulassungen, Mindeststandards, Kartellrecht.

**Harmonien:**
- **Preisboden/-decke:** Erweitert bestehende Preiskontrollen
- **Zertifizierung:** Kann Zertifizierungen vorschreiben
- **Schwarzmarkt:** Regulierung treibt Schwarzmarktaktivität
- **Abstimmungssystem:** Demokratische Entscheidung über Regulierungen
- **Gruppen:** Gruppenspezifische Regulierungen

**Nutzen:** Ermöglicht die Untersuchung von Regulierungsökonomie, unbeabsichtigten Folgen und optimal intervention design.

**Implementierung:**
- `Regulation` System mit verschiedenen Interventionstypen
- Höchst-/Mindestpreise pro Fähigkeit
- Berufszulassungen (Lizenzen erforderlich)
- Qualitätsstandards (Mindestqualität für Verkauf)
- Compliance-Kosten für Anbieter

### 5. Erweiterte Analyse

#### 5.1 Kausalanalyse-Framework
**Beschreibung:** Eingebautes Framework für kausale Inferenz: A/B-Tests mit Kontrollgruppen, synthetische Kontrollmethoden, Difference-in-Differences Analyse.

**Harmonien:**
- **Parameter-Sweeps:** Erweitert Sweeps mit kausaler Interpretation
- **Szenario-Vergleich:** Ermöglicht rigorose Wirkungsanalyse
- **Monte-Carlo-Simulation:** Statistische Signifikanz für Kausalschätzungen
- **Gruppen:** Gruppen können als Treatment/Control dienen

**Nutzen:** Rigoros wissenschaftliche Evaluation von Policy-Interventionen und Mechanismus-Designs.

**Implementierung:**
- `CausalAnalysis` Modul mit verschiedenen Methoden
- Automatische Behandlungs-/Kontrollgruppen-Zuweisung
- Statistische Tests für kausale Effekte
- Confounder-Kontrolle durch Randomisierung oder Matching

### 6. Verschiedene Agentenstrategien

#### 6.1 Adaptive Strategien
**Beschreibung:** Agenten passen ihre Verhaltensstrategien basierend auf Erfolg an. Lernen aus Erfahrung durch Reinforcement Learning oder evolutionäre Strategien.

**Harmonien:**
- **Verhaltensstrategien:** Erweitert statische Strategien mit Lernen
- **Reputation:** Erfolg korreliert mit Reputationsaufbau
- **Krisen:** Strategieanpassung als Krisenreaktion
- **Bildung:** Lernen neuer Fähigkeiten als strategische Anpassung

**Nutzen:** Realistische Agenten mit adaptiven Fähigkeiten, emergente Strategien, evolutionäre Dynamiken.

**Implementierung:**
- Strategie-Parameter werden pro Person getrackt
- Erfolgsmetriken: Vermögenswachstum, Handelsvolumen
- Einfache Lernregel: Erfolgreiche Strategien werden verstärkt
- Mutation und Exploration (ε-greedy oder ähnlich)

#### 6.2 Spezialisierung und Diversifikation
**Beschreibung:** Agenten entscheiden strategisch, ob sie sich auf wenige Fähigkeiten spezialisieren (Experte) oder viele Fähigkeiten lernen (Generalist).

**Harmonien:**
- **Bildungssystem:** Spezialisierung erfordert intensivere Bildungsinvestition
- **Qualität:** Spezialisierte Fähigkeiten haben höhere Qualität
- **Risiko:** Diversifikation reduziert Einkommensrisiko
- **Produktionssystem:** Spezialisten produzieren höherwertige Outputs

**Nutzen:** Untersucht Trade-offs zwischen Spezialisierung und Diversifikation, Expertenbildung.

**Implementierung:**
- `SpecializationStrategy` Parameter pro Person
- Spezialisten: Höhere Qualität, höhere Preise, aber engerer Markt
- Generalisten: Breiterer Markt, flexibler, aber niedrigere Preise
- Dynamische Entscheidung basierend auf Marktnachfrage

## 🔧 Code-Verbesserungen

### 1. Architektur und Design

#### 1.1 Event-System vollständige Integration
**Beschreibung:** Das vorhandene Event-System-Framework vollständig in die Simulationslogik integrieren, um detailliertes Event-Tracking zu ermöglichen.

**Nutzen:** Ermöglicht Timeline-Analyse, detailliertes Debugging und Forschung über Kausalzusammenhänge in der Simulation.

**Implementierung:**
- Events bei jedem Trade, Preisupdate, Reputationsänderung emittieren
- Event-Filter und -Subscriptions für selektives Tracking
- Event-Replay-Funktionalität für Debugging
- Performance-optimiertes Event-Logging mit optionaler Kompression

#### 1.2 Erweiterbare Agentenarchitektur
**Beschreibung:** Refactoring der Person-Struktur zu einer modularen, komponenten-basierten Architektur (Entity-Component-System inspiriert).

**Nutzen:** Einfachere Erweiterung mit neuen Features ohne Monolith-Struktur, bessere Testbarkeit, modulare Aktivierung von Features.

**Implementierung:**
- `Component` Trait für verschiedene Fähigkeiten (Trading, Learning, Investing, etc.)
- `ComponentManager` zur Verwaltung von Komponenten pro Person
- Feature-Flags zur Compile-Zeit-Aktivierung von Komponenten
- Dependency Injection für Komponenten-Kommunikation

### 2. Performance-Optimierungen

#### 2.1 Parallele Trade-Matching
**Beschreibung:** Optimierung des Trade-Matchings durch Parallelisierung konfliktfreier Trades unter Verwendung von Rayon.

**Nutzen:** Deutliche Performance-Verbesserung bei großen Simulationen (>1000 Personen), bessere CPU-Auslastung.

**Implementierung:**
- Konfliktgraph für Trade-Paare berechnen
- Konfliktfreie Trades parallel ausführen
- Atomare Operationen für gemeinsame Ressourcen
- Benchmark-Suite zur Performance-Messung

#### 2.2 Inkrementelle Statistikberechnung
**Beschreibung:** Statt volle Neuberechnung in jedem Schritt inkrementelle Updates von Statistiken (Mittelwert, Median, Gini).

**Nutzen:** Reduzierung der Berechnungskomplexität von O(n) zu O(1) pro Update, ermöglicht größere Simulationen.

**Implementierung:**
- Inkrementelle Algorithmen für Mittelwert, Varianz
- Approximative inkrementelle Median-Berechnung (Quantil-Sketch)
- Effiziente Gini-Updates unter Verwendung von sortiertem Index
- Validierung gegen exakte Berechnung in Tests

### 3. Code-Qualität

#### 3.1 Konfigurationsvalidierung und -dokumentation
**Beschreibung:** Automatisch generierte Dokumentation aller Konfigurationsparameter mit Ranges, Defaults, Abhängigkeiten.

**Nutzen:** Bessere Benutzererfahrung, weniger Konfigurationsfehler, selbstdokumentierender Code.

**Implementierung:**
- Schema-basierte Validierung mit detaillierten Fehlermeldungen
- Automatische Generierung von Markdown-Dokumentation aus Code
- Interactive Config-Builder (CLI Wizard)
- Validierung von Feature-Dependencies (z.B. Loans benötigen Reputation)

### 4. Datenmanagement

#### 4.1 Zeitreihen-Datenbank-Integration
**Beschreibung:** Integration mit Zeitreihen-Datenbanken (InfluxDB, TimescaleDB) für effiziente Speicherung und Abfrage historischer Simulationsdaten.

**Nutzen:** Skalierbare Datenspeicherung, schnelle Zeitreihen-Abfragen, Integration mit Visualisierungs-Tools (Grafana).

**Implementierung:**
- Optionale TimescaleDB-Integration via Feature-Flag
- Batch-Inserts für Performance
- Vordefinierte Dashboards für Grafana
- Retention-Policies für große Datensätze

#### 4.2 Daten-Versionierung und Reproduzierbarkeit
**Beschreibung:** Vollständige Versionierung aller Simulationsläufe mit Git-ähnlichem Modell für Reproduzierbarkeit und Vergleichbarkeit.

**Nutzen:** Wissenschaftliche Reproduzierbarkeit, Audit-Trail, einfache Vergleiche zwischen Läufen.

**Implementierung:**
- Content-addressable Storage für Simulationsstates
- Metadaten-Tracking (Git-Hash, Parameter, Timestamp)
- Diff-Tool für Simulationsvergleiche
- Export zu standardisierten Formaten (Frictionless Data Package)

## 📊 Analyse und Forschung

### 1. Wirtschaftliche Analysen

#### 1.1 Marktmacht und Monopolanalyse
**Beschreibung:** Detaillierte Analyse von Marktmacht, Monopolbildung und Wettbewerbsintensität für einzelne Fähigkeitenmärkte.

**Harmonien:**
- **Herfindahl-Index:** Erweitert bestehenden HHI auf Skill-Ebene
- **Handelspartner-Statistiken:** Identifiziert dominante Anbieter
- **Preishistorie:** Zeigt monopolistische Preissetzung
- **Qualitätsbewertung:** Monopole können Qualität reduzieren

**Nutzen:** Untersuchung von Marktmacht, Preissetzungsverhalten und Wohlfahrtsverlusten durch Monopole.

**Implementierung:**
- Per-Skill HHI und Konzentrationsverhältnisse (CR4, CR8)
- Lerner-Index für Markup-Messung
- Marktzutrittsbarrieren-Analyse
- Consumer-Surplus und Deadweight-Loss-Berechnung

#### 1.2 Konjunkturzyklen-Detektion
**Beschreibung:** Automatische Identifikation und Analyse von Wirtschaftszyklen (Expansion, Peak, Rezession, Trough) in der Simulation.

**Harmonien:**
- **Zyklische Nachfrage:** Interagiert mit exogenen Zyklen
- **Krisen-Events:** Unterscheidung von exogenen vs. endogenen Krisen
- **Handelsvolumen:** Primärer Indikator für Konjunktur
- **Preishistorie:** Pro-zyklisches Preisverhalten

**Nutzen:** Verständnis endogener Zyklen, Krisenprogression und stabilisierender Mechanismen.

**Implementierung:**
- Hodrick-Prescott-Filter zur Trend-Zyklus-Zerlegung
- Peak/Trough-Detektion mit NBER-Methodik
- Zyklus-Charakterisierung: Dauer, Amplitude, Asymmetrie
- Leading/Lagging Indikatoren-Analyse

#### 1.4 Externalitäten-Analyse
**Beschreibung:** Messung und Analyse von positiven und negativen Externalitäten zwischen Agenten und Aktivitäten.

**Harmonien:**
- **Umwelt-Ressourcen:** Negative Umwelt-Externalitäten
- **Bildungssystem:** Positive Externalitäten durch Wissensverbreitung
- **Produktionssystem:** Produktionsketten mit Spillover-Effekten
- **Freundschaft/Netzwerke:** Soziale Externalitäten

**Nutzen:** Bewertung von Marktversagen, optimaler Pigou-Steuern und Subventionen.

**Implementierung:**
- Tracking von Externalitäten pro Transaktion/Aktivität
- Soziale vs. private Kosten-Nutzen-Analyse
- Optimale Korrektursteuern-Berechnung (Pigou-Steuer)
- Coase-Theorem-Experimente mit Verhandlungen

## 🛠️ Entwickler-Tools

### 1. CLI-Verbesserungen

#### 1.1 Interaktive Szenario-Konfiguration
**Beschreibung:** Erweiterte interaktive Konfiguration mit Vorschlägen, Validierung in Echtzeit und Feature-Dependency-Resolution.

**Harmonien:**
- **Interaktiver Modus:** Ergänzt REPL mit Setup-Phase
- **Konfigurationsdateien:** Generiert YAML/TOML aus interaktiver Session
- **Presets:** Bietet Presets als Startpunkt

**Nutzen:** Niedrigere Einstiegshürde für neue Benutzer, weniger Fehler durch geführte Konfiguration.

**Implementierung:**
- Inquire-basierter CLI-Wizard
- Kontext-sensitive Hilfe und Dokumentation
- Dependency-Checks (z.B. "Loans benötigt Reputation")
- Config-Export am Ende des Wizards

#### 1.2 Simulation-Dashboards
**Beschreibung:** Live-Dashboard im Terminal während der Simulation mit Sparklines, Gauges und Histogrammen für Schlüsselmetriken.

**Harmonien:**
- **Progress Bar:** Ersetzt oder ergänzt einfache Progress Bar
- **Streaming-Output:** Nutzt JSONL-Stream als Datenquelle
- **Farbausgabe:** Erweitert bestehendes Farbschema

**Nutzen:** Bessere Überwachung langer Simulationen, frühe Probleme-Detektion.

**Implementierung:**
- TUI-Framework (tui-rs oder ratatui)
- Multiple Panels: Handelsvolumen, Gini-Koeffizient, Top-Trader
- Sparklines für Zeitreihen
- Tastenkombinationen für Panel-Wechsel

### 2. Debugging-Tools

#### 2.1 Simulation-Debugger
**Beschreibung:** Interaktiver Debugger mit Breakpoints, Step-Execution, State-Inspektion und Time-Travel-Debugging.

**Harmonien:**
- **Checkpoint-System:** Nutzt Checkpoints für Time-Travel
- **Interaktiver Modus:** Erweitert REPL mit Debug-Kommandos
- **Event-System:** Events als Debug-Trail
- **Logging:** Integration mit strukturiertem Logging

**Nutzen:** Drastisch verbesserte Debugging-Erfahrung, schnellere Bug-Diagnose.

**Implementierung:**
- Breakpoint-System (Step, Trade, Price-Threshold)
- State-Inspektion mit Pretty-Printing
- Time-Travel: Zurückspulen zu früheren Steps via Checkpoints
- Conditional Breakpoints mit Expression-Evaluator

#### 2.2 Simulation-Recorder und Playback
**Beschreibung:** Aufzeichnung aller Aktionen für exakte Replay-Funktionalität, nützlich für Bug-Reports und Demonstrations.

**Harmonien:**
- **Event-System:** Events als Aufzeichnungsformat
- **Checkpoint-System:** Snapshots für schnelles Spulen
- **Streaming-Output:** Parallele Aufzeichnung
- **Reproduzierbarkeit:** Deterministisches Replay

**Nutzen:** Bug-Reproduktion, Demos, Lehre, Regression-Testing.

**Implementierung:**
- Binäres Aufzeichnungsformat für Effizienz
- Playback mit variablen Geschwindigkeiten
- Annotations/Kommentare während Aufzeichnung
- Export zu Video (ASCII-Cast Format)

## 🌍 Erweiterungen für spezifische Anwendungsfälle

#### 1. Epidemiologie-Integration
**Beschreibung:** Krankheitsausbreitung im Handelsnetzwerk, mit wirtschaftlichen Auswirkungen (Arbeitsausfall, Gesundheitskosten).

**Harmonien:**
- **Handelsnetzwerk:** Übertragung erfolgt durch Handelsbeziehungen
- **Krisen-Events:** Epidemie als spezielle Krise
- **Versicherung:** Krankenversicherung relevant
- **Geografie:** Räumliche Ausbreitung

**Nutzen:** Modellierung von Pandemien und ihren ökonomischen Auswirkungen.

#### 2. Klimawandel-Szenarios
**Beschreibung:** Integration von Klimawandel-Dynamiken mit steigenden Umweltkosten, Ressourcenknappheit und Anpassungsbedarf.

**Harmonien:**
- **Umwelt-Ressourcen:** Beschleunigte Ressourcenerschöpfung
- **Krisen-Events:** Klimabedingte Schocks (Dürren, Fluten)
- **Technologischer Fortschritt:** Grüne Technologien
- **Regulierung:** Klimapolitik (CO2-Steuer)

**Nutzen:** Erforschung von Klimaökonomie, Anpassungsstrategien und Kosten des Nicht-Handelns.

#### 3. Post-Konflikt-Wiederaufbau
**Beschreibung:** Simulation von Wirtschafts-Wiederaufbau nach Krisen mit beschädigter Infrastruktur, Vertrauensverlust und knappen Ressourcen.

**Harmonien:**
- **Krisen-Events:** Extreme Startbedingungen
- **Reputation:** Vertrauens-Wiederaufbau ist zentral
- **Infrastruktur-Investitionen:** Wiederaufbau-Investitionen
- **Internationale Hilfe:** Externe Geldinfusion

**Nutzen:** Politikbewertung für Post-Konflikt-Situationen, Hilfsallokation.

#### 4. Gig-Economy-Simulation
**Beschreibung:** Modellierung von Plattformökonomie mit vermittelten Trades, Plattformgebühren, Ratings und algorithmischem Matching.

**Harmonien:**
- **Transaktionsgebühren:** Plattformgebühren
- **Reputation:** Platform-Ratings
- **Verträge:** Gig-Verträge (kurzfristig)
- **Preisdynamik:** Surge-Pricing

**Nutzen:** Untersuchung von Plattformökonomie, Worker-Outcomes, Plattformmacht.

## 🎯 Priorisierung

### Hohe Priorität (Kurzfristig - hohe Harmonie-Effekte)

1. **Versicherungssystem** - Starke Synergie mit Reputation, Krediten, Krisen
2. **Qualitätsbewertungssystem** - Fügt wichtige Marktdimension hinzu (NOTE: Quality rating is already implemented!)
3. **Mentorschaft** - Natürliche Erweiterung von Bildung + Freundschaft
4. **Event-System Integration** - Infrastruktur-Verbesserung mit breitem Nutzen

### Mittlere Priorität (Mittelfristig - gute Harmonien)

6. **Investitionssystem** - Verbindet Ersparnisse, Kredite, Produktion
7. **Handelsabkommen** - Erweitert Freundschaft und Verträge
8. **Zertifizierungssystem** - Qualitätssicherung mit Reputation-Link
9. **Vertrauensnetzwerke** - Ausgefeiltes soziales Kapital-Modell
10. **Adaptive Strategien** - Macht Agenten realistischer

### Niedrige Priorität (Langfristig - Spezialfälle)

11. **Technologieschocks** - Interessant aber komplex
12. **Regulatorische Interventionen** - Erfordert umfangreiche Modellierung
13. **Gemeinschaftliche Ressourcenpools** - Nischenanwendung
14. **Spezialisierung/Diversifikation** - Erfordert große Überarbeitung
15. **Kausalanalyse-Framework** - Fortgeschrittenes Research-Tool
16. **Konjunkturzyklen-Detektion** - Ausgefeilte Analyse
17. **Externalitäten-Analyse** - Theoretisch wichtig, praktisch herausfordernd
18. **Simulation-Debugger** - Nice-to-have für Entwicklung
19. **Spezielle Anwendungsfälle** - Domänenspezifisch

### Code-Verbesserungen (Kontinuierlich)

- **Parallele Trade-Matching** - Performance bei großen Simulationen
- **Inkrementelle Statistiken** - Skalierbarkeit
- **Integration-Tests** - Qualitätssicherung
- **Konfigurationsvalidierung** - Benutzererfahrung
- **Zeitreihen-DB** - Enterprise-Integration
- **Erweiterbare Architektur** - Langfristige Wartbarkeit

## 💡 Harmonien-Matrix

Diese Matrix zeigt, welche Features besonders gut zusammenwirken:

### Reputation als Zentrum
- **Reputation** ↔ **Versicherung**: Günstigere Prämien
- **Reputation** ↔ **Kredite**: Bessere Konditionen
- **Reputation** ↔ **Qualität**: Gegenseitige Verstärkung
- **Reputation** ↔ **Mentorschaft**: Effektivere Mentoren
- **Reputation** ↔ **Zertifizierung**: Schnellerer Aufbau

### Bildung als Wachstumsmotor
- **Bildung** ↔ **Mentorschaft**: Reduzierte Kosten, höherer Erfolg
- **Bildung** ↔ **Qualität**: Erlernte Skills starten mit niedriger Qualität
- **Bildung** ↔ **Zertifizierung**: Bildung führt zu Zertifizierung
- **Bildung** ↔ **Investitionen**: Investition in Bildung anderer
- **Bildung** ↔ **Spezialisierung**: Spezialist vs. Generalist-Strategie

### Soziales Kapital
- **Freundschaft** ↔ **Vertrauensnetzwerke**: Basis für Vertrauen
- **Freundschaft** ↔ **Handelsabkommen**: Präferenzielle Behandlung
- **Freundschaft** ↔ **Mentorschaft**: Mentorschaft führt zu Freundschaft
- **Freundschaft** ↔ **Gemeinschaftspools**: Solidarität in Gruppen
- **Verträge** ↔ **Handelsabkommen**: Formalisierte Abkommen

### Risikomanagement
- **Versicherung** ↔ **Kredite**: Kreditausfallversicherung
- **Versicherung** ↔ **Krisen**: Schutz vor Schocks
- **Versicherung** ↔ **Gemeinschaftspools**: Gegenseitige Versicherung
- **Ersparnisse** ↔ **Investitionen**: Kapitalallokation
- **Ersparnisse** ↔ **Kreditrating**: Besseres Rating

### Marktstruktur
- **Qualität** ↔ **Preise**: Qualitätswettbewerb
- **Qualität** ↔ **Zertifizierung**: Qualitätsgarantie
- **Produktion** ↔ **Investitionen**: Produktionskapazitäts-Investitionen
- **Produktion** ↔ **Qualität**: Qualitäts-Vererbung in Produktionsketten
- **Schwarzmarkt** ↔ **Regulierung**: Regulierung treibt Schwarzmarkt

## 🔄 Implementations-Roadmap

### Phase 1: Grundlegende Harmonien (3-6 Monate)
1. Versicherungssystem implementieren
2. Qualitätsbewertungssystem implementieren (NOTE: Already implemented!)
3. Event-System vollständig integrieren
4. Mentorschaftssystem implementieren

**Warum diese Reihenfolge?**
- Versicherung baut auf Reputation und Kredite auf
- Qualität ist relativ unabhängig und bringt sofort Mehrwert (bereits implementiert!)
- Event-System ist Infrastruktur für besseres Debugging aller Features
- Mentorschaft rundet soziale Features ab

### Phase 2: Erweiterte Interaktionen (6-12 Monate)
6. Investitionssystem implementieren
7. Handelsabkommen implementieren
8. Zertifizierungssystem implementieren
9. Vertrauensnetzwerke implementieren
10. Adaptive Strategien implementieren

**Warum diese Reihenfolge?**
- Investitionen nutzen Ersparnisse, Kredite, Reputation aus Phase 1
- Handelsabkommen bauen auf Freundschaften und Verträgen auf
- Zertifizierung ergänzt Qualitätssystem aus Phase 1
- Vertrauensnetzwerke erweitern Freundschaftssystem
- Adaptive Strategien profitieren von allen bisherigen Features

### Phase 3: Fortgeschrittene Analysen (12+ Monate)
11. Kausalanalyse-Framework
12. Konjunkturzyklen-Detektion
13. Externalitäten-Analyse

**Warum diese Reihenfolge?**
- Analysen profitieren von den reicheren Daten aus Phasen 1-2
- Mobilitätsanalyse ist relativ einfach zu implementieren
- Kausalanalyse ist methodisch anspruchsvoll
- Konjunktur- und Externalitäten-Analyse benötigen reife Simulation

### Kontinuierlich: Code-Qualität und Performance
- Parallele Trade-Matching bei Bedarf (wenn N > 1000)
- Inkrementelle Statistiken bei Performance-Problemen
- Integration-Tests für jedes neue Feature
- Refactoring zu komponenten-basierter Architektur
- Dokumentations-Updates

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

### Beispiele erfolgreicher Harmonien im aktuellen System

- **Reputation + Freundschaft + Verträge**: Reputation beeinflusst Vertragspreise, Freundschaft beschleunigt Reputationsaufbau, Verträge schaffen langfristige Beziehungen
- **Kredite + Reputation + Savings**: Gute Reputation ermöglicht günstige Kredite, Ersparnisse verbessern Kreditwürdigkeit, Kreditrückzahlungen stärken Reputation
- **Bildung + Produktion + Qualität**: Bildung ermöglicht Produktion, Produktion erzeugt hochwertige Skills, Qualität beeinflusst Bildungseffizienz
- **Steuern + Umverteilung + Ungleichheit**: Steuern finanzieren Umverteilung, Umverteilung reduziert Ungleichheit, Ungleichheit beeinflusst politische Stabilität (via Voting)

### Vorschläge zur Feature-Priorisierung

Verwende diese Kriterien für die Priorisierung zusätzlicher Features:

1. **Harmonie-Score** (0-10): Anzahl und Stärke der Verknüpfungen mit existierenden Features
2. **Implementierungs-Aufwand** (S/M/L/XL): Geschätzter Entwicklungsaufwand
3. **Forschungs-Relevanz** (0-10): Wie interessant ist das Feature für ökonomische Forschung?
4. **Praxis-Relevanz** (0-10): Wie relevant ist das Feature für reale Wirtschaftssysteme?
5. **Lehr-Eignung** (0-10): Wie gut eignet sich das Feature für Lehrzwecke?

**Beispiel-Scoring:**

| Feature | Harmonie | Aufwand | Forschung | Praxis | Lehre | Gesamt |
|---------|----------|---------|-----------|--------|-------|--------|
| Versicherung | 9 | M | 8 | 10 | 8 | 35/50 |
| Kreditrating | 10 | S | 7 | 10 | 7 | 34/50 |
| Qualität | 8 | M | 7 | 9 | 9 | 33/50 |
| Mentorschaft | 7 | S | 6 | 7 | 10 | 30/50 |

Contributions sind willkommen! Bitte öffnen Sie ein Issue oder Pull Request, um Diskussionen zu starten oder Änderungen vorzuschlagen.

### Wie man beiträgt

1. **Issue öffnen**: Beschreibe das Feature und seine Harmonien mit bestehenden Features
2. **Design diskutieren**: Community-Feedback zu Implementierungsdetails
3. **Pull Request**: Implementation mit Tests und Dokumentation
4. **Review**: Code-Review mit Fokus auf Harmonien und Qualität
5. **Integration**: Merge und Aktualisierung dieser Features-Liste
