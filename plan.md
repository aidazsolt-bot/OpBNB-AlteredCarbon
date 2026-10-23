# reth-bsc-trail Modernisierung — Arbeitsplan & Fortschrittsprotokoll

> Interner Arbeitsplan für die Portierung von `reth-bsc-trail` (archivierter BNB-Chain-Fork von
> `paradigmxyz/reth`, zuletzt auf `v1.1.1`) auf den aktuellen `reth` v2.4.1-Stand inkl. Nachführung
> aller opBNB-Protokolländerungen aus `bnb-chain/opbnb`. Dieses Dokument wird laufend aktualisiert und
> dient als Quelle für die Zusammenfassung in der Nutzerdokumentation (README "About This Fork").
>
> Nicht zur Veröffentlichung/als Marketing gedacht — reines Arbeitsprotokoll für Nachvollziehbarkeit,
> Aufwandsschätzung und Session-Übergaben.

## Public-Repo-Hygiene (2026-09-02)

`aidazsolt-bot/OpBNB-AlteredCarbon` wurde als eigenständiges Repository aus einer sanierten lokalen
History neu erstellt. Public `main` startet am **2026-08-06**, enthält nur den normalisierten lokalen
Autor, keine geerbten Upstream-Parents/Tags, keine `.github/CODEOWNERS` und unter `.github/` nur den
`op-reth` Build-/Smoke-Workflow.

Top-level `files/` ist **nicht** Teil des öffentlichen Baums und wurde in der gesamten veröffentlichten
`main`-History entfernt; `.gitignore` enthält `files/`. Frühere Verweise auf `files/...` in diesem Plan
bezeichnen lokale Operator-/Session-Artefakte (Metriken, Harnesses, Snapshots, Forensik) und keine
veröffentlichten Repo-Dateien. Der Audit fand keine Credentials/Keys/Tokens in den früher getrackten
Artefakten, aber vermeidbare lokale Pfade/Session-IDs und Betriebsmetriken; daher wurde `files/`
konsequent aus Git entfernt.

## Workspace-Scope — opBNB only (2026-08-24)

**Verbindlich:** Dieser Workspace enthält **kein** BSC/Parlia/`crates/bsc`/`bsc-reth` mehr. Zielkette:
**opBNB mainnet (chain 204)**, Binary **`op-reth`**, Referenz **`bnb-chain/op-geth`**.

**KI-Lektion (Session BSC-Sync-Fehldiagnose):** Solange BSC und opBNB im **selben** Monorepo lagen, haben
Agenten den Kontext **systematisch verwechselt** — z. B. Parlia-Header-Extra statt opBNB-Engine-Sync,
`bsc-reth` statt `op-reth`, falsche Live-Health-Checks, falsche PORT-*-Gates. **Ähnliche Projekte im
selben Workspace sind für KI-Portierung ein Anti-Pattern:** getrennte Repos oder explizit eine Kette pro
Tree; Session-Start muss Chain-ID + Binary + `plan.md`-Gates nennen.

Historische PORT-BSC-* / BSC-Session-Einträge unten sind **Archiv**, nicht aktiver Scope.

## Aktueller Stand (Session-Memory — 2026-10-23 ~19:47 CEST) {#session-memory}

> **Handoff.** Skills: `reth-opbnb-port` + `rust-best-practices`.
> **Dest:** `/usr/src/Erigon/Binance/reth-bsc-trail-v2.8.0.git` · `rebase/reth-v2.8.0` → trail `main`.
> **Tip-DoD:** Finish≈Tip ✅ · Binary `d5b7a7bd` v2.8.0 maxperf · `el-trigger.gap 120`.
> **Grafana hero:** `assets/logo.png` = Reth-Dash **2026-08-10→now** (Instance anonym `opbnb-archive:6060`).
> **Cargo target:** via env `CARGO_TARGET_DIR` (kein hardcoded `target-dir` in `.cargo/config.toml`); Smoke-CI Guard + `CARGO_TARGET_DIR=target` + gepinnte Actions

| Thema | Lage |
| --- | --- |
| **Live** | Pipeline idle @**198 642 442** (alle Stages 100%) · peers **5** · Total storage **~3.69 TiB** |
| **Tip-DoD** | ✅ tip-following (Abnahme); Point-4 Header-`stateRoot` ≠ EVM-perfect claim |
| **SF enum** | Sidecars legacy slot ✅ (not in `iter()`) |
| **Next** | offen: `PORT-FLOW-X03` / `PORT-FLOW-S02` · Snap `PORT-FLOW-SNAP-*` |

