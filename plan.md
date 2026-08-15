# reth-bsc-trail Modernisierung — Arbeitsplan & Fortschrittsprotokoll

> Interner Arbeitsplan für die Portierung von `reth-bsc-trail` (archivierter BNB-Chain-Fork von
> `paradigmxyz/reth`, zuletzt auf `v1.1.1`) auf den aktuellen `reth` v2.4.1-Stand inkl. Nachführung
> aller opBNB-Protokolländerungen aus `bnb-chain/opbnb`. Dieses Dokument wird laufend aktualisiert und
> dient als Quelle für die Zusammenfassung in der Nutzerdokumentation (README "About This Fork").
>
> Nicht zur Veröffentlichung/als Marketing gedacht — reines Arbeitsprotokoll für Nachvollziehbarkeit,
> Aufwandsschätzung und Session-Übergaben.

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
| **PORT-FLOW** | Welche **Zustandsübergänge / Wire-/Persistenz-Pfade** müssen stimmen, damit die Stage überhaupt laufen kann? | `PORT-FLOW-E/H/B/X/S*` | Dataflow skizziert; Invarianten benannt; Transition-Test oder bewusster `📝 n/a`; erst dann Live |

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

**Verboten:** Live-Restart als primäres Analysewerkzeug; „Folgebug“-Label für Zustände, die der Dataflow vorher hätte enthalten müssen; PIPE-Status `✅ live`, während FLOW für dieselbe Stage offen ist.

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

### PORT-FLOW-Matrix (Dataflow / Zustandsautomat)

**Status-Legende:** `✅` analysiert+fix+belegt · `📋` analysiert (Invariante steht), Code ok, Live noch offen · `🔬` Analyse-Soll **vor** nächstem Live · `🐛` bekannt falsch · `➖` n/a

| ID | Pfad | Invariante / Zustandsübergang (Soll) | Maps to | Status |
| --- | --- | --- | --- | --- |
| PORT-FLOW-E01 | Engine poll | `downloader.poll` vor CL-`incoming_requests`, sonst keine `DownloadedBlocks` → kein Backfill | ENGINE-001, PIPE-001 | ✅ live Backfill |
| PORT-FLOW-E02 | Missing block | Gap ≫ Buffer → **Backfill**, nicht endlos `Download(single)` / Tip-Chase | ENGINE-001, PIPE-001 | ✅ |
| PORT-FLOW-E03 | Tip-Header-Quelle | Tip-Hash aus **CL/NewPayload/HeaderSeed**, nicht allein P2P `GetBlockHeaders(hash,1)`; Empty ≠ Ban (esp. trusted) | ENGINE-003 | ✅ Code · 📋 Checkpoint nach ETL |
| PORT-FLOW-H01 | eth/68 Status | Tip oft nur **Hash** → Number-Resolve bevor `HeadersAtLeast(CL)` | P2P-003 | ✅ live |
| PORT-FLOW-H02 | Peer-Auswahl / Empty | `HeadersAtLeast` + miss-map; Empty → backoff **ohne** Peer-Drop/Ban; eth/69 `max(best,range.latest)` + Range-Filter | P2P-003 | ✅ live |
| PORT-FLOW-H03 | Working-Tip-Cap | `eventual_CL` ≠ `working=max_peer_best`; Cap **idempotent** (kein Re-Cap-Loop) | P2P-004 | ✅ live |
| PORT-FLOW-H04 | Cap → Falling | `SyncTarget::Number(N)` / Tip-Outcome `old==new` **primt** Falling-Tracker (`next_request_*`) | P2P-005 | ✅ live |
| PORT-FLOW-H05 | Headers Persistenz | ETL=`TempDir` → Checkpoint/Metriken erst nach `Writing headers`; Restart vor Write = Download von Tip neu | Upstream #6154 | ✅ **live** (2026-08-11T16:35–~16:47Z): Write `173369140` → Headers checkpoint=tip; Bodies gestartet |
| PORT-FLOW-B01 | Bodies Peer/Range | Body-Requests nur an Peers mit Range/Fähigkeit; eth/69 hard-filter analog Headers | PIPE-005 | ✅ Bodies tip (08-12 ~03:02 CEST); FLOW-B beobachtet OK |
| PORT-FLOW-B02 | Bodies Empty/Timeout | Empty/Timeout-Politik: kein Ban-Sturm; Retry/Backoff explizit | PIPE-005 | ✅ Bodies durch · Empty/Ban kein Stall |
| PORT-FLOW-B03 | Bodies Buffer→Stage | In-flight / buffered / flush → Checkpoint; Stall-Zustände benennen | PIPE-005 | ✅ Bodies Checkpoint=Tip |
| PORT-FLOW-B04 | Bodies↔Headers Kopplung | Bodies startet erst nach Headers-Checkpoint; kein stilles Warten ohne Metrik | PIPE-005 | ✅ Headers→Bodies ~18:58 CEST (08-11) |
| PORT-FLOW-R01 | Deposit Sender | Deposit `from` ohne ECDSA (Feld im Deposit-TX, kein `ecrecover`); Fehlerpfad ≠ Peer-Ban | PIPE-006 | ✅ **live OK** — Sender läuft spez-konform (~25 %, keine Recovery-Fails/Stall) |
| PORT-FLOW-X01 | Historische Overlays | Precompiles/Flags am **Blockzeitpunkt** (Fermat/Haber-Fenster), nicht nur Tip-Fork | PIPE-007/008 | ✅ **Fermat live** · ⛔ **blocked** Receipt-Root @ `21591154` (vor Haber) · Haber ⏳ |
| PORT-FLOW-X02 | Wright L1-Fee | op-geth: L1-Fee-Skip nur `gasPrice==0`; Reth-Diff dokumentieren/fixen vor Root-Verify | PIPE-009 | 🐛 Diff bekannt · **nicht** Ursache von `21591154` (pre-Wright) |
| PORT-FLOW-X03 | Exec Persistenz | Commit/Unwind-Pfad storage.v2 (SF changesets, hashed readers) konsistent mit PIPE-012 | STOR-007/008 | 📋 Code · 🔬 Archive-Last |
| PORT-FLOW-X04 | Einzelblock Receipt-Diff | Bei Receipt-/State-Root-Mismatch: Single-block exec → Dump `(idx,status,gasUsed,cumGas,logs)` → Diff vs public `eth_getBlockReceipts` → **erster** divergenter Index vor Fix | PIPE-014 | 🔬 offline: Bodies+Sender →`21591154`; Exec →`21591153` (SF tip ≠ Bodies Cap — 08-15 heal @`20365614`); dann `re-execute --from 21591154 --to 21591155` (half-open). DoD: first-mismatch Index · s. Harness-README |
| PORT-FLOW-X05 | Pipeline Unwind-Sturm | Exec-/Merkle-Validation-Fail darf **nicht** stillschweigend ~10⁸ Headers via O(N) `HeaderNumbers`-Loop vernichten; Status `checkpoint=tip` bis `UnwindOutput` ≠ Idle; Headers loggt **kein** batch-`Stage unwound done=false` (Observability-Inkonsistenz vs Sender/Hashing) | PIPE-014, EXEC-001 | 🐛 **3×** live (2× Receipt @`21591154` + **08-14 ~13:43** Merkle @`21579110`→unwind_to=0); Tip gerettet per Kill vor Headers-Commit; **Ops:** Process-Stop ≫ `max-block` als Park |
| PORT-FLOW-S01 | SF Segment-Routing | Jedes Segment eigene Datei/Mask; kein Headers-Reuse (STOR-001-Klasse) | STOR-004…006 | ✅ |
| PORT-FLOW-S02 | Prune/History v2 | EitherWriter/RocksDB unwind verdrahtet; tote Helper ≠ stiller No-Op ohne FLOW-Notiz | STOR-008, PIPE-U10/11 | 📋 |
| PORT-FLOW-S03 | Metrics/Healing | Alle `StaticFileSegment`s in Metrics registriert (STOR-009-Klasse) | STOR-009 | ✅ |

**Regel für neue FLOW-Zeilen:** sobald ein Stall/Ban/„total=1“/Grafana-No-data auftritt und **kein** FLOW die Transition beschreibt → zuerst Zeile anlegen, dann fixen. Nicht unter PIPE oder Chat begraben.

### Anti-Patterns (erweitert — Session 10)

- Live-Sync debuggen ohne **PIPE und FLOW** für die Stage
- Nur Konsensregel portieren, Downloader-/Engine-Automat ignorieren
- Cap/Seed/Tracker-Logik als „Folgebug“ nach dem Restart entdecken
- Checkpoint 0 als „Headers broken“ lesen, ohne FLOW-H05 (ETL-TempDir)
- eth/68 Tip-Hash mit `best_number` verwechseln
- Ban auf Empty Headers/Bodies
- „Workspace kompiliert“ / nextest grün als Protokoll-Done

## Phasenübersicht (Soll)

1. **Phase 1 — Bestandsaufnahme & Diff-Baseline** ✅ erledigt
2. **Phase 2 — Kern-Crates auf v2.4.1 rebasen** ✅ Merge/Konflikte erledigt, Detailarbeit läuft (s.u.)
3. **Phase 3 — BSC-Crate (`crates/bsc`) aktualisieren** ✅ Compile-Meilenstein: `reth-bsc-node --features bsc` grün (2026-08-09)
4. **Phase 4 — Optimism/opBNB-Crate + Snow/Volta/Fourier-Hardforks** 🔄 Hardforks+stack through **node/cli/op-reth bin** compile-green; nextest prim/consensus/evm/node/rpc ✅; trie/proofs deferred
5. **Phase 5 — Build/Lint/Test/EF-Tests** ✅ check/Clippy/nextest stages+op-stack; EF **v17.0** + Bytecode Compact → **62/62** nach nextest-Timeout-Override (`valid_blocks`/`invalid_blocks` re-verified)
6. **Phase 6 — Doku & Freigabe** 🔄 Effort-Log Session 6+8+9+**10**; **Migrations-Gate PIPE+FLOW** nachgezogen; Human Catch-up/Full-Sync + finale Zahlen nach Live-Tests

### Sync-Tests (Human-owned)

- **Catch-up** und **Full Sync** startet/führt **nur ein Human** durch — sobald die AI den Port als
  **lauffähig** einstuft (Compile + Boot/RPC-Smoke + Kern-Tests ohne Blocker).
