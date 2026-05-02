# PhraseDJ – Vollständiges Projekt-Audit (Stand: 2. Mai 2026)

## 1) Executive Summary

Dieses Audit bewertet den aktuellen Projektstand als **fortgeschrittenes Prototyp-/Alpha-Repository mit starker Diskrepanz zwischen Kommunikation und Realität**.

**Positiv:**
- Breiter technischer Unterbau ist vorhanden (Rust-Workspace, C++-Engine, Frontend-Tests).
- Test-Substanz in einzelnen Bereichen ist sichtbar.
- Architektur und Vision sind für ein Solo-/Hobbyprojekt überraschend klar dokumentiert.

**Kritisch:**
- Offizielle Kommunikation ist widersprüchlich (README „Pre-Alpha / nicht runnable“ vs. tatsächlich vorhandene Module und Tests).
- CI-/Build-Reproduzierbarkeit ist auf Standard-Linux-Umgebung nicht gegeben (fehlende native Systemabhängigkeiten).
- Plan-/Status-Management wirkt unzuverlässig (viele Phasen als „[x]“ markiert, obwohl laut README noch Vor-Phase).
- Sicherheits-/Lieferfähigkeitsrisiko durch eingecheckte `node_modules`.

**Gesamteinschätzung:**
- **Produktreife:** 4/10
- **Engineering-Reife:** 6/10
- **Lieferfähigkeit (für externe Mitwirkende):** 3/10
- **Dokumentationskonsistenz:** 2/10

---

## 2) Scope & Methode

Geprüfte Dimensionen:
1. Produktstrategie & Scope
2. Architektur & Modularität
3. Build/Test/CI-Reproduzierbarkeit
4. Code-/Repo-Hygiene
5. Risiko-, Qualitäts- und Release-Management

Durchgeführte Checks (lokal):
- `cargo test --workspace`
- `pnpm test` (apps/desktop)
- `cmake ... && ctest` (native/audio)

---

## 3) Findings (kritisch priorisiert)

## A. Governance, Planung, Erwartungsmanagement

### A1 — **Kritisch:** Status-Kommunikation ist widersprüchlich
**Befund:** README sagt „Pre-Alpha — specification and planning phase“ und „Not yet runnable“, während der Plan weit fortgeschrittene Phasen bereits als abgeschlossen markiert und echte Test-Suiten existieren.

**Auswirkung:**
- Verwirrt Contributor:innen und mögliche frühe Nutzer.
- Erzeugt Vertrauensverlust in Projektkommunikation.

**Empfehlung:**
- Einheitliche „Single Source of Truth“ für Projektstatus einführen.
- README-Status und PROJECT_PLAN-Checkboxen innerhalb 1 PR synchronisieren.

### A2 — **Hoch:** Roadmap-Tracking wirkt inflationär
**Befund:** In `PROJECT_PLAN.md` sind große Deliverables als erledigt markiert, ohne öffentliche Evidenz (z. B. Demo, QA-Abnahme, messbare Artefakte im Root-Dokumentationspfad).

**Auswirkung:**
- Fortschritt schwer auditierbar.
- Risiko von Selbsttäuschung im Projektcontrolling.

**Empfehlung:**
- Done-Kriterien pro Meilenstein mit verlinkten Artefakten verpflichtend machen (Video, Benchmarks, Testreport).

---

## B. Build, Test, CI, Reproduzierbarkeit

### B1 — **Kritisch:** Workspace-Tests nicht reproduzierbar in Standard-Linux
**Befund:** `cargo test --workspace` schlägt fehl (fehlendes `glib-2.0` via `pkg-config`).

**Auswirkung:**
- Neue Contributor können den Qualitätsstatus nicht verifizieren.
- CI-Drift zwischen Zielplattform und Developer-Umgebung.

**Empfehlung:**
- Plattformprofile trennen (macOS-first explizit).
- Linux-Check mit benötigten Systempaketen dokumentieren oder optionalisieren.
- Smoke-Test-Set definieren, das überall läuft.

### B2 — **Hoch:** Native Audio-Tests nicht ausführbar ohne PortAudio-Systempaket
**Befund:** CMake/CTest bricht ab, da `portaudio-2.0` fehlt.

**Auswirkung:**
- Native Kernkomponente schwer verifizierbar.

**Empfehlung:**
- Install-Anleitung pro OS (apt/brew/choco) verbindlich dokumentieren.
- Optional: statische/vendored Build-Option für lokale Tests.

### B3 — **Positiv:** Frontend-Testbasis ist funktionsfähig
**Befund:** `pnpm test` läuft erfolgreich (30 Tests, 3 Files).

**Empfehlung:**
- Dieses funktionierende Setup als Referenzstandard für andere Module nutzen.

---

## C. Repository-Hygiene & Betriebsfähigkeit

### C1 — **Kritisch:** `node_modules` ist im Repository vorhanden
**Befund:** Unter `apps/desktop/node_modules` liegen Dateien im Repo-Arbeitsbaum.

**Auswirkung:**
- Riesiges VCS-Rauschen, Merge-Konflikte, unnötige Repository-Größe.
- Sicherheits-/Supply-Chain-Risiko durch ungeprüften Vendor-Müll.

**Empfehlung:**
- `node_modules` konsequent aus Git entfernen und `.gitignore` absichern.
- CI auf lockfile-basierte reproducible installs (`pnpm install --frozen-lockfile`).

### C2 — **Mittel:** Struktur signalisiert mehrere Phasen parallel
**Befund:** Viele Crates existieren bereits, teils als Stubs.