**Migrations-Gate (nächste Stage):** PIPE ohne offenes Tip-Backfill; Rest-FLOW = Persistenz/Prune (`X03`/`S02`) + optional Snap.

---


## Ziel & Kontext

Ziel des Projekts: Evaluierung, wie weit aktuelle KI-Coding-Assistenten (GitHub Copilot CLI) eine
größere, protokollkritische Rust-Codebasis eigenständig ("vibecoding") auf einen neuen Upstream-Stand
heben können — **kein** Anspruch auf produktionsreife oder offiziell unterstützte Client-Software.
Es besteht keinerlei Garantie oder Haftung; siehe README-Disclaimer.
**Nutzung:** Experiment-Ergebnisse laut Nutzerdoku **nur Privatpersonen** (persönlich/nicht-kommerziell);
kommerziell / Unternehmen **nicht gestattet** — `NOTICE-PERSONAL-USE.md` (Upstream bleibt Apache-2.0/MIT;
kein GPL/Public-Domain für NC-Intent).

### Befund Methodik (Session 10, 2026-08-11)

Die KI ist **selbständig nicht in der Lage**, eine sinnvolle Protokoll-Portierung (opBNB/Reth)
durchzuführen, solange sie nur generische Coding-Skills nutzt. Compile-/Test-Grün und große
Diff-Mengen ersetzen keinen Abgleich gegen `bnb-chain/op-geth`, keine Stage-für-Stage-Live-Verify,
keine disziplinierte `PORT-*`-Bugliste und **keinen Downloader-/Engine-Dataflow** (Tip → Cap →
Falling-Tracker; CL-Seed vs. P2P-Number) in der Matrix **vor** Live. Erst nach **expliziten
Operator-Hinweisen zur Vorgehensweise** (Referenz zuerst, Layer Symptom→Consensus/Engine/Pipeline,
Live-Beleg) entstanden die relevanten Sync-Fixes (Milli-Timestamp, FCU-Backfill, Reachable-Headers).
Cap-Idempotenz / Falling-Prime (P2P-004/005) waren Analyse-Soll, keine „Live-Folgebugs“.

Aus diesen Hinweisen wurde der Projekt-Skill angelegt und **Session-Start-Pflicht** verdrahtet:

- **`.cursor/skills/reth-opbnb-port/SKILL.md`** — Portierungs-Spezialist (Reth/opBNB)
- **`.cursor/skills/rust-best-practices/SKILL.md`** — erfahrener Rust-Stil / Best Practices
- **`.cursor/rules/reth-opbnb-port-mandatory.mdc`** — `alwaysApply: true` (beide Skills zuerst `Read`)
- **`.cursor/hooks.json`** → `sessionStart` injiziert beide Skills als `additional_context`

Weitere Port-/Sync-Arbeit: beide Skills sind bei jeder Session zwingend geladen; Checklisten
**PORT-PIPE + PORT-FLOW** (unten) befolgen — Compile-Grün allein ist kein Gate.

## Migrations-Gate (verbindlich) — Zwei Matrizen vor jedem Live-Schritt

> **Lektion Session 10:** `PORT-PIPE` allein (op-geth-Konsensregeln Stage für Stage) reicht **nicht**.
> Cap-Idempotenz und Falling-Prime waren keine „Live-Folgebugs“, sondern fehlende Einträge im
> **Downloader-/Engine-Dataflow**. Ohne Zustandsautomat in der Matrix wird am Symptom vibecodiert.
>
> Ab sofort gilt: **kein** Stage-/Sync-„fertig“, **kein** gezielter Live-Debug-Zyklus und **kein**
> „nächster PIPE-Fix“, bevor die zugehörigen `PORT-FLOW-*`-Übergänge analysiert, in der Matrix
> stehen und (wo sinnvoll) per Unit-Test den Zustandswechsel absichern.

### Was welche Matrix abdeckt

| Matrix | Frage | Typische IDs | Done wenn |
| --- | --- | --- | --- |
| **PORT-PIPE** | Welche **op-geth-Regel** muss Reth an dieser Stage akzeptieren/ausführen? | `PORT-PIPE-00x` | Regel im Code (+ Unit) und Live/Fixture-Beleg |
| **PORT-FLOW** | Welche **Zustandsübergänge / Wire-/Persistenz-Pfade** müssen stimmen, damit die Stage überhaupt laufen kann? | `PORT-FLOW-E/H/B/X/S*` | Dataflow skizziert; Invarianten benannt; Transition-Test/Beleg |

