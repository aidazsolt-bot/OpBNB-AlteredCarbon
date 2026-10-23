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

## Aktueller Stand (Session-Memory — 2026-10-21 ~12:50 CEST) {#session-memory}

> **Handoff.** Skills: `reth-opbnb-port` + `rust-best-practices` + `network-linux-sysadmin`.
> **Evidence-only.** `scripts/sync-eta.sh` + Logs + Mimir + op-node Journal.
> **Offene Gates:** `PORT-FLOW-E03` Live-DoD Tip-Follow (Gap≪32, Finish≈public) · `FLOW-H07` Live-DoD · `PORT-PIPE-009`.

| Thema | Lage |
| --- | --- |
| **Binary** | live `reth/v2.4.1-84a5caa` maxperf · FLOW-E03 Label-Advance **deployed** (Working-Tree-Build → Commit folgt) |
| **Pipeline** | Headers/Bodies/Sender **@197 844 013** · **Execution aktiv** (von 197 256 674 → 197 844 013, ~587 k) · Finish noch @197 256 674 |
| **Tip-Follow** | CL `newPayload` wieder fließt (~197 850 k) · FCU=`SYNCING` · Public **~197 851 k** · Gap Finish→Horizon **~587 k** |
| **CL (op-node)** | Engine-Stream **wieder aktiv** nach EL-Restart mit Label-Fix |
| **Labels** | `latest=safe=finalized=197256674` (nicht mehr Wright) — Label-DoD ✅; steigen mit Finish |
| **FLOW-H06** | ✅ |
| **FLOW-H07** | 📋 Code+Deploy · Gap-Fill **live beobachtet** (Headers 197.26M→197.84M nach NewPayload-Seed) |
| **Health** | peers **11** · body/invalid-msg errors **0** · Point4 (1k/Fermat/1e8/tip−1k) **MATCH** |

### Tip-Quelle (CL vs Pipeline)

| Gap | Wer liefert Header/Blöcke |
| --- | --- |
| **Tip-Follow** (nahe Head) | **op-node → Engine API** (`newPayload` + FCU). Tip-Hash/Seed = CL (FLOW-E03), nicht allein P2P `GetBlockHeaders(hash)`. |
| **Gap ≫ Buffer (64)** | Engine → `SYNCING`, merkt CL-Tip; **Pipeline-Backfill** füllt die Zwischenkette (Headers-Stage; Ziel aus HeaderSeed/CL). |

### Stall-Analyse (2026-10-19, Evidence — kein Headers/H07)

1. **CT-Reboot ~20:46:38 CEST:** `OPNodeBinance` startet **vor** Auth (`127.0.0.1:8551` erst ~20:46:48); Unit nur `After=network-online`, JWT via BlockChain `ExecStartPost`.
2. Boot-Backfill: `initial_target` = lokaler Tip-Hash @197 256 674 → **No-Op**.
3. Danach EL: **0×** `Received new payload`. op-node: Dauer-`deadline exceeded` am Engine-Driver (PID 171, nach Restart 533/617 wieder).
4. Restart op-node @21:37: `Optimistically inserting` Tip → sofort **`Skipping EL sync … finalized :32984676`** (`engine_controller.go`) → CL-Sync **`Walking back L1Block`** vom Tip rückwärts.
5. Semantik: Archive-Blöcke bis Finish **vorhanden**; hängende Labels `safe`/`finalized`@Wright + übersprungener EL-Sync → kein Tip-Stream → Pipeline idle. **Nicht** FLOW-H07 (P2P Headers noch gar nicht im Spiel).

### Upstream-Vergleich (2026-10-19, `bnb-chain/reth-bsc-trail`) — neue Notiz

**Anlass:** Verdacht „beim Port wegrationalisiert / nicht korrekt verdrahtet“. Ziel: rekonstruieren,
wie der initiale Upstream-Live-Flow gedacht war, und wo im Port vermutlich Entkopplungen entstanden.

#### Befund A — Upstream-Design ist klar Engine-API-zentriert (CL→EL)

- `bin/reth-bench/README.md`: Live-Sync wird explizit über alternierende `engine_newPayload` +
  `engine_forkchoiceUpdated` modelliert (nicht über P2P-only Tip-Chase).
- `bin/reth-bench/src/valid_payload.rs`: Helper loopen bis `VALID` (bei `SYNCING` retry), d. h.
  der Datenpfad ist auf kontinuierliche CL→EL-Impulse ausgelegt.