**Auswirkung:**
- Onboarding wird schwer, wenn nicht klar ist, welche Module „active“ vs. „placeholder“ sind.

**Empfehlung:**
- Pro Crate Reifegradlabel einführen (`stub`, `prototype`, `active`, `stable`).

---

## D. Produkt- & Technikstrategie

### D1 — **Hoch:** Scope-Risiko für 5h/Woche weiterhin sehr hoch
**Befund:** Featureumfang (Stems, Phrase AI, Macro-Automation, Lyrics Alignment, MIDI, CLAP, Scripting, MCP) ist extrem breit.

**Auswirkung:**
- Gefahr: Kein stabiler Kern für frühe Nutzer, Fokusverlust.

**Empfehlung:**
- Hartes Core-Produkt definieren: „2 Decks + stabiles Audio + Library + Basis-Transitions“.
- Alles andere nach „Live-Set-Stabilität“ priorisieren.

### D2 — **Mittel:** Erfolgsmessung braucht operativere KPIs
**Befund:** Metriken sind gut, aber Messverfahren fehlen oft (wer misst wann womit?).

**Empfehlung:**
- KPI-Ritual einführen (monatlich, feste Benchmark-Skripte, historisierte Werte).

---

## 4) Priorisierte Maßnahmen (Backlog-ready)

## Sofort (0–2 Wochen)
1. **Status-Alignment PR**: README + PROJECT_PLAN synchronisieren.
2. **Repo-Hygiene PR**: `node_modules` entfernen, Ignore-Regeln härten.
3. **Build-Doku PR**: Native Abhängigkeiten + OS-spezifische Installschritte dokumentieren.
4. **Minimal Quality Gate**: Ein universell laufender Testpfad (`pnpm test` + ausgewählte Rust-Tests ohne Systemlibs).

## Kurzfristig (2–6 Wochen)
5. **Definition of Done 2.0** mit Artefaktpflicht pro Meilenstein.
6. **Crate-Reifegradmatrix** in Doku.
7. **CI-Matrix** (macOS required, Linux best-effort klar markiert).

## Mittelfristig (6–12 Wochen)
8. **Core-Freeze**: Keine neuen Großfeatures vor Stabilitätsziel.
9. **Live-Reliability Paket**: Crash-Recovery, Panic-Stop, Device-Hotswap priorisieren.
10. **KPI-Dashboard light**: Latenz, Startzeit, Crash-freie Session, Testabdeckung.

---

## 5) Schrittweises Abarbeiten (Vorschlag in 4 Wellen)

### Welle 1 — „Vertrauen herstellen“
- Kommunikations- und Plan-Konsistenz reparieren.
- Reproduzierbare Build-Anleitung liefern.

### Welle 2 — „Lieferfähigkeit absichern“
- CI/Tests auf realistische Plattformziele ausrichten.
- Contributor-Onboarding in <30 Minuten ermöglichen.

### Welle 3 — „Kernprodukt stabilisieren“
- Audio-Engine, Deck-Steuerung, Library als höchste Priorität.
- Strikter Bugfix-/Stabilitätsfokus.

### Welle 4 — „Differenzierung ausbauen“
- Erst dann AI-Mehrwert (Phrase/Macros/Lyrics) produktreif machen.
- Plugin/Scripting/MCP as Erweiterung, nicht als Blocker.

---

## 6) Produktverbesserungen: Was könnte PhraseDJ besser machen?

1. **„First Gig Mode“**: Ein abgesicherter Modus mit minimalen Features und maximaler Stabilität.
2. **Transition Coach statt Vollautomatik zuerst**: Empfehlungen + visuelle Hilfen vor Auto-Mix-Komplexität.
3. **Live-Sicherheit als USP**: Panic-Stop, Auto-Recovery, Device-Fallback prominenter als AI-Marketing.
4. **Auditierbare Transparenz**: Sichtbare lokale/online-Aktivität (insb. Lyrics-Lookups) direkt in UI.
5. **Performance-Profiling in App**: Echtzeit-Anzeige für CPU/Audio-Headroom pro Deck.

---

## 7) Abschlussbewertung

Das Projekt ist **viel weiter als „nur Spezifikation“**, aber aktuell in einer riskanten Zwischenlage: technisch ambitioniert, kommunikativ inkonsistent und operativ noch nicht robust genug für verlässliche externe Mitarbeit.

Die nächsten 4–8 Wochen sollten **nicht** primär neue Features liefern, sondern **Klarheit + Reproduzierbarkeit + Stabilität**. Das erhöht die Wahrscheinlichkeit, dass PhraseDJ als ernstzunehmendes Open-Source-DJ-Produkt nachhaltig wächst.


## Addendum (2. Mai 2026, nach Remote-Änderungen)

Seit dem ursprünglichen Audit wurden CI-Workflows angepasst und installieren auf Ubuntu/macOS nun explizit die nativen Abhängigkeiten (u. a. PortAudio, libsndfile sowie Linux-GUI/libs für Tauri). Dadurch ist ein wesentlicher Teil des ursprünglichen Reproduzierbarkeitsrisikos bereits reduziert.

Konsequenz für die Priorisierung:
- Der Punkt „Build-Abhängigkeiten dokumentieren“ bleibt wichtig, ist aber nicht mehr „kritisch ungeklärt“.
- Fokus sollte jetzt stärker auf Statuskonsistenz, belastbaren Done-Kriterien und Stabilität des Audio-Kerns liegen.