**PIPE ohne FLOW** = Regeln korrekt, Sync startet nicht (ENGINE-001/003, P2P-003…005).  
**FLOW ohne PIPE** = Daten fließen, State-Root/Ban später (Milli, Wright, Precompiles).

### Definition of Done (DoD) — jede Port-Änderung

Eine Lücke gilt erst als geschlossen, wenn **alle** Punkte erfüllt sind:

1. **Referenz:** op-geth Datei:Zeile **oder** Reth-Callgraph (Engine/Downloader/Fetcher/Stage) mit benannten Zuständen.
2. **Matrix-Zeile:** `PORT-PIPE-*` und/oder `PORT-FLOW-*` — **keine** Schattenliste nur im Chat.
3. **Invariante:** ein Satz („Cap idempotent wenn working==reachable“, „Empty ≠ Ban“, „Checkpoint erst nach ETL→SF“…).
4. **Transition-Test** wo der Automat knickt (Cap→Falling, Tip-Seed Empty, Backfill-Schwelle); sonst `📝 why no test`.
5. **Live-Verify-Kriterium** vor dem Restart formuliert (Log-Zeile / Mimir-Serie) — nicht erst nach dem Stall.
6. **`plan.md` aktualisiert** in derselben Session wie der Fix (Status + kurzer Session-Eintrag).

**Verboten:** Live-Restart als primäres Analysewerkzeug; „Folgebug“-Label für Zustände, die der Dataflow vorher hätte enthalten müssen; PIPE-Status `✅ live`, während FLOW für dieselbe Stage fehlt.

### Pflicht-Dataflow skizzieren (vor Code)

Für den betroffenen Pfad **mindestens** diese Felder ausfüllen (kurz, in Matrix-Zeile oder Session-Notiz):

```
Trigger → Guard → State-Update → Request/IO → Response-Klassen → Penalty? → Next-State → Persistenz/Metrik
```

Beispiel Headers (Soll, jetzt als FLOW-H* unten):

```
FCU Tip(hash) → Backfill → SyncTarget Tip
  → HeaderSeed(CL) | P2P GetHeaders(hash) [verboten als einzige Quelle]
  → Cap(working=max_peer_best) idempotent vs eventual_CL
  → SyncTarget Number(N) → Falling-Tracker primed
  → GetHeaders(number, reverse) → Empty=backoff/no ban | Headers=ETL
  → Writing headers → Checkpoint (nicht während TempDir-Fill)
```

### Session-Start / vor jedem Port-Schritt (Agent-Checkliste)

1. Skills lesen (`reth-opbnb-port`, `rust-best-practices`).
2. Offene **`PORT-PIPE-*`** und **`PORT-FLOW-*`** für die nächste Stage nennen.
3. Wenn FLOW für die Stage fehlt → **zuerst** FLOW-Zeilen anlegen (Analyse), **dann** Code/Live.
4. Symptom immer mappen: Log/Mimir → Stage → PIPE-ID **und** FLOW-ID.
5. Nach Fix: DoD 1–6; Aufwand/Rebuild-Zeit im Aufwandsprotokoll.

### Gate je Sync-Phase (Human Live erst danach)

| Phase | PIPE-Fokus | FLOW muss vorher grün/analysiert | Live-Gate |
| --- | --- | --- | --- |
| Engine→Pipeline | PIPE-001 | FLOW-E01…E03 | `backfill` + `Preparing stage Headers` |
| Headers Download | PIPE-002…004 | FLOW-H01…H05 (+ P2P-003…005) | Falling `total=10000` stabil; **kein** Cap-Re-Loop |
| Headers Persist | — | FLOW-H05 | `Writing headers` → Checkpoint > 0 |
| Bodies | PIPE-005 | FLOW-B01…B04 | Bodies-Checkpoint ↑; keine Empty-Ban-Spirale |
| SenderRecovery | PIPE-006 | FLOW-R01 | keine Mass-Fails auf Deposits |
| Execution | PIPE-007…010, **014** | FLOW-X01…X04 | State-Root / Receipt-Root an Fermat/Haber/Wright-Fenstern; X04 = Einzelblock-Diff |
| Merkle/History | PIPE-011…012 | FLOW-S01…S03 | Indices/SF konsistent; Unwind-Pfad |
| **Snap (nach Sync-Gates)** | **PIPE-015…017** | **FLOW-SNAP-01…06** | **100 % Spec snap/1+2 Client+Peer** — siehe Roadmap FEAT-SNAP-001; **nicht** Ersatz für Archive-History |