- `crates/consensus/beacon/src/engine/message.rs`: `BeaconEngineMessage::{NewPayload, ForkchoiceUpdated}`
  ist Kernobjekt des internen Engine-Streams.
- `crates/rpc/rpc-engine-api/src/metrics.rs`: getrennte Counter für FCU/newPayload inkl.
  `*_syncing` bestätigen das beabsichtigte Betriebsmodell (SYNCING als Übergangszustand, nicht Dauerzustand).
- `book/run/mainnet.md`: ohne CL keine echte Tip-Fortschreibung; EL erwartet korrektes JWT/AuthRPC-Handshake.

**Interpretation:** Der initiale Code funktioniert nur stabil, wenn die CL→EL-Message-Kette
zuverlässig startet und dauerhaft läuft.

#### Befund B — unser Live-Stall entspricht exakt einer unterbrochenen Engine-Kette

- Symptom lokal: `newPayload=0`, FCU/newPayload antworten praktisch nur `SYNCING`, canonical bleibt auf Finish.
- Das passt zu entkoppelter oder zu früh gestarteter CL→EL-Verbindung (Race bei Auth/JWT/8551)
  **mehr** als zu fehlenden Header-/Body-/Archive-Daten.

#### Befund C — konkrete Verdachtsstellen (Port-/Wiring-Ebene)

1. **Startup-Ordering außerhalb Rust (systemd/ops):**
   - `op-node` startet vor stabilem EL-AuthRPC/JWT → erste Engine-Driver-Requests laufen in Deadlines.
   - Danach CL-Controller-Pfad mit vorhandener Finalized-Markierung (`Skipping EL sync`) verhindert
     erneutes aggressives EL-Kickoff.
2. **Launcher/Wiring-Pfad im Node-Boot:**
   - Diff-Kandidaten: `crates/node/builder/src/launch/engine.rs`, `.../launch/mod.rs`.
   - Risiko: Hook-/Stream-Reihenfolge oder Kanalverdrahtung (EngineRx/Message-Stream) wurde beim Port
     vereinfacht/umgestellt.
3. **Engine-Seed/Backfill-Übergang nach Finish:**
   - Wenn CL-Tip nicht sauber als HeaderSeed/Engine-Target re-injiziert wird, bleibt Pipeline nach
     Horizon-Completion idle (auch bei gesunden P2P-Peers).

#### Arbeits-Hypothesen (priorisiert)

- **H1 (hoch):** Primärer Triggerverlust durch Boot-Race Auth/JWT/8551 + op-node Controller-Verhalten bei
  vorhandener Finalized-Linie.
- **H2 (mittel):** Zusätzlich ein Port-Diff in Launch/Engine-Wiring, das Re-Arm nach transientem CL-Fehler
  nicht robust genug macht.
- **H3 (niedriger):** Historische Pipeline-/P2P-Flow-Lücken sind aktuell nicht primär; sie werden erst
  relevant, wenn CL→EL-Stream wieder läuft.

#### Nächste technische Verifikation (upstream vs current)

1. **Line-by-line Diff** der folgenden Pfade zwischen
   `bnb-chain/reth-bsc-trail` und `aidazsolt-bot/OpBNB-AlteredCarbon`:
   - `crates/node/builder/src/launch/engine.rs`
   - `crates/node/builder/src/launch/mod.rs`
   - `crates/consensus/beacon/src/engine/message.rs`
   - `crates/rpc/rpc-engine-api/src/metrics.rs`
   - `crates/engine/util/src/engine_store.rs`
2. **Ops-Fix vor Code-Fix:** systemd Reihenfolge/JWT-Readiness hart machen (`After`/`Requires`,
   preflight auf `8551` + JWT file existence).
3. **Success-Kriterium:** im EL-Log wiederkehrend `Received new payload` + FCU nicht permanent `SYNCING`,
   `safe/finalized` laufen von Wright weg Richtung Tip.

> Hinweis: Code-Search-Ergebnisse können unvollständig sein (Tool-Limit). Für vollständige Sicht
> zusätzlich GitHub-UI-Search nutzen.

### Live Sync / ETA (2026-10-21 ~12:50 CEST / 10:50Z)