- AI macht höchstens Boot-Smoke / kurze Pipeline-Sanity; keine langen Sync-Läufe.
- **Stand 2026-08-15 ~10:50 CEST:** Offline FLOW-X04: Bodies+Sender Cap→`21591154` ✅; Exec `stage run` von ChangeSets-SF-Tip **`20365614`→`21591153`** (nicht Bodies-Cap — Gap = Exec hinter Bodies nach Unwind#3; heal kappte nur Sidecar). Danach `re-execute --from 21591154 --to 21591155` (half-open). Headers Tip **174.0 M**. Details: **Live Sync Progress** + `files/harness-receipt-diff-21591154/README.md`.

## Todo-Status (Stand 2026-08-11)

| ID | Titel | Status |
| --- | --- | --- |
| inventory-diff | Bestandsaufnahme & Diff-Baseline erstellen | ✅ done |
| core-rebase | Kern-Crates auf reth v2.4.1 rebasen | ✅ done |
| bsc-crate-update | BSC-Crate (crates/bsc) aktualisieren | ✅ done (compile: bsc-node grün) |
| opbnb-hardforks | Optimism/opBNB-Crate + Snow/Volta/Fourier | 🔄 H Tip **174 M**; offline X04: Bodies/Sender→`21591154`; Exec SF→`21591153` (from `20365614`); Fail-Block noch offen |
| build-test-validate | Build, Lint, Tests, EF-Tests | ✅ stages/op-stack nextest; EF v17.0 → **62/62** |
| docs-release | Doku aktualisieren, Freigabe vorbereiten | 🔄 Migrations-Gate PIPE+FLOW in plan/Skill; finale Zahlen nach Human-Sync |

## Portierungs-Bugliste (v2.4.1 rebase)

Regressions / CLI-Drift, die beim Rebase untergegangen sind (nicht Upstream-Feature-Gaps).

| ID | Symptom | Ursache | Status |
| --- | --- | --- | --- |
| PORT-CLI-001 | `--storage.v2` fehlte an `op-reth`/`reth` (`node`, `init`, …); neue DBs liefen effektiv über `StaticFilesArgs::to_settings()` → oft **v1** | `StorageArgs` beim Phase-3/4-Port aus `EnvironmentArgs`/`NodeCommand`/`NodeConfig` entfernt; Genesis nutzte falschen Settings-Pfad | ✅ fixed (Session 8): wieder verdrahtet wie Upstream v2.4.1; Default `true`; `ArgAction::Set` + optionaler Wert |
| PORT-CLI-002 | README empfiehlt noch `--enable-prefetch` / `--optimize.enable-execution-cache` | Alte BSC-Fork-Toggles; CLI + Engine-Gating beim Port verloren; Upstream ersetzt durch `--engine.*` Prewarm/Cache | 📝 docs: Flags als obsolet markiert; Runtime-Port von `TriePrefetch` bewusst nicht wiederbelebt |
| PORT-CLI-003 | `--ipcpath /tmp/foo.ipc` wurde zu `/tmp/foo.ipc-1` | `NodeConfig.instance` war `u16` mit Default `1`; `adjust_instance_ports` hängte immer `-{instance}` an | ✅ fixed: `instance: Option<u16>` (None ohne `--instance`), wie Upstream |
| PORT-CLI-004 | Log `Storage settings settings=None`; trotz `--storage.v2` keine v2-Persistenz / kein „Loaded storage settings“ | `init_genesis_with_settings` war Stub (ignorierte Settings); Log lief **vor** Genesis | ✅ fixed: Settings bei frischer DB schreiben; bestehende DB: fehlende Metadata = v1 + Warn bei CLI-Mismatch; Log nach Genesis |
| PORT-CLI-005 | OTLP (`--tracing-otlp` / `--logs-otlp`) wirkt in Live-/maxperf-`op-reth` nicht; Grafana sieht nur Prometheus | Code pfad ist verdrahtet (`reth-tracing-otlp`, `TraceArgs`, Optimism/Eth CLI), aber hinter optionalen Features `otlp` / `otlp-logs` — **nicht** in `default`, **nicht** in `make maxperf-op` (`jemalloc,asm-keccak,keccak-cache-global`). Ohne Feature: Warn „compile with the `otlp` feature“ | 📝 bewusst so (wie Upstream Feature-Gate). **Ops:** `--metrics` (Prometheus) reicht für Grafana; OTLP nur bei Bedarf mit `--features …,otlp[,otlp-logs]` bauen |
| PORT-STOR-001 | Fresh start crash: `append Headers #0 but expected #1` | Incomplete port: AccountChangeSets SF stub wrote into **Headers** during `write_state` (genesis); Senders stub similarly unsafe | ✅ closed via PORT-STOR-004/005 (dedicated segments; no Headers/Tx reuse) |
| PORT-STOR-004 | TransactionSenders SF stub reused Transactions/Receipts | Wrong segment literals + prune stub; v2 expected senders in SF | ✅ fixed: dedicated TransactionSenders writer/prune/readers; `transaction_senders_in_static_files() → storage_v2` |
| PORT-STOR-005 | AccountChangeSets SF incomplete (Headers corruption) | Missing `.csoff` sidecar / header len / writer heal; stubs wrote Headers | ✅ fixed: SegmentHeader `changeset_offsets_len` + sidecar writer/heal/truncate; `account_changesets_in_static_files() → storage_v2` |
| PORT-STOR-006 | StorageChangeSets stub always routed to MDBX (`TODO(opbnb-port)` `Headers` placeholders in rocksdb invariants, migrate-v2, `db state`) | `StaticFileSegment::StorageChangeSets` variant, mask, writer/reader, and `either_writer` routing were never ported after AccountChangeSets SF landed | ✅ fixed (Session 9): dedicated `StorageChangeSets` segment (`.csoff` sidecar, same change-based model as AccountChangeSets); `storage_changesets_in_static_files() → storage_v2`; `EitherWriter`/`EitherReader` routing in `write_state_reverts`/`StorageReader`; `migrate-v2` now really migrates `StorageChangeSets` into static files instead of skipping |
| PORT-STOR-002 | Kein `rocksdb/` trotz `--storage.v2` (Default true) | Feature `reth-provider/rocksdb` war nicht verdrahtet; API-Drift (0.24 CF refs, snapshot/batch, history tip, SF stub); prune Batch-Lifetimes | ✅ fixed: provider+prune rocksdb-Pfad kompiliert; `op-reth` default `rocksdb`; `cargo check -p op-reth` grün |
| PORT-P2P-001 | opBNB EL: `peerCount=0`, Sync hängt bei Genesis trotz Tip-Feeding | Stale Bootnodes; discv4 default aus; `--addr ::` → discv5 dialte UDP-discport statt `tcp4`; **ForkId mismatch** vs op-geth; discv5 admitted **opstack CL** ENRs (TCP ~9222) without fork-id gate | 🔄 code: Bootnodes+ForkId-Filter; discv5 DualStack; **OPSTACK must-not-include + `enforce_enr_fork_id` for Optimism**; live: eth-Session zu `a624…` ✅; Header-empty vom Peer = History, nicht Dial-Noise |
| PORT-P2P-002 | `--nat upnp` / `--nat any` nutzen **kein** echtes UPnP/IGD | `NatResolver::Upnp` ist Stub: alias zu HTTP Public-IP (`ipinfo.io`/…); kein SSDP, **kein Port-Mapping**; Router-UPnP „an“ ändert an Reth nichts | 🐛 open (Upstream-Lücke in `reth-net-nat`). **Ops-Workaround:** manuell TCP+UDP forwarden + `--nat extip:<public-v4>` |
| PORT-P2P-003 | Headers: Empty-Spam auf Tip-Range (`best_number`≪CL-Tip); Lagging-Peers ungenutzt; Stage hängt an unreachable Tip | **Dataflow-Soll (vor Live):** eth/68 Status oft nur Tip-**Hash** → Tip-Number-Resolve; Peer-Auswahl `HeadersAtLeast` / miss-map; Empty → Backoff **ohne** Ban; eth/69 `tip_number=max(best,range.latest)` + Range-Filter. | ✅ **live** (2026-08-11T14:40Z): Tip-Resolve + Falling ab Peer-Head ~173369140 @ ~22k hdr/s (2 Peers). Code: HeadersAtLeast/miss-map; eth/69 Range; ENGINE-003 Tip-Seed. Note: Headers-ETL=`TempDir` (Upstream [#6154](https://github.com/paradigmxyz/reth/pull/6154)) — Restart vor Write = Neustart von Tip; Checkpoint erst nach ETL→SF |
| PORT-P2P-004 | Working-Tip-Cap vs eventual CL-Tip: Cap darf Tip/Falling nicht periodisch verwerfen | **Dataflow-Soll:** `eventual_tip` (CL) ≠ `working_tip` (max peer best). Cap einmalig auf reachable Head; `maybe_recap` **idempotent** wenn already capped — sonst Re-Loop verwirft Tip-Header. Gehört in Matrix **vor** Live, nicht erst nach Stall. | ✅ fixed + live: Cap 1×; Unit-Regression Cap→Falling |
| PORT-P2P-005 | Cap setzt `SyncTargetBlock::Number(N)` → Falling-Tracker bleiben ungesetzt → nur Tip `total=1` dann Stall | **Dataflow-Soll:** Tip-Outcome `Number(N)` mit `old==new` (lokaler Head schon N−ε) muss `next_request_block_number` / Falling-Tracker **primen**. Gehört in Matrix mit P2P-003/004 (Downloader-Zustandsautomat), nicht als „Live-Folgebug“. | ✅ fixed + live (14:40Z): Falling `total=10000` durchgehend; Test Cap→Falling-Prime |
| PORT-STOR-003 | Neue MDBX-DBs mit 4 KiB Pagesize (OS-default) | `default_page_size()` clampte nur auf OS-Pagesize (≥4 KiB); keine Begründung gegen 16 KiB | ✅ fixed: Floor 16 KiB (max OS/libmdbx 64 KiB); nur bei DB-Erstellung wirksam |
| PORT-STOR-007 | `test_pipeline_v2` State-Root-Mismatch / SF unwind; history `IntegerList UnsortedInput` | Incomplete v2 port: plain readers under hashed-canonical; StorageChangeSets keys wrongly hashed; take/remove_state plain-only; hashing/history unwind ignored SF; duplicate block nums in history collect | ✅ fixed: hashed `AccountReader`/`StorageReader`; plain keys in changesets; hashed take/remove; SF hashing/history unwind; dedupe history indices; test un-ignored |
| PORT-STOR-008 | Index Account/Storage History under `storage.v2` still wrote MDBX; unwind no-op without rocksdb | Incomplete EitherWriter history load (`load_*_history`) + RocksDB clear/unwind wiring | ✅ fixed: EitherWriter append/upsert/get_last; stages use `with_rocksdb_batch_auto_commit`; MDBX fallback when rocksdb feature off |
| PORT-STOR-009 | Startup panic: `segment operation metrics should exist` (static_file/metrics.rs) after metrics endpoint | Metrics `Default` only registered Headers/Tx/Receipts/Sidecars; heal/init-cursor hits Account/StorageChangeSets + TransactionSenders | ✅ fixed: register via `StaticFileSegment::iter()` (upstream pattern) |
| PORT-STOR-010 | `--dev` / frische v2-DB: `Persistence … UnexpectedStaticFileBlockNumber(TransactionSenders, 1, 0)` → Fatal engine | `init_genesis` tippte nur Receipts+Transactions auf Block 0; unter `storage_v2` liegen Senders in SF, Genesis hat 0 Txs → Segment blieb untipped; erster Persist `increment_block(1)` scheitert | ✅ fixed (Session 11): wie Upstream/`bnb-chain/reth` bei `storage_v2` `get_writer`+`set_block_range(0,0)` für `TransactionSenders`; Verify: `files/dev-250ms` init zeigt `static_file_transaction-senders_*`, kein Persistence-Crash |
| PORT-DEV-001 | `--dev --dev.block-time 250ms`/`1s`: nach ~5–7 Blöcken Dauer-Spam `Error advancing the chain: No payload`; Tip bleibt stehen | **Nur `--dev` / `LocalMiner`:** (1) `advance()` = FCU+Attrs → `resolve_kind(payload_id)`; `resolve` liefert `None` wenn Job nicht (mehr) in `payload_jobs` (Race: Job noch nicht inserted / schon removed / ID stale). (2) Parallel hartcodiert `fcu_interval=1s` mit **bare FCU** (`attrs=None`) im selben `select!` — verschärft Timing; Interval-`MissedTickBehavior::Burst` feuerzt Catch-up-Ticks. (3) Persistence/SF (STOR-010) ist **nicht** die Ursache (nach Fix kein Fatal mehr). **Mainnet/Archive-Follow:** trifft **nicht** denselben Pfad — kein `LocalMiner`; Tip-Follow = CL `newPayload` + FCU oft **ohne** Attrs. Sequencer-Build (FCU+Attrs → `getPayload`) steuert die CL zeitlich; kein 1s-bare-FCU aus LocalMiner. Ähnliches Risiko nur, wenn ein Client Attrs-Build und bare-FCU unsynchronisiert spamt (nicht op-node Normalbetrieb). | 🧊 **parked** (2026-08-11): keine Prio / kein maxperf-Rebuild dafür. Soll irgendwann funktionieren **oder** `--dev`/LocalMiner dekommissionieren (kein klarer Mehrwert für Archive-Port). Fix-Idee bleibt: bare-FCU während pending Build unterdrücken; Burst ab; Job vor Resolve. Reproduce: `files/dev-250ms` tip≈7 |
| PORT-DEV-002 | `--dev.payload-wait-time` wirkte nicht | `DebugNodeLauncher` spawnte `LocalMiner::new` ohne `with_payload_wait_time_opt` | ✅ fixed (Session 11): Flag an `LocalMiner` durchgereicht; allein **kein** Ersatz für DEV-001 (Race bleibt) |
| PORT-CONS-001 | Headers-Stage: `TimestampIsInPast` trotz gültiger opBNB-Kette; Peers `BadMessage`-Ban; Checkpoint 0 | Eth-`validate_against_parent_timestamp` (Sekunden). opBNB speichert Subsekunden in `mixHash` (`MilliTimestamp = Time*1000 + mixHash[:2]`, bnb-chain/op-geth); gleiche Unix-Sekunde + steigende Milli ist gültig | ✅ fixed (Session 10): `validation/milli_timestamp.rs` + `OpBeaconConsensus` für Chain-ID **204/5611**; Unit-Tests live equal-second + OP-Mainnet reject |
| PORT-EXEC-001 | Archive Execution: `receipt root mismatch` @ **`21591154`** (`got 0x61c1b64b…` ≠ `expected 0x579924c8…` = public header); danach Unwind-Sturm | Pre-Canyon / post-Snow / pre-Haber/Wright (`ts=1713344877`). Hardfork-Gating + Regolith deposit-nonce-Strip wirken korrekt; **PIPE-009 Wright nicht Ursache**. Vermutlich falsche Receipt-**Inhalte** (status/gasUsed/logs) ≥1 User-Tx. 2× live Unwind auf Floor **`21579118`**; Headers Tip wieder **~174.0 M** | 🐛 **open** · PIPE-014 / FLOW-X04/X05 · Fixture+Harness ready · **Ops:** Process-Stop vor Exec-Fail ≫ Cap; offline dump mit `target/maxperf/op-reth` |
| PORT-ENGINE-004 | `systemctl` Reload/Stop: Panic `SelectNextSome polled after terminated` in Critical task `consensus engine` | Shutdown-Pfad Engine/`futures_util::SelectNextSome` nach Stream-Ende noch gepollt (Reload 08-14 13:29 + Stop 17:49) | 🧊 **parked** — später analysieren; Tip/DB nicht primär betroffen |
| PORT-OPS-001 | `--debug.max-block H` als „Park vor Fail“ → Merkle-Fail @ H + `unwind_to=0` | Wenn Stage-Checkpoints **bereits > H**: Pipeline skippt Bodies/Exec (`Stage reached target… skipping`) → Hashing/Merkle auf Restzustand; 08-14 13:43 `bad_block=21579110` state-root mismatch (`got 0x99a6…` / `expected 0x1817…` ≠ Public `0x7b77…`) → Unwind Tip→0 | 🐛 **Ops-Gate** · Cap nur für **Clean-Rebuild** 0…H wenn Checkpoints ≤ H; sonst Process-Stop |
| PORT-ENGINE-001 | Nach Tip-FCU: Status `latest_block=0` **ohne** `stage=…`; Grafana Stages **No data**; Pipeline startet nicht (oder nur kurz) | (1) Engine API Flood: `incoming_requests` vor `downloader.poll` → keine `DownloadedBlocks` → kein Backfill. (2) `handle_missing_block` nur `Download(single_block)` bei gleitendem Buffer (Limit 64) → Tip-Chase, nie Pipeline. (3) `NewDownloadStarted` als Poll-Ready blockierte Inflight-Advance | ✅ fixed + **live** Backfill/`Preparing stage Headers` (FLOW-E01/E02). Checkpoint Headers weiter 0 bis FLOW-H05 |
| PORT-ENGINE-002 | Grafana Stages „0 Blöcke“ vs „No data“ verwechselt | „0“ = Pipeline aktiv, Checkpoint 0. „No data“ = keine Stage-Series (Pipeline idle / Backfill nie gestartet) | 📝 docs only (kein Code) |
| PORT-ENGINE-003 | Headers nach Backfill-Start: Tip-Hash per P2P `GetBlockHeaders(limit=1)` → empty → `BadMessage`-Ban → `connected_peers=0`, Checkpoint 0 | **op-geth Beacon/Skeleton-Sync:** Tip-Header kommt von CL/NewPayload, **nicht** von Peers per Hash. Skeleton-Mock: `RequestHeadersByHash` / remote `Head()` → panic (`eth/downloader/skeleton_test.go:191-196`); Sync füllt per Number (`beaconsync.go` `fetchBeaconHeaders`). Reth `ReverseHeadersDownloader` + `SyncTarget::Tip(hash)` forderte Tip per P2P — Lücke nach PIPE-001. | ✅ Code + live Falling nach Tip-Seed (FLOW-E03). 📋 Headers-Checkpoint > 0 = FLOW-H05 |

### Pipeline-Verify-Matrix (PORT-PIPE) — op-geth ↔ Reth, Stage für Stage

**Zweck:** Systematische Live-/Code-Verifikation der opBNB-EL-**Konsensregeln** entlang `DefaultStages`.
Abgeleitet aus Diff gegen `bnb-chain_op-geth.git`. **Pflicht-Partner:** Abschnitt *Migrations-Gate →
PORT-FLOW-Matrix* — ohne FLOW-Analyse für dieselbe Stage kein Live-„fertig“.

Pipeline-Reihenfolge: Headers → Bodies → SenderRecovery → Execution → MerkleUnwind → AccountHashing → StorageHashing → MerkleExecute → TxLookup → IndexStorageHistory → IndexAccountHistory → Prune → Finish.

**Status-Legende:** `✅ umgesetzt` = Code gegen op-geth portiert (ggf. Unit-Tests); `⏳ live ungetestet` = noch kein Stage-/Archive-Lauf-Beleg; `🐛` = bekannte Regel-Lücke; `➖` = kein Extra-EL-Port; `📝`/`📋` = Hinweis; `♻️`/`⚠️`/`🔜` = siehe Unused-Tabelle (PORT-PIPE-U*).

| ID | Stage / Gate | op-geth-Regel (Soll) | Reth-Stand (Code) | FLOW-Gate | Verify / Status |
| --- | --- | --- | --- | --- | --- |
| PORT-PIPE-001 | Engine → Pipeline | Tip-Gap → Backfill/Pipeline, nicht endlos Tip-Chase | ✅ `handle_missing_block` Backfill + downloader-first (PORT-ENGINE-001) | E01–E03 ✅ | ✅ **live** (2026-08-11T09:15Z): `backfill` + `Preparing stage Headers`. Tip-Fetch → ENGINE-003/FLOW-E03 |
| PORT-PIPE-002 | Headers | `MilliTimestamp` streng steigend (`mixHash[:2]`) | ✅ `milli_timestamp.rs` + OpBeaconConsensus 204/5611; Unit-Tests | H01–H05 (H05 📋) | ✅ umgesetzt · 🔄 live Falling; Milli bis Write beobachten |
| PORT-PIPE-003 | Headers | Wright `baseFee == 0` | ✅ Consensus-Check + `next_block_base_fee` → 0 | H* | ✅ umgesetzt · ⏳ live ungetestet (ab Wright-Höhe) |
| PORT-PIPE-004 | Headers | Pre-Wright EIP-1559 elast=2, denom=8 | ✅ `BaseFeeParams::ethereum()` in `OPBNB_*` | H* | ✅ umgesetzt · ⏳ live ungetestet (Pre-Wright-Range) |
| PORT-PIPE-005 | Bodies | Canyon empty withdrawals; Ecotone `blobGasUsed=0` | ✅ OP `validate_block_pre_execution` / blob-gas=0 | **B01–B04 ✅** | ✅ umgesetzt · ✅ **live** Bodies=Tip @08-12 ~03:02 CEST (s. Live Sync Progress) |
| PORT-PIPE-006 | SenderRecovery | Deposit `from` ohne ECDSA | ✅ OP Deposit-Primitives / Recovery (`OpTransactionSigned::recover_signer` → Deposit.`from`) | **R01 ✅** | ✅ umgesetzt · ✅ **live OK** Tip @15:54 CEST (s. Live Sync Progress) |
| PORT-PIPE-007 | Execution @ Fermat `9397477` | Precompiles `0x66`/`0x67` | ✅ `opbnb_precompiles` Overlay + Flag-Tests | **X01 ✅ Fermat** | ✅ umgesetzt · ✅ **live** Exec≫Fermat; IPC stateRoot MATCH an `9397477`± (s. Live Sync Progress) |
| PORT-PIPE-008 | Execution Haber→Fjord | Early `p256` @ `0x100` nur vor Fjord | ✅ `haber_p256` Flags in `evm/src/config.rs` + Overlay-Tests | **X01 Haber ⏳** | ✅ umgesetzt · ⏳ live ab Haber-Timestamp `1718872200` |
| PORT-PIPE-009 | Execution Wright+ | L1-Fee **nur** wenn `gasPrice==0` → 0 | ⚠️ **Diff:** Reth `skip_l1_data_fee=true` für **alle** Txs post-Wright (`factory.rs`); op-geth nur `GasPrice==0` | **X02 🐛** | 🐛 **nicht umgesetzt**; Live-Root erst nach FLOW-X02-Entscheidung |
| PORT-PIPE-010 | Execution L1-Attr | Snow/Volta/Fourier nur CL → Deposit-Calldata | ➖ EL braucht keine Snow-Logik (Deposit-Parse stock OP) | — | ➖ n/a EL · 📝 CL liefert L1-Info |
| PORT-PIPE-011 | MerkleExecute | Root = Execution-Ergebnis | ➖ Generic Stages; kein opBNB-Extra-Port | X03 | ➖ kein Extra-Port · ⏳ live hängt an PIPE-007…009 |
| PORT-PIPE-012 | History / TxLookup | storage.v2 Indices | ✅ Code + Unit (PORT-STOR-007/008) | S01–S02 | ✅ umgesetzt · ⏳ live ungetestet (Archive-Last / SF-Unwind) |
| PORT-PIPE-013 | Testnet only | PreContract @ `5805494` | ✅ Hardfork + `is_pre_contract_fork_block`; Mainnet ohne Fork | — | ✅ umgesetzt · 📋 Verify nur bei Testnet-Archive (Mainnet n/a) |
| PORT-PIPE-014 | Execution pre-Canyon | Receipt-**Content**-Parity vs op-geth (status/gasUsed/logs/cumGas), nicht nur Regolith-Nonce-Strip / Canyon-`deposit_receipt_version` | Code: Overlay+OP-Receipt-Pfad; Live Fail @ `21591154` | **X04 🔬** | 🐛 **live Fail** 2026-08-13T11:36Z · expected=public · DoD: first divergent tx via FLOW-X04 · siehe PORT-EXEC-001 |

#### Unused / ersetzt / Orphan (PORT-PIPE-U*)

Portierte Helper oder Alt-Pfade ohne Call-Site — **nicht** mit „fehlender Stage“ verwechseln. Legende: `📝 by design` = absichtlich unverdrahtet; `♻️ ersetzt` = Logik läuft woanders; `⚠️ orphan` = tote Datei/API; `🔜 ggf. nachportieren` = nur wenn Live/Produkt es braucht.

| ID | Symbol / Artefakt | Warum unused? | Bewertung |
| --- | --- | --- | --- |
| PORT-PIPE-U01 | `OptimismHardforks::opbnb_block_interval_ms_at_timestamp` (`hardforks/src/lib.rs`) | **0 Call-Sites** außerhalb der Definition. Portiert als Convenience (Volta 500 ms / Fourier 250 ms), aber **EL-Konsens prüft kein festes Δt** — op-geth verlangt nur `MilliTimestamp` streng steigend; Kadenz steuert die CL (`opbnb`). Milli-Validierung (PIPE-002) deckt Sync ab. | 📝 by design · **kein Sync-Blocker** · 🔜 optional nur für Metrics/RPC/Diagnose verdrahten, nicht für Header-Reject |
| PORT-PIPE-U02 | `is_snow_active_at_timestamp` / `is_volta_*` / `is_fourier_*` | Nur von U01 referenziert → ebenfalls **tot**. Die **Forks selbst** stehen in `ChainHardforks` (Aktivierungszeiten) und werden aus dem EIP-2124-ForkId **ausgefiltert** (`opbnb_fork_filter`) — das ist verdrahtet. Snow-L1-Gas-Median bleibt CL (PIPE-010). | 📝 by design (Helper) · Forks ≠ unused · 🔜 Helper nur mit U01 |
| PORT-PIPE-U03 | `is_fermat_active_at_block` | **0 Call-Sites.** Execution nutzt inline `chain_spec.fork(Fermat).active_at_block` in `opbnb_precompile_flags` (PIPE-007). | ♻️ ersetzt durch `fork(Fermat)…` · Helper darf bleiben oder auf Flags umgestellt werden |
| PORT-PIPE-U04 | `is_haber_active_at_timestamp` (Optimism-Trait) | In **optimism**-Crates unbenutzt; Overlay nutzt `fork(Haber)` in `config.rs`. (BSC-Crates haben eigene Haber-API.) | ♻️ ersetzt (OP-Pfad) · BSC separat |
| PORT-PIPE-U05 | `crates/optimism/cli/src/commands/build_pipeline.rs` (`build_import_pipeline`) | **Nicht** in `commands/mod.rs` eingebunden → kompiliert nicht mit. Live-Import: `reth_cli_commands::import::build_import_pipeline` + Node-`DefaultStages`. Stale Merge-Artefakt (falsche `DefaultStages::new`-Arity historisch). | ⚠️ orphan · löschen oder an CLI anbinden · **kein Live-Sync-Pfad** |
| PORT-PIPE-U06 | Alt-CLI `TriePrefetch` / `--enable-prefetch` / `--optimize.enable-execution-cache` | Beim v2.4.1-Port verloren; Upstream ersetzt durch `--engine.*` Prewarm/Cache (siehe PORT-CLI-002). | ♻️ ersetzt (Upstream-Engine) · 📝 bewusst nicht wiederbelebt · 🔜 nur falls Produkt-Parity zu altem BSC-Fork |
| PORT-PIPE-U07 | `COMETBFT_LIGHT_BLOCK_VALIDATION` (+ `…_run`) und `…_PASTEUR` (+ `…_run_pasteur`, per-byte gas) in `opbnb_precompiles/cometbft.rs` | **dead_code** bei `maxperf-op`. Overlay (`opbnb_precompiles/mod.rs`) injiziert nur `COMETBFT_LIGHT_BLOCK_VALIDATION_BEFORE_HERTZ`. op-geth opBNB (`contracts.go`) hat **eine** `cometBFTLightBlockValidate` @ `0x67` mit flat gas — kein Hertz/Pasteur-Switch. Hertz/Pasteur = BSC-Era-Varianten, mitkopiert. | 📝 by design für **opBNB** (BEFORE_HERTZ ≈ op-geth) · ⚠️ BSC-Varianten tot im OP-Crate · 🔜 nachportieren/verdrahten nur wenn `reth-bsc` denselben Precompile-Pfad braucht; sonst löschen/cfg-gaten |
| PORT-PIPE-U08 | Unused imports in `optimism/hardforks/src/hardfork.rs` (+ Spiegel `bsc/hardforks`) | `Box`/`format`/`String`/`Display`/`FromStr` nach Macro-Refactor übrig (`maxperf-op` warn). | 🧹 CLEANUP-A01 · kein Port-Gap |
| PORT-PIPE-U09 | `reth-engine-tree`: unused crate dep `reth_trie_prefetch` | Prefetch-Crate hängt noch als Dependency, Code-Pfad entfernt (U06). | ♻️ ersetzt · 🧹 Dep entfernen (CLEANUP-A02) |
| PORT-PIPE-U10 | `reth-stages`: `load_history_indices` / `load_indices` / `LoadMode` | storage.v2/EitherWriter-Port hat alten History-Load-Pfad obsolet gemacht; Funktionen blieben liegen. | ♻️ ersetzt (v2 history writers) · 🧹 löschen oder hinter Feature · Verify: PIPE-012 |
| PORT-PIPE-U11 | `reth-prune`: `AccountHistory::prune_static_files` / `StorageHistory::prune_static_files` | SF-Changeset-Prune vorbereitet, Call-Site fehlt (v2-Port unvollständig verdrahtet). | ⚠️ unwired · 🔜 nachportieren wenn SF-History-Prune live nötig · sonst dead entfernen |
| PORT-PIPE-U12 | `reth-beacon-consensus`: `MAX_INVALID_HEADERS`, `StaticFileHook` fields, Metrics-Imports | Engine-Tree ersetzt Beacon-Engine-Laufzeit; Crate oft nur noch Stub/Compat → tote Felder. | ♻️ Architektur (engine-tree) · 🧹 Stub aufräumen oder Crate schrumpfen (CLEANUP-B) |
| PORT-PIPE-U13 | `reth-blockchain-tree-api` + Deps in `engine-tree`/`provider` | Deprecated `SealedBlockWithSenders` / Tree-API; Upstream-Tree weg, Fork behält Compat-Schicht. | ♻️ ersetzt durch engine-tree · 🧹 Deprecate-Migration / Deps streichen (CLEANUP-B) |
| PORT-PIPE-U14 | `ConsistentProvider.chain_spec` never read; mass unused imports in `reth-provider` | storage.v2/RocksDB-Port ließ Imports & Feldreste. | 🧹 CLEANUP-A03 · kein Konsens-Gap |
| PORT-PIPE-U15 | `NETWORK_PEER_SCOPE` (`net/network/metrics.rs`) | Metric-Scope-Konstante ohne Verwendung nach Metrics-Refactor. | 🧹 trivial |
| PORT-PIPE-U16 | CLI checksum helpers (`write_entry`, `split_storage_changeset_row`, `checksum_change_based_segment`, …) | Change-based checksum für v2-Rows geplant, nicht verdrahtet. | ⚠️ orphan helpers · 🔜 Tooling nachportieren oder löschen |
| PORT-PIPE-U17 | `reth-rpc-types-compat` mass missing-docs + deprecated aliases | Compat-Crate für alte RPC-Typen; Upstream migriert zu `RecoveredBlock` / neuen Traits. | 🧹 CLEANUP-C (docs/deprecate) · niedrige Prio vs Sync |

**Nicht als fehlende Pipeline-Stages portieren:** Holocene/Isthmus/Jovian (in opBNB-Hardfork-Liste inaktiv); Snow (CL-only); exaktes Δt Volta/Fourier (op-geth prüft nur milli ↑) — siehe U01/U02.

### Final Cleanup Plan (nach Sync-Gates) — CLEANUP-*

Quelle: `make maxperf-op` / `cargo build --profile maxperf … --bin op-reth` Warning-Dump (2026-08-11).  
**Reihenfolge:** Sync-Korrektheit (PORT-PIPE live) **vor** mechanischem Aufräumen. Cleanup darf keine Konsens-/EVM-Semantik ändern.

| Prio | ID | Scope | Aktion | Done wenn |
| --- | --- | --- | --- | --- |
| **P0** | — | **PORT-EXEC-001 / PIPE-014** Receipt-Root @ `21591154` + FLOW-X04; FLOW-X05 3×; **PORT-OPS-001** max-block Skip-Falle | First-mismatch Index; Fix; Process-Stop vor Fail; Cap nur Clean-Rebuild | Exec past `21591154` ohne Receipt-Mismatch; plan Status ✅ |
| **P0** | — | Execution live → FLOW-X01 Haber + X02/X03 (+ PIPE-008/009) | Fermat ✅; Haber/Wright; X02 L1-Fee Diff | Execution ohne Unwind-Sturm; Root-Stichproben an Fork-Fenstern |
| **P1** | CLEANUP-A01 | `reth-optimism-forks` / `reth-bsc-forks` unused imports | `cargo fix -p …` oder Imports streichen | 0 unused_imports in hardfork.rs |
| **P1** | CLEANUP-A02 | Dead crate deps (engine-tree `trie_prefetch`, engine-local/service/util, payload-builder, prune `rayon`, static-file-types, trie-sparse/parallel, db, provider, rpc-*, optimism-rpc, …) | `Cargo.toml` deps entfernen **oder** `use x as _;` nur wo Feature-gated nötig; danach `zepter` + `make lint-toml` | `maxperf-op` ohne `unused_crate_dependencies` in angefassten Crates |
| **P1** | CLEANUP-A03 | `reth-provider` unused imports + `chain_spec` field + rocksdb unreachable-pub | fix imports; Feld nutzen/`_`/entfernen; `pub` → `pub(crate)` wo intern | `cargo fix -p reth-provider` clean für unused |
| **P1** | CLEANUP-A04 | PORT-PIPE-U05 orphan `build_pipeline.rs` | Datei löschen **oder** korrekt in CLI verdrahten | nicht mehr unreferenced on disk |
| **P2** | CLEANUP-A05 | PORT-PIPE-U07 CometBFT Hertz/Pasteur | Entscheiden: (a) `cfg(feature = "bsc")` / nach `reth-bsc-evm` verschieben, oder (b) im OP-Crate löschen, BEFORE_HERTZ behalten | `reth-optimism-evm` ohne dead_code CometBFT-Warnungen |
| **P2** | CLEANUP-A06 | PORT-PIPE-U10/U11/U16 stages/prune/cli checksum | Toten History/Prune/Checksum-Code entfernen **oder** an storage.v2 anbinden + Test | keine dead_code auf genannten Symbolen **oder** nextest-Beleg |
| **P2** | CLEANUP-A07 | Trivial unused imports (payload-primitives, config, eth-wire, txpool, rpc, node-builder, stages HeaderTy, …) | `cargo fix` sweep gezielt | Warning-Anzahl spürbar ↓ |
| **P3** | CLEANUP-B | Legacy tree/beacon: `blockchain-tree-api` deprecated types; beacon stub dead fields; engine-tree deps auf tree-api | Migrate Callers → `RecoveredBlock` / Engine-Events; unnötige deps streichen | keine `SealedBlockWithSenders` Warnungen in Hot-Crates **oder** crate `allow`/entfernen dokumentiert |
| **P3** | CLEANUP-C | Docs/cfg noise: `missing-docs` (primitives-traits Maybe*, rpc-types-compat, provider writer); `unexpected_cfgs` mdbx in db-api; `missing-debug-implementations` beacon | Docs ergänzen **oder** lint-Allow nur lokal; Feature `mdbx` in db-api oder cfg entfernen | `maxperf-op` ohne missing-docs/unexpected-cfgs in Fork-touched files |
| **P3** | CLEANUP-D | Deprecated API surface: `PruneSegment::Headers/Transactions`, revm `gas_used`, ringbuffer `push`, rpc-types-compat aliases | Upstream-Ersatz nutzen wo billig | Warnungen weg oder bewusst `#[allow(deprecated)]` mit Ticket |
| **P4** | CLEANUP-E | Workspace hygiene | `files/*.log` nicht committen; untracked build logs; optional warning-baseline script `scripts/maxperf-op-warnings.sh` | Repro: eine Command → Warning-Count Trend |

**Exit-Kriterium „final cleanup done“:**  
1) `make maxperf-op` (bzw. gleicher `cargo build --profile maxperf … --bin op-reth`) **ohne** `unused`/`dead_code` in `crates/optimism/**` und ohne neue Konsens-Diffs;  
2) Workspace-Warnungen dokumentiert (Baseline) oder auf Upstream-Parity;  
3) PORT-PIPE-U* alle entweder gelöscht, verdrahtet, oder als `📝 by design` abgehakt.