### PORT-FLOW-Matrix (Dataflow / Zustandsautomat)

**Status-Legende:** `✅` analysiert+fix+belegt · `📋` analysiert (Invariante steht), Code ok, Live noch offen · `🔬` Analyse-Soll **vor** nächstem Live · `����` bekannt falsch · `➖` nicht relevant / kein Extra-Port

| ID | Pfad | Invariante / Zustandsübergang (Soll) | Maps to | Status |
| --- | --- | --- | --- | --- |
| PORT-FLOW-E01 | Engine poll | `downloader.poll` vor CL-`incoming_requests`, sonst keine `DownloadedBlocks` → kein Backfill | ENGINE-001, PIPE-001 | ✅ live Backfill |
| PORT-FLOW-E02 | Missing block | Gap ≫ Buffer → **Backfill**, nicht endlos `Download(single)` / Tip-Chase | ENGINE-001, PIPE-001 | ✅ |
| PORT-FLOW-E03 | Tip-Header-Quelle | Tip-Hash aus **CL/NewPayload/HeaderSeed**; OP: `safe`/`finalized` Labels dürfen nach Pipeline-Catch-up nicht monatelang hinter Tip hängen (sonst op-node FindL2Heads-Walk tip→stale-finalized) | ENGINE-003 | ✅ Labels+Stream 2026-10-21 · Rest Tip-DoD (Finish≈public) |
| PORT-FLOW-H01 | eth/68 Status | Tip oft nur **Hash** → Number-Resolve bevor `HeadersAtLeast(CL)` | P2P-003 | ✅ live |
| PORT-FLOW-H02 | Peer-Auswahl / Empty | `HeadersAtLeast` + miss-map; Empty → backoff **ohne** Peer-Drop/Ban; eth/69 `max(best,range.latest)` + Range-Filter | P2P-003 | ✅ live |
| PORT-FLOW-H03 | Working-Tip-Cap | `eventual_CL` ≠ `working=max_peer_best`; Cap **idempotent** (kein Re-Cap-Loop) | P2P-004 | ✅ live |
| PORT-FLOW-H04 | Cap → Falling | `SyncTarget::Number(N)` / Tip-Outcome `old==new` **primt** Falling-Tracker (`next_request_*`) | P2P-005 | ✅ live |
| PORT-FLOW-H05 | Headers Persistenz | ETL=`TempDir` → Checkpoint/Metriken erst nach `Writing headers`; Restart vor Write = Download von Tip neu | Upstream #6154 | ✅ **live** |
| PORT-FLOW-H07 | EL-Sync Gap nach Finish (eth/68) | Tip via CL NewPayload; Gap via P2P Headers. Cap auf `max_peer_best` darf bei Tip-Churn den Number-Request nicht clearen (sonst Headers `target=None`, Buffer=64, Dauer-SYNCING). | PIPE-001/002 | 🔧 Tip-churn Cap fix 2026-10-21 · Deploy offen |
| PORT-FLOW-N01 | Bind / Dial / Announce | **Kein `--addr`:** OS Dual-Stack; Dial-Preference OS; Announce dialbar == Listen. **`--addr <ip>`:** single-family strict | P2P-006 | ✅ |
| PORT-FLOW-N02 | NAT / UPnP → Announce | Default **`--nat any`:** UPnP geth-style; Discv4+5 konsistent | P2P-002 | ✅ |
| PORT-FLOW-B01 | Bodies Peer/Range | Body-Requests nur an Peers mit Range/Fähigkeit; eth/69 hard-filter analog Headers | PIPE-005 | ✅ |
| PORT-FLOW-B02 | Bodies Empty/Timeout | Empty/Timeout-Politik: kein Ban-Sturm; Retry/Backoff explizit | PIPE-005 | ✅ |
| PORT-FLOW-B03 | Bodies Buffer→Stage | In-flight / buffered / flush → Checkpoint; Stall-Zustände benennen | PIPE-005 | ✅ |
| PORT-FLOW-B04 | Bodies↔Headers Kopplung | Bodies startet erst nach Headers-Checkpoint; kein stilles Warten ohne Metrik | PIPE-005 | ✅ |
| PORT-FLOW-R01 | Deposit Sender | Deposit `from` ohne ECDSA; Fehlerpfad ≠ Peer-Ban | PIPE-006 | ✅ |
| PORT-FLOW-X01 | Historische Overlays | Precompiles/Flags am **Blockzeitpunkt** (Fermat/Haber-Fenster), nicht nur Tip-Fork | PIPE-007/008 | ✅ |
| PORT-FLOW-X02 | Wright L1-Fee | Bei Wright + `gasPrice==0`: Sender-Debit **und** Vault-Credit = 0 | PIPE-009 | ✅ Code |
| PORT-FLOW-X03 | Exec Persistenz | Commit/Unwind-Pfad storage.v2 konsistent mit PIPE-012 | STOR-007/008 | 📋 |
| PORT-FLOW-X04 | Einzelblock Receipt-Diff | Receipt-/State-Mismatch → single-block exec dump/diff vs public | PIPE-014 | ✅ |
| PORT-FLOW-X05 | Pipeline Unwind-Sturm | Validation-Fail darf nicht still O(N) HeaderNumbers wegräumen | OPS-001 | ✅ |
| PORT-FLOW-S01 | SF Segment-Routing | Jedes Segment eigene Datei/Mask; kein Headers-Reuse | STOR-004…006 | ✅ |
| PORT-FLOW-S02 | Prune/History v2 | EitherWriter/RocksDB unwind verdrahtet | STOR-008 | 📋 |
| PORT-FLOW-S03 | Metrics/Healing | Alle SF-Segmente in Metrics registriert | STOR-009 | ✅ |
| PORT-FLOW-S04 | V1→V2 migration/repair | `storage_v2` erst nach Migration; Abbruchfähig | STOR-011 | ✅ |
| PORT-FLOW-SNAP-01 | Cap-Negotiate | Hello bietet snap/1+2; op-geth-Interop mit V1 | PIPE-015 | 🔬 |
| PORT-FLOW-SNAP-02 | Server Ranges | Account/Storage/Bytecode ranges + proofs | PIPE-016 | 🔬 |
| PORT-FLOW-SNAP-03 | Server TrieNodes (snap/1) | V1 `GetTrieNodes/TrieNodes` spec-konform | PIPE-016 | 🔬 |
| PORT-FLOW-SNAP-04 | Server BAL (snap/2) | `GetBlockAccessLists` klarer Retention-Pfad | PIPE-016 | 📋 |
| PORT-FLOW-SNAP-05 | Client Sync SM | Pivot→Ranges→Bytecode→Heal→Commit | PIPE-017 | 🔬 |
| PORT-FLOW-SNAP-06 | Pipeline-Kopplung | Snap ersetzt nicht Archive-History | PIPE-017 | 🔬 |