| | |
| --- | --- |
| Catch-up Horizon | **197 844 013** (Headers/Bodies/Sender fertig) |
| Canonical / RPC tip | **197 256 674** (Finish/Execution noch hinterher) |
| `safe` / `finalized` | **197 256 674** (= local tip; Wright-Lag behoben) |
| Public tip | **~197 851 k** · Δ Horizon **~7 k** · Δ Finish **~595 k** |
| Active | **Execution** (Rate in `sync-eta` noch 0 — Stage gerade gestartet nach Bodies) |
| **ETA Tip** | offen — Execution→Merkle→…→Finish; Tip-`newPayload` parallel |
| Point1 | errors **0** · Point4 **MATCH** |

### Deploy / FLOW-E03 (2026-10-21)

1. **Code:** `advance_op_safe_finalized_if_stale` (Engine-Tree spawn / missing-block / post-backfill) + Persist `SaveFinalized`/`SaveSafe` immediate commit.
2. **op-node:** kein Patch — Upstream `return nil` bei `Skipping EL sync` intentional (OP #8968); lokaler Fall-through **reverted**.
3. **Deploy:** maxperf `op-reth-bnb` gebaut + Archive-Restart ~12:03 CEST → Labels Wright→Finish; danach CL Tip-Stream + Pipeline-Backfill Headers/Bodies bis Horizon.
4. **Rest-DoD:** Finish/Execution bis Horizon; Gap public ≪32; dann H07 Live-DoD abschließen.

### FLOW-E03 Backfill-Semantik-Fix (2026-10-22)

**Regression:** OP `backfill_target_hash → head` + post-Finish Re-Arm trotz `finalized=0` → Pipeline nie Idle → Buffer=64 / `newPayload_valid=0`.

**Fix (an `bnb-chain/reth-bsc-trail`):** `backfill_target_hash` wieder **finalized**; bei `finalized=0` nach Finish **Idle** + `try_connect` (kein sofortiger Head-Re-Arm). `advance_op_safe_*` bleibt. FCU Tip-buffered Start mit Optimistic-Head-Fallback bleibt für Catch-up. Deploy/Live-Verify ausstehend.

### Archive-Verify vs Public Archive (2026-10-19 ~16:15 CEST)

**Referenz:** local IPC `op-reth` vs **NodeReal** `opbnb-mainnet.nodereal.io` (historische State OK).  
**Nicht-Archive:** `opbnb-mainnet-rpc.bnbchain.org` → `missing trie node` @Fermat (nur Full).

#### A) Header `hash` / `stateRoot` / `transactionsRoot` / `receiptsRoot`

| Höhe | Gate | Ergebnis |
| --- | --- | --- |
| 1, 1k, 100k, 1M | früh | **MATCH** |
| **9 397 477** | Fermat / PIPE-007 | **MATCH** |
| **21 591 154** | Fail / PIPE-014 | **MATCH** |
| 25M, 50M, 100M, 150M | mid | **MATCH** |
| **32 984 677** | Wright≈ | **MATCH** |
| **34 367 717** | Wright Receipt / X02 | **MATCH** |
| 197 111 706, 197 200 000 | Finish / catch-up | **MATCH** |

→ **14/14** — ältere Exec-/Consensus-Fixes (Fermat/Fail/Wright) auf Header-Ebene bestätigt.

#### B) `eth_getBalance` (8 System-/Vault-Adressen × 8 Höhen)

- **60/64 MATCH**, inkl. Fermat, Fail, Wright-Gates, 50M–150M.
- **4 MISMATCH nur @197 111 706** (canonical/`latest` während Catch-up): local `latest` == public Balance **@Execution-Tip 197 206 567**, nicht post-state @Finish. **@197 111 705 und tiefer MATCH**.
- Ursache: während Pipeline `eth_blockNumber`=Finish, Plain-State/`latest` schon bei Exec-Head — **kein** historischer Root-Fehler (Header-`stateRoot` @Finish MATCH). Nach Finish-Catch-up erneut prüfen.
- Höhen **>Finish** (noch kein Index/Finish): local `no state found for block` (erwartet bis History/Finish durch sind).

#### C) Code / Storage / Logs / Tx

| Check | Ergebnis |
| --- | --- |
| `eth_getCode` + `eth_getStorageAt(0)` @Fermat/Wright/Finish (GPO, WETH) | **MATCH** |
| `eth_getLogs` Bridge @Fermat/Wright (gleiche Tx/Topics/Data) | **MATCH** (Reth liefert extra `blockTimestamp`; NodeReal nicht) |
| `eth_getTransactionByHash` / `Receipt` @Fermat | **MATCH** |
| `eth_getBlockReceipts` / `ots_getApiLevel` | local **OK** |
| `eth_getProof` weit zurück | local: **max proof window**; NodeReal @Finish: missing trie (deren Limit) |

#### D) Reth-Archive vs op-geth-Archive (funktional)

**Legende:** ✅ = funktioniert wie erwartet · ❌ = fehlt/schlägt fehl · ⚠️ = **kein Datenfehler**, aber Limit oder Sync-Semantik (Details unten).

| Fähigkeit | local reth | Public archive (NodeReal≈op-geth) | Full bnbchain.org |
| --- | --- | --- | --- |
| Historische Block-Header/Roots | ✅ | ✅ | ✅ Header |
| Historische Balances/Code/Storage **< tip** | ✅ | ✅ | ❌ `missing trie node` |
| `eth_getLogs` historisch | ✅ | ✅ | — |
| `latest` während staged catch-up | ⚠️¹ | Tip=Chainhead | Tip |
| `eth_getProof` deep history | ⚠️² window | ⚠️³ teils | ❌ |

1. **⚠️¹ `latest`:** Während Pipeline-Catch-up meldet `eth_blockNumber` noch **Finish**, Plain-State/`eth_getBalance(latest)` aber schon **Execution-Head**. Historische Tags **< Finish** bleiben korrekt.
2. **⚠️² `eth_getProof`:** Reth begrenzt den Abstand Tip→Zielblock (`distance to target block exceeds maximum proof window`) — absichtliches Limit, kein fehlender Archive-State.
3. **⚠️³ NodeReal proof:** kann an manchen Höhen `missing trie node` liefern (Provider-Limit), während `getBalance` dort noch geht.

#### E) Mikro-Benchmark (2026-10-19 ~16:23 CEST, n=5, 1 Warmup, ms End-to-End)

**local reth IPC** vs **NodeReal HTTPS Archive** vs **bnbchain.org Full**.  
IPC ohne Netz-RTT → misst vor allem Node-Arbeit; Public inkl. RTT+Proxy (~0.5–1.5 s).

| Workload | local p50 | NodeReal p50 | Full p50 / Status |
| --- | --- | --- | --- |
| `getBalance` @Fermat | **0.2** | 631 | ERR missing trie (~597) |
| `getBalance` @Wright | **0.3** | 566 | ERR missing trie |
| `getBalance` @100M | **0.2** | 1506 | ERR missing trie |
| `getBalance` @tip−1 | **0.3** | 483 | ERR missing trie |
| `getCode` @Fermat | **0.2** | 472 | ERR missing trie |
| `storageAt(0)` @Fermat | **0.2** | 533 | ERR missing trie |
| `getBlock` @Fermat | **0.1** | 541 | **OK** 963 (nur Header/Full) |
| `getLogs` Bridge Fermat±50 | **0.3** | 896 | OK 644 (Logs oft auch Full) |
| `getProof` @tip−1 / Fermat | ⚠️ window (~0.2) | ERR missing trie | ERR missing trie |

**p50 über alle OK-Calls:** local **0.2 ms** (n=9) · NodeReal **~540 ms** median · Full nur 3 OK (Block/Logs), State historisch **nein**.

**Urteil:** Historische Ausführung (alte Bugs) **verifiziert** über Roots+State unter Tip. Archive-RPC **parity-tauglich** vs op-geth-Archive; ⚠️ = Semantik/Limits, nicht falsche Roots.

### op-node Dashboard-Audit (`BSCRethArchiveNode:6062`, now-15m) — Stand 15:45 CEST

Dashboard: [op-node / opBNB](https://gra.zsolttech.com/d/opbnb-op-node-dash/op-node-opbnb-bnb-chain?orgId=1&from=now-15m&to=now&var-instance=BSCRethArchiveNode:6062) · Metriken via Mimir `instance=BSCRethArchiveNode:7300`.

**Sollbild (sauberer Verifier am Tip):** `up=1` · L1-refs laufen mit L1 · L2-refs (`l2_unsafe`/`safe`/`finalized`) laufen mit L2 · Engine `newPayload`+`FCU` ~Blockzeit · Antworten **VALID** (syncing nur kurz) · EL canonical ≈ public tip.

| Check | Ist (15 m) | Soll | Urteil |
| --- | --- | --- | --- |
| `op_node_default_up` | 1 | 1 | ✅ |
| L1 `refs_number` (head/safe/finalized) | ~128 564 k, Δhead **+1996**/15 m | L1 läuft | ✅ |
| L1 `refs_latency` | ≈ −0.6…−1.9 s | \|lat\| klein | ✅ (leichte Clock-Skew) |
| L2 `refs_number` (`l2_*`) | **fehlen komplett** | vorhanden + steigend | ⚠️ / Folge von EL-Stall |
| Engine RPC → EL | FCU+newPayload **~4/s**, `error=<nil>`, avg **0.3–0.6 ms** | ähnlich, Fehler 0 | ✅ Client-Seite |
| EL `forkchoice_updated_valid` | **0** | ≫0 am Tip | ❌ |
| EL `forkchoice_updated_syncing` | **= messages (122 145)** | selten/kurz | ❌ Dauer-SYNCING |
| EL `new_payload_syncing` | **= messages** | selten | ❌ |
| EL `block_buffer_blocks` | **64** | klein / drainend | ❌ Buffer voll |
| EL canonical / Finish | **197 111 706** flat | ≈ public tip | ❌ Stall |
| `derivation_errors` / `pipeline_resets` / channel_timeout | 0 | 0 | ✅ |
| `unsafe_payloads_total` / frame/channel counters | 0 (idle) | oft >0 bei Batch-Derive | ℹ️ Tip läuft hier über **P2P gossip→Engine** (DELIVER_MESSAGE≈newPayload-Rate) |
| Sequencer_* / `transactions_sequenced` | 0 | 0 (Verifier) | ✅ |
| P2P `peer_count` | **17** | >0 | ✅ |
| Gossip 15 m | DELIVER≈3589, REJECT≈13, DUPLICATE≈3252 | Peers+Traffic | ✅ (REJECT vernachlässigbar) |
| L1 URL switch / storage/publishing errors | 0 | 0 | ✅ |

**Urteil:** op-node ist **kein** Fault — L1, P2P und Engine-Client sind clean. Unhealthy Tip-Follow = **EL nach Finish** (FLOW-H06): jede FCU/`newPayload` → `SYNCING`, Tree-Buffer **64**, Canonical bleibt auf dem lokalen Finish.

### Datadir-Analyse (Archive vollständig?)

| Segment | Größe | Block-Range / Jars |
| --- | --- | --- |
| static_files total | ~2.9 TiB | — |
| storage-change-sets | ~1041 GiB | **0..197.5 M**, **395/395** jars |
| transactions | ~702 GiB | 0..197.5 M, 395 jars |
| receipts | ~511 GiB | 0..197.5 M, 395 jars |
| account-change-sets | ~172 GiB | 0..197.5 M, 395 jars |
| transaction-senders | ~99 GiB | 0..197.5 M, 395 jars |
| headers | ~80 GiB | 0..197.5 M, 395 jars |
| db (MDBX) | ~476 GiB | State/Trie/Hashed… |
| rocksdb | ~385 GiB | History-Indices |
| **Summe** | **~3.7 TiB** | prune.segments **leer** = Full-Archive |

**Urteil:** 3.1–3.7 TiB ist für opBNB-Archive @~197 M **plausibel und vollständig** (alle SF-Segmente lückenlos). opBNB ist L2 (kleinere Bodies als L1/BSC); kein Hinweis auf fehlende History.

### Bugquellen — Status

| ID | Status |
| --- | --- |
| **STOR-TRIE-001** | ✅ live |
| **FLOW-H06** | ✅ live — HeaderSeed Tip-Bind; Catch-up-Wellen bis Finish **197 256 674** |
| **FLOW-H07** | 🔧 Code 2026-10-21: Tip-Churn während Cap wischte `sync_target_request` → Headers `target=None` Stall; Fix + Unit. Deploy/Live offen |
| **FLOW-E03 / ENGINE-EL** | ✅ Label-Advance bleibt · 🔧 2026-10-22: OP Head-Chase Re-Arm reverted (upstream Finalized/Idle bei `finalized=0`) — Live-DoD Tip-Follow offen |
| **METRICS-RDB-001** | ✅ live — RocksDB Size **~384 GiB** in Mimir |

### Nächste Schritte

1. Execution→Finish bis Horizon **197 844 013+** beobachten (`sync-eta`); Point4 nahe neuem Finish.
2. Tip-DoD: Finish≈public, Gap≪32, FCU nicht dauerhaft `SYNCING`.
3. Ops härten (optional): `OPNodeBinance` `After=BlockChain` + wait `8551`/JWT; `--syncmode=execution-layer --el-trigger.gap 120`.
4. **FLOW-H07** Live-DoD formal schließen · `eth_getBalance` @Finish vs NodeReal.

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