## Feature-Requests (nicht Port-Regressions)

Geplante Produkt-/Explorer-Fähigkeiten. **Start erst nachdem PORT-P2P-001 live belegt ist** (`net_peerCount>0`, eth-Session zu Peers stabil) — Sync/P2P vor Index-Disk-Kosten.

Quelle / Bench (kanonisch):
`<external-analysis>/Analysis/reth-vs-erigon-history-index-gap-2026-08-10.md`
(WalletHistory / Otterscan / `eth_getLogs` Address-Topic vs Erigon 3.7 am Tip ≈25.7 M).

| ID | Ziel | Scope (kurz) | Gate / Abhängigkeit | Status |
| --- | --- | --- | --- | --- |
| FEAT-HIST-001 | Historische Adress-Analysen auf **≥ Erigon-Niveau** (WalletHistory/OTS + sparse `getLogs`) | CallTrace From/To-Index (+ optional internal frames); echte `ots_searchTransactionsBefore/After` auf Index (nicht 100-Block-Scan); Log Address/Topic inverted index (FilterMaps / EIP-7745, Upstream [#16999](https://github.com/paradigmxyz/reth/issues/16999) / [#18305](https://github.com/paradigmxyz/reth/pull/18305)); optional Sender→Nonce Accessor + `eth_getBlockReceipts`-Pfad | **nach** PORT-P2P-001 live ✅; Archive-Disk-Budget operatorseitig (Call-Index Ballpark 100–400 GiB-Klasse); Upstream-Bezug [#15394](https://github.com/paradigmxyz/reth/issues/15394), stale [#22626](https://github.com/paradigmxyz/reth/pull/22626), [#13499](https://github.com/paradigmxyz/reth/issues/13499) | 📋 geplant · blocked on P2P live |

**Acceptance (nach Implementierung, gleiches Tip-Pin wie Bench-Doc):**

1. `ots_searchTransactionsBefore/After` OK; Vollscan tip→First-Activity (Bench-Wallet) ≤5 s (Erigon-Referenz ≤3 s).
2. `eth_getLogs` Transfer.from lookback 50k @chunk1k wall ≤0.5 s (Erigon ~0.13 s).
3. Optional: WalletHistoryExport Reth-only ≤120 s; `du`/DB-Stats Before/After dokumentieren.
4. Repro-Scripts aus dem Analysis-Doc (`bench-erigon-vs-reth-ipc.py`, `bench-wallet-history-nodes.py`).

**Priorisierte Teilschritte innerhalb FEAT-HIST-001:** (1) CallTraceFrom/To-Index + Stage → (2) `ots_search*` → (3) Log inverted index → (4) Sender+Nonce / BlockReceipts-Tuning.

## Chronologisches Änderungsprotokoll (wichtigste Meilensteine)

### Phase 1+2 (frühere Sessions, zusammengefasst)
- Merge-Branch `rebase/reth-v2.4.1` gegen Upstream v2.4.1 angelegt, 212 Konflikte identifiziert, alle
  aufgelöst (Commit `9260a53e3` u.a.).
- `crates/ethereum-forks` → `crates/ethereum/hardforks` migriert, inkl. opBNB-Fermat-ForkID-Fix.
- Kritischer Fund: `crates/optimism` existiert in `paradigmxyz/reth` v2.4.1 nicht mehr (OP-Stack seit
  Anfang 2025 nach `ethereum-optimism/op-reth` ausgelagert) — eigene Strategieentscheidung nötig
  (unser Fork behält eigene `crates/optimism`, wird schrittweise nachgezogen).
- Kritischer Fund: `crates/blockchain-tree` upstream komplett entfernt, ersetzt durch `engine-tree`
  Architektur — größter struktureller Umbau, `bnb-chain/reth` dient als Referenzimplementierung.
- `Block`/`SealedBlock`/`RecoveredBlock`-Trait-Subsystem in `primitives-traits` wiederhergestellt.
- `eth-wire-types`: `BlobSidecars`-Import, `EthMessageID::UpgradeStatus`-Match, `Receipt`
  `Encodable`/`Decodable`-Bounds gefixt.
- `ExecutionOutcome` generisch über Receipt-Typ gemacht; `NodePrimitives::BlockBody`-Bound gefixt.
- Kona-node vs. op-node Evaluierung für opBNB-Rollup-Node dokumentiert (Commit `4324d21d2`) — siehe
  Abschnitt "Kona-node vs. op-node" unten.
- README: Binance/BNB-Marketingsprache entfernt, No-Warranty-Disclaimer + Effort-Log ergänzt
  (Commit `9781c77e7`).

### Session vom 2026-08-06 (aktuelle Session, `a95758da-...`)

**Größter Blocker gelöst: `crates/primitives` + `crates/primitives-traits` komplett neu geschrieben**

Hintergrund: Upstream v2.4.1 hat `reth-primitives` radikal verschlankt — `Transaction`,
`TransactionSigned`, `Block`, `BlockBody`, `Receipt` sind dort nur noch dünne Typ-Aliase/Re-Exports
über `alloy_consensus`/`reth_ethereum_primitives`/`reth_primitives_traits`, keine eigenen Structs mehr.
Unser Fork hatte noch die alte, ~4200 Zeilen umfassende v1.1.1-Architektur mit eigenen
Transaction/Receipt/Block-Implementierungen — nicht mehr kompatibel mit den neuen Traits.

Vorgehen:
- Vergleich mit `bnb-chain_reth.git` (v2.4.1-Referenz) bestätigt: kein `parlia`/BSC-Code dort — unsere
  BSC-Erweiterungen (`parlia/`, `proofs.rs`, `compression/`, `constants/`) sind rein Fork-eigen und
  mussten erhalten bleiben.
- `crates/primitives/src`: `receipt.rs`, `block.rs`, `alloy_compat.rs`, `transaction/` (alt),
  `traits/`, `compression/` (inkl. .bin-Dictionaries) gelöscht; neue Shim-Versionen aus
  `bnb-chain_reth.git` adaptiert (`block.rs`, `receipt.rs`, `proofs.rs`, `transaction/{mod,pooled,
  signature,tx_type,util}.rs`).
- `lib.rs` neu geschrieben nach Upstream-Struktur, BSC-Module (`parlia`, `constants`, `proofs`)
  erhalten.
- `Cargo.toml` verschlankt, aber `derive_more`/`modular-bitfield`/`bytes`/`reth-codecs`/`serde` bewusst
  behalten (werden von BSC-Parlia-Structs mit `Compact`-Derive und `wrap_fixed_bytes!`-Makro benötigt).
- `crates/primitives-traits`: fehlende Upstream-Module ergänzt — `crypto.rs`, `proofs.rs`,
  `transaction/error.rs`, `transaction/execute.rs` (alle aus Upstream kopiert/adaptiert),
  `gas_spent_by_transactions`-Funktion in `receipt.rs` ergänzt (bewusst als freie Funktion, nicht als
  Trait-Bound-Änderung, um vorherige `Encodable+Decodable`-Fixes an `Receipt`-Trait nicht zu
  gefährden).
- Folgefehler behoben: `crates/bsc/consensus/src/validation.rs`
  (`calculate_receipt_root_ref` → `calculate_receipt_root`), unused-Import in
  `crates/evm/execution-types/src/chain.rs` (`TxHashRef`).
- **Ergebnis:** `cargo check -p reth-primitives-traits`, `-p reth-primitives`,
  `-p reth-execution-types` kompilieren sauber.

**`crates/bsc/chainspec` API-Drift behoben**

- `EthChainSpec`-Trait in v2.4.1 stark verändert: neues assoziiertes `type Header`, neue Methode
  `blob_params_at_timestamp`, entfernt: `base_fee_params_at_block`, `max_gas_limit`;
  `final_paris_total_difficulty` jetzt ohne `block_number`-Argument hier definiert.
- `EthereumHardforks`-Trait kommt jetzt aus externer `alloy-hardforks`-Crate, kollabiert auf eine
  Pflichtmethode `ethereum_fork_activation(&self, fork) -> ForkCondition`.
- `ChainSpec` jetzt generisch über Header-Typ (`ChainSpec<H: BlockHeader = Header>`), `genesis_header:
  SealedHeader<H>`-Feld statt altem `genesis_hash: OnceCell<B256>`/`once_cell_set()`-Muster.
- `crates/bsc/chainspec/src/{lib,bsc,bsc_chapel,bsc_rialto,dev}.rs` entsprechend umgeschrieben
  (Konstruktionsmuster `genesis_header: SealedHeader::new(make_genesis_header(&genesis, &hardforks),
  HASH)` aus `bnb-chain_reth.git/crates/optimism/chainspec/src/op.rs` übernommen).
- `alloy-eips`-Dependency zu `crates/bsc/chainspec/Cargo.toml` ergänzt (für `BlobParams`).
- No-std-Leak in `crates/chainspec/src/spec.rs:653` gefixt (`std::fmt::Debug` → `core::fmt::Debug`).
- **Ergebnis:** `cargo check -p reth-bsc-chainspec` kompiliert sauber.

**Commits dieser Session:**
```
3621e69ea Fix reth-bsc-chainspec + reth-chainspec for v2.4.1 EthChainSpec/EthereumHardforks API
fc022d486 Cleanup unused imports (chain.rs TxHashRef)
4be769cb2 Rewrite crates/primitives + primitives-traits per upstream v2.4.1 shim pattern
```

**Wichtige Nutzer-Vorgabe (dauerhaft, alle Sessions/Repos):** Niemals Backups/Scratch-Dateien nach
`/tmp` schreiben — stattdessen Session-Workspace `files/`-Verzeichnis nutzen. (Nutzer-Korrektur am
2026-08-06, als user-scoped Memory hinterlegt.)

## Aktuell offene Blocker (Stand jetzt, vor `cargo check -p reth-bsc-evm`)

Kompiliert NICHT sauber, in ungefährer Abhängigkeitsreihenfolge:

1. **`reth-storage-db-api`** (`crates/storage/db-api/src/models/mod.rs`) — Orphan-Rule-Verletzung:
   Makro `impl_compression_fixed_compact!` versucht `Decompress`/`Compress` für externe Typen
   (`B256`, `Address`) zu implementieren, die nicht lokal in dieser Crate definiert sind. Muss
   untersucht werden, ob Upstream hier eine andere Struktur (z.B. Newtype-Wrapper oder
   Blanket-Impl-Standort) nutzt.
2. **`crates/storage/db-api/src/tables/mod.rs`** — Selbstreferenz `reth_db_api::DatabaseError` nicht
   gefunden (sollte vermutlich `crate::DatabaseError` sein, da dies bereits die `reth-db-api`-Crate
   ist).
3. **`reth-consensus`** (`crates/consensus/consensus/src/lib.rs`, `noop.rs`):
   - `reth_execution_types::BlockExecutionResult` existiert in unserem Fork nicht (unser
     `BlockExecutionOutput` vereint, was Upstream in `BlockExecutionOutput` + `BlockExecutionResult`
     aufteilt) — braucht Adaption ähnlich der bereits gemachten Anpassung in `execution_outcome.rs`.
   - Fehlende Konstanten `GAS_LIMIT_BOUND_DIVISOR`/`MAXIMUM_GAS_LIMIT_BLOCK` in
     `reth_primitives_traits::constants`.
   - `#[display(...)]`-Attribut auf `ConsensusError`-Varianten ohne zugehöriges
     `#[derive(Display)]` von `derive_more` (vermutlich Merge-Artefakt, Derive und Attribut getrennt).
4. **`reth-bsc-primitives`** (`crates/bsc/primitives/src/system_contracts/mod.rs`) — `.get()`-Methode
   auf `BSC_TESTNET_CONTRACTS`/`BSC_QA_CONTRACTS` nicht gefunden (vermutlich `lazy_static!`/HashMap-
   API-Mismatch, noch nicht untersucht).
5. **`reth-trie-sparse`** (`crates/trie/sparse/src/state.rs:398`) — `.par_bridge_buffered()`-Methode
   nicht gefunden auf `IntoIter` (vermutlich rayon-Versions-Drift, ähnlich dem früheren
   `revm::db`→`revm::database`-Pfadwechsel). Muss geprüft werden, ob `.par_bridge()` semantisch
   gleichwertig ist oder ob eine gepatchte rayon-Erweiterung fehlt.
6. **`reth-bsc-evm`** — blockiert transitiv durch alle obigen Punkte.

Zusätzlich bekannt, aber noch nicht angegangen:
- **`crates/optimism/chainspec/src/lib.rs`** hat exakt dasselbe kaputte `EthChainSpec`/
  `EthereumHardforks`/`once_cell_set`-Muster wie `crates/bsc/chainspec` vor dem Fix — plus fehlende
  Modul-Dateien (`base.rs`, `base_sepolia.rs`, `dev.rs`, `op.rs`, `op_sepolia.rs` werden per `mod`
  referenziert, existieren aber nicht). Größerer separater Task.

## Kona-node vs. op-node Evaluierung (Zusammenfassung, Details in Commit `4324d21d2`)

- Anlass: `kona-node` (moderneres Rollup-Node-Projekt) benötigt `--l1-beacon`
  (Konsensschicht-/Beacon-Endpunkt), das BSC in dieser Form/diesem Datenformat nicht liefert.
  Live-Test (2026-08-06, `ETHGravityRethMinimalNode`) zeigte den erwarteten Crash: `kona-node`
  verlangt zwingend `--l1-beacon <L1_BEACON>`, ohne Fallback.
- Bewertung: `kona-node` ist für BSC/opBNB aktuell **nicht** ohne zusätzliche Anpassung (Beacon-Daten-
  Adapter oder Bypass) einsetzbar; `op-node`-kompatibler Ansatz bleibt für jetzt die pragmatischere
  Wahl. Rollup-Defaults, die ggf. schon in neuem reth/op-reth enthalten sind, wurden im Rahmen der
  Evaluierung mitgeprüft (Details siehe Commit-Dokumentation).
- **TODO (steht noch aus):** Nach echten Live-Tests gegen opBNB-Testnet erneut prüfen und Doku/README
  aktualisieren (Nutzer-Vorgabe: "nach Live-Tests die Infos aktualisieren").

## Aufwandsprotokoll (für README "About This Fork" — wird bei jedem Meilenstein nachgeführt)

| Session | Zeitraum (UTC) | Modelle | Input-Tokens | Output-Tokens | Turns / Events | Wichtigste Ergebnisse |
| --- | --- | --- | --- | --- | --- | --- |
| Frühere Sessions (kumulativ, vor `a95758da`) | mehrere Tage | Claude Sonnet 5 (primär), GPT-5.4 | ~58,7M (Sonnet 5) + ~38,5M (GPT-5.4) | ~231K (Sonnet 5) + ~78K (GPT-5.4) | ~800 | Merge/Rebase auf v2.4.1, Konflikte, Blockchain-Tree→Engine-Tree, Kona-Node-Eval, README-Disclaimer |
| Copilot CLI `a95758da` (Snapshot **2026-08-09**, final DB) | 2026-08-06 09:50 – 2026-08-07 18:05 | Sonnet 5, GPT-5.4, Sonnet 4.6, GPT-5.3-Codex, GPT-5.4-mini | ~356,9M + ~135,4M + ~88,4M + ~63,1M + ~6,4M = **~650,1M** (+ ~636M cache-read) | ~1,163K + ~298K + ~260K + ~124K + ~16K = **~1,861K** | 32 Turns / 5.803 Events / ~8,1h request-wall | Compile-Loop bis node-core/stages/rpc-Typen; Phase-2/3 Vorarbeit |
| Cursor Composer YOLO (Session 6, Chat `42f88fe7…`, Snapshot **2026-08-09 12:05 UTC**) | 06:45 – ~12:05 UTC (**~5,34 h** Wall) | **composer-2.5-fast** (4.986 `modelName`-Hits) + **cursor-grok-4.5-high-fast** (178); Parent `default` | **Kein lokaler Billed-Token-Ledger.** Content-Proxy: Transcripts ~2,34M chars ≈ **~0,58M Tokens** (÷4); Cleartext-Chat-JSON ≈ **~0,33M Tokens** (Untergrenze, Tool/Context unterzählt). Erwartete billed/context-Wiederholung deutlich höher | (Proxy, s. Input-Spalte) | **15 Agents** (1 Parent + 14 Subs); 2.582 Assistant-Msgs; 5.861 Tool-Blobs; ~11.722 Tool-Calls; 74.482 `ai_code_hashes` | **`reth-bsc-node --features bsc` + workspace `--no-default-features` grün**; Phase-4 op-forks/chainspec/primitives/consensus; Details: `files/cursor-session-metrics.json` |
| Cursor Session 8 (Chat `d6ebb428…`, Snapshot **2026-08-09 ~14:30 UTC**) | ~12:18 – ~14:25 UTC (**~2,1 h** Commit-Span; ~1,4 h Chat-Wall) | Auto/Composer (kein per-request Model-Ledger im Transcript) | Transcript-Proxy **~0,11M Tokens** (÷4); billed meter n/a | (Proxy) | ~816 Tool-Calls; 11.288 `ai_code_hashes`; 350 assistant / 18 user msgs | op-evm→payload/rpc/node/cli/`op-reth` grün; opBNB init+RPC smoke; nextest chainspec/forks 23/23; Details: `files/cursor-session8-metrics.json` |
| Cursor Session 9 (Chat `6a6455c9…` + Vorabend `9be255b9…` PORT-STOR-006, Snapshot **2026-08-10 ~08:30 UTC**) | Vorabend SCS-Port unterbrochen; Resume **05:57–~08:30 UTC** (**~2,5 h** Chat-Wall inkl. EF-Rootcause); Commit-Span **06:06–~08:27 UTC** | Auto/Composer + Task-Subagents (inherit); kein per-request Model-Ledger | Transcript-Proxy kombiniert **~97K+** Tokens (÷4, früher Snapshot ~97K; Session fortgesetzt); billed meter n/a | (Proxy) | Resume früh: 12 user / 118 assistant; **250** tool_use; danach EF-Deep-Dive (Bytecode Compact) | **PORT-STOR-006**; stages **106**; op-stack nextest; EF **v17.0** + Compact-Fix → **61/62** suites; Details: `files/cursor-session9-metrics.json` |
| Cursor Session 10 cont. (Chat `84eb0b61…`, Snapshot **2026-08-11 ~16:50 UTC+2**) | Live-Sync P2P-003/004/005: **~12:00–16:50** (**~4,8 h** Wall) inkl. Nachziehen der Dataflow-Lücken + **3× maxperf-op** (~20–23 min/Link, JOBS=1) | Auto/Composer | Transcript-Proxy n/a | (Proxy) | Matrix-Soll Tip-Resolve/Cap/Falling (Analyse nachgezogen); eth/69; Unit-Tests; Live-Verify | **P2P-003/004/005 live ✅** Falling @~22k hdr/s. Rebuilds: eth69~23 min, Cap~20 min, Falling~21 min. Tests: fetch 43/43, reverse_headers 11/11. ETL-TempDir = Upstream-Design |
| Cursor Session 12 (Chat `ea987bef…`, Snapshot **2026-08-15 ~10:54 CEST**, kumulativ 08-12→15) | Kalender **~66,5 h** (08-12 16:27–08-15 10:54); **6** Interaktiv-Cluster **~4,5 h** Span (Gap>90 min; +Pad ≈**~6 h**) | Auto/Composer (+1 Task) | Transcript-Proxy: Msg-Text **~72 K** Tok (÷4); File **~216 K** Tok (÷4); billed n/a | (Proxy) | **84** user / **367** asst; **567** tool_use (Shell 219, Read 113, StrReplace 108, Grep 93); Details: `files/cursor-session12-metrics.json` | **EXEC-001** open; PIPE-014/X04/X05; Harness+dump-flag; OPS-001/ENGINE-004; Cap Bodies/Sender; offline X04 Exec `20365614→21591153`; SF≠Cap dokumentiert |
| Cursor Session 12 cont. (Teil-Snapshots 08-13…08-15) | s. Cluster in Metrics-JSON | Auto/Composer | (in kumulierter Zeile) | (Proxy) | Fail#1–3; Tip-Rettung; Cap; offline X04/SF-Heal; CLI inkl. vs half-open | Dump `re-execute 54..55` nach Exec-fertig |

> Hinweis: Copilot-Token-Zahlen sind kumulative Modellaufrufe inkl. Tool-Nutzung/Kontext-Wiederholung pro
> Turn. Cursor speichert hier **keinen** äquivalenten `assistant_usage_events`-Zähler (Chat-Blobs teils
> verschlüsselt) — daher Activity-Counts + Content-Size-Proxies. Kein Effizienz-Benchmark.
> **Kosten (illustrativ, kein Invoice):** Copilot `a95758da` allein ~650M in / ~1,9M out ≈ **USD 1,5–2k**
> bei öffentlichen Sonnet/GPT-Listenpreisen ohne Cache-Rabatt; **Cursor Session 12** nur Proxy
> (~72 K–216 K Tok Content, **~4,5–6 h** Interaktiv-Wall) — **billed** nur Account-Dashboard /
> Abo (Context-Resend ≫ Content-Proxy). Quellen: Copilot `<copilot-session-store>`;
> Cursor `agent-transcripts/` + `files/cursor-session12-metrics.json`.

## Nächste Schritte (unmittelbar, in Reihenfolge)

1. `reth-storage-db-api`: Orphan-Rule-Fix für `Compress`/`Decompress` auf `B256`/`Address`.
2. `reth-storage-db-api`: `reth_db_api`-Selbstreferenz in `tables/mod.rs` fixen.
3. `reth-consensus`: `BlockExecutionResult`-Ersatz, fehlende Gas-Limit-Konstanten, `Display`-Derive
   auf `ConsensusError` reparieren.
4. `reth-bsc-primitives`: `BSC_TESTNET_CONTRACTS`/`BSC_QA_CONTRACTS`-`.get()`-Problem lösen.
5. `reth-trie-sparse`: `.par_bridge_buffered()`-Ersatz klären und fixen.
6. `cargo check -p reth-bsc-evm` erneut, iterieren bis grün.
7. `cargo check -p reth-bsc-node` / breitere BSC-Crate-Prüfung, dann `cargo check --workspace`.
8. `crates/optimism/chainspec` analog zu BSC fixen (inkl. fehlender Modul-Dateien) — Phase 4 Vorarbeit.
9. Nach jedem grünen Meilenstein: Commit mit ausführlicher Begründung, `plan.md` + Todo-Status
   aktualisieren.
10. Danach: Phase 4 (opBNB Snow/Volta/Fourier Hardforks), Phase 5 (Build/Lint/Test/EF-Tests),
    Phase 6 (Doku-Feinschliff, Live-Test-Nachträge in README).

## Session `a95758da` Fortsetzung (2026-08-06, `cargo check -p reth-bsc-evm` Kompilier-Loop)

Weiter Richtung `reth-bsc-evm` grün: iterativer Loop "Check → nächster Blocker → Fix (direkt oder
per Hintergrund-Agent) → Commit → nächster Blocker". Reihenfolge der in dieser Fortsetzung
behobenen Blocker (jeweils mit `cargo check -p <crate> --no-default-features` verifiziert):

- **`reth-prune-types`** (`segment.rs`): gleiches Feature-Gating-Muster wie zuvor bei `target.rs`
  auf `Compact`/`Serialize`/`Deserialize`-Derives angewendet (`cfg_attr(any(test, feature = ...))`).
- **`reth-trie-sparse`** (`state.rs`): totes/nie definiertes
  `reth_primitives_traits::ParallelBridgeBuffered` (Merge-Artefakt) durch Standard-
  `rayon::iter::{ParallelBridge, ParallelIterator}`/`.par_bridge()` ersetzt.
- **`reth-execution-types`** (`execute.rs`): fehlendes `use alloc::vec::Vec;` in `no_std`-Kontext
  ergänzt.
- **`reth-storage-api`**: `lib.rs` hatte ~14 fehlende `mod`/`pub use`-Deklarationen für bereits
  vorhandene Dateien (`bal.rs`, `chain.rs`, `metadata.rs`, `state_writer.rs`, u.v.m.) — komplett
  neu geschrieben mit vollständiger, Feature-gegateter Mod-Liste. `withdrawals.rs` (komplett
  fehlend) aus Git-Historie (`95558cb45~1`) wiederhergestellt. `BlockReader`/`BlockReaderIdExt`/
  `noop.rs`/`state_writer.rs` per Hintergrund-Agent auf v2.4.1-Form gebracht (`BlockExecutionOutput`
  bleibt bewusst flach: `state`/`receipts`/`requests`/`gas_used`/`snapshot`, keine `.result`-
  Verschachtelung). Zweite Runde: `StorageSettings`-Re-Export (`models/metadata.rs` in
  `models/mod.rs` verdrahtet), `FullBlockHeader`/`FastInstant`-Kompatibilität in
  `primitives-traits` ergänzt, `BlockOmmers`-Tabellenzugriff auf nicht-generische Form umgestellt.
- **`reth-db`/`reth-db-api`**: fehlende `StaticFileMap<T>`-Typalias, fehlender `TableSet`-Trait
  (+ `impl TableInfo`/`TableSet for Tables`). Static-File-Masken (`mask.rs`/`masks.rs`) auf
  Upstream-Muster "je Verwendung ein eigener Marker-Typ" umgestellt (`HeaderWithHashMask`,
  `TDWithHashMask`, `BlockHashMask`, `SidecarWithHashMask`, `RawTransactionMask`) — behebt einen
  echten Rust-Trait-Overlap-Checker-Fehlalarm (E0119) bei generischen Structs über assoziierten
  Typ-Projektionen.
- **`reth-network-p2p`**: `SealedBlockWith<B,T>`-Struct (BSC/BAL-Feature) war durch einen
  Merge-Commit ("refactor: reuse primitive sealed block wrapper") entfernt worden, in der
  Annahme, sie sei nach `reth_primitives_traits` gewandert — das war nie passiert (totes
  Merge-Artefakt). Lokal wiederhergestellt. `SealedHeader::size()` von nicht-generischer
  Inherent-Methode auf generischen `impl<H: InMemorySize> InMemorySize for SealedHeader<H>`
  umgestellt (passend zu Upstream).
- **`reth-blockchain-tree-api`**: `error.rs` war komplett gelöscht (nicht wiederhergestellt worden)
  — aus Git-Historie (`53ccb5d46~1`) restauriert, dann alle Fehler-Enum-Formen (`BlockExecutionError`/
  `BlockValidationError`) auf die neue `alloy_evm::block`-Form migriert (kein `Consensus`-Variant
  mehr, kein `BlockPreMerge`/`StateRoot` mehr — konservativ auf `false`/entfernt abgebildet, siehe
  Kommentare im Code für Semantik-Lücken). `try_seal_with_senders()` → `try_recover()` (neue
  `RecoveredBlock`-API), `BlockRecoveryError::into_inner()` zum Auspacken des Fehlerfalls ergänzt.
  `MaybeCompact`-Hilfstrait fehlte komplett in `primitives-traits/src/lib.rs` (für
  `FullBlockHeader: BlockHeader + MaybeCompact`) — nach Upstream-Vorbild (Feature-gegatet auf
  `reth-codec`) ergänzt.
- **`reth-network-api`**: `EngineMessage`/`BlockHashesEvent`/`BlockEvent<N>`-Typen fehlten in
  `events.rs` (BSC-spezifische Engine-Message-Weiterleitung network↔engine) — aus Git-Historie
  (Commit `41d092253`) wiederhergestellt und auf aktuelle generische `NetworkPrimitives`/
  `NewBlock<N::Block>`-Form angepasst.
- **`reth-chain-state`**: größerer Blocker (~33 Fehler) — fehlende Workspace-Deps
  (`reth-ethereum-primitives`, `reth-primitives-traits`, `alloy-consensus` fest statt optional),
  `StateProvider`-Trait-Drift (`&Address`/`&B256`-Signaturen statt Wert, neue Pflicht-Trait-Teile
  `HashedPostStateProvider`/`BytecodeReader`, `storage_multiproof`, `MultiProofTargets`-Typ,
  zusätzlicher `ExecutionWitnessMode`-Parameter bei `witness()`), `BlockExecutionOutput`-Zugriffe
  ohne `.state`-Präfix (Fork-spezifische Flachstruktur beachten), `ExecutedBlock`-Feldumbenennung
  (`block`→`recovered_block`, `bundle`→`state`), `BundleState::into_plain_state`→`to_plain_state`.
  Per Hintergrund-Agent gegen `bnb-chain_reth.git`-Referenz gelöst und verifiziert
  (`cargo check -p reth-chain-state --no-default-features` grün).
- **`reth-trie-db`/`reth-db-api`**: `trie_cursor.rs` erwartete bereits die gepackten Tabellen-Views
  (`PackedAccountsTrie`/`PackedStoragesTrie`, 33-Byte-Keys) aus Upstream-PR #22158
  ("pack StoredNibblesSubKey from 65→33 bytes"), aber `db-api` hatte nie die zugehörigen
  Tabellen-Definitionen/`Encode`/`Decode`/`Compress`-Impls erhalten. Aus dem Upstream-Commit
  `80bf5532a` 1:1 portiert (`PackedAccountsTrie`/`PackedStoragesTrie`-Wrapper-Structs in
  `tables/mod.rs`, `Encode`/`Decode`-Impls + `impl_compression_for_compact!`-Registrierung in
  `models/mod.rs`). Verifiziert grün.

**Laufend/delegiert bei Session-Fortsetzung:**
- Hintergrund-Agent `fix-evm-evm`: `crates/evm/evm` (Kern-`reth-evm`-Crate) fehlten mehrere
  Workspace-Deps (`alloy-evm`, `alloy-consensus`, `reth-trie-common`, `reth-storage-api`,
  `derive_more`) und `execute.rs` hatte fehlende Trait-Bounds (`BlockExecutorFactory`-assoziierte
  Typen `ExecutionCtx`/`Transaction`/`Receipt`, `Evm`-assoziierter Typ) — gegen
  `bnb-chain_reth.git`-Referenz (identische, nicht-BSC-spezifische Datei) delegiert. Ergebnis
  noch ausstehend bei Verfassung dieses Plan-Updates.

**Commits dieser Fortsetzung** (chronologisch, jeweils mit `Co-authored-by: Copilot`-Trailer):
`545faf637`, `f01a93c66`, `d27471397`, `8739fa87c`, `cd4943f34`, `4cb4603fd`, `8315201e0`,
`bd1fc9591` (+ ggf. `fix-evm-evm`-Commit sobald Agent abgeschlossen).

**Nächste Schritte nach `reth-evm`-Fix:**
1. `cargo check -p reth-bsc-evm --no-default-features` erneut ausführen, nächsten Blocker
   identifizieren (BSC-eigene EVM-Konfiguration, Precompile-Registrierung).
2. Iterieren bis `reth-bsc-evm` komplett grün — das ist der aktuelle Phase-3-Meilenstein.
3. Danach `reth-bsc-node`, dann breitere Workspace-Prüfung.
4. `plan.md` weiterhin nach jedem größeren Fund/Fix aktualisieren (Nutzer-Vorgabe: lückenlos, für
   spätere Übernahme in die User-Doku inkl. Token-/Zeit-Aufwand).

## Session-Fortsetzung: SignedTransaction-Fix, reth-provider-Blocker (Commit 717a5743c)

- **reth-transaction-pool grün:** `EthPooledTransaction::new()` rief `max_fee_per_gas()`/
  `gas_limit()`/`value()`/`blob_gas_used()`/`max_fee_per_blob_gas()` auf `Recovered<T>` auf.
  `alloy_consensus::Recovered<T>` hat **keinen `Deref`-Impl** (reiner Wrapper) — `T` selbst
  muss `alloy_consensus::Transaction` implementieren. Root Cause: unser `SignedTransaction`-
  Trait (`crates/primitives-traits/src/transaction/signed.rs`) hatte den Supertrait-Bound
  `+ alloy_consensus::Transaction` verloren (upstream vorhanden). Ein einziger Bound-Zusatz
  behob alle 5 verbleibenden Fehler. Zusätzlich vorher schon erledigt: `FullBlock`/
  `FullSignedTx`/`FullBlockBody`-Marker-Traits (komplett gefehlt), `try_into_recovered()`,
  `TipZero`-Match-Arm-Fix. Verifiziert grün: `reth-primitives-traits`, `reth-node-types`,
  `reth-transaction-pool`.
- **Neuer Blocker:** `cargo check -p reth-bsc-evm` → `reth-provider`
  (`crates/storage/provider`) mit **~167 Fehlern** — großer, nicht-BSC-spezifischer
  Kern-Crate. Fehlerkategorien: fehlende/umbenannte Trait-Methoden auf
  TransactionsProvider/HeaderProvider/BlockReader/WithdrawalsProvider/CanonChainTracker,
  fehlende Imports (BlobSidecars, Withdrawals, reth_primitives_traits,
  revm::primitives::{BlockEnv,CfgEnvWithHandlerCfg}, reth_evm::ConfigureEvmEnv,
  reth_static_file_types, rocksdb), Signatur-Mismatches (basic_account &Address,
  Associated-Type-Header/Block-Rückgaben), `tables::Receipts<N::Receipt>`-Generic-Mismatch,
  6 fehlende Modul-Dateien (dead merge artifacts: trie, tree_viewer, stats, state, history,
  header_sync_gap), 2x `gen`-Keyword-Kollisionen, 2x unvollständige Trait-Impls
  (Header-Assoc-Type + local_tip_header fehlt).
- An Hintergrund-Agent `fix-reth-provider` delegiert (detaillierter Prompt mit allen
  Kategorien, Referenz `bnb-chain_reth.git`, Schutz der `bd1fc9591`-Packed-Trie-Tables,
  Anweisung BSC-spezifischen Code wie evtl. RocksDB-Secondary-Storage zu erhalten).

**Commits dieser Fortsetzung (aktualisiert):** ..., `717a5743c` (SignedTransaction-Bound-Fix).

**Nächste Schritte:**
1. Warten auf `fix-reth-provider`-Agent-Ergebnis, verifizieren, committen falls nicht
   bereits durch Agent geschehen.
2. `cargo check -p reth-bsc-evm --no-default-features` erneut, nächsten Blocker angehen.
3. Iterieren bis `reth-bsc-evm` grün (Phase-3-Meilenstein), danach `reth-bsc-node`,
   dann breitere Workspace-Prüfung.
4. `plan.md` nach jedem Meilenstein weiter aktualisieren (inkl. Token-/Zeit-Aufwand für
   spätere User-Doku-Übernahme).

## Session-Fortsetzung: reth-provider grün (156→0), reth-network grün — 2026-08-06

**reth-provider (crates/storage/provider) ist jetzt vollständig grün** (sowohl
`--no-default-features` als auch mit Default-Features/RocksDB), nach einer sehr langen
iterativen Fehlerbehebungs-Session, die über mehrere vorherige Kompaktierungen lief.
Fehleranzahl-Verlauf (dieser Fortsetzungs-Abschnitt): 156 → 126 → 108 → 100 → 85 → 75 → 72
→ 67 → 65 → 54 → 34 → 31 → 17 → 10 → 8 → 6 → **0**. Der Agent (`fix-reth-provider`) hat den
Großteil (156→17) automatisiert erledigt; die letzten ~17 Fehler wurden direkt von mir (ohne
Agent) gelöst, da der Agent bei größeren Verallgemeinerungen (Bound-Propagation) wiederholt
Regressionen verursachte und konservativ zurückrollte — bei architektonisch kniffligen
Restfehlern ist direktes Eingreifen effizienter als weitere Agent-Iteration.

**Kernerkenntnisse / Root Causes:**
- `tables::Receipts`/`tables::Headers` sind in diesem Fork **nicht generisch** (fixiert auf
  konkrete `reth_ethereum_primitives::Receipt`/`alloy_consensus::Header`), im Gegensatz zum
  Referenz-Repo (`bnb-chain_reth.git`), wo `table Receipts<R = Receipt> { ... }` generisch mit
  Default-Typparameter ist. Der `tables!`-Macro dieses Forks unterstützt nur Single-Line-Syntax
  ohne generische Default-Parameter — eine "richtige" architektonische Lösung würde den Macro
  erweitern (nicht gemacht, zu invasiv für die verbleibende Zeit).
- Da aktuell **nur `EthPrimitives` `NodePrimitives` implementiert**, sind `N::Receipt`/
  `N::BlockHeader` in der Praxis immer konkret `EthereumReceipt`/`alloy_consensus::Header` —
  der Compiler kann das aber generisch nicht wissen. Lösung: neue Helper-Funktion
  `crate::compact_convert<From: Compact, To: Compact>()` in `storage/provider/src/lib.rs` —
  eine sichere, generische Byte-Roundtrip-Konvertierung über den bereits garantierten
  `Compact`-Trait-Bound (aus `NodeTypesForProvider`). Angewandt an 5 Stellen:
  `header_by_number`, `receipt`, `receipts_by_tx_range`, `write_execution_outcome`
  (Receipt-Schreiben via `EitherWriter::append_receipt`), und einem `.collect()`-Call beim
  Sammeln von `(u64, Receipt)`-Tupeln.
- `EitherWriter::append_receipt`'s `where`-Bound wurde von `N::Receipt: Into<TableValue>`
  (was einen expliziten `From`-Impl je `NodePrimitives`-Implementierung verlangt hätte, den es
  nicht gibt) auf `N::Receipt: Value + Clone` gelockert, Konvertierung erfolgt jetzt intern via
  `compact_convert`.
- `BlockHeader`-Trait fehlte der Supertrait-Bound `+ AsRef<Self>` (im Referenz-Repo vorhanden)
  — ein einzeiliger Fix, der mehrere `AsRef<HeaderTy<N>>`-Fehler in generischen
  Block-Lese-Funktionen (`recovered_block`, `block_range`, `block_with_senders_range`) behob.
- `Range<u64>` (aus dem lokalen `to_range()`-Helper) hat `.start`/`.end` als **Felder**, nicht
  Methoden — mehrere `E0689`-Fehler durch fälschliche `.start()/.end()`-Methodenaufrufe (Agent
  hatte das per Sed-Fix falsch gemacht) korrigiert via Feldzugriff +
  `saturating_sub(1)`-Rekonstruktion zu `RangeInclusive` wo nötig.
- `write_storage_trie_updates_sorted`: Cursor für `tables::StoragesTrie`/`PackedStoragesTrie`
  muss **innerhalb** des `with_adapter!`-Makro-Closures geöffnet werden (via
  `<A as TrieTableAdapter>::StorageTrieTable`, vollqualifizierte Syntax wegen `E0223`), nicht
  davor mit fest codiertem Tabellentyp — der Makro wählt den Adapter (Legacy vs. Packed) zur
  Laufzeit.
- `tx_hash()`/`recover_signer_unchecked()`: fehlender Import `use
  reth_primitives_traits::SignedTransaction;` — klassischer "Trait-Methode nicht gefunden, weil
  Trait nicht importiert"-Rust-Fallstrick.
- **Wichtigste Lektion:** Ein `where`-Bound auf einen **ganzen impl-Block** (statt einer
  einzelnen Methode) kann durch Bound-Propagation dramatische Regressionen an entfernten
  Stellen verursachen (Beispiel: ein Versuch sprang von 17 auf 63-69 Fehler). Lokale,
  call-site-spezifische Konvertierungen (wie `compact_convert`) sind der sicherere Weg.

**Commit:** `fix(provider): resolve final reth-provider compile errors, reach green`
(direkt nach den `fix-reth-provider`-Agent-Commits `bdf13bfc3` … `869e3066d`).

**reth-network (crates/net/network + net/eth-wire-types) danach direkt (ohne Agent, schnell)
gelöst — 6 Fehler:**
- `SignedTransaction`-Trait fehlte `is_broadcastable_in_full()`-Default-Methode (nutzt
  bereits verfügbares `is_eip4844()` aus `alloy_consensus::Transaction`-Supertrait) —
  Upstream-Parität hergestellt.
- `NetworkPrimitives::PooledTransaction` fehlte `IsTyped2718`-Bound (für
  `N::PooledTransaction::is_type(ty)` in `transactions/config.rs`-Announcement-Policies).
- `NetworkPrimitives::BroadcastedTransaction` fehlte `TxHashRef + IsTyped2718`-Bounds (für
  `PropagateTransaction::new()`/`BroadcastPoolTransaction`-Bound in `transactions/mod.rs`).
- `NetworkHandle::fetch_client()` gab fälschlich nicht-generisches `FetchClient` zurück statt
  `FetchClient<N>`.
- Commit: `fix(network): resolve reth-network compile errors (Typed2718/TxHashRef bounds)`.

**Verifiziert grün (dieser Fortsetzungs-Abschnitt):** `reth-provider` (beide Feature-Varianten),
`reth-network` (`--no-default-features`).

**Neuer Blocker:** `cargo check -p reth-bsc-evm --no-default-features` → als nächstes
`reth-bsc-consensus` (crates/bsc/consensus, **BSC-spezifisch**) mit 10 Fehlern. Root Cause:
die `Consensus`/`FullConsensus`/`HeaderValidator`-Trait-Hierarchie wurde in v2.4.1
grundlegend umgebaut (drei separate Traits statt einem monolithischen `Consensus`-Trait;
`PostExecutionInput`-Struct entfernt zugunsten von `&BlockExecutionResult<N::Receipt>` +
neuen Optional-Parametern `receipt_root_bloom`/`block_access_list_hash`). Zusätzlich:
`revm_primitives`/`reth_revm::primitives::Account`-Importpfade veraltet,
`alloy_eips::eip4844::MAX_DATA_GAS_PER_BLOCK`-Konstante umbenannt/verschoben. An
Hintergrund-Agent `fix-bsc-consensus` delegiert (detaillierter Prompt inkl. exaktem neuen
Trait-API-Shape, Referenz auf `bnb-chain_reth.git`s Ethereum/Optimism-Consensus-Impls als
Template für die Drei-Trait-Aufspaltung).

**Commits dieser Fortsetzung:** `fix(provider): resolve final reth-provider compile errors,
reach green`, `fix(network): resolve reth-network compile errors (Typed2718/TxHashRef bounds)`
(+ ggf. `fix-bsc-consensus`-Agent-Commit sobald abgeschlossen).

**Wichtiger Hinweis für User-Doku (Standing-Notiz, bereits vom User bestätigt):** nach den
ersten Live-Tests müssen alle Aufwands-/Token-/Zeit-Angaben in der User-Doku aktualisiert
werden — dieser Plan.md-Log ist die Rohdaten-Quelle dafür.

**Nächste Schritte:**
1. Warten auf `fix-bsc-consensus`-Agent-Ergebnis, verifizieren, ggf. nachbessern, committen.
2. `cargo check -p reth-bsc-evm --no-default-features` erneut, nächsten Blocker identifizieren.
3. Iterieren bis `reth-bsc-evm` grün (Phase-3-Meilenstein), danach `reth-bsc-node`, dann
   breitere Workspace-Prüfung (`cargo check --workspace`).
4. `plan.md` nach jedem Meilenstein weiter aktualisieren.

## Session-Fortsetzung: reth-bsc-node-Blocker (nach reth-bsc-evm-Meilenstein)

**Stand:** `reth-bsc-evm` erreicht grünen Zustand (Phase-3-Meilenstein, beide Feature-
Varianten). Fortsetzung mit `reth-bsc-node` als nächstem Meilenstein.

**Behobene Blocker-Kette (chronologisch):**
1. `reth-primitives-traits` fehlte komplett das `serde_bincode_compat`-Modul (Datei nie
   nach dem Merge wiederhergestellt) — neu erstellt in
   `crates/primitives-traits/src/serde_bincode_compat.rs` nach Vorbild `bnb-chain_reth.git`.
2. `reth-config`: serde/toml/humantime-serde waren optional+feature-gated, `config.rs`
   nutzt sie aber unconditional — als Pflicht-Deps gemacht statt `config.rs` komplett
   umzubauen.
3. `reth-node-core`/`build.rs`: **Lektion gelernt** — Workspace pinnt vergen/vergen-git2
   auf **10.0.1** (ältere API: `Build::builder()...build()` ohne `?`), NICHT auf 9.1.0 wie
   `bnb-chain_reth.git` (neuere API: `BuildBuilder`-Suffix, `.build()?`). Erst fälschlich
   auf die neuere API "gefixt", dann korrekt zurückgerollt. Immer die tatsächlich gepinnte
   Version prüfen, nicht vom Referenz-Repo übernehmen!
4. `reth-node-metrics`: `jsonrpsee::server::serve_with_graceful_shutdown` → korrekt
   `jsonrpsee_server::serve_with_graceful_shutdown` (Crate-Pfad-Fix).
5. `reth-execution-types`: `execution_outcome.rs` fehlte das ganze
   `serde_bincode_compat`-Submodul; `chain.rs`s Bincode-`Chain`-Struct hatte kaputten Typ
   `ExecutionOutcome<'a>` (fehlender Generic-Parameter). Erst mit einem custom
   `RlpBincode`-Trait-Ansatz gefixt (erforderte `N::Receipt: SerdeBincodeCompat`), später
   **vereinfacht** auf Upstream-Design zurückgebaut: Receipts werden direkt via
   `T: Receipt`-Bound (der bereits `Encodable`/`Decodable` erfordert) zu rohen RLP-`Bytes`
   serialisiert — kein zusätzlicher `SerdeBincodeCompat`-Bound auf `N::Receipt` mehr nötig.
   Das behebt gleichzeitig `reth-exex-types`, das `Chain<N>` generisch über JEDE
   `NodePrimitives`-Impl instanziiert (inkl. BSCs Receipt-Typ, der nie `RlpBincode`
   implementierte).
6. `reth-ethereum-primitives`: Orphan-Rule-Falle — `impl RlpBincode for
   alloy_consensus::EthereumReceipt<T>` geht nur in `primitives-traits` (Trait-Eigentümer),
   nicht in `reth-ethereum-primitives` (Alias-Nutzer). Mittlerweile durch Punkt 5 überholt
   (kein `RlpBincode` mehr für Receipts nötig).
7. **`reth-rpc-traits`-Duplikat-Problem gelöst:** Diese Crate war als externe
   crates.io-Dependency gepinnt (`version = "0.5.0"`, exakt wie Upstream-v2.4.1s eigener
   Pin — verifiziert, kein Fork-Fehler). Problem: die publizierte `reth-rpc-traits-0.5.2`
   hängt an ihrer EIGENEN crates.io-`reth-primitives-traits` (0.5.2), einem strukturell
   identischen aber DISTINKTEN Typ zu unserem lokalen `crates/primitives-traits`
   (2.4.1-Pfad-Member) → `SealedHeader<T>`-Typkonflikte überall wo `reth-rpc-traits`-
   Methoden auf unsere eigenen Typen treffen. Ein `[patch.crates-io]`-Override griff NICHT
   (publizierte `reth-rpc-traits` verlangt `^0.5`, inkompatibel mit unserer `2.4.1`).
   **Lösung:** `reth-rpc-traits` als lokalen Workspace-Member vendort
   (`crates/rpc/rpc-traits/`, Quellcode 1:1 aus der publizierten 0.5.2 kopiert, nur
   Cargo.toml auf Workspace-lokale Deps umgeschrieben).
8. Fehlende Re-Exports aus `reth-primitives-traits` wiederhergestellt: `SealedHeaderFor<N>`
   (Type-Alias, neu in `header/sealed.rs`), `TransactionMeta`/`SignerRecoverable`/
   `TxHashRef` (Re-Export von `alloy_consensus::transaction`, sowohl im
   `transaction`-Modul als auch im Root-`lib.rs`). Fehlender
   `impl From<SealedHeader<H>> for alloy_consensus::Sealed<H>` ergänzt (benötigt von
   `reth-rpc-traits::FromConsensusHeader`).
9. `SignedTransaction::try_recover()`-Default-Methode ergänzt (Trait-Definition +
   `EthereumTxEnvelope`-Blanket-Impl) — war komplett verschwunden, Upstream hat sie als
   fehlerbehaftete Variante von `recover_signer` (nicht zu verwechseln mit
   `alloy_consensus`s eigenem `SignerRecoverable`, das dieser Alloy-Consensus-Pin gar nicht
   anbietet).
10. `reth-chain-state`: `rayon`-Feature-Flag deklariert (leer, keine echte Dependency) um
    `unexpected-cfg`-Warnungen für bestehende `#[cfg(feature = "rayon")]`-Gates in
    `state_trie_overlay.rs` stumm zu schalten; `serde`-Dependency optional ergänzt +
    `dep:serde`-Wiring (war referenziert aber nie deklariert → E0433 in
    `notifications.rs`).

**Commit:** `b8891d6da` "fix(primitives-traits,rpc-traits,execution-types): unify
reth-rpc-traits dependency, restore missing exports, simplify bincode-compat" — deckt
Punkte 5, 7, 8, 9, 10 ab (Punkte 1-4, 6 waren bereits in früheren Commits dieser Sitzung).

**Verifiziert grün:** `reth-rpc-convert`, `reth-evm-ethereum`, `reth-chain-state`,
`reth-execution-types` (beide Feature-Varianten), `reth-exex-types` (beide
Feature-Varianten), `reth-rpc-traits` (neu vendort).

**Nächster Blocker-Cluster (an Background-Agent `fix-static-file-prune-cluster`
delegiert):** `reth-static-file`, `reth-prune`, `reth-db-common`, `reth-trie-parallel` —
gemeinsamer Nenner: `StaticFileProvider` hat jetzt einen Generic-Parameter `N`
(`StaticFileProvider<N>`), `Segment::copy_to_static_files`-Trait-Signatur hat sich
geändert (3 statt 4 Parameter erwartet), fehlende Re-Exports aus `reth_prune_types`
(`MINIMUM_UNWIND_SAFE_DISTANCE`, `PruneLimiter`), Receipt-Typ-Borrow-Trait-Mismatch in
`append_receipts`. Agent hat Auftrag: alle 4 Crates einzeln grün bekommen, dann
`reth-bsc-node` erneut prüfen und nächsten Blocker NUR berichten (nicht selbst fixen),
eigenen Commit erstellen.

**Nächste Schritte:**
1. Warten auf `fix-static-file-prune-cluster`-Agent-Ergebnis, verifizieren, ggf.
   nachbessern.
2. Nächsten von Agent gemeldeten Blocker angehen (selbst oder erneut delegieren, je nach
   Umfang).
3. Iterieren bis `reth-bsc-node --no-default-features` grün ist (nächster großer
   Meilenstein nach `reth-bsc-evm`).
4. Danach breitere Workspace-Prüfung (`cargo check --workspace --no-default-features`),
   dann mit Default-Features, dann `crates/optimism/*` (opBNB) analog zu BSC.

**Update:** `fix-static-file-prune-cluster`-Agent lieferte `reth-static-file` fertig grün
(committed `44b0e41fa`), aber `reth-prune` erwies sich als deutlich größerer Umbau als
angenommen (~31 tiefere Fehler nach ersten API-Anpassungen: entfernte Static-File-Segmente
`AccountChangeSets`/`StorageChangeSets`/`TransactionSenders`, alte RocksDB-Batch-APIs,
veraltete `Bodies`/Segment-Enum-Varianten) — Agent-Versuch dort wurde verworfen (hätte
Fehlerzahl von 14 auf 32 erhöht), stattdessen sauber zurückgesetzt. `reth-prune` braucht
einen eigenen, breiteren Portierungs-Task gegen die v2.4.1-Prune-Architektur.

**Nächste Schritte (aktualisiert):**
1. `reth-prune` als eigenständigen, größeren Task angehen (nicht als Quick-Fix) —
   entfernte Static-File-Segmente/Enum-Varianten/RocksDB-Batch-APIs gegen v2.4.1 neu
   aufbauen, `bnb-chain_reth.git`s `crates/prune/*` als Referenz.
2. Danach `reth-db-common`, `reth-trie-parallel`.
3. `cargo check -p reth-bsc-node --no-default-features` erneut, nächsten Blocker
   identifizieren.

## Session-Fortsetzung: rpc-traits-Vendoring, writer-Modul-Wiederherstellung, prune/db-common

**Commits dieser Runde:**
- `b8891d6da` — rpc-traits vendored als lokales Workspace-Member (crates/rpc/rpc-traits/),
  SealedHeaderFor/TransactionMeta-Exports, try_recover()-Default-Methode,
  ExecutionOutcome/Chain bincode-compat vereinfacht.
- `9cbedc78f`, `14ca866da` — Doku-Updates.
- `44b0e41fa` — reth-static-file auf v2.4.1-Receipts-only-Segmentmodell portiert
  (Agent-Arbeit, verifiziert vor Commit).
- `6f8e21394` — reth-prune/-types erster Portierungs-Durchgang (Agent-Arbeit),
  reth-prune-types wurde grün, reth-prune selbst blieb transitiv durch
  reth-provider-Fehler blockiert.
- `faa534b33` — reth-provider writer-Modul auf v2.4.1-Architektur restauriert (Agent-Arbeit):
  `UnifiedStorageWriter` existiert in v2.4.1 weiterhin als eigener Typ in
  `crates/storage/provider/src/writer/mod.rs` (keine Verschiebung nach `DatabaseProvider`,
  wie zunächst vermutet) — die Datei war nach dem Merge nur unvollständig wiederhergestellt
  (deklarierte `mod database;`/`mod static_file;` auf nicht existente Dateien). Agent hat sie
  gegen v2.4.1 neu aufgebaut inkl. Parlia-Snapshot/Sidecar-Handling.
- `6519c8d12` — Eigene Fertigstellung: `pub mod writer;` in provider/lib.rs ergänzt,
  db-common/init.rs vollständig auf v2.4.1 portiert (`StateChangeWriter`→`StateWriter`,
  `UnifiedStorageWriter::commit_unwind`→`provider_rw.commit()`, `UnifiedStorageWriter::
  from_database(&p).write_state(...)`→`provider.write_state(...)` direkt, HashMap-Typen auf
  `alloy_primitives::map::HashMap` umgestellt wegen BundleStateInit/RevertsInit-Kompatibilität,
  `StaticFileProvider`/`insert_genesis_header` generisch über `N: NodePrimitives` gemacht,
  `header.difficulty`/`.state_root` auf Methodenaufrufe (`alloy_consensus::BlockHeader`-Trait)
  umgestellt, `StateRootComputer::from_tx` über vollqualifizierte `DatabaseStateRoot`-Trait-
  Syntax mit `LegacyKeyAdapter` aufgerufen). Zusätzlich `PruneSegment::Sidecars`-Variante
  (BSC/opBNB-Blob-Sidecar-Pruning) wieder ergänzt, die der Prune-Port-Agent beim Angleichen
  an Upstream fälschlich entfernt hatte (angehängt ans Enum-Ende gemäß Stabilitätsvertrag).

**Verifiziert grün:** reth-rpc-convert, reth-evm-ethereum, reth-chain-state,
reth-execution-types, reth-exex-types, reth-rpc-traits (vendored), reth-static-file,
reth-prune-types, reth-provider, **reth-db-common** (neu).

**Noch offen / delegiert:** reth-prune (tiefere API-Drift: PruneLimiter-Reexport,
Pruner::new/new_with_factory-Signaturen, PrunedSegmentInfo, RocksDBProviderFactory-Bounds,
AccountChangeSets-Static-File-Segment-Schicksal, Batch-API in account_history.rs) und
reth-trie-parallel (Cargo.toml stark veraltet: fehlende crossbeam-channel/reth-tasks/
reth-storage-api/alloy-evm/revm-Deps, Quellcode nutzt bereits neuere APIs) — beide an
Background-Agent `port-prune-trieparallel-v2-4-1` delegiert, Ergebnis steht noch aus.

**Nächste Schritte:**
1. Agent-Ergebnis für reth-prune/reth-trie-parallel abwarten, verifizieren, committen.
2. `cargo check -p reth-bsc-node --no-default-features` erneut, nächsten Blocker-Cluster
   identifizieren.
3. Iterieren bis reth-bsc-node grün.

---

## Session-Log: Sitzung 5 (Fortsetzung Phase 2 – Compile-Fix Loop)

**Zeitraum:** (laufend)

### Grün geschaltet in dieser Sitzung:
- `reth-trie-parallel` → `StorageRootTargets` aus Reference-Repo hinzugefügt (`crates/trie/parallel/src/storage_root_targets.rs`), in lib.rs exportiert; workspace Cargo.toml: `reth-trie-parallel` mit `default-features=false`
- `reth-trie-prefetch` → Agent `port-trie-prefetch` hat alle 12 Fehler behoben (Commit `4f112f438`); eigene Nachkorrekturen: `StateRootTaskError`-Import entfernt, `LegacyKeyAdapter`-Annotation; metrics immer aktiviert (Feature-Boundary-Problem umgangen)
- `reth-primitives-traits` → `block_timestamp: None` im `TransactionInfo`-Initialisierer ergänzt (alloy-consensus 2.3.0 hat neues Pflichtfeld)
- `reth-rpc-eth-types` → Agent `port-rpc-eth-types`: bereits aus Vorsitzung grün; Restkorrekturen von Agent (cache bounds, simulate.rs)
- `reth-beacon-consensus` → Komplett durch Agent `port-node-api-beacon` neu aufgebaut als Stub (re-exportiert von `reth-engine-primitives`); `BeaconConsensusEngineHandle` = `ConsensusEngineHandle`
- `reth-node-api` → Agent: `FullNodeTypes`, `FullNodeComponents` auf v2.4.1-API portiert (`NodeTypes::Payload` statt `NodeTypesWithEngine::Engine`, `DB` type param, `FullConsensus` statt `Consensus`, `payload_builder_handle()`)
- `reth-basic-payload-builder` → Komplett durch Reference-Repo ersetzt (keine BSC-spezifischen Teile); `PayloadBuilderAttributes`-Trait in `reth-payload-primitives/traits.rs` hinzugefügt; `PayloadJobGenerator::new_payload_job` Signatur auf 1 Param geändert; `PayloadJob::PayloadAttributes: PayloadBuilderAttributes` (statt `PayloadAttributes`)

### Commits (Session 5):
- `df67c8807` — trie-parallel StorageRootTargets, prefetch fixes, primitives-traits TransactionInfo
- `4f112f438` — trie/prefetch: Agentenfix
- `e1102d8df` — rpc-eth-types: Restfehler
- `10425cc11` — node-api + beacon-consensus: v2.4.1-Port (Agent)
- `cae0c7058` — payload traits + basic-payload-builder

### Noch offen:
- `reth-stages` (Restdrift nach Node/Payload-API-Umstellung)
- `reth-node-builder` (größerer API-Drift-Cluster)
- `reth-bsc-node` (abhängig von obigen Restclustern)

### Agent-Token-Verbrauch (geschätzt, Session 5):
- `port-trie-prefetch`: ~85k Tokens, ~24 min
- `port-rpc-eth-types`: ~120k Tokens, ~38 min
- `port-node-api-beacon`: ~65k Tokens, ~16 min
- `port-nodecore-bctree`: ~35 min (abgeschlossen; node-core + blockchain-tree grün)

---

## Session-Log: Sitzung 6 (2026-08-09, YOLO Compile-Loop → `reth-bsc-node` grün)

**Zeitraum:** 2026-08-09 ~06:46–~10:45 UTC (Cursor/Composer YOLO-Session)
**Branch:** `rebase/reth-v2.4.1`
**Meilenstein:** `cargo check -p reth-bsc-node --features bsc` → **0 errors** (verifiziert).
Zusätzlich: `reth-node-ethereum --no-default-features` → **0 errors**.
**Aufwand:** ~12 Composer-Sub-Agents, ~3–4h Wall; Token-Meter nicht in Copilot-DB (README aktualisiert 2026-08-09 mit finalem Copilot-`a95758da`-Snapshot ~650.1M in / ~1.861M out).

### Grün geschaltet (Kette, chronologisch):
- `reth-rpc-eth-api` — HeaderMut, RpcNodeCore-Bounds, nested `BlockExecutionOutput.result`, Trace::inspect generic DB
- `reth-ethereum-payload-builder` / `reth-bsc-payload-builder --features bsc` — BlockBuilder-Pattern (v2.4.1)
- `reth-engine-tree` — monolithic tree/mod.rs → v2.4.1 EngineValidator-Handler; nested `BlockExecutionOutput`
- `reth-engine-service` — fehlendes `service.rs` wiederhergestellt; `reth-engine-util` Reorg auf BlockBuilder
- `reth-bsc-evm --features bsc` — ConfigureEvm/ExecuteEvm-Port, revm-41 Account-API, Parlia-Executor
- `reth-eth-wire` — StreamExt/`EthMessage::<N>`
- `reth-bsc-engine --features bsc` — ParliaClient BlockClient, SealedHeader/FCU ohne version
- `reth-rpc` — Sync + lokale Anpassungen (ExecutionWitnessMode, EthBuiltPayload-Signatur)
- `reth-rpc-builder` — v2.4.1 RpcModuleBuilder; BSC `with_bsc_trace_helper` erhalten
- `reth-stages` — Provider/StaticFile/HashedPostState-Brücken (34→0)
- `reth-node-builder` — Reference-Port + Trail-Engine-API-Anpassung; PayloadBuilderService nutzt `T::PayloadBuilderAttributes`; FullProvider-Sync
- `reth-node-ethereum` / **`reth-bsc-node --features bsc`** — EthExecutorSpec für BscChainSpec, ConfigureEngineEvm, TryIntoTxEnv→BscTxEnv, NodeComponentsBuilder-Wiring

### Wichtige Root Causes / Lektionen:
1. **`PayloadBuilderService`** muss `PayloadJob<PayloadAttributes = T::PayloadBuilderAttributes>` bounden (nicht RPC-`PayloadAttributes`) — sonst ~6 E0271 in node-builder.
2. **`BlockExecutionOutput`** ist nested (`result` + `state` + `snapshot`); flache Feldzugriffe brechen RPC/pending_block.
3. Reference-Copy von `node-builder`/`stages` ohne Angleich an Trail-Engine (2-param `EngineApiRequest`, 17-arg `EngineService::new`) erzeugt große E0271/E0277-Wellen — Bounds/Service-Signaturen lokal halten oder Engine auf Reference zurückziehen.
4. Scratch/Logs nur unter `files/` (nicht `/tmp`).

### Arbeitsbaum:
- Session-7 WIP-Snapshot committed (Code + `plan.md`/`README.md`; Scratch-Logs unter `files/` bleiben untracked).
- Logs: `files/verify-bsc-node-final.log`, `files/bsc-node-check-final.log`, diverse `files/*-check*.log`

### Session 7 (2026-08-09 ~14:10 UTC) — Snow/Volta/Fourier
- `OptimismHardfork::{Snow,Volta,Fourier}` + Schedules aus `bnb-chain/opbnb` `op-node/chaincfg/chains.go`
  (Mainnet/Testnet/QA/Dev). Helpers: `is_*_active_at_timestamp`, `opbnb_block_interval_ms_at_timestamp`.
- Semantik: Snow = L1-Gaspreis-Median (op-node); Volta = 500ms; Fourier = 250ms — keine eigenen revm-`SpecId`s.
- Verify: `reth-optimism-{forks,chainspec,consensus,primitives}`, `reth-bsc-node --features bsc`, workspace `--no-default-features` → grün.
- **op-evm Hard Wall:** lokal noch v1.1.1 (`#![cfg(feature = "optimism")]`, `EvmBuilder::optimism()`, `OptimismFields`); revm 41 hat kein `optimism`-Feature. Referenz `bnb-chain_reth` nutzt `op-revm` 15 — Port = Rewrite, nicht Quick-Fix. Noch nicht in Workspace-Members.

### Nächste Schritte:
1. Großen WIP-Diff reviewen und in sinnvolle Commits splitten (rpc / engine / stages / bsc / node-builder / optimism-forks).
2. ~~`cargo check --workspace --no-default-features`~~ ✅ grün (stale examples excluded: custom-state-root, custom-engine-types, custom-payload-builder, custom-auth-http-middleware, custom-beacon-withdrawals, custom-node-components).
3. Phase 4: `reth-optimism-evm` auf `op-revm`/`ConfigureEvm` v2.4.1 porten (Reference: `bnb-chain_reth.git/crates/optimism/evm`), dann payload/rpc/node.
4. Phase 5: Clippy, nextest, EF-Tests; Default-Features/`bsc`-Workspace; Phase 6: Live-Tests + finale Token-Zahlen.


### Session 7 cont. (2026-08-09 ~14:20 UTC) — op-evm compile milestone
- Path-deps auf lokales `optimism.git/rust` (`op-revm`, `alloy-op-evm`, `alloy-op-hardforks`, `op-alloy-*`) für revm41/alloy-evm0.37.
- `reth-optimism-{primitives,consensus,evm}` aus op-reth gesynct + `primitives-traits` Feature `op` (InMemorySize/SignedTransaction für OpTxEnvelope inkl. PostExec).
- `OpHardforks` Bridge auf `OpChainSpec`; `basefee.rs` (`decode_holocene_base_fee`) ergänzt.
- Verify: `cargo check -p reth-optimism-{forks,chainspec,primitives,consensus,evm}` + `reth-bsc-node --features bsc` → grün.
- Snow/Volta/Fourier bleiben in `OptimismHardfork`; workspace member `crates/optimism/evm/` aktiv.

### Session 6 Docs-Update (2026-08-09 ~10:45–12:45 UTC):
- README Effort-Log: Copilot `a95758da` final **~650.1M in / ~1.861M out / 5803 events**.
- Cursor-Metriken ergänzt (nicht mehr nur „unmetered“): Wall **~5.34 h**, 15 Agents, Models **composer-2.5-fast** / **cursor-grok-4.5-high-fast**, Activity (Msgs/Tool-Calls), Content-Token-Proxies (~0.58M / ~0.33M), **74.482** AI-code hashes; Snapshot `files/cursor-session-metrics.json`.
- plan.md Aufwandsprotokoll-Tabelle synchronisiert.
- Phase 4 Start: `reth-optimism-{forks,chainspec,primitives,consensus}` compile-fähig.
- Phase 5: Workspace `--no-default-features` **0 errors**; WIP weiterhin uncommitted.

### Session 8 (Cursor YOLO) — op-evm green
- Ported `reth-optimism-{primitives,consensus,evm,chainspec basefee}` from local op-reth + path deps (`op-revm`/`alloy-op-evm`/`op-alloy`/`alloy-op-hardforks`) for revm 41.
- `primitives-traits` feature `op`: InMemorySize + SignedTransaction + SerdeBincodeCompat for Op types; reth-codec/serde-bincode-compat wire `op-alloy?/…` for workspace feature unification.
- Verified: `reth-optimism-evm` + stack, `reth-bsc-node --features bsc`, `cargo check --workspace --no-default-features`.
- Next: optimism payload / rpc / node / cli.

### Session 8 cont. — payload/txpool/storage
- Synced `reth-optimism-{storage,txpool,payload-builder}` from op-reth; workspace members + `op-alloy-flz`.
- Adapted payload to trail v2.4.1 `PayloadConfig`/`BuildArguments`/`PayloadTypes::PayloadBuilderAttributes`.
- Added `SignedTransaction::try_clone_into_recovered`.
- Verify: storage/txpool/payload + bsc-node + workspace `--no-default-features` green.
- Next: optimism rpc / node / cli.

### Session 8 cont. — flashblocks/rpc
- Synced `post-exec-replay`, `flashblocks`, `rpc`; stubbed trie-backed `debug`/`state`/`proofs`.
- Flashblocks: trail `BlockExecutionOutput.snapshot` + `recover_transactions::<T>`.
- RPC: drop blanket-conflicting EthSubscriptions/GetBlockAccessList; align receipt/witness APIs.
- Verify: post-exec-replay, flashblocks, rpc green. Next: optimism-node/cli.

### Session 8 cont. — optimism-node WIP
- Synced `reth-optimism-node` from op-reth; stubbed `proof_history` (needs trie/exex).
- Partial trail adapt: PayloadBuilderAttributes, pool-before-executor, RpcAddOns arity,
  PoolBuilder without Evm generic, `OpHardfork` re-export, engine `Fn` (not `FnOnce`) for
  post-exec hashed-state validation.
- **Not green yet**: RpcConvert/SignableTxRequest/OpTransactionRequest + PayloadAttributes
  mismatch (~38 errors). Next: wire OP RpcConverter / SignableTxRequest like ethereum path.

### Session 8 cont. — optimism-node green
- Added `reth-rpc-traits` feature `op` (SignableTxRequest/TryIntoSimTx/FromConsensusTx for Op types).
- Constrained `OpNodeTypes` (Header/EthChainSpec) + PoolBuilder/PayloadBuilderAttributes bounds.
- Verify: `reth-optimism-node`, `reth-bsc-node --features bsc`, workspace `--no-default-features` green.
- Next: `reth-optimism-cli` + `op-reth` bin (opBNB-focused).

### Session 8 cont. — optimism-cli + op-reth bin green
- `SUPPORTED_CHAINS` / `generated_chain_value_parser` in chainspec (opBNB-first + OP/Base carriers).
- Stripped `op_proofs` / slot-preimages seed (needs stages API not in trail yet).
- Adapted CLI to trail command APIs (`env.init` without Runtime, `Arc<DatabaseEnv>`, import pipeline arity).
- `reth-db-api` feature `optimism`: Compact→DB Compress bridge for `OpTxEnvelope`/`OpReceipt` (launch path).
- `proof_history::launch_node` stub launches plain `OpNode` (trie/exex deferred).
- New `crates/optimism/bin` binary package `op-reth` (bsc-reth pattern).
- Verify: `reth-optimism-cli`, `op-reth`, `reth-bsc-node --features bsc` green.
- Next: live opBNB smoke / Clippy / optional trie-proofs; keep BSC green.

### Session 8 cont. — Phase 5 smoke + Clippy + nextest (chainspec/forks)
- Smoke: `op-reth init --chain opbnb` + short `node` boot → RPC up; hardfork list shows Snow/Volta/Fourier.
- Clippy on op-stack finished (no errors); minor chainspec/node cleanups.
- `EthChainSpec::next_block_base_fee` → `Option<u64>` via Holocene `decode_holocene_base_fee`.
- nextest: `reth-optimism-chainspec` + `reth-optimism-forks` **23/23 passed**.
- Next: nextest primitives/consensus/evm; EF-tests; optional trie/proofs (Human: catch-up/full sync).

### Session 8 cont. — docs + maxperf-op (binary not committed)
- README/plan Effort-Log: Session 8 metrics + illustrative Copilot API-equivalent cost (~USD 1.5–2k).
- Snapshots: `files/cursor-session8-metrics.json`; Makefile `maxperf-op` features fixed for trail `op-reth`.
- Build: `make maxperf-op` / `cargo build --profile maxperf -p op-reth` → local `target/maxperf/op-reth` only (**do not commit binaries**).

### Session 8 cont. — PORT-CLI-001 `--storage.v2` restored
- Bug: Flag missing after rebase; genesis settings derived from static-files heuristic instead of `--storage.v2`.
- Fix: flatten `StorageArgs` into `EnvironmentArgs`/`NodeCommand`/`NodeConfig`; `storage_settings()` → genesis; remove bogus `StaticFilesArgs::to_settings`.
- Docs: Portierungs-Bugliste + README run examples (drop obsolete prefetch/exec-cache flags).
- Rebuild maxperf `op-reth` after commit (binary stays local / gitignored).

### Session 8 cont. — PORT-CLI-003/004 IPC + storage-settings stub
- Bug: `--ipcpath` got `-1` suffix (`instance` defaulted to 1); `--storage.v2` still ineffective because `init_genesis_with_settings` ignored settings and log ran before genesis (`settings=None`).
- Fix: `NodeConfig.instance: Option<u16>`; real genesis settings persist on fresh DB; existing DB treats missing metadata as v1 + warn on CLI mismatch; `Loaded storage settings` after genesis.
- Note: DBs already opened under the stub without persisted settings effectively ran **v1** — wipe for true v2, or keep syncing as v1 (expect mismatch warn with `--storage.v2`).

### Session 8 cont. — PORT-STOR-001/002 genesis Headers crash + rocksdb gap
- Bug: fresh `op-reth node` crashed `append Headers #0 but expected #1`; no `rocksdb/` despite `--storage.v2` default true.
- Cause: AccountChangeSets SF stub wrote into Headers during genesis `write_state`; RocksDB feature not enabled / does not compile.
- Fix (001): disable SF account-changesets/senders accessors; route changesets to MDBX. Genesis + `Loaded storage settings { storage_v2: true }` verified.
- Gap (002): ~~`rocksdb` feature flags sketched but enabling fails compile~~ → **closed (Session 8 YOLO):** provider rocksdb-API an v2.4.1; prune Batch-Lifetimes; `op-reth` default `rocksdb`; `cargo check -p op-reth` / `reth-provider --features rocksdb` grün. Live: nach maxperf-Rebuild muss `rocksdb/` unter Datadir erscheinen.

### Session 8 YOLO — PORT-P2P-001 + PORT-STOR-002
- P2P: op-geth EL-Bootnodes in `OPBNB_*_BOOTNODES`; OpBNB erzwingt discv4; bei RLPx `::` ohne `--discovery.v5.addr` → discv5 auf `0.0.0.0`.
- Ops-Hinweis: bevorzugt `--addr 0.0.0.0` (+ optional IPv6 discv5), `--nat extip:<public>`; nach Rebuild `net_peerCount` / eth-Session gegen Bootnodes prüfen.
- Offen danach: ~~echte AccountChangeSets/TransactionSenders-SF-Segmente~~ → PORT-STOR-004/005; ~~StorageChangeSets-SF~~ → PORT-STOR-006; EF Gas-Mismatch-Cluster; Archive-Sync; live P2P nach Rebuild; Human Catch-up/Full-Sync.

### Session 9 — Resume + YOLO Phase-5 (2026-08-10)

**Kontext:** Vorabend-Chat `9be255b9…` (PORT-STOR-006 StorageChangeSets SF) war NW-technisch unterbrochen mitten in der Schlussverifikation. Resume-Chat `6a6455c9…` schloss den Port ab und lief YOLO weiter auf nextest/EF.

**Storage / PORT-STOR-006 (Commit `d93bd7ea0`):**
- Dedicated `StaticFileSegment::StorageChangeSets` mit `.csoff` (wie AccountChangeSets); Mask `StorageChangesetMask` / `StorageBeforeTx`.
- Writer/jar/manager/either_writer Routing; `storage_changesets_in_static_files() → storage_v2`; migrate-v2 + `db state` + stage drop/config_gen verdrahtet.
- Verify: `reth-static-file-types` / `reth-provider` / `op-reth` check grün; sf-types nextest 14/14.

**Compile/Test-Fixes (Folgecommits):**
- `f6f05157e` — `RocksDBProvider::clear` auf Stub (ohne `rocksdb`-Feature); prune Full-Clear wie Upstream.
- `b4eabb869` — `StorageSettings::legacy`/`with_*`-Shims, `SealedHeader::clone_header`, `StoragePath`/`db_path`, stages `--tests` Dev-Deps.
- `c73eaa747` — echtes `header_td_by_number` aus Headers-Jar (Stub → `UnsupportedProvider` hatte `insert_block` ab Block 1 gebrochen).
- `35c2bb28a` — Sidecars-Consistency nur bei vorhandenem SF-Daten (sonst falsches `Unwind(0)`); op-node/rpc Lagoon/Sepolia-Aktivatoren + Payload-`From`.

**Nextest-Meilensteine:**
| Package | Result | Log |
| --- | --- | --- |
| `reth-optimism-primitives --lib` | 26/26 | `files/yolo-nextest-op-prim.log` |
| `reth-optimism-consensus` + `evm --lib` | 53/53 | `files/yolo-nextest-op-ce-verify.log` |
| `reth-optimism-node` + `rpc --lib` | 51 pass / 1 skip | `files/yolo-nextest-op-node-rpc-verify.log` |
| `reth-stages --features test-utils` | **106 pass / 8 skip** | `files/yolo-nextest-stages-final.log` |

**Ignored (bewusst, Trie/preimage deferred):** 6× `tests/preimage.rs` Cancun-preimage. ~~`tests/pipeline.rs::test_pipeline_v2`~~ → ✅ PORT-STOR-007.

**EF-Tests (Inventory, kein Grün-Meilenstein):**
- Fixtures: ethereum/tests **v12.2** unter `testing/ef-tests/ethereum-tests/` (gitignored).
- `cargo nextest run -p ef-tests --features ef-tests --no-fail-fast`: **62 Suites → 32 pass / 29 fail / 1 timeout**; Case-Proxy aus `Ran`-Zeilen ~**1303 pass / ~240 fail**.
- Dominantes Muster: `block gas used mismatch` (z.B. `stBadOpcode/measureGas.json`). Log: `files/yolo-ef-tests-full.log`.
- Nächster Deep-Dive: Gas-Accounting / Spec-ID / bad-opcode Pfad vs. Upstream v2.4.1.

**Aufwand Session 9 (Proxy, kein Invoice):** siehe Tabelle oben + `files/cursor-session9-metrics.json`.

### Nächste Schritte (Stand Session 9 Ende)

1. ~~EF Gas-Mismatch / `valid_blocks`~~ → ✅ (v17.0 + Bytecode Compact; 62/62).
2. ~~`test_pipeline_v2`~~ → ✅ PORT-STOR-007/008.
3. Preimage-Aux-DB für Cancun-Selfdestruct — deferred mit Trie/Proofs.
4. PORT-P2P-001 live: `net_peerCount` nach maxperf-Rebuild.
5. Human Catch-up / Full Sync; danach Effort-Zahlen + README finalisieren.
6. **FEAT-HIST-001** (nach P2P live): Reth History-Index ≥ Erigon für WalletHistory/`eth_getLogs`.

### Session 9 cont. — EF fixtures v17.0 + Bytecode Compact (2026-08-10)

**Rootcause Gas-Mismatch:** Makefile noch auf ethereum/tests **v12.2** (Upstream v2.4.1: **v17.0**). Zusätzlich `Bytecode::Compact` restore via `new_analyzed` ohne revm-41-Padding → Interpreter-UB / falsches Gas (PUSH0-Suite).

**Fixes:**
- `Makefile`: `EF_TESTS_TAG=v17.0` + EEST v4.5.0 Targets (wie Upstream).
- `Bytecode::from_compact`: Legacy-analyzed → `new_raw` Re-Analyse (wie revm serde).
- Test-Factory: RocksDB/`static_files` TempDirs via `keep()` (kein Drop unter offenen Handles).

**Verify:** Shanghai + zuvor failende Cluster (`st_bugs`, `st_code_size_limit`, `st_eip150`, `st_mem_expanding_eip150_calls`, `st_call_create_call_code`, `st_delegate_call_test_homestead`, `st_eip150_gas_prices`) → **8/8 PASS**. Logs: `files/yolo-ef-shanghai-after-bytecode-fix.log`, `files/yolo-ef-cluster-after-bytecode-fix.log`.

**Full EF suite (nach Fix):** `cargo nextest run -p ef-tests --features ef-tests --retries 0 --no-fail-fast` → **61 passed / 1 timed out** (`valid_blocks`, Default-Nextest 60s). Log: `files/yolo-ef-tests-v17-after-bytecode.log`. Nextest-Override für `valid_blocks`/`invalid_blocks` (2m×5) nachgezogen; Re-Verify: **beide PASS** (`files/yolo-ef-valid-blocks-reverify.log`, ~22s).

### Nächste Schritte (nach Bytecode-Fix / Pipeline-v2)

1. ~~`valid_blocks` mit erhöhtem Nextest-Timeout grün verifizieren.~~ ✅
2. ~~`test_pipeline_v2` State-Root unter `storage.v2`~~ → ✅ PORT-STOR-007 (+ PORT-STOR-008 history EitherWriter).
3. Preimage-Aux-DB für Cancun-Selfdestruct — deferred mit Trie/Proofs.
4. ~~**PORT-P2P-001 live:** eth-Session~~ → ✅ Session zu opBNB-Peers (2026-08-11); Peer-Count weiterhin flüchtig.
5. **PORT-PIPE live:** zuerst **001/002** nach maxperf-Rebuild; danach 003–008/012. Offen als Code-Lücke: **009** (Wright L1-Fee). 010/011 kein Extra-Port. Unused: **U01–U17**; mechanisch: **Final Cleanup Plan (CLEANUP-A…E)** nach Sync-Gates.
6. Human Catch-up / Full Sync; spätere Stages über PIPE-007…012 (Execution/Merkle/History).
7. **Danach FEAT-HIST-001:** History-/Explorer-Indizes → Erigon-Parität (Gate: stabiler Sync).
8. **Final cleanup** laut CLEANUP-* (nicht vor P0 Live-Verify).

### Session 10 — Live opBNB Archive Sync Blocker (2026-08-11)

**Setup:** `op-reth-bnb` maxperf, Datadir Archive opBNB mainnet (chain 204), CL federt Tip ~173M, Datadir genesis-only.

**Beobachtete Sequenz:**
1. Engine Fairness: ohne Downloader-vor-CL blieb Sync bei Tip-Inserts ohne Backfill.
2. Pipeline startete (`Preparing stage Headers`, `pipeline_stages=1/13`) — Stages sind **vorhanden**.
3. Headers stuck Checkpoint 0: `TimestampIsInPast` bei Blöcken mit gleicher Unix-Sekunde (live 173253771/772, `ts=1786430373`) → **PORT-CONS-001**.
4. Nach Milli-Fix-Rebuild: Tip-FCU wieder ohne Pipeline; Status nur `latest_block=0`; Grafana **No data** → **PORT-ENGINE-001** (Tip-Chase + Download-Poll).
5. Empty-response / TooManyPeers / fremde Genesis weiter Rauschen, aber nicht der Hauptblocker nachdem ein guter Peer (`a624`) Headers/Bodies liefert.
6. Nach PIPE-001 live: Headers Tip-Fetch `GetBlockHeaders(hash,limit=1)` → empty → Ban → peers=0 → **PORT-ENGINE-003** (op-geth Beacon: Tip von CL, P2P nur by number).

**Code:**
- `crates/optimism/consensus/src/validation/milli_timestamp.rs` (+ Wire in `OpBeaconConsensus`).
- `crates/engine/tree`: `handle_missing_block` Backfill-Shortcut; `engine.rs` Downloader-first; `download.rs` ohne `NewDownloadStarted`-Ready-Starvation.
- `Makefile` `maxperf-op` installiert `dist/bin/op-reth-bnb` (kein PATH-Overwrite von generischem `op-reth`).
- **PORT-ENGINE-003:** `reth-network-p2p` `HeaderSeed`/`SeededBlockClient`; Engine seedet Tip aus NewPayload/Buffer; EmptyResponse ohne Penalize; Trusted ohne BadMessage-Ban.

**Verify (Code):** `cargo check -p reth-engine-tree` ✅; Consensus-Unit-Tests für equal-second opBNB / reject OP-Mainnet; seed + trusted-reputation Unit-Tests ✅.

**Noch offen live:** Headers-ETL fertig (`Writing headers…`) → Checkpoint > 0 → Bodies/PIPE-003…; Milli-Rejects während Falling beobachten.

### Session 10 cont. — PORT-P2P-003/004/005 Reachable Headers Dataflow (2026-08-11)

**Methodik:** Cap-Idempotenz und Falling-Prime sind **keine Live-Folgebugs**. Sie sind Zustandsübergänge im
`ReverseHeadersDownloader`-Dataflow (CL eventual tip → working tip Cap → `SyncTarget::Number(N)` →
Falling-Tracker), die in der Portierungsmatrix **vor** dem Archive-Lauf als Soll hätten stehen müssen.
Live hat nur nachgezogen, was die Analyse ausgelassen hat.

**Dataflow-Soll (Matrix) → IDs:**
1. **P2P-003:** eth/68 Tip-Hash → Number-Resolve; `HeadersAtLeast`/miss-map; Empty ohne Ban; eth/69 Range-Tip.
2. **P2P-004:** Cap auf `max_peer_best`, **idempotent** vs. eventual CL-Tip (kein Re-Cap-Loop).
3. **P2P-005:** Cap-`Number(N)` primt Falling (`next_request_*`), auch wenn Tip-Outcome `old==new`.

**Live-Beleg (Analyse-Lücke sichtbar gemacht):**
1. ENGINE-003/Tip-Seed: Falling kurz ~149M — Restart ohne P2P-004.
2. ohne Tip-Resolve: `best_number=0` → Stall (P2P-003).
3. Tip-Resolve + Cap, aber Re-Loop über eventual tip → Tip/`total=1` verworfen (P2P-004).
4. Cap idempotent, aber Falling-Tracker ungesetzt → Stall nach Tip (P2P-005).
5. P2P-005 + Restart **14:40Z**: Cap 1× `working_tip≈173369140` → `total=10000` @ **~22 k hdr/s**.

**Code:**
- `StateFetcher`: `HeadersAtLeast` + `header_miss`; `tip_number=max(best,range.latest)`; `FullBlockRange` hard-filter; `on_block_range_update`.
- `NetworkState`: Status tip-number resolve; `BlockRangeUpdate` → Fetcher.
- `ReverseHeadersDownloader`: Working-Tip-Cap; Empty-Backoff (**kein** Ban); Cap **idempotent** (P2P-004); Cap-`Number` tip **primed Falling** (P2P-005).
- `HeaderSeed` / `SeededBlockClient` (ENGINE-003); Trusted: `BadMessage` nicht bannen.
- discv5: OPSTACK must-not-include + `enforce_enr_fork_id`; Bootnodes TCP `30303?discport=30304`.

**Verify (Code):** `cargo test -p reth-network --lib fetch::tests` 43/43; `cargo test -p reth-downloaders --lib headers::reverse_headers::tests` 11/11 (inkl. Cap→Falling-Regression für P2P-004/005).

**Maxperf rebuild wall (Session, `CARGO_BUILD_JOBS=1`, fat LTO):**
| Binary / Log | Wall |
| --- | --- |
| eth69-conform (`files/maxperf-op-eth69-conform-*.log`) | ~23 min |
| Cap-Loop-Fix (`files/maxperf-op-cap-loop-fix-*.log`) | ~20 min |
| Falling-Prime (`files/maxperf-op-cap-falling-fix-*.log`) | ~21 min |
| tipresolve (früher) | FAIL SIGKILL (sccache/LTO) |

**Ops-Hinweise:**
- Headers-Stage: Checkpoint/Metriken erst nach ETL→SF (`Writing headers`). INFO `total=10000` = ETL-Batch, nicht DB.
- ETL = `tempfile::TempDir` — **Upstream-by-design** ([PR #6154](https://github.com/paradigmxyz/reth/pull/6154)); Restart vor Write = Download von Tip neu. Kein Mid-Stage-Resume.

**Noch offen live:** FLOW-H05 ETL-Write → Checkpoint > 0; danach **FLOW-B01…B04 analysieren** bevor Bodies-Live; PIPE-002 Milli bis Stage-Ende.

### Methodik-Notiz Session 10 — Skill statt Vibecoding

- Operator-Befund: AI ohne zusätzlichen Portierungs-Skill / ohne Vorgehens-Hints liefert keine
  belastbare Protokoll-Portierung (siehe Abschnitt *Befund Methodik* oben).
- Daraus erstellt: `.cursor/skills/reth-opbnb-port/SKILL.md`.
- Zusätzlich: `.cursor/skills/rust-best-practices/SKILL.md` (erfahrener Rust / Best Practices).
- **Zwingend Session-Start:** Rule `reth-opbnb-port-mandatory` + Hook lädt **beide** Skills.
- README *About This Fork → Method finding* entsprechend aktualisiert.

### Session 10 — Migrationsplan-Reanalyse (2026-08-11 ~17:55 local)

**Anlass:** Cap/Falling waren Dataflow-Soll, fälschlich als Live-Folgebugs gerahmt. PIPE allein
reicht nicht.

**Änderung am Migrationsplan:**
- Neuer Abschnitt **Migrations-Gate**: DoD, Session-Checkliste, Phase-Gates, Anti-Patterns.
- Neue Matrix **`PORT-FLOW-*`** (E/H/B/R/X/S) mit Headers bereits gemappt; Bodies/Exec **🔬 vor Live**.
- **PORT-PIPE**-Tabelle um Spalte **FLOW-Gate** erweitert; Bodies/Sender/Exec als „gesperrt bis FLOW“.
- Skill + mandatory Rule + sessionStart-Hook auf **PIPE+FLOW** umgestellt.
- README Method finding nachgezogen.

**Nächstes Gate:** FLOW-H05 (`Writing headers`) → dann FLOW-B* Analyse → erst dann Bodies-Live.

### Session 11 — Dev-Miner / Genesis SF + Checkpoint (2026-08-11 evening)

**Checkpoint:** `ae662153f` — *started syncing, ok for testing* (Headers done, Bodies live; docs/rules).

**PORT-STOR-010:** `--dev` Fatal `UnexpectedStaticFileBlockNumber(TransactionSenders, 1, 0)` —
`init_genesis` tippte Senders-SF nicht unter `storage_v2`. Fix wie Upstream: `set_block_range(0,0)`.
maxperf → `Cargo/bin/op-reth-bnb` only; Smoke `files/dev-250ms` ohne Persistence-Crash.

**PORT-DEV-001 (parked):** LocalMiner `No payload` nach ~5–7 Blöcken — **keine Prio**, ggf. später fixen oder `--dev` dekommissionieren.
**PORT-DEV-002:** `payload_wait_time` verdrahtet (hilft allein nicht gegen DEV-001).

**Live Archive (parallel):** s. **Live Sync Progress** — offline FLOW-X04 Exec `20365614→21591153`; Bodies/Sender bis Fail-Block; PORT-OPS-001/ENGINE-004; ChangeSets-SF ≠ Bodies-Cap.

### Session 12 — Receipt-Root Fail / Unwind / Harness Binary (2026-08-13 → 08-15)

**Chat:** `ea987bef…` · Gates: **PORT-PIPE-014** + **PORT-FLOW-X04/X05** · **PORT-OPS-001** · **PORT-ENGINE-004**.

**Aufwand (Snapshot 08-15 ~10:54 CEST):** Kalender ~66,5 h; Interaktiv-Cluster **~4,5–6 h**; **84**/ **367** user/asst; **567** tools; Token-Proxy **~72 K** (Msg) / **~216 K** (File÷4); billed n/a → `files/cursor-session12-metrics.json`.

| Thema | Ergebnis |
| --- | --- |
| Live Fail #1 | 08-13 ~13:36 CEST @ **`21591154`** → FLOW-X05 inkl. Headers O(N) ~152 M |
| Live Fail #2 | 08-14 **10:49 CEST** — **gleicher** Block/Hashes; Exec nur Floor→Fail (~12 k Blk, ~5 min) weil Bodies/Sender schon Tip |
| Live Fail #3 | 08-14 **13:43 CEST** — MerkleExecute state-root @ **`21579110`** nach dirty `max-block` (Stages schon > H → Skip) → **unwind_to=0** |
| Bodies-Referenz | Grafana: 1. Tip-Lauf **0→173.37 M in 8.25 h @ ~5.8 k/s**; Rebuilds nach Unwind |
| Harness | `files/harness-receipt-diff-21591154/` + `re-execute --dump-receipts-on-fail`; Binary `target/maxperf/op-reth` |
| CLI ranges | **`stage run`**: inkl. `from..=to` · Exec max **`21591153`** · Bodies/Sender bis **`21591154`** · Bodies `--from` = Cap (**nicht** Cap+1). **`re-execute`**: half-open `from..to` → `--from 21591154 --to 21591155` = nur Fail-Block |
| SF vs Cap (08-15) | ChangeSets tip **`20365614`** nach three-way heal (`header_claims=365615`); Bodies Cap **`21579110`** ≠ Exec; `missing … 20365615` wenn Exec `--from` Cap |
| Debug-Flags | **`--debug.max-block <H>`** = Pipeline-Höhen-Cap (Stages mit Checkpoint **> H** werden **geskippt** — kein Re-Exec) · **`--debug.terminate`** nach Pipeline · **`--debug.skip-fcu <N>`** = nur N Engine-FCUs (**kein** Block-Stop) · **`--debug.skip-stages` existiert nicht** |
| Reload/Stop | **PORT-ENGINE-004:** Panic `SelectNextSome polled after terminated` (consensus engine) — parked |
| Headers Unwind-Log | Kein batch-`Stage unwound done=false` (nur Start + finales `done=true`); Fortschritt via `headers init-cursor` / CPU — Observability-Bug unter FLOW-X05 |
| Journal | `journalctl -D <archive-journal> |
| Upstream-Lage | Trail **2.4.1**; kein Sprung auf 2.5 bis op/bnb catch-up |

**Parked:** PORT-DEV-001 LocalMiner · PORT-ENGINE-004 Shutdown-Panic.

### Live Sync Progress — opBNB Archive (`<archive-ct>` / `op-reth-bnb`) {#live-sync-progress}

**Stichprobe:** 2026-08-15 **~10:50 CEST** · offline FLOW-X04 (kein Live-Pipeline-Exec am Fail-Block) · Headers Tip **174 027 661** · chain **204**

| Stage | Checkpoint / Target | Status |
| --- | ---: | --- |
| Headers | **174 027 661** | ✅ Tip gerettet (Kill ~17:49 während Tip→0-Unwind vor Commit) |
| Bodies | Cap **`21579110`** → offline **`21591154`** | ✅ Cap 08-14; X04 Bodies `stage run` Cap→Fail-Block 08-15 |
| SenderRecovery | Cap **`21579110`** → offline **`21591154`** | ✅ analog Bodies |
| Execution | ChangeSets SF tip war **`20365614`** → target **`21591153`** | 🔄 offline `stage run` (nicht Cap `21579110` als `--from`); Fail-Block **`21591154`** absichtlich **nicht** committen |
| MerkleExecute | **0** / n/a | ⏳ nach PIPE-014 Fix + sauberem Exec past Fail |
| History / Finish | — | ⏳ |

#### ALERT — ChangeSets SF ≠ Bodies Cap (08-15)

Account/Storage-ChangeSets-SF folgen **Execution**, nicht Bodies. Nach Unwind#3 + Cap-Rebuild lagen Bodies/Sender bei Cap, Exec/SF weit dahinter. Three-way heal am Jar `…_20000000_20499999`: `header_claims=365615`, `sidecar_has=379384` → Truncate Sidecar auf Header → Tip **`20365614`**. Heal hat **nicht** von Cap nach unten geschnitten — nur uncommitted Sidecar (~14 k). Danach: Exec `--from 20365614 --to 21591153`.

#### ALERT — PORT-EXEC-001 / PIPE-014 (Fails 08-13 + **08-14**)

```
Stage encountered a validation error: receipt root mismatch:
  got      0x61c1b64b0df2fc07a64c4d8fabde08bf8be235bdbfa6b8543c00b9683a9fbe6b
  expected 0x579924c85d951e538e7b9c5358a1acda6d1fb379af748b01274c60a283d5e50c
  stage=Execution bad_block=21591154
```

| Feld | Wert |
| --- | --- |
| Block | **`21591154`** · hash `0x33377a22…6f9a81` · 68 txs |
| Timestamp | **`1713344877`** (2024-04-17T09:07:57Z) |
| Forks | Regolith ✅ · **Snow ✅** · Canyon/Haber/Wright/Ecotone **❌** · Fermat ✅ |
| Public `receiptsRoot` | = error **expected** |
| Ruled out | PIPE-009 Wright · Canyon deposit_version · „anderer“ Bad-Block am 08-14 |
| Fixture / Harness | `files/receipts-21591154-public.json` · `files/harness-receipt-diff-21591154/` |

#### FLOW-X04 — Offline CLI (verbindliche Höhen)

| Step | Command | `--from` | `--to` | Notes |
| --- | --- | ---: | ---: | --- |
| Bodies | `stage run … bodies` | **`21579110`** (Cap) | **`21591154`** | inkl.; **nicht** Cap+1 |
| Sender | `stage run … senderrecovery` | **`21579110`** | **`21591154`** | inkl. |
| Execution | `stage run … execution` | **SF/Exec tip** (z. B. `20365614`) | **`21591153`** | inkl. Parent; **nie** `21591154` bis Fix |
| Dump | `re-execute --dump-receipts-on-fail` | **`21591154`** | **`21591155`** | half-open = nur Fail-Block; State @ `21591153` |

Details + SF-Erklärung: `files/harness-receipt-diff-21591154/README.md`.

#### FLOW-X05 — Unwind-Stürme

| # | Zeit (CEST) | Effekt |
| --- | --- | --- |
| 1 | 08-13 13:36→ | Exec/Sender/Bodies → **`21579118`**; Headers Tip→Floor (O(N) ~152 M); Tip später **~174.0 M** |
| 2 | 08-14 10:49→ | **Same** `bad_block=21591154`; Sender/Bodies erneut → Floor; **Headers Tip blieb** |
| 3 | 08-14 13:43→ | Merkle state-root @ **`21579110`** (dirty Cap) → Hashing/Exec/Sender/Bodies/Headers **unwind_to=0**; Kill ~17:49 rettet Headers-Tip-Commit |
| Cap rebuild | 08-14 ~17:53→ | Bodies **0→21579110** clean; Headers Tip vorhanden |
| X04 offline | 08-15 → | Bodies/Sender→`21591154`; Exec SF `20365614`→`21591153` |

**Ops (verbindlich bis PIPE-014 Fix):**

1. **Stabilster Park vor Bad-Block:** Process **stop** (nicht Reload), bevor Execution den Fail-Block anfasst.
2. **`--debug.max-block <H>`** nur für **Clean-Rebuild** wenn alle relevanten Stage-Checkpoints **≤ H** (sonst Skip → PORT-OPS-001). Optional `--debug.terminate`.
3. **`--debug.skip-fcu`** ist **kein** Höhen-Stop.
4. Offline Harness: Bodies/Sender→`21591154`, Exec→`21591153`, dann `re-execute --from 21591154 --to 21591155 --dump-receipts-on-fail` → `diff_receipts.py`.
5. Exec `--from` = **ChangeSets-SF tip**, nicht Bodies-Cap (sonst `missing static file data`).
6. Journal: `journalctl -D <archive-journal>
7. Headers-Unwind: Journal ohne Batch-Progress ≠ Hang — Fortschritt an `reth_static_files_jar_provider_calls_total{…init-cursor}` / CPU messen.
8. Point4/RPC: Live-Node hat **nur IPC** (`--ipcpath /tmp/<archive-ct>.ipc`); HTTP erst mit `--http`. Raw JSON-RPC über Unix-Socket.

#### Health / Anomalien (~10:50 08-15)

| Check | Befund |
| --- | --- |
| Fail-Block divergiert? | **nein** — 2× Receipt `21591154`; #3 war Merkle @ Cap-Höhe |
| Bodies Cap vs Exec SF | Cap **`21579110`** vs SF tip **`20365614`** (geheilt) — erwartet nach Unwind#3 |
| Headers Tip | ✅ **174 027 661** |
| FLOW-X04 | 🔬 Exec Catch-up →`21591153` dann Dump |
| Reload/Stop Panic | 🧊 ENGINE-004 parked |

#### FLOW-X01 / PIPE-007 — Fermat (hist. OK)

Siehe `files/fermat-point4-20260812.txt`. **Haber** noch nicht erreicht.

#### Bereits durchlaufen

| Phase | Start → Ende (CEST) | Elapsed | Ergebnis |
| --- | --- | --- | --- |
| Bodies (1. Lauf) | 08-11 ~18:58 → 08-12 ~03:02 | **~8.25 h** | Tip @ ~5.8 k blk/s |
| SenderRecovery | 08-12 → ~15:54 | ~12.9 h | Tip |
| Execution #1 | 08-12 ~15:54 → 08-13 ~13:36 | ~22 h | FAIL @ **21591154** |
| Unwind #1 + Headers Tip | 08-13 | — | Floor **`21579118`**; Tip **~174 M** |
| Bodies (2. Lauf) | 08-13→08-14 | ~halb–1 d | Tip wieder; dann Exec #2 |
| Execution #2 | 08-14 10:44→10:49 | **~5 min** | FAIL @ **21591154** (Floor→+12 k) |
| Dirty Cap Merkle | 08-14 13:29→13:43 | ~14 min | FAIL state-root @ **21579110** → unwind_to=0 |
| Headers Tip→0 (abgebrochen) | 08-14 17:30→17:49 | ~19 min | Kill; Tip-Commit verhindert |
| Bodies Cap clean | 08-14 17:53→19:49 | ~2 h | **0→21579110** ✅ |
| Sender Cap | 08-14 →20:48 | ~1 h | **21579110** ✅ |
| Execution Cap | 08-14 ~20:48→ | — | gestoppt / SF tip später **`20365614`** |
| X04 Bodies/Sender | 08-15 | — | Cap→**`21591154`** ✅ |
| X04 Execution | 08-15 ~08:32→ | 🔄 | **`20365614`→`21591153`** |

#### Network usage (CT `<archive-ct>:9100`, `node_network_*`)

| Phase | RX (typ.) | TX (typ.) | Notes |
| --- | --- | --- | --- |
| Headers Falling (08-11) | ~**25–60 Mbit/s** | — | P2P Header-Batches |
| Bodies | ~**140–200 Mbit/s** | niedrig | Peak ~200 Mbit/s |
| SenderRecovery | ~**0.5–0.8 Mbit/s** | ~**0.7–0.9 Mbit/s** | CPU-lokal |
| Execution (bis Fail) | ~**0.6 Mbit/s** | ~**0.7 Mbit/s** | peers~12–13 |

Kein `reth_network_*_bytes` auf `:6060` — Bandbreite über CT-Exporter `:9100`.