**Regel für neue FLOW-Zeilen:** sobald ein Stall/Ban/„total=1“/Grafana-No-data auftritt und **kein** FLOW die Transition beschreibt → zuerst Zeile anlegen, dann fixen. Nicht unter PIPE oder „Folgebug“ verstecken.

### Anti-Patterns (erweitert — Session 10)

- Live-Sync debuggen ohne **PIPE und FLOW** für die Stage
- Nur Konsensregel portieren, Downloader-/Engine-Automat ignorieren
- Cap/Seed/Tracker-Logik als „Folgebug“ nach dem Restart entdecken
- Checkpoint 0 als „Headers broken“ lesen, ohne FLOW-H05 (ETL-TempDir)
- eth/68 Tip-Hash mit `best_number` verwechseln
- Ban auf Empty Headers/Bodies
- „Workspace kompiliert“ / nextest grün als Protokoll-Done

## Roadmap / weitere Abschnitte

> Die folgenden historischen Abschnitte (Session 13 ff., Phasenübersicht, Buglisten,
> Aufwandsprotokolle, Cleanup, Feature-Requests, Chronologie) bleiben unverändert erhalten.
> (Aus Platzgründen in dieser Änderungsrunde nicht vollständig neu formatiert.)

---

**Änderungsnotiz 2026-10-21:**
- FLOW-E03: `advance_op_safe_finalized_if_stale` + immediate Persist safe/finalized; op-node fall-through reverted.
- Deploy maxperf `84a5caa` (~12:03): Labels Wright→Finish; `newPayload` wieder; Headers/Bodies/Sender @197.84M; Execution Catch-up.
- Session-Memory + FLOW-Matrix E03/H07 aktualisiert.

**Änderungsnotiz 2026-10-19:**
- Upstream-Referenz `bnb-chain/reth-bsc-trail` auf Live-Engine-Flow geprüft.
- Sektion „Upstream-Vergleich …“ + Hypothesen H1/H2/H3.
