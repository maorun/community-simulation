# Simulation Framework

Dieses Repository enthält ein Hochleistungs-Simulationsframework, das in Rust geschrieben wurde. Es wurde für Geschwindigkeit und Effizienz entwickelt und nutzt Parallelverarbeitung mit Rayon, um Simulationen mit einer großen Anzahl von Entitäten und Zeitschritten durchzuführen.

## Funktionen

- **Hochleistung:** Nutzt Rust und Rayon für schnelle Simulationsberechnungen.
- **Konfigurierbar:** Ermöglicht die Anpassung von Simulationsparametern wie der Anzahl der Zeitschritte, der Anzahl der Entitäten und mehr.
- **Ausgabe:** Kann Simulationsergebnisse zur weiteren Analyse in einer Datei speichern.
- **Befehlszeilenschnittstelle:** Bietet eine benutzerfreundliche CLI zum Ausführen von Simulationen mit unterschiedlichen Konfigurationen.

## Erste Schritte

### Voraussetzungen

- Rust-Toolchain (siehe [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) für Installationsanweisungen)

### Erstellen des Projekts

1. Klone das Repository:
   ```bash
   git clone https://github.com/your-username/simulation-framework.git
   cd simulation-framework
   ```
2. Erstelle das Projekt im Release-Modus für optimale Leistung:
   ```bash
   cargo build --release
   ```

### Ausführen der Simulation

Nach dem Erstellen befindet sich die ausführbare Datei unter `target/release/simulation`. Du kannst sie direkt ausführen.

**Grundlegende Ausführung:**

```bash
./target/release/simulation
```

Dies führt die Simulation mit den Standardparametern aus (1000 Schritte, 10000 Entitäten, 4 Threads).

**Befehlszeilenargumente:**

Das Simulationsprogramm akzeptiert die folgenden Befehlszeilenargumente, um sein Verhalten anzupassen:

*   `--steps <ANZAHL_SCHRITTE>` oder `-s <ANZAHL_SCHRITTE>`:
    *   Legt die Gesamtzahl der Simulationsschritte fest.
    *   Standardwert: `1000`
*   `--entities <ANZAHL_ENTITÄTEN>` oder `-e <ANZAHL_ENTITÄTEN>`:
    *   Legt die Anzahl der zu simulierenden Entitäten fest.
    *   Standardwert: `10000`
*   `--output <DATEIPFAD>` oder `-o <DATEIPFAD>`:
    *   Gibt den Pfad an, in dem die Simulationsergebnisse gespeichert werden sollen.
    *   Wenn nicht angegeben, werden die Ergebnisse nicht in einer Datei gespeichert, sondern nur eine Zusammenfassung wird in der Konsole ausgegeben.
*   `--threads <ANZAHL_THREADS>`:
    *   Legt die Anzahl der Threads fest, die für die Simulation verwendet werden sollen.
    *   Standardwert: `4`

**Beispiel mit benutzerdefinierten Parametern:**

```bash
./target/release/simulation --steps 5000 --entities 20000 --output results.json --threads 8
```

Dies führt die Simulation für 5000 Schritte mit 20000 Entitäten aus, verwendet 8 Threads und speichert die Ergebnisse in `results.json`.

## Code-Struktur

*   `src/main.rs`: Enthält die Hauptfunktion, die Befehlszeilenargumente verarbeitet und die Simulation initialisiert.
*   `src/lib.rs`: Das Haupt-Bibliotheks-Crate, das die Kernmodule der Simulation exportiert.
*   `src/config.rs`: Definiert die `SimulationConfig`-Struktur zur Aufnahme von Simulationsparametern.
*   `src/engine.rs`: Enthält die `SimulationEngine`-Logik, die für die Ausführung der Simulationsschleife verantwortlich ist.
*   `src/entity.rs`: Definiert die `Entity`-Struktur und zugehörige Zustände.
*   `src/physics.rs`: (Vermutlich) Enthält die Physikberechnungen für die Entitäteninteraktionen.
*   `src/result.rs`: Definiert die `SimulationResult`-Struktur und Methoden zum Speichern und Zusammenfassen von Ergebnissen.

## Lizenz

Dieses Projekt ist unter den Bedingungen der MIT-Lizenz lizenziert. Siehe die `LICENSE`-Datei für weitere Details.
