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
| PORT-FLOW-E03 | Tip-Header-Quelle | Tip-Hash aus **CL/NewPayload/HeaderSeed**, nicht allein P2P `GetBlockHeaders(hash,1)`; Empty ≠ Ban (esp. trusted) | ENGINE-003 | ✅ Code + live · Headers Tip |
| PORT-FLOW-H01 | eth/68 Status | Tip oft nur **Hash** → Number-Resolve bevor `HeadersAtLeast(CL)` | P2P-003 | ✅ live |
| PORT-FLOW-H02 | Peer-Auswahl / Empty | `HeadersAtLeast` + miss-map; Empty → backoff **ohne** Peer-Drop/Ban; eth/69 `max(best,range.latest)` + Range-Filter | P2P-003 | ✅ live |
| PORT-FLOW-H03 | Working-Tip-Cap | `eventual_CL` ≠ `working=max_peer_best`; Cap **idempotent** (kein Re-Cap-Loop) | P2P-004 | ✅ live |
| PORT-FLOW-H04 | Cap → Falling | `SyncTarget::Number(N)` / Tip-Outcome `old==new` **primt** Falling-Tracker (`next_request_*`) | P2P-005 | ✅ live |
| PORT-FLOW-H05 | Headers Persistenz | ETL=`TempDir` → Checkpoint/Metriken erst nach `Writing headers`; Restart vor Write = Download von Tip neu | Upstream #6154 | ✅ **live** (2026-08-11T16:35–~16:47Z): Write `173369140` → Headers checkpoint=tip; Bodies gestartet |
| PORT-FLOW-N01 | Bind / Dial / Announce | **Kein `--addr`:** OS Dual-Stack; Dial-Preference OS (modern AAAA→A); Announce = preferierte **dialbare** Adresse ≡ Listen. **`--addr <ip>`:** nur diese Familie; andere nicht initialisieren. **Live-Repro 08-15 (anonymisiert):** Bind-Default `0.0.0.0:30303` (v4-only); NAT/`admin_nodeInfo` announcte zeitweise **`<host-global-ipv6>`** der Node-NIC (SLAAC/EUI-64, gleiche MAC wie privates v4 — **nicht** Router); Ping auf announced v6 OK, **TCP:30303 refused** (kein v6-Listener); effektive Sessions über `<host-lan-ip>`; inbound=0; Announce kann später auf Public-v4 aus HTTP-NAT umspringen. | P2P-006 | ✅ **09-03**: `NetworkArgs::addr` jetzt `Option<IpAddr>` (`wants_os_dual_stack()`); ohne `--addr` bindet discv5 nachweislich `mode=DualStack` auf beiden UDP-Sockets (`0.0.0.0:9200` + `[::]:9200`, Dev-Host-Isolationstest), NAT announct beide Familien separat (`Announced dialable enode` für IPv6 + `Announced additional discv5 dual-stack NAT endpoint` für IPv4). `--addr` weiterhin single-family (RLPx-TCP/discv4 bleiben single-stack by design). **Folgebug 09-03 (Session 16, gefixt+live bestätigt):** UPnP-Family-Handling im Dual-Stack-Pfad verwarf gültige IPv4-Mapping, erzwang flakige zweite SSDP-Suche → jetzt UPnP nur noch für IPv4-Ziele versucht. |
| PORT-FLOW-N02 | NAT / UPnP → Announce | Default **`--nat any`:** UPnP geth-style (kein Hijack) → ENR/enode/Logs konsistent; Discv4 **und** Discv5 ENR (TCP=mapped RLPx; UDP=v5-Port, ggf. extra UPnP). Kein IGD → HTTP+Listen-Ports, **Familie ≡ `--addr`**. | P2P-002 | ✅ **live** 08-15 ~21:37: Alt-Ports (30303 bei Erigon), `via_upnp=true`, hairpin OK, inbound_conn≥2; `eth_*_requests_received` noch 0 |
| PORT-FLOW-B01 | Bodies Peer/Range | Body-Requests nur an Peers mit Range/Fähigkeit; eth/69 hard-filter analog Headers | PIPE-005 | ✅ Bodies tip (08-12 ~03:02 CEST); FLOW-B beobachtet OK |
| PORT-FLOW-B02 | Bodies Empty/Timeout | Empty/Timeout-Politik: kein Ban-Sturm; Retry/Backoff explizit | PIPE-005 | ✅ Bodies durch · Empty/Ban kein Stall |
| PORT-FLOW-B03 | Bodies Buffer→Stage | In-flight / buffered / flush → Checkpoint; Stall-Zustände benennen | PIPE-005 | ✅ Bodies Checkpoint=Tip |
| PORT-FLOW-B04 | Bodies↔Headers Kopplung | Bodies startet erst nach Headers-Checkpoint; kein stilles Warten ohne Metrik | PIPE-005 | ✅ Headers→Bodies ~18:58 CEST (08-11) |
| PORT-FLOW-R01 | Deposit Sender | Deposit `from` ohne ECDSA (Feld im Deposit-TX, kein `ecrecover`); Fehlerpfad ≠ Peer-Ban | PIPE-006 | ✅ **live OK** Tip-Lauf; Catch-up 08-15: Sender wartet auf Bodies-Yield (@ Fail-Höhe) |
| PORT-FLOW-X01 | Historische Overlays | Precompiles/Flags am **Blockzeitpunkt** (Fermat/Haber-Fenster), nicht nur Tip-Fork | PIPE-007/008 | ✅ **Fermat live** · PIPE-014/X04 Hertz ✅ · ⏳ Haber live ab `1718872200` |
| PORT-FLOW-X02 | Wright L1-Fee | op-geth: L1-Fee-Skip nur `gasPrice==0`; Reth setzt `skip_l1_data_fee=true` ab Wright. Das vendorte Workspace-`crates/optimism/op-revm` gated die L1-Datenkosten ebenfalls mit `gas_price==0` und erhält das Flag beim L1-Info-Reload; Isthmus-Operatorgebühren bleiben erhalten. | PIPE-009 | ✅ Code + Unit `wright_gasless_transactions_skip_only_l1_data_fee` und Reload-Test · 🔬 portabler Gesamtbuild / Live-Stichprobe ab ~**32984677** |
| PORT-FLOW-X03 | Exec Persistenz | Commit/Unwind-Pfad storage.v2 (SF changesets, hashed readers) konsistent mit PIPE-012 | STOR-007/008 | 📋 Code · 🔬 Archive-Last |
| PORT-FLOW-X04 | Einzelblock Receipt-Diff | Bei Receipt-/State-Root-Mismatch: Single-block exec → Dump `(idx,status,gasUsed,cumGas,logs)` → Diff vs public `eth_getBlockReceipts` → **erster** divergenter Index vor Fix | PIPE-014 | ✅ **closed** · idx=10 `syncLightBlock`/`0x67` · Hertz-Overlay · `re-execute 54..55` ✅ (08-15 ~14:13 CEST, kein Dump) |
| PORT-FLOW-X05 | Pipeline Unwind-Sturm | Exec-/Merkle-Validation-Fail darf **nicht** stillschweigend ~10⁸ Headers via O(N) `HeaderNumbers`-Loop vernichten; Status `checkpoint=tip` bis `UnwindOutput` ≠ Idle; Headers loggt **kein** batch-`Stage unwound done=false` (Observability-Inkonsistenz vs Sender/Hashing) | PIPE-014, EXEC-001 | 🐛 **3×** live (2× Receipt @`21591154` + **08-14 ~13:43** Merkle @`21579110`→unwind_to=0); Tip gerettet per Kill vor Headers-Commit; **Ops:** Process-Stop ≫ `max-block` als Park |
| PORT-FLOW-S01 | SF Segment-Routing | Jedes Segment eigene Datei/Mask; kein Headers-Reuse (STOR-001-Klasse) | STOR-004…006 | ✅ |
| PORT-FLOW-S02 | Prune/History v2 | EitherWriter/RocksDB unwind verdrahtet; tote Helper ≠ stiller No-Op ohne FLOW-Notiz | STOR-008, PIPE-U10/11 | 📋 |
| PORT-FLOW-S03 | Metrics/Healing | Alle `StaticFileSegment`s in Metrics registriert (STOR-009-Klasse) | STOR-009 | ✅ |
| PORT-FLOW-S04 | V1→V2 layout migration / repair backfill | `storage_v2` darf erst nach vollständiger Datenmigration gesetzt werden; ein Teilabbruch muss aus MDBX in die bestehenden Zielsegmente fortsetzbar sein. Prune-Checkpoints repräsentieren absichtlich entfernte Static-File-Abdeckung, nicht Korruption. Bei einem unterbrochenen Reparaturlauf startet `initial_backfill_target` absichtlich am letzten konsistenten Header; eintreffende FCUs werden als neuestes Sync-Ziel gehalten und dürfen nach Pipeline-Idle nicht verloren gehen. | STOR-007/008, ENGINE-001 | 📋 Code: direkte Metadatenmutation gesperrt; ChangeSet-Resume + Prune-Coverage portiert. Live 09-02: lokaler Repair-Target `71185159` (`0x005603…`) korrekt geschlossen, CL-Head ~`180.94M` als `SYNCING` registriert; externer Backfill nach Merkle/Pipeline-Idle offen. ✅ 09-03 Dev-Host: sauberer (nicht unterbrochener) `db migrate-v2` End-zu-Ende-Test (V1-Sync 0→300 via `--debug.tip`/`--debug.terminate`, dann Migration, dann Rebuild-Neustart) reproduzierte **keinen** Fehler; alle 13 Stage-Checkpoints konsistent @300 nach Rebuild. Crash-Resume-Semantik (Abbruch *während* `migrate-v2`) bleibt weiterhin ungetestet. |

**Regel für neue FLOW-Zeilen:** sobald ein Stall/Ban/„total=1“/Grafana-No-data auftritt und **kein** FLOW die Transition beschreibt → zuerst Zeile anlegen, dann fixen. Nicht unter PIPE oder Chat begraben.

### Anti-Patterns (erweitert — Session 10)

- Live-Sync debuggen ohne **PIPE und FLOW** für die Stage
- Nur Konsensregel portieren, Downloader-/Engine-Automat ignorieren
- Cap/Seed/Tracker-Logik als „Folgebug“ nach dem Restart entdecken
- Checkpoint 0 als „Headers broken“ lesen, ohne FLOW-H05 (ETL-TempDir)
- eth/68 Tip-Hash mit `best_number` verwechseln
- Ban auf Empty Headers/Bodies
- „Workspace kompiliert“ / nextest grün als Protokoll-Done

### Session 13 — Storage-v2 recovery (2026-09-02)

**Korrektur (15:02 CEST):** Der Archive-Datadir lief mindestens seit 2026-08-14 durchgehend mit
`storage_v2=true`; es gab weder einen manuellen `storage_v2`-Metadatenwechsel noch einen
`migrate-v2`-Lauf. Die frühere Layoutwechsel-Hypothese ist damit verworfen. Der
`ExecutionStage::ensure_consistency`-Befund bleibt real: Receipt-Static-File-Daten fehlen ab
`71185160` trotz V2-Betrieb und lösten den abhängigen Unwind aus:
Execution `71242925→71185159`, danach SenderRecovery/Bodies/Headers
`174027661→71185159`. Der anschließende vollständige Merkle-Rebuild bestätigte eine zweite
V2-Integritätsverletzung: für `71185159` berechnete er `0x0edebae1…`, während der lokale
kanonische Header und der öffentliche opBNB-RPC `0x68efa1b2…` liefern. Das beobachtete
Header-Verhalten war Folge dieser Unwinds, nicht ein Header-Downloader-Validierungsfehler.

**Umgesetzt:** `db settings set storage_v2` verweigert Layoutwechsel; nur gleiche Werte bleiben
No-op. `db migrate-v2` setzt vorhandene Account-/StorageChangeSet-Static-File-Segmente fort.
Der Upstream-Prune-Coverage-Fix ist für Receipts, Senders und beide ChangeSet-Segmente portiert.

**Header-/Engine-Analyse (14:13–14:23 CEST):** Der Logeintrag `Headers target=None` bedeutet
nicht, dass die Header-Stage keinen Tip kennt: Pipeline-Events zeigen dort nur keinen numerischen
Zielblock. Wegen `MerkleExecute=0` erkennt `check_pipeline_consistency()` einen unterbrochenen
Lauf und setzt `initial_backfill_target` auf den letzten konsistenten Header
`71185159`/`0x005603ad…`. Die Header-Stage erhält diesen Hash intern und schließt den lokalen
Gap folgerichtig sofort mit `Target block already reached`. Direkt danach lieferte der
Konsens-Client `newPayload`/FCU bei Block `180937060` und fortlaufend weitere Payloads. Während
der exklusive Reparatur-Backfill aktiv ist, antwortet der Engine-Tree auf FCUs absichtlich
`SYNCING`; `ForkchoiceStateTracker::set_latest` speichert dabei den neuesten Sync-Target. Nach
`BackfillSyncFinished` soll `on_backfill_sync_finished()` daraus den nächsten Backfill bis zum
Netz-Head anfordern. Es gibt bislang keinen Header-/P2P-Validierungsfehler und keinen Beleg für
einen stale Tip; der Live-Beleg dieses Übergangs nach Pipeline-Idle steht noch aus.

**Live-Status:** Der lokale Reparatur-Pass ist bis `71185159` geschlossen. Der vollständige
Merkle-Rebuild des vorhandenen Hashed State (2.84B Entitäten) hält die Pipeline weiter exklusiv;
danach müssen Headers/Bodies über den vorgemerkten Engine-Target bis zum dann aktuellen CL-Tip
weiterlaufen. Dieser Übergang sowie vollständige Crash-Resume-Semantik von `migrate-v2` bleiben
offen.

**Auswirkung / Status:** Der State-Root-Fail um 14:54:12 CEST erzwingt aktuell einen vollständigen
Unwind (`StorageHashing 71185159→0` in 100k-Block-Chunks), bevor irgendein externer
Header-Backfill möglich ist. Das ist kein Stillstand und kein bloßer Grafana-Anzeigefehler.
Die Ursache liegt im V2-Sync-/State-/Static-File-Datenpfad und ist offen; die zuvor ergänzten
Migrations-Guidrails sind präventiv, aber keine bestätigte Fehlerbehebung für diesen Datadir.

**Root-Cause-Analyse `main` (16:00–16:30 CEST): zwei belegte Portierungsdefekte.**

*(1) Static-File-Block-Index falsch verschlüsselt — behoben in `34f19ebdd4`.*
`StaticFileProvider::update_index` trug die erwarteten Block-Ranges lokal (Commit `9fdf8220cc`)
unter `segment_max_block` ein, während der `or_insert_with`-Zweig und die Leseseite
`find_fixed_range_with_block_index` diese Schlüssel als *Range-Enden* interpretieren. Für eine
angeschriebene, aber noch nicht gefüllte Datei (Schlüssel `1`, Range `0..=499999`) schlägt der
Lookup deshalb fehl und fällt in die Range-Ableitung:
`let blocks_after_last_range = block - range.end();` → u64-Underflow. Im Debug-Build ein Panic,
im maxperf-/Release-Build ein **stiller Wrap** mit anschließend absurder Range, sodass real
vorhandene Static-File-Daten als fehlend gemeldet werden. Das erklärt den Auslöser der gesamten
Kette (`Stage is missing static file data … segment=Receipts` bei `71185160`) direkt.
Der Index ist jetzt wieder konsistent per `fixed_range.end()` verschlüsselt, plus Guard gegen den
stillen Wrap. Nachweis: `tests/preimage.rs` panickte zuvor reproduzierbar an genau dieser Stelle.

*(2) Slot-Preimage-DB (Upstream #22379) nie portiert — nachgeholt in `333ba71e10`.*
`write_state_reverts` liest die vor einem Storage-Wipe gültigen Slots aus `PlainStorageState`.
Unter `storage_v2` wird diese Tabelle nie geschrieben (`write_state_changes` kehrt früh zurück).
Der Codekommentar behauptete, die Execution-Stage injiziere die Plain-Keys per Preimage — diese
Implementierung fehlte im Port vollständig; aus Upstream `815037e27d` waren nur die Tests
übernommen und in `5b8488b606` per `#[ignore]` stillgelegt worden. Folge: bei jedem
Selfdestruct mit nicht-leerem Storage blieb `wiped_storage` leer, die Vor-Wipe-Slots fehlten in
den StorageChangeSets, und ein Unwind über solche Blöcke konnte den Storage nicht rekonstruieren.
Portiert sind jetzt `execution/slot_preimages.rs`, die Pre-Cancun-Anbindung samt Aufräumen der
Hilfs-DB ab Cancun sowie `reject_cancun_boundary_unwind`; alle sechs Regressionstests sind
reaktiviert und grün.

**Wichtige Einschränkung zur Zuordnung:** opBNB aktiviert Cancun bei ts `1718871600`
(2024-06-20). Block `71185159` (2025-08-01) liegt damit **post-Cancun**, wo SELFDESTRUCT keinen
Storage mehr zerstört. Defekt (2) kann den Merkle-Mismatch vom 09-02 daher **nicht** verursacht
haben; er ist aber zwingende Voraussetzung für den laufenden Re-Sync ab Genesis, der alle
pre-Cancun-Selfdestructs durchquert (u. a. PreContract bei `5805494`). Ursächlich für den
Live-Vorfall ist nach aktuellem Stand Defekt (1); der konkrete Merkle-Mismatch bei `71185159`
bleibt formal unbewiesen — plausibelste offene Hypothese ist, dass der erzwungene Unwind
`71242925→71185159` bereits auf einem durch den Index-Bug beschädigten Static-File-Zustand lief.

**Neuer Blocker (PORT-STAGE-006), Live-Befund 16:31 CEST.** Beim Neustart mit der Binary
`333ba71e10` zeigt das Journal:
`Preparing StorageHashing checkpoint=70885156 target=71185159` → `Finished … 71185159` in <1 ms.
Der gestrige Unwind war mitten in `StorageHashing` bei `70885156` abgebrochen worden; Execution
stand unverändert auf `71185159`, da die Stage-Reihenfolge Hashing **vor** Execution unwindet.
Beim Neustart setzt `check_pipeline_consistency()` wegen `MerkleExecute=0` einen Forward-Backfill
an, statt den offenen Unwind fortzusetzen. Da die Hashing-Stages unter V2 im Forward-Pfad reine
No-ops sind (`use_hashed_state()` → nur Checkpoint auf Target setzen), wird der Hashed State für
`70885157..=71185159` **nicht wiederhergestellt** — der Checkpoint springt lediglich hoch.
Der aktuell laufende MerkleExecute-Pass (ETA 2–3 h) wird deshalb mit hoher Wahrscheinlichkeit
erneut einen falschen State Root liefern. Upstream v2.4.1 enthält denselben ungeschützten
No-op; es fehlt ein Konsistenz-Guard „Hashing-Checkpoint < Execution-Checkpoint“ bzw. eine
persistierte Unwind-Fortsetzung. **Gefixt lokal:** Startup-Guard bricht nun unter Storage-V2 ab,
wenn Execution bereits auf dem Header-Tip steht, aber Account-/StorageHashing darunter liegen.

### Forensik 09-02 17:00–17:50 CEST — Datadir-Obduktion (Node gestoppt, read-only)

Vor dem Verwerfen des Datadirs wurde der State offline vermessen (`db state`,
`db static-file-header`, `mdbx_stat`) und gegen `https://opbnb.drpc.org` (einziger
gefundener öffentlicher Archive-Endpoint mit historischem State) abgeglichen. Ergebnis: **ein
Datadir mit drei verschiedenen Höhen** — die eigentliche Ursache des Root-Mismatch.

| Komponente | Tatsächliche Höhe (gemessen) | Checkpoint behauptete |
| --- | --- | --- |
| `HashedAccounts` | **71 242 925** | 71 185 159 |
| `HashedStorages` | **70 885 156** | 71 185 159 |
| Blockdaten / Static Files | 71 185 159 | 71 185 159 |

Belege (je zwei unabhängige Indikatoren):
*Accounts* — `0x4200…0006` (WBNB) Balance `864879264746320059412` und `0x4200…0011`
(SequencerFeeVault) `7907590433894700094` matchen **exakt und nur** bei Block `71242925`.
*Storage* — `0x4200…0015` (`L1Block`) Slot 0 = `l1num 55898140 | ts 1753921625` matcht
**exakt und nur** bei Block `70885156`.

**Ursachenkette (vollständig rekonstruiert):**
1. **PORT-STOR-009** meldete durch den Index-Underflow fälschlich fehlende Receipts bei
   `71185160`; der Startup-Consistency-Check kürzte daraufhin **alle** Static-File-Segmente auf
   `71185159` (per `static-file-header` bestätigt: Headers/Receipts/Transactions sowie
   Account-/StorageChangeSets enden alle exakt dort).
2. Der anschließende Execution-Unwind lief ins Leere — **PORT-STOR-011**: in
   `remove_state_above` ist `range = block + 1..=self.last_block_number()?`; `last_block_number()`
   war durch (1) bereits `71185159`, die Range damit leer, und `if range.is_empty() { return
   Ok(()) }` kehrt **still** zurück. Der Hashed-Account-Revert `71242925 → 71185159` fand nie
   statt, der Checkpoint wurde trotzdem gesetzt.
3. MerkleExecute rechnete bei `71185159` gegen Accounts von `71242925` → der Mismatch war
   zwingend, nicht zufällig. Damit ist der Root-Fail vom 09-02 **restlos erklärt**; die zuvor
   notierte Hypothese „beschädigter Static-File-Zustand“ ist damit erledigt.
4. Der ausgelöste Unwind Richtung Genesis rollte `HashedStorages` batchweise auf `70885156` und
   wurde dort unterbrochen.
5. Neustart: **PORT-STAGE-006** hob die Hashing-Checkpoints per Forward-No-op zurück auf
   `71185159`, ohne Daten zu rekonstruieren.

**Warum keine Teilrettung möglich war.** `unwind_account_hashing` ist rein changeset-getrieben
(`changesets.map(|(_, e)| (keccak256(e.address), e.info))`) — Accounts ohne Changeset im
unwindeten Bereich werden nie angefasst. Die AccountChangeSets für `71185160..=71242925` waren
durch (1) mitgekürzt und lokal unwiederbringlich; insbesondere in diesem Fenster **neu
entstandene** Accounts hätten als Phantome im State überlebt und jeden weiteren Merkle-Lauf
erneut vergiftet. Ein Reparatur-Unwind auf `70885156` war damit ausgeschlossen. Eine Reparatur
über einen Referenz-Archive-Node (`eth_getProof`-Descent) wäre theoretisch möglich, scheiterte
praktisch am Rate-Limit der öffentlichen Endpoints.

**PORT-STAGE-007 (daraus abgeleitet, sicherheitsrelevant).** `unwind()` in `hashing_account.rs`
/ `hashing_storage.rs` besitzt **keinen** Table-Clear-Pfad — auch nicht bei Unwind-Ziel 0. Ein
„Unwind auf Genesis“ leert `HashedAccounts`/`HashedStorages` also **nicht**; Phantom-Einträge
überleben und würden einen nachfolgenden Sync ab Block 0 von Beginn an korrumpieren. Konsequenz
für den Betrieb war bis zum Fix: nach einem V2-State-Schaden **Datadir verwerfen, nicht auf 0
unwinden**. **Gefixt lokal:** Hashing-Unwind auf Ziel 0 leert unter Storage-V2 die jeweiligen
Hashed-State-Tabellen explizit.

**Recovery-Guard-Fixes 19:34–19:50 CEST.** Nach Vergleich mit Reth v2.4.1/v2.5.0/main,
`bnb-chain/reth` v0.1.2 und den go-geth/op-geth Recovery-Pfaden wurden drei lokale
Sicherungen umgesetzt:
1. `stage drop Execution` ist nun upstream-konform Storage-V2-aware und leert
   `HashedAccounts`/`HashedStorages` statt der leeren Plain-State-Tabellen.
2. `remove_state_above` bricht unter Storage-V2 ab, wenn der Execution-Checkpoint über den
   verfügbaren Blockdaten liegt, statt einen leeren Range-Revert still als Erfolg zu behandeln.
3. Account-/StorageHashing-Unwind auf Genesis leert unter Storage-V2 die Hashed-State-Tabellen.

Der bewusst **nicht** in den Hashing-Stages selbst gesetzte Forward-Guard wurde verworfen: ein
naiver `checkpoint < target`-Abort blockiert legitimen frischen V2-Sync (`0 → 1`). Der Guard sitzt
daher im Startup-Consistency-Check, wo der konkrete gefährliche Zustand erkennbar ist
(Execution == Header-Tip, Hashing darunter).

**Entscheidung 17:53 CEST:** Datadir (inkl. Static Files, verifiziert: 2861 → 21 Dateien)
verworfen, Re-Sync ab Genesis mit `333ba71` gestartet. Log sauber:
`Wrote storage settings genesis_storage_settings=StorageSettings { storage_v2: true }` —
Storage-V2 also **ab Genesis** statt via Migration; Headers laden rückwärts ab `179 239 838`.
Der Re-Sync durchquert erstmals scharf die pre-Cancun-Selfdestructs, womit die in `333ba71e10`
portierte Slot-Preimage-DB (**PORT-STAGE-005**) produktiv zum Tragen kommt; `preimage/` wird
erwartungsgemäß erst mit Beginn der Execution-Stage angelegt.

**Aufwand/Kosten:** Journal-/Mimir-Analyse, drei fokussierte Rust-Dateien plus Dokumentation;
`cargo +nightly check -p reth-cli-commands --lib`, fokussierte Settings-Tests,
`cargo +nightly check -p reth-provider --lib` und ein `make maxperf-op` erfolgreich. Für diese
Copilot-Session liegt kein belastbarer Billed-Token-/Kosten-Ledger vor; keine Kostenschätzung
ergänzt.

## Phasenübersicht (Soll)

1. **Phase 1 — Bestandsaufnahme & Diff-Baseline** ✅ erledigt
2. **Phase 2 — Kern-Crates auf v2.4.1 rebasen** ✅ Merge/Konflikte erledigt, Detailarbeit läuft (s.u.)
3. **Phase 3 — BSC-Crate (`crates/bsc`) aktualisieren** 🔄 Compile ✅ (08-09); **Fork-Nachzug** `bnb-chain/reth-bsc` → Pasteur (08-23 Slice 1: Hardforks+System-Contracts+SpecId)
4. **Phase 4 — Optimism/opBNB-Crate + Snow/Volta/Fourier-Hardforks** 🔄 Hardforks+stack through **node/cli/op-reth bin** compile-green; nextest prim/consensus/evm/node/rpc ✅; trie/proofs deferred
5. **Phase 5 — Build/Lint/Test/EF-Tests** ✅ check/Clippy/nextest stages+op-stack; EF **v17.0** + Bytecode Compact → **62/62** nach nextest-Timeout-Override (`valid_blocks`/`invalid_blocks` re-verified)
6. **Phase 6 — Doku, CI & Freigabe** 🔄 Effort-Log Session 6+8+9+**10**; **Migrations-Gate PIPE+FLOW** nachgezogen; die gekoppelten Optimism-Crates (`op-revm`, `alloy-op-evm`, `alloy-op-hardforks`, `op-alloy-*`) sind für den Wright-/PreContract-Port im Workspace vendort, damit keine nicht-portablen `/usr/src`-Pfade oder doppelten Cargo-Typidentitäten verbleiben; Human Catch-up/Full-Sync + finale Zahlen nach Live-Tests

### Sync-Tests (Human-owned)

- **Catch-up** und **Full Sync** startet/führt **nur ein Human** durch — sobald die AI den Port als
  **lauffähig** einstuft (Compile + Boot/RPC-Smoke + Kern-Tests ohne Blocker).
- AI macht höchstens Boot-Smoke / kurze Pipeline-Sanity; keine langen Sync-Läufe.
- **Stand 2026-09-01 ~18:52 CEST:** H+Bodies+Sender Tip ✅ `174 027 661`. Exec **`65 828 907`** (~38 %, ~19–33 blk/s cooled). Haber Point-4 ✅; **past Wright**. **ETA Tip ~1¼–2¼ Mo** (current bands). X02/PIPE-009 ✅. P2P-002 ✅; P2P-006 offen. Live-Node: **kein Restart**.

## Roadmap (aktuell — Exec-Fenster)

| Fenster | Ziel | Aufwand (Schätzung) | Status |
| --- | --- | --- | --- |
| **≤48 h** | Point-4 stateRoot @ Haber (~27.1 M) dann Wright (~33.0 M) vs public RPC | ~0.5 h Agent/Stichprobe je Gate | ✅ Haber MATCH (08-17); Wright height **passed** — optional Point-4 sample still open |
| **≤48 h** | Journal/Mimir: kein Unwind / receipt-root / peers>0 | ~5–10 min / Check | 🔄 laufend |
| **diese Woche** | CLEANUP-A02 Rest (provider/rpc/db/…) + A03/A04 | ~2–4 h Agent | 🔄 A02 partial |
| **bei geplantem Restart** | PORT-P2P-006 Dual-Stack; optional Serve-RX / ENGINE-004 | ~0.5–1 d Code+Live | 📋 geparkt bis Restart |
| **aktuell: Re-Sync ab Genesis** | Headers → Bodies → Execution neu aufbauen; Gates Haber/Wright Point-4 erneut ziehen | unsupervised + Spot-Checks | 🔄 **Neustart 09-02 17:53** mit `333ba71`, Datadir verworfen (Static Files 2861 → 21). `storage_v2: true` **ab Genesis** statt via Migration; Headers laden rückwärts ab `179 239 838`. Vorheriger Repair-Versuch aufgegeben — Ursache forensisch geklärt (**PORT-STOR-011**), Teilrettung wegen gekürzter AccountChangeSets unmöglich. CL-Head ~`180.99M` |
| **nach Repair-Backfill** | Execution Tip → Merkle/History/Finish | unsupervised + Spot-Checks | ⏳ erneut zu berechnen, sobald die Header- und Execution-Rate nach dem Wiederanlauf messbar ist |
| **nach Tip** | Snapshot-Manifest/`download` für op-reth verdrahten; FEAT-HIST-* | groß | 📋 nach Sync-Gates |
| **erledigt 09-02** | **V2-State-Integrität — drei gekoppelte Guards.** (a) **PORT-STOR-011**: `remove_state_above` kehrt still zurück, wenn Static Files bereits gekürzt sind → State-Revert entfällt, Checkpoint wird trotzdem gesetzt. (b) **PORT-STAGE-006**: `AccountHashing`/`StorageHashing` sind unter `use_hashed_state()` im Forward reine No-ops; nach abgebrochenem Unwind bleibt die Differenz ungehasht. (c) **PORT-STAGE-007**: Hashing-`unwind()` hat keinen Table-Clear-Pfad, „Unwind auf 0“ leert den Hashed State nicht. Zusammen erzeugen sie **stillen, nicht reparierbaren State-Verlust** | ~1–2 d Code + Regressionstests | ✅ lokal gehärtet: Storage-V2-aware `stage drop Execution`, `remove_state_above`-Abort bei Execution>Blockdaten, Startup-Consistency-Guard bei Execution==Header-Tip aber Hashing darunter, Hashing-Unwind-to-0 clear. Upstream-Issue/PR später prüfen bzw. melden |
| **nicht jetzt** | Rebase → reth 2.5.0; Live-Datadir snapshotten während Exec | — | ⛔ |

**P0-Gates (Exec):** PIPE-014 live past Fail ✅ · X02 Unit ✅ · Haber Point-4 ✅ · Wright height passed (optional Point-4 sample) · Tip ⏳ (~1¼–2¼ Mo) · FLOW-X05 watch.

### BSC Mainnet Port (parallel, opBNB unverändert) — Stand 2026-08-23

Referenz: `github.com/bnb-chain/reth-bsc` main (live Tip); Workspace bleibt **reth v2.4.1** Monorepo.

| ID | Inhalt | Status |
| --- | --- | --- |
| PORT-BSC-001 | Hardforks Tycho→Pasteur + Prague (`crates/bsc/hardforks`) | ✅ Slice 1 |
| PORT-BSC-002 | System-Contracts Pascal/Pasteur/Maxwell/Lorentz/Fermi | ✅ Slice 1 (dirs from reth-bsc) |
| PORT-BSC-003 | `revm_spec_by_timestamp_and_block_number` + SpecId map | ✅ Slice 1 |
| PORT-BSC-004 | Prague HistoryStorage + `apply_blockhashes` in executor | ✅ Slice 2 legacy · ✅ Slice 3 `BscAlloyBlockExecutor` (+ Eth `apply_pre`) |
| PORT-BSC-005 | Pasteur state-hook on system-contract upgrade (trie) | ✅ Slice 2 legacy · 🔬 main path (upgrade `mark_touch`; engine hook noch nicht alloy-verdrahtet) |
| PORT-BSC-006 | Parlia/network live-sync PIPE+FLOW | ✅ Slice 8: `ParliaEngineBuilder::build(true)` + `BscBlockImport`/`ImportService` beim Node-Launch; engine-handle oneshot wie upstream reth-bsc. **Offen:** live verify (007). |
| PORT-BSC-007 | Live smoke / Point-4 post-Pascal | 🔜 Human |
| PORT-BSC-008 | `reth-bsc-cli` v2.4.1 (stage/import/launcher) | ✅ Slice 4 |
| PORT-BSC-009 | Legacy `static_files/*.conf` → `tag for enum is not valid, found 247` on node/`db` init | Official reth-bsc datadir: pre-v2.4.1 `SegmentHeader` + jars with serialized `filter`/`phf` | ✅ fix: `load_segment_nippy_jar` + `NippyJar::load_from_bytes` legacy paths |
| PORT-BSC-CLI-009 | `--datadir` semantics: explicit path = chain data dir (no auto `/bsc` suffix); default = `~/.local/share/reth/<chain>` | `DatadirArgs::resolve_datadir` / `MaybePlatformPath::unwrap_or_chain_default` | 📝 ops doc |
| PORT-BSC-SF-002 | Hybrid v2: `TransactionSenders` SF genesis-only, SenderRecovery @ 117M, MDBX still has senders → unwind target 0 panic | `storage_v2` flipped before senders migrated to SF | ✅ skip unwind when MDBX has senders; warn to backfill SenderRecovery |
| PORT-BSC-SF-003 | Orphan SF / wrong slot path on heal (`117500000` without `.conf`) | Interrupted append + legacy filename inside slot | ✅ orphan delete + `jar_data_path` legacy resolve + missing jar → create (reth-bsc datadir pattern) |

**PORT-BSC-FLOW-006** (Parlia checkpoint persistenz): Checkpoint-Blöcke (`N % 1024 == 0`) → `put` keyed by `block_hash`; Execution-Unwind (`take_state_above` / `remove_state_above`) → `delete` alle Einträge mit `snapshot.block_number > unwind_to`; Read-Pfad: MDBX vor LRU-Cache an Checkpoints.

**PORT-BSC-FLOW-007** (Node-Launch live-sync): `BscNode::new()` + oneshot → `ImportService::from_channels` + `BscBlockImport` in `BscNetworkBuilder::network_config`; nach Launch `engine_handle_tx.send(beacon_engine_handle)`; `on_node_started` → `spawn_parlia_engine` (`ParliaEngineBuilder::build(true)` → FCU/newPayload an Engine-Tree). Invariante: ohne beides bleibt Pipeline-`target=Hash(<db-tip>)` frozen („Target block already reached“).

**Nächster Slice:** PORT-BSC-007 Live smoke post-Pascal (human deploy + verify FCU/tip advance).

## Todo-Status (Stand 2026-08-20)

| ID | Titel | Status |
| --- | --- | --- |
| inventory-diff | Bestandsaufnahme & Diff-Baseline erstellen | ✅ done |
| core-rebase | Kern-Crates auf reth v2.4.1 rebasen | ✅ done |
| bsc-crate-update | BSC-Crate (crates/bsc) aktualisieren | ✅ done (compile: bsc-node grün) |
| opbnb-hardforks | Optimism/opBNB-Crate + Snow/Volta/Fourier | 🔄 Storage-v2 recovery forced local checkpoints to **71 185 159**; Root-Cause im Port lokalisiert und behoben (`34f19ebdd4` Static-File-Index-Underflow, `333ba71e10` Slot-Preimage-DB). Neuer aktiver Blocker **PORT-STAGE-006** (Hashed State nach abgebrochenem Unwind nicht rekonstruiert). Engine Backfill zu CL ~**180.97M** weiterhin offen. X02 ✅; P2P-002 ✅; P2P-006 todo |
| build-test-validate | Build, Lint, Tests, EF-Tests | ✅ stages/op-stack nextest; EF v17.0 → **62/62** |
| docs-release | Doku aktualisieren, Freigabe vorbereiten | 🔄 Gate+ETA 09-01; finale Zahlen nach Exec Tip |

## Portierungs-Bugliste (v2.4.1 rebase)

Regressions / CLI-Drift, die beim Rebase untergegangen sind (nicht Upstream-Feature-Gaps).

| ID | Symptom | Ursache | Status |
| --- | --- | --- | --- |
| PORT-CLI-001 | `--storage.v2` fehlte an `op-reth`/`reth` (`node`, `init`, …); neue DBs liefen effektiv über `StaticFilesArgs::to_settings()` → oft **v1** | `StorageArgs` beim Phase-3/4-Port aus `EnvironmentArgs`/`NodeCommand`/`NodeConfig` entfernt; Genesis nutzte falschen Settings-Pfad | ✅ fixed (Session 8): wieder verdrahtet wie Upstream v2.4.1; Default `true`; `ArgAction::Set` + optionaler Wert |
| PORT-CLI-002 | README empfiehlt noch `--enable-prefetch` / `--optimize.enable-execution-cache` | Alte BSC-Fork-Toggles; CLI + Engine-Gating beim Port verloren; Upstream ersetzt durch `--engine.*` Prewarm/Cache | 📝 docs: Flags als obsolet markiert; Runtime-Port von `TriePrefetch` bewusst nicht wiederbelebt |
| PORT-CLI-003 | `--ipcpath /tmp/foo.ipc` wurde zu `/tmp/foo.ipc-1` | `NodeConfig.instance` war `u16` mit Default `1`; `adjust_instance_ports` hängte immer `-{instance}` an | ✅ fixed: `instance: Option<u16>` (None ohne `--instance`), wie Upstream |
| PORT-CLI-004 | Log `Storage settings settings=None`; trotz `--storage.v2` keine v2-Persistenz / kein „Loaded storage settings“ | `init_genesis_with_settings` war Stub (ignorierte Settings); Log lief **vor** Genesis | ✅ fixed: Settings bei frischer DB schreiben; bestehende DB: fehlende Metadata = v1 + Warn bei CLI-Mismatch; Log nach Genesis |
| PORT-CLI-005 | OTLP (`--tracing-otlp` / `--logs-otlp`) wirkt in Live-/maxperf-`op-reth` nicht; Grafana sieht nur Prometheus | Code pfad ist verdrahtet (`reth-tracing-otlp`, `TraceArgs`, Optimism/Eth CLI), aber hinter optionalen Features `otlp` / `otlp-logs` — **nicht** in `default`, **nicht** in `make maxperf-op` (`jemalloc,asm-keccak,keccak-cache-global`). Ohne Feature: Warn „compile with the `otlp` feature“ | 📝 bewusst so (wie Upstream Feature-Gate). **Ops:** `--metrics` (Prometheus) reicht für Grafana; OTLP nur bei Bedarf mit `--features …,otlp[,otlp-logs]` bauen |
| PORT-BSC-SF-001 | BSC prod: node + `db` crash `tag for enum is not valid, found 247` at `StaticFileProviderBuilder::build` | Legacy reth-bsc `static_files/*.conf` incompatible with v2.4.1 `SegmentHeader` / skipped `filter`/`phf` in `NippyJar` | ✅ + legacy **offsets-before-segment** change-set headers (`117470000` jar) |
| PORT-BSC-SF-003 | Node crash `failed to read static file config …117500000_117999999.conf: No such file` | Orphan data file without committed `.conf` after interrupted append | ✅ RW startup: `remove_orphan_static_files` deletes incomplete bundle (not skip) |
| PORT-STOR-001 | Fresh start crash: `append Headers #0 but expected #1` | Incomplete port: AccountChangeSets SF stub wrote into **Headers** during `write_state` (genesis); Senders stub similarly unsafe | ✅ closed via PORT-STOR-004/005 (dedicated segments; no Headers/Tx reuse) |
| PORT-STOR-004 | TransactionSenders SF stub reused Transactions/Receipts | Wrong segment literals + prune stub; v2 expected senders in SF | ✅ fixed: dedicated TransactionSenders writer/prune/readers; `transaction_senders_in_static_files() → storage_v2` |
| PORT-STOR-005 | AccountChangeSets SF incomplete (Headers corruption) | Missing `.csoff` sidecar / header len / writer heal; stubs wrote Headers | ✅ fixed: SegmentHeader `changeset_offsets_len` + sidecar writer/heal/truncate; `account_changesets_in_static_files() → storage_v2` |
| PORT-STOR-006 | StorageChangeSets stub always routed to MDBX (`TODO(opbnb-port)` `Headers` placeholders in rocksdb invariants, migrate-v2, `db state`) | `StaticFileSegment::StorageChangeSets` variant, mask, writer/reader, and `either_writer` routing were never ported after AccountChangeSets SF landed | ✅ fixed (Session 9): dedicated `StorageChangeSets` segment (`.csoff` sidecar, same change-based model as AccountChangeSets); `storage_changesets_in_static_files() → storage_v2`; `EitherWriter`/`EitherReader` routing in `write_state_reverts`/`StorageReader`; `migrate-v2` now really migrates `StorageChangeSets` into static files instead of skipping |
| PORT-STOR-002 | Kein `rocksdb/` trotz `--storage.v2` (Default true) | Feature `reth-provider/rocksdb` war nicht verdrahtet; API-Drift (0.24 CF refs, snapshot/batch, history tip, SF stub); prune Batch-Lifetimes | ✅ fixed: provider+prune rocksdb-Pfad kompiliert; `op-reth` default `rocksdb`; `cargo check -p op-reth` grün |
| PORT-P2P-001 | opBNB EL: anfangs `peerCount=0` / Sync-Genesis trotz Tip | Stale Bootnodes; discv4; `--addr ::` discv5; ForkId; opstack CL ENRs | ✅ **live** eth-Sessions (typ. ~5–7 Peers outbound); Rest: P2P-006 Announce/Bind |
| PORT-P2P-002 | Default `--nat any`: kein echtes UPnP; Announce ohne dialbares Mapping; 0 inbound | **Fix 08-15:** `reth-net-nat` + `igd-next` — UPnP zuerst (`add_port` preferred, sonst `add_any_port`, **kein** Hijack); SSDP-Timeout 10 s; Announce-IP-Familie ≡ `--addr`; discv4/discv5 `apply_nat_endpoint`; `advertised_nat` → enode/ENR/Logs; Lease-Refresh 8 min; HTTP-Fallback same-family. **Live:** Alt-Ports wenn `:30303` fremd (Erigon); `via_upnp=true`; hairpin LAN→WAN-IP OK; `incoming_connections≥2`; `eth_*_requests_received` noch 0 (Serve später). | ✅ **live** Map+Announce · FLOW-N02 · Serve-RX watch |
| PORT-P2P-003 | Headers: Empty-Spam auf Tip-Range (`best_number`≪CL-Tip); Lagging-Peers ungenutzt; Stage hängt an unreachable Tip | **Dataflow-Soll (vor Live):** eth/68 Status oft nur Tip-**Hash** → Tip-Number-Resolve; Peer-Auswahl `HeadersAtLeast` / miss-map; Empty → Backoff **ohne** Ban; eth/69 `tip_number=max(best,range.latest)` + Range-Filter. | ✅ **live** (2026-08-11T14:40Z): Tip-Resolve + Falling ab Peer-Head ~173369140 @ ~22k hdr/s (2 Peers). Code: HeadersAtLeast/miss-map; eth/69 Range; ENGINE-003 Tip-Seed. Note: Headers-ETL=`TempDir` (Upstream [#6154](https://github.com/paradigmxyz/reth/pull/6154)) — Restart vor Write = Neustart von Tip; Checkpoint erst nach ETL→SF |
| PORT-P2P-004 | Working-Tip-Cap vs eventual CL-Tip: Cap darf Tip/Falling nicht periodisch verwerfen | **Dataflow-Soll:** `eventual_tip` (CL) ≠ `working_tip` (max peer best). Cap einmalig auf reachable Head; `maybe_recap` **idempotent** wenn already capped — sonst Re-Loop verwirft Tip-Header. Gehört in Matrix **vor** Live, nicht erst nach Stall. | ✅ fixed + live: Cap 1×; Unit-Regression Cap→Falling |
| PORT-P2P-005 | Cap setzt `SyncTargetBlock::Number(N)` → Falling-Tracker bleiben ungesetzt → nur Tip `total=1` dann Stall | **Dataflow-Soll:** Tip-Outcome `Number(N)` mit `old==new` (lokaler Head schon N−ε) muss `next_request_block_number` / Falling-Tracker **primen**. Gehört in Matrix mit P2P-003/004 (Downloader-Zustandsautomat), nicht als „Live-Folgebug“. | ✅ fixed + live (14:40Z): Falling `total=10000` durchgehend; Test Cap→Falling-Prime |
| PORT-P2P-006 | Ohne `--addr`: Bind/Dial/Announce folgen **OS-Netzwerk** (Dual-Stack); mit `--addr`: **nur** die IP-Familie des Werts | **Soll:** kein `--addr` → Dual-Stack nach OS (modern IPv6 first); Announce = preferierte dialbare Adresse ≡ Listen. Mit `--addr <ip>` → single-family; andere Familie tot (Docs/Help klar). **Ist heute:** CLI-Default Bind = v4-unspec `0.0.0.0:30303`; `--nat any` schreibt ENR/enode unabhängig davon (HTTP-Public-IP und/oder Interface). **Live 08-15 (IPs anonymisiert):** (1) Bind v4-only, Sessions über `<host-lan-ip>`; (2) Announce zeitweise **`<host-global-ipv6>`** derselben NIC (SLAAC/EUI-64, MAC = Node, **nicht** Router-GW) — Host pingbar, **TCP 30303 refused** weil kein v6-Listen; (3) Peers **0 inbound / N outbound**; (4) Announce kann auf Public-v4 aus NAT-API flippen — weder konsistent noch „OS preference“. | ✅ **09-03** (Commits `4bbdd60fd6`, `45db221aeb`) · FLOW-N01 · DoD (a)+(b)+(c) erfüllt: `NetworkArgs::addr: Option<IpAddr>` unterscheidet jetzt "kein `--addr`" von explizit `0.0.0.0`/`::`; `wants_os_dual_stack()` aktiviert discv5 Dual-Stack nur ohne `--addr`. Dev-Host-Isolationstest (isolierter Datadir, `--chain opbnb-mainnet`, kein `--addr`) bestätigt: `discv5::service: Discv5 Service started mode=DualStack`, echte UDP-Bindings auf `0.0.0.0:9200` **und** `[::]:9200`, NAT announct beide Familien (`Announced dialable enode` [IPv6] + `Announced additional discv5 dual-stack NAT endpoint` [IPv4]). RLPx-TCP/discv4 bleiben bewusst single-stack. **Update (Session 16):** Live-Node läuft tatsächlich bereits ohne `--addr` (Dual-Stack aktiv); dabei Folgebug im UPnP-Family-Handling gefunden+gefixt+live bestätigt (kein Family-Mismatch mehr, sauberes IPv4-UPnP-Mapping via_upnp=true, 5 Peers reconnected). |
| PORT-STOR-003 | Neue MDBX-DBs mit 4 KiB Pagesize (OS-default) | `default_page_size()` clampte nur auf OS-Pagesize (≥4 KiB); keine Begründung gegen 16 KiB | ✅ fixed: Floor 16 KiB (max OS/libmdbx 64 KiB); nur bei DB-Erstellung wirksam |
| PORT-STOR-007 | `test_pipeline_v2` State-Root-Mismatch / SF unwind; history `IntegerList UnsortedInput` | Incomplete v2 port: plain readers under hashed-canonical; StorageChangeSets keys wrongly hashed; take/remove_state plain-only; hashing/history unwind ignored SF; duplicate block nums in history collect | ✅ fixed: hashed `AccountReader`/`StorageReader`; plain keys in changesets; hashed take/remove; SF hashing/history unwind; dedupe history indices; test un-ignored |
| PORT-STOR-008 | Index Account/Storage History under `storage.v2` still wrote MDBX; unwind no-op without rocksdb | Incomplete EitherWriter history load (`load_*_history`) + RocksDB clear/unwind wiring | ✅ fixed: EitherWriter append/upsert/get_last; stages use `with_rocksdb_batch_auto_commit`; MDBX fallback when rocksdb feature off |
| PORT-STOR-009 | Startup panic: `segment operation metrics should exist` (static_file/metrics.rs) after metrics endpoint | Metrics `Default` only registered Headers/Tx/Receipts/Sidecars; heal/init-cursor hits Account/StorageChangeSets + TransactionSenders | ✅ fixed: register via `StaticFileSegment::iter()` (upstream pattern) |
| PORT-STOR-010 | `--dev` / frische v2-DB: `Persistence … UnexpectedStaticFileBlockNumber(TransactionSenders, 1, 0)` → Fatal engine | `init_genesis` tippte nur Receipts+Transactions auf Block 0; unter `storage_v2` liegen Senders in SF, Genesis hat 0 Txs → Segment blieb untipped; erster Persist `increment_block(1)` scheitert | ✅ fixed (Session 11): wie Upstream/`bnb-chain/reth` bei `storage_v2` `get_writer`+`set_block_range(0,0)` für `TransactionSenders`; Verify: `files/dev-250ms` init zeigt `static_file_transaction-senders_*`, kein Persistence-Crash |
| PORT-DEV-001 | `--dev --dev.block-time 250ms`/`1s`: nach ~5–7 Blöcken Dauer-Spam `Error advancing the chain: No payload`; Tip bleibt stehen | **Nur `--dev` / `LocalMiner`:** (1) `advance()` = FCU+Attrs → `resolve_kind(payload_id)`; `resolve` liefert `None` wenn Job nicht (mehr) in `payload_jobs` (Race: Job noch nicht inserted / schon removed / ID stale). (2) Parallel hartcodiert `fcu_interval=1s` mit **bare FCU** (`attrs=None`) im selben `select!` — verschärft Timing; Interval-`MissedTickBehavior::Burst` feuerzt Catch-up-Ticks. (3) Persistence/SF (STOR-010) ist **nicht** die Ursache (nach Fix kein Fatal mehr). **Mainnet/Archive-Follow:** trifft **nicht** denselben Pfad — kein `LocalMiner`; Tip-Follow = CL `newPayload` + FCU oft **ohne** Attrs. Sequencer-Build (FCU+Attrs → `getPayload`) steuert die CL zeitlich; kein 1s-bare-FCU aus LocalMiner. Ähnliches Risiko nur, wenn ein Client Attrs-Build und bare-FCU unsynchronisiert spamt (nicht op-node Normalbetrieb). | 🧊 **parked** (2026-08-11): keine Prio / kein maxperf-Rebuild dafür. Soll irgendwann funktionieren **oder** `--dev`/LocalMiner dekommissionieren (kein klarer Mehrwert für Archive-Port). Fix-Idee bleibt: bare-FCU während pending Build unterdrücken; Burst ab; Job vor Resolve. Reproduce: `files/dev-250ms` tip≈7 |
| PORT-DEV-002 | `--dev.payload-wait-time` wirkte nicht | `DebugNodeLauncher` spawnte `LocalMiner::new` ohne `with_payload_wait_time_opt` | ✅ fixed (Session 11): Flag an `LocalMiner` durchgereicht; allein **kein** Ersatz für DEV-001 (Race bleibt) |
| PORT-CONS-001 | Headers-Stage: `TimestampIsInPast` trotz gültiger opBNB-Kette; Peers `BadMessage`-Ban; Checkpoint 0 | Eth-`validate_against_parent_timestamp` (Sekunden). opBNB speichert Subsekunden in `mixHash` (`MilliTimestamp = Time*1000 + mixHash[:2]`, bnb-chain/op-geth); gleiche Unix-Sekunde + steigende Milli ist gültig | ✅ fixed (Session 10): `validation/milli_timestamp.rs` + `OpBeaconConsensus` für Chain-ID **204/5611**; Unit-Tests live equal-second + OP-Mainnet reject |
| PORT-EXEC-001 | Archive Execution: `receipt root mismatch` @ **`21591154`**; Unwind-Sturm 2× | FLOW-X04 idx=10 `syncLightBlock`→`0x67`; Fix Hertz-Overlay. `re-execute` ✅ 08-15. **Live 09-01:** Exec `65 828 907` ≫ Fail/Haber/Wright. | ✅ **fix + live past Fail** · FLOW-X05 watch · Tip ~1¼–2¼ Mo |
| PORT-ENGINE-004 | `systemctl` Reload/Stop: Panic `SelectNextSome polled after terminated` in Critical task `consensus engine` | Shutdown-Pfad Engine/`futures_util::SelectNextSome` nach Stream-Ende noch gepollt (Reload 08-14 13:29 + Stop 17:49) | 🧊 **parked** — später analysieren; Tip/DB nicht primär betroffen |
| PORT-OPS-001 | `--debug.max-block H` als „Park vor Fail“ → Merkle-Fail @ H + `unwind_to=0` | Wenn Stage-Checkpoints **bereits > H**: Pipeline skippt Bodies/Exec (`Stage reached target… skipping`) → Hashing/Merkle auf Restzustand; 08-14 13:43 `bad_block=21579110` state-root mismatch (`got 0x99a6…` / `expected 0x1817…` ≠ Public `0x7b77…`) → Unwind Tip→0 | 🐛 **Ops-Gate** · Cap nur für **Clean-Rebuild** 0…H wenn Checkpoints ≤ H; sonst Process-Stop |
| PORT-ENGINE-001 | Nach Tip-FCU: Status `latest_block=0` **ohne** `stage=…`; Grafana Stages **No data**; Pipeline startet nicht (oder nur kurz) | (1) Engine API Flood: `incoming_requests` vor `downloader.poll` → keine `DownloadedBlocks` → kein Backfill. (2) `handle_missing_block` nur `Download(single_block)` bei gleitendem Buffer (Limit 64) → Tip-Chase, nie Pipeline. (3) `NewDownloadStarted` als Poll-Ready blockierte Inflight-Advance | ✅ fixed + **live** Backfill/`Preparing stage Headers` (FLOW-E01/E02). Checkpoint Headers weiter 0 bis FLOW-H05 |
| PORT-ENGINE-002 | Grafana Stages „0 Blöcke“ vs „No data“ verwechselt | „0“ = Pipeline aktiv, Checkpoint 0. „No data“ = keine Stage-Series (Pipeline idle / Backfill nie gestartet) | 📝 docs only (kein Code) |
| PORT-ENGINE-003 | Headers nach Backfill-Start: Tip-Hash per P2P → empty → Ban | Tip muss von CL/HeaderSeed kommen (op-geth Skeleton), nicht P2P Hash | ✅ **closed** · Tip-Seed + Falling live; Headers Tip **174 M** (FLOW-E03/H05) |
| PORT-ENGINE-005 | Restart mit inkonsistenten Stages: lokaler Konsistenz-Target wird als „Headers stuck“ fehlgedeutet; Netz-FCU trifft während Repair-Pipeline ein | `check_pipeline_consistency()` setzt absichtlich den letzten konsistenten Header als initialen Backfill-Target. Während Backfill `Active` ist, antwortet FCU `SYNCING`, aber `ForkchoiceStateTracker::set_latest` muss den neuesten Head erhalten; nach `BackfillSyncFinished` löst `on_backfill_sync_finished()` den nächsten Backfill aus. **Live 09-02:** lokaler Hash `0x005603…` bei `71185159` korrekt erreicht; anschließend FCU/newPayload ~`180937060+` beobachtet. | 📋 Codepfad nachvollzogen; durch bestätigten Merkle-State-Root-Fail bei `71185159` wieder blockiert. Nach sauberem V2-State-Recovery: `Preparing Headers` mit aktuellem externem Target und Checkpoint-Anstieg nachweisen |
| PORT-STOR-009 | Static-File-Block-Index unter `segment_max_block` statt Range-Ende verschlüsselt → u64-Underflow in `find_fixed_range_with_block_index` | Lokale Abweichung aus `9fdf8220cc`: Leseseite und `or_insert_with` behandeln die BTreeMap-Keys als Range-Enden. Bei angeschriebener, ungefüllter Datei schlägt der Lookup fehl → `block - range.end()` wrappt im Release-Build still → vorhandene Static-File-Daten gelten als fehlend (`missing static file data … segment=Receipts` @ `71185160`) | ✅ **fixed** `34f19ebdd4` · Key wieder `fixed_range.end()` + Underflow-Guard; `tests/preimage.rs` reproduzierte den Panic vorher zuverlässig |
| PORT-STAGE-005 | Slot-Preimage-DB (Upstream #22379) im Port fehlend → unvollständige V2-Wipe-ChangeSets | `write_state_reverts` liest Vor-Wipe-Slots aus `PlainStorageState`, das unter `storage_v2` nie geschrieben wird. Die kommentierte Preimage-Injektion existierte nicht; nur die Tests waren übernommen und in `5b8488b606` per `#[ignore]` deaktiviert. Selfdestructs mit Storage verloren dadurch ihre Revert-Daten | ✅ **fixed** `333ba71e10` · `execution/slot_preimages.rs` + Pre-Cancun-Anbindung + `reject_cancun_boundary_unwind`; 6/6 Tests reaktiviert und grün. **Nicht** ursächlich für `71185159` (post-Cancun), aber Pflicht für den Re-Sync ab Genesis |
| PORT-STAGE-006 | Abgebrochener Unwind + V2-Forward-No-op ⇒ Hashed State wird nie wiederhergestellt | Pipeline unwindet Hashing **vor** Execution. Bricht der Lauf dort ab (live: `StorageHashing` bei `70885156`, Execution weiter `71185159`), setzt `check_pipeline_consistency()` beim Neustart einen Forward-Backfill an statt den Unwind fortzusetzen. Unter V2 sind `AccountHashing`/`StorageHashing` im Forward reine No-ops (Checkpoint → Target), sodass `70885157..=71185159` ungehasht bleibt. Upstream v2.4.1 hat denselben ungeschützten No-op | ✅ **fixed lokal** · Startup-Guard in `check_pipeline_consistency`: wenn Storage-V2 aktiv ist und Execution bereits auf Header-Tip steht, aber Account-/StorageHashing darunter liegt, bricht der Node laut ab statt die Hashing-Stages per No-op hochzusetzen. Naiver Stage-Guard verworfen, da er frischen V2-Sync (`0→1`) blockiert |
| PORT-STOR-011 | `remove_state_above` überspringt den State-Revert **still**, wenn Static Files bereits gekürzt sind | `let range = block + 1..=self.last_block_number()?; if range.is_empty() { return Ok(()) }`. Nachdem der Startup-Consistency-Check die Static Files (ausgelöst durch PORT-STOR-009) auf `71185159` gekürzt hatte, war `last_block_number()` bereits `71185159` → Range leer → der Hashed-Account-Revert `71242925 → 71185159` wurde übersprungen, der Checkpoint aber gesetzt. Forensisch belegt: `HashedAccounts` stand real auf `71242925` (WBNB- und SequencerFeeVault-Balance matchen exakt nur dort) | ✅ **fixed lokal** · unter Storage-V2 Fehler statt stillem `Ok(())`, wenn Execution-Checkpoint über den verfügbaren Blockdaten liegt; verhindert erneuten stillen Hashed-State-Rewind-Skip |
| PORT-STAGE-007 | Hashing-`unwind()` besitzt keinen Table-Clear-Pfad — „Unwind auf 0“ leert den Hashed State nicht | `unwind()` in `hashing_account.rs`/`hashing_storage.rs` rief ausschließlich `unwind_*_hashing_range(range)`, das rein changeset-getrieben arbeitet. Accounts ohne Changeset im Bereich — insbesondere im unwindeten Fenster neu entstandene — blieben stehen. Auch bei Ziel 0 wurden `HashedAccounts`/`HashedStorages` daher **nicht** geleert; Phantom-Einträge überlebten und korrumpierten einen nachfolgenden Sync ab Genesis | ✅ **fixed lokal** · Account-/StorageHashing-Unwind leert bei `unwind_progress == 0` unter Storage-V2 die jeweilige Hashed-State-Tabelle explizit; Regressionstests ergänzt |

### Pipeline-Verify-Matrix (PORT-PIPE) — op-geth ↔ Reth, Stage für Stage

**Zweck:** Systematische Live-/Code-Verifikation der opBNB-EL-**Konsensregeln** entlang `DefaultStages`.
Abgeleitet aus Diff gegen `bnb-chain_op-geth.git`. **Pflicht-Partner:** Abschnitt *Migrations-Gate →
PORT-FLOW-Matrix* — ohne FLOW-Analyse für dieselbe Stage kein Live-„fertig“.

Pipeline-Reihenfolge: Headers → Bodies → SenderRecovery → Execution → MerkleUnwind → AccountHashing → StorageHashing → MerkleExecute → TxLookup → IndexStorageHistory → IndexAccountHistory → Prune → Finish.

**Status-Legende:** `✅ umgesetzt` = Code gegen op-geth portiert (ggf. Unit-Tests); `⏳ live ungetestet` = noch kein Stage-/Archive-Lauf-Beleg; `🐛` = bekannte Regel-Lücke; `➖` = kein Extra-EL-Port; `📝`/`📋` = Hinweis; `♻️`/`⚠️`/`🔜` = siehe Unused-Tabelle (PORT-PIPE-U*).

| ID | Stage / Gate | op-geth-Regel (Soll) | Reth-Stand (Code) | FLOW-Gate | Verify / Status |
| --- | --- | --- | --- | --- | --- |
| PORT-PIPE-001 | Engine → Pipeline | Tip-Gap → Backfill/Pipeline, nicht endlos Tip-Chase | ✅ `handle_missing_block` Backfill + downloader-first (PORT-ENGINE-001) | E01–E03 ✅ | ✅ **live** (2026-08-11T09:15Z): `backfill` + `Preparing stage Headers`. Tip-Fetch → ENGINE-003/FLOW-E03 |
| PORT-PIPE-002 | Headers | `MilliTimestamp` streng steigend (`mixHash[:2]`) | ✅ `milli_timestamp.rs` + OpBeaconConsensus 204/5611; Unit-Tests | **H01–H05 ✅** | ✅ umgesetzt · ✅ **live** Headers Tip |
| PORT-PIPE-003 | Headers | Wright `baseFee == 0` | ✅ Consensus-Check + `next_block_base_fee` → 0 | H* | ✅ umgesetzt · ⏳ live ungetestet (ab Wright-Höhe) |
| PORT-PIPE-004 | Headers | Pre-Wright EIP-1559 elast=2, denom=8 | ✅ `BaseFeeParams::ethereum()` in `OPBNB_*` | H* | ✅ umgesetzt · ⏳ live ungetestet (Pre-Wright-Range) |
| PORT-PIPE-005 | Bodies | Canyon empty withdrawals; Ecotone `blobGasUsed=0` | ✅ OP `validate_block_pre_execution` / blob-gas=0 | **B01–B04 ✅** | ✅ umgesetzt · ✅ **live** Bodies=Tip @08-12 ~03:02 CEST (s. Live Sync Progress) |
| PORT-PIPE-006 | SenderRecovery | Deposit `from` ohne ECDSA | ✅ OP Deposit-Primitives / Recovery (`OpTransactionSigned::recover_signer` → Deposit.`from`) | **R01 ✅** | ✅ umgesetzt · ✅ **live OK** Tip @15:54 CEST (s. Live Sync Progress) |
| PORT-PIPE-007 | Execution @ Fermat `9397477` | Precompiles `0x66`/`0x67` | ✅ `opbnb_precompiles` Overlay + Flag-Tests | **X01 ✅ Fermat** | ✅ umgesetzt · ✅ **live** Exec≫Fermat; IPC stateRoot MATCH an `9397477`± (s. Live Sync Progress) |
| PORT-PIPE-008 | Execution Haber→Fjord | Early `p256` @ `0x100` nur vor Fjord | ✅ `haber_p256` Flags in `evm/src/config.rs` + Overlay-Tests | **X01 Haber ✅** | ✅ umgesetzt · ✅ **live** Haber Point-4 MATCH (08-17) |
| PORT-PIPE-009 | Execution Wright+ | L1-Fee **nur** wenn `gasPrice==0` → 0 | `factory.rs` setzt `skip_l1_data_fee=true` ab Wright. Das vendorte Workspace-`op-revm` überspringt L1-Kosten nur bei Flag **∧** `gas_price==0`, bewahrt das Flag über `try_fetch` und berechnet post-Isthmus weiterhin die Operatorfee — ≡ op-geth `core/state_transition.go::buyGas`. Frühere Plan-Lesart „skip für alle Txs“ war falsch. Wright-Höhe Mainnet ~**32984677** (`ts=1724738400`). | **X02 ✅** | ✅ fokussierte Units · 🔬 portabler `op-reth`-Build und Live stateRoot @ Wright-Fenster |
| PORT-PIPE-010 | Execution L1-Attr | Snow/Volta/Fourier nur CL → Deposit-Calldata | ➖ Snow erzeugt den Median-L1-Gaspreis im op-node und schreibt ihn in die L1-Info-Deposit-Tx. Volta/Fourier erzeugen Millisekundenzeit plus Fourier-Intervallzähler in `prevRandao[0..4]`; der OP-Engine-Pfad übernimmt diesen unverändert als Header-`mix_hash`, während EL nur monotonen Millisekundenfortschritt prüft. Kadenz-/Span-Batch-Regeln sind op-node-Consensus. | — | ➖ n/a zusätzliche EL-Logik · 📝 CL liefert L1-Info und `prevRandao` |
| PORT-PIPE-011 | MerkleExecute | Root = Execution-Ergebnis | ➖ Generic Stages; kein opBNB-Extra-Port | X03 | ➖ kein Extra-Port · ⏳ live hängt an PIPE-007…009 |
| PORT-PIPE-012 | History / TxLookup | storage.v2 Indices | ✅ Code + Unit (PORT-STOR-007/008) | S01–S02 | ✅ umgesetzt · ⏳ live ungetestet (Archive-Last / SF-Unwind) |
| PORT-PIPE-013 | Testnet only | PreContract @ `5805494` | `OpEvmConfig` setzt am exakten Forkblock `OpBlockExecutionCtx::apply_pre_contract_hardfork`; der vendorte `OpBlockExecutor::apply_pre_execution_changes` mutiert vor allen System-/Nutzer-Txs WBNB Slot 0/1 und selfdestructed das Governance-Predeploy, entsprechend op-geth `StateProcessor`. | — | ✅ Hook, Zustandsmutation und Executor-Transition-Test (WBNB Slots + Governance-Löschung) · 🔬 Testnet-Archive-Verifikation (Mainnet n/a) |
| PORT-PIPE-014 | Execution pre-Canyon | Receipt-**Content**-Parity vs op-geth | Hertz @ `0x67`; Fail war `21591154` | **X04 ✅** | ✅ Fix + `re-execute` ✅ · 🔄 live Exec≫Fail nach Bodies/Sender Catch-up |

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
| PORT-PIPE-U07 | `COMETBFT_LIGHT_BLOCK_VALIDATION_BEFORE_HERTZ` + `…_PASTEUR` in `opbnb_precompiles/cometbft.rs` | Overlay injectiert jetzt **`COMETBFT_LIGHT_BLOCK_VALIDATION`** (Hertz = op-geth). BEFORE_HERTZ/Pasteur = BSC-Era; tot im OP-Pfad. **Früher falsch:** BEFORE_HERTZ ≈ op-geth — op-geth hat immer pre-update `validatorSetChanged` (PIPE-014). | ✅ Overlay-Fix 08-15 · ⚠️ BEFORE_HERTZ/Pasteur tot · 🔜 cfg-gaten/löschen oder nach `reth-bsc` |
| PORT-PIPE-U08 | Unused imports in `optimism/hardforks/src/hardfork.rs` (+ Spiegel `bsc/hardforks`) | `Box`/`format`/`String`/`Display`/`FromStr` nach Macro-Refactor übrig (`maxperf-op` warn). | 🧹 CLEANUP-A01 · kein Port-Gap |
| PORT-PIPE-U09 | `reth-engine-tree`: unused crate dep `reth_trie_prefetch` | Prefetch-Crate hängt noch als Dependency, Code-Pfad entfernt (U06). | ✅ CLEANUP-A02 (08-16): Dep entfernt; `reth-engine-tree` check grün |
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
| **P0** | — | **PORT-EXEC-001 / PIPE-014** offline ✅; **live Exec≫`21591154` ✅** (08-16 ~03:46); FLOW-X05 watch; **PORT-OPS-001** | — | ✅ **P0 live** (Fail durch); Unwind-Sturm weiter beobachten |
| **P0** | — | Execution live → FLOW-X01 Haber + X02/X03 (+ PIPE-008/009) | Fermat ✅; past Fail ✅; Haber Point-4 ✅; past Wright; **X02/PIPE-009 aligned**; Tip ETA ~1¼–2¼ Mo | Optional Wright Point-4 sample; Tip catch-up |
| **P1** | — | **PORT-P2P-002 / FLOW-N02** `--nat any`: UPnP geth-style + konsistente Announce/Logs | ✅ **live** 08-15 ~21:37 (Alt-Ports, hairpin, inbound_conn); Serve-RX noch 0 | DoD Map/ENR ✅; Serve optional post-Bodies |
| **P1** | — | ✅ **PORT-P2P-006 / FLOW-N01** Dual-Stack Default + single-family `--addr` (09-03, `4bbdd60fd6`/`45db221aeb`, Dev-Test verifiziert) | Bind≡Announce dialbar; `--addr` = Familie only (NAT matched family schon) | DoD (a)+(b)+(c) ✅ |
| **P1** | CLEANUP-A02 | Dead crate deps (engine-tree `trie_prefetch`/tree/beacon, engine-local/service/util, payload-builder, prune `rayon`, static-file-types, …) | 🔄 **partial 08-16:** engine-tree, engine-local, engine-service, engine-util, payload-builder, prune, static-file-types bereinigt; Rest (provider/rpc/db/…) offen | betroffene Crates ohne `unused_crate_dependencies` |
| **P1** | CLEANUP-A03 | `reth-provider` unused imports + `chain_spec` field + rocksdb unreachable-pub | fix imports; Feld nutzen/`_`/entfernen; `pub` → `pub(crate)` wo intern | `cargo fix -p reth-provider` clean für unused |
| **P1** | CLEANUP-A04 | PORT-PIPE-U05 orphan `build_pipeline.rs` | Datei löschen **oder** korrekt in CLI verdrahten | nicht mehr unreferenced on disk |
| **P2** | CLEANUP-A05 | PORT-PIPE-U07 CometBFT BEFORE_HERTZ/Pasteur | Overlay nutzt Hertz (op-geth). Entscheiden: (a) BEFORE_HERTZ/Pasteur nach `reth-bsc-evm`, oder (b) im OP-Crate löschen | `reth-optimism-evm` ohne dead_code CometBFT-Warnungen |
| **P2** | CLEANUP-A06 | PORT-PIPE-U10/U11/U16 stages/prune/cli checksum | Toten History/Prune/Checksum-Code entfernen **oder** an storage.v2 anbinden + Test | keine dead_code auf genannten Symbolen **oder** nextest-Beleg |
| **P2** | CLEANUP-A07 | Trivial unused imports (payload-primitives, config, eth-wire, txpool, rpc, node-builder, stages HeaderTy, …) | `cargo fix` sweep gezielt | Warning-Anzahl spürbar ↓ |
| **P3** | CLEANUP-B | Legacy tree/beacon: `blockchain-tree-api` deprecated types; beacon stub dead fields; engine-tree deps auf tree-api | Migrate Callers → `RecoveredBlock` / Engine-Events; unnötige deps streichen | keine `SealedBlockWithSenders` Warnungen in Hot-Crates **oder** crate `allow`/entfernen dokumentiert |
| **P3** | CLEANUP-C | Docs/cfg noise: `missing-docs` (primitives-traits Maybe*, rpc-types-compat, provider writer); `unexpected_cfgs` mdbx in db-api; `missing-debug-implementations` beacon | Docs ergänzen **oder** lint-Allow nur lokal; Feature `mdbx` in db-api oder cfg entfernen | `maxperf-op` ohne missing-docs/unexpected-cfgs in Fork-touched files |
| **P3** | CLEANUP-D | Deprecated API surface: `PruneSegment::Headers/Transactions`, revm `gas_used`, ringbuffer `push`, rpc-types-compat aliases | Upstream-Ersatz nutzen wo billig | Warnungen weg oder bewusst `#[allow(deprecated)]` mit Ticket |
| **P4** | CLEANUP-E | Workspace hygiene | ✅ public `main`: top-level `files/` komplett aus Git/History entfernt und via `.gitignore` blockiert; nur `.github/workflows/op-reth-build-smoke.yml` bleibt | Repro: `git ls-tree -r --name-only main files` = leer |

**Exit-Kriterium „final cleanup done“:**  
1) `make maxperf-op` (bzw. gleicher `cargo build --profile maxperf … --bin op-reth`) **ohne** `unused`/`dead_code` in `crates/optimism/**` und ohne neue Konsens-Diffs;  
2) Workspace-Warnungen dokumentiert (Baseline) oder auf Upstream-Parity;  
3) PORT-PIPE-U* alle entweder gelöscht, verdrahtet, oder als `📝 by design` abgehakt.

## Feature-Requests (nicht Port-Regressions)

Geplante Produkt-/Explorer-Fähigkeiten. **Start erst nachdem PORT-P2P-001 live belegt ist** (`net_peerCount>0`, eth-Session zu Peers stabil) — Sync/P2P vor Index-Disk-Kosten.

Quelle / Bench (kanonisch):
`/usr/src/Erigon/Ethereum-MEV-BOT/Analysis/reth-vs-erigon-history-index-gap-2026-08-10.md`
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
| Cursor Composer YOLO (Session 6, Chat `42f88fe7…`, Snapshot **2026-08-09 12:05 UTC**) | 06:45 – ~12:05 UTC (**~5,34 h** Wall) | **composer-2.5-fast** (4.986 `modelName`-Hits) + **cursor-grok-4.5-high-fast** (178); Parent `default` | **Kein lokaler Billed-Token-Ledger.** Content-Proxy: Transcripts ~2,34M chars ≈ **~0,58M Tokens** (÷4); Cleartext-Chat-JSON ≈ **~0,33M Tokens** (Untergrenze, Tool/Context unterzählt). Erwartete billed/context-Wiederholung deutlich höher | (Proxy, s. Input-Spalte) | **15 Agents** (1 Parent + 14 Subs); 2.582 Assistant-Msgs; 5.861 Tool-Blobs; ~11.722 Tool-Calls; 74.482 `ai_code_hashes` | **`reth-bsc-node --features bsc` + workspace `--no-default-features` grün**; Phase-4 op-forks/chainspec/primitives/consensus; Detailmetriken lokal archiviert, nicht publiziert |
| Cursor Session 8 (Chat `d6ebb428…`, Snapshot **2026-08-09 ~14:30 UTC**) | ~12:18 – ~14:25 UTC (**~2,1 h** Commit-Span; ~1,4 h Chat-Wall) | Auto/Composer (kein per-request Model-Ledger im Transcript) | Transcript-Proxy **~0,11M Tokens** (÷4); billed meter n/a | (Proxy) | ~816 Tool-Calls; 11.288 `ai_code_hashes`; 350 assistant / 18 user msgs | op-evm→payload/rpc/node/cli/`op-reth` grün; opBNB init+RPC smoke; nextest chainspec/forks 23/23; Detailmetriken lokal archiviert, nicht publiziert |
| Cursor Session 9 (Chat `6a6455c9…` + Vorabend `9be255b9…` PORT-STOR-006, Snapshot **2026-08-10 ~08:30 UTC**) | Vorabend SCS-Port unterbrochen; Resume **05:57–~08:30 UTC** (**~2,5 h** Chat-Wall inkl. EF-Rootcause); Commit-Span **06:06–~08:27 UTC** | Auto/Composer + Task-Subagents (inherit); kein per-request Model-Ledger | Transcript-Proxy kombiniert **~97K+** Tokens (÷4, früher Snapshot ~97K; Session fortgesetzt); billed meter n/a | (Proxy) | Resume früh: 12 user / 118 assistant; **250** tool_use; danach EF-Deep-Dive (Bytecode Compact) | **PORT-STOR-006**; stages **106**; op-stack nextest; EF **v17.0** + Compact-Fix → **61/62** suites; Detailmetriken lokal archiviert, nicht publiziert |
| Cursor Session 10 cont. (Chat `84eb0b61…`, Snapshot **2026-08-11 ~16:50 UTC+2**) | Live-Sync P2P-003/004/005: **~12:00–16:50** (**~4,8 h** Wall) inkl. Nachziehen der Dataflow-Lücken + **3× maxperf-op** (~20–23 min/Link, JOBS=1) | Auto/Composer | Transcript-Proxy n/a | (Proxy) | Matrix-Soll Tip-Resolve/Cap/Falling (Analyse nachgezogen); eth/69; Unit-Tests; Live-Verify | **P2P-003/004/005 live ✅** Falling @~22k hdr/s. Rebuilds: eth69~23 min, Cap~20 min, Falling~21 min. Tests: fetch 43/43, reverse_headers 11/11. ETL-TempDir = Upstream-Design |
| Cursor Session 12 (Chat `ea987bef…`, Snapshot **2026-08-15 ~10:54 CEST**, kumulativ 08-12→15) | Kalender **~66,5 h** (08-12 16:27–08-15 10:54); **6** Interaktiv-Cluster **~4,5 h** Span (Gap>90 min; +Pad ≈**~6 h**) | Auto/Composer (+1 Task) | Transcript-Proxy: Msg-Text **~72 K** Tok (÷4); File **~216 K** Tok (÷4); billed n/a | (Proxy) | **84** user / **367** asst; **567** tool_use (Shell 219, Read 113, StrReplace 108, Grep 93); Detailmetriken lokal archiviert, nicht publiziert | **EXEC-001** open; PIPE-014/X04/X05; Harness+dump-flag; OPS-001/ENGINE-004; Cap Bodies/Sender; offline X04 Exec `20365614→21591153`; SF≠Cap dokumentiert |
| Cursor Session 12 cont. (Teil-Snapshots 08-13…08-15) | s. Cluster in Metrics-JSON | Auto/Composer | (in kumulierter Zeile) | (Proxy) | Fail#1–3; Tip-Rettung; Cap; offline X04/SF-Heal; CLI inkl. vs half-open | Dump `re-execute 54..55` nach Exec-fertig |
| Cursor Session 12 cont. (Chat `ea987bef…`, Snapshot **2026-08-15 ~11:47 CEST**) | op-geth↔Reth Root-Pipeline-Doku | Auto/Composer | (Session-12-Proxy) | (Proxy) | ValidateState eager vs Exec+Merkle staged; alloy-op-evm Path-Dep; PIPE-014 bleibt Content | FLOW-X04 Dump; Merkle später |
| Cursor Session 12 cont. (Chat `ea987bef…`, Snapshot **2026-08-15 ~14:20 CEST**) | PIPE-014 Hertz-Fix + Verify + Live Restart | Auto/Composer | (Session-12-Proxy) | (Proxy) | FLOW-X04 idx=10 `syncLightBlock`; Overlay Hertz; `re-execute` ✅; maxperf→`dist/bin`; live Bodies Catch-up | Live Exec≫`21591154`; FLOW-X05 watch |
| Cursor Session 12 cont. (Chat `ea987bef…`, Snapshot **2026-08-16 ~08:35 CEST**) | UPnP+Bodies Tip+Exec past Fail+X02+A02; Kalender 08-12→16 **~88 h**; Interaktiv +~4 h (Abend 15 + Morgen 16) | Auto/Composer | Transcript File **~1.58 MB** → Proxy **~396 K** Tok (÷4); billed n/a | (Proxy) | jsonl **~1063** lines; Detailmetriken lokal archiviert, nicht publiziert | **P2P-002** UPnP ✅; H/B/S Tip; Exec≪Tip past Fail; **X02 ✅**; CLEANUP-A02 partial; Roadmap ETAs |

> Hinweis: Copilot-Token-Zahlen sind kumulative Modellaufrufe inkl. Tool-Nutzung/Kontext-Wiederholung pro
> Turn. Cursor speichert hier **keinen** äquivalenten `assistant_usage_events`-Zähler (Chat-Blobs teils
> verschlüsselt) — daher Activity-Counts + Content-Size-Proxies. Kein Effizienz-Benchmark.
> **Kosten (illustrativ, kein Invoice):** Copilot `a95758da` allein ~650M in / ~1,9M out ≈ **USD 1,5–2k**
> bei öffentlichen Sonnet/GPT-Listenpreisen ohne Cache-Rabatt; **Cursor Session 12** nur Proxy
> (~72 K–388 K Tok Content-Proxy über Snapshots, **~4,5 h** früh + **~4 h** 08-15/16) — **billed** nur Account-Dashboard /
> Abo (Context-Resend ≫ Content-Proxy). Quellen: lokale Copilot-/Cursor-Sessiondaten; die früheren
> `files/`-Metrikartefakte sind bewusst nicht mehr Teil der öffentlichen Git-History.

## Nächste Schritte (unmittelbar — Stand 2026-08-20)

> Historische Compile-Loop-Liste (Session `a95758da`, 2026-08-06) ist erledigt und bleibt unten
> im Session-Protokoll. **Aktuelle Priorität = Roadmap (Exec-Fenster)** oben.

1. **Live unsupervised:** Execution weiterlaufen lassen — **kein Restart**; FLOW-X05 Unwind-Watch.
2. **Optional:** Point-4 stateRoot-Stichprobe im Wright-Fenster (Höhe bereits passiert).
3. **diese Woche (parallel, kein Live-Restart):** CLEANUP-A02 Rest + A03/A04.
4. **bei geplantem Restart:** P2P-006 Dual-Stack; optional Serve-RX / ENGINE-004.
5. **nach Tip (~1½–2½ Mo @ current rate):** Merkle/History-Gates; op-reth `download`/`snapshot-manifest`; FEAT-HIST-*.
6. Nach Meilensteinen: `plan.md` Todo/Roadmap + README Effort-Log + Metrics-JSON nachziehen.

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
5. **PORT-PIPE live:** 001–008/012/014 weitgehend ✅ (014 live past Fail). **009/X02 ✅** Code≡op-geth (Unit); Live Point-4 @ Haber/Wright. Unused: **U01–U17**; mechanisch: **CLEANUP-A…E** (A02 partial 08-16).
6. Human Archive Sync: Exec Tip (~3–4 Wo) → Merkle/History; Spot-Checks unsupervised.
7. **Danach FEAT-HIST-001:** History-/Explorer-Indizes → Erigon-Parität (Gate: stabiler Sync).
8. **Final cleanup** laut CLEANUP-* (A02 Rest parallel ok; große Semantik-Änderungen erst nach Tip).
9. **P2P-006** + ENGINE-004 nur bei geplantem Node-Restart.
10. **op-reth `download` / `snapshot-manifest`:** verdrahten nach Tip (Bootstrap), nicht während Exec.

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

**Live Archive (parallel):** s. **Live Sync Progress** — H/B/S Tip ✅; Exec ~37 M↑ (~21 %) past Haber/Wright; ETA Tip ~1½–2½ Mo; X02 ✅; Haber Point-4 ✅; P2P-002 ✅; P2P-006 offen; CLEANUP-A02 partial.

### Session 12 — Receipt-Root Fail / Unwind / Harness Binary (2026-08-13 → 08-15)

**Chat:** `ea987bef…` · Gates: **PORT-PIPE-014** + **PORT-FLOW-X04/X05** · **PORT-OPS-001** · **PORT-ENGINE-004**.

**Aufwand (Snapshot 08-16 ~08:35 CEST):** Kalender 08-12→16 **~88 h**; Interaktiv früh **~4,5–6 h** + Abend 15 / Morgen 16 **~4 h**; Transcript **~1.58 MB** → Proxy **~396 K** Tok; billed n/a → `files/cursor-session12-metrics.json` + `…-20260816.json`.

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
| Journal | `journalctl -D /var/lib/machines/<archive-ct>/var/log/journal/` |
| Upstream-Lage | Trail **2.4.1**; kein Sprung auf 2.5 bis op/bnb catch-up |

**Parked:** PORT-DEV-001 LocalMiner · PORT-ENGINE-004 Shutdown-Panic.

### Live Sync Progress — opBNB Archive (`<archive-ct>` / `op-reth-bnb`) {#live-sync-progress}

**Stichprobe:** 2026-08-20 **~10:28 CEST** · Execution past Haber/Wright · chain **204** · peers **16**

| Stage | Checkpoint / Target | Status |
| --- | ---: | --- |
| Headers | **174 027 661** | ✅ Tip (parked; public ~**176.4 M**) |
| Bodies | **174 027 661** | ✅ Tip; validation_errors **0** |
| SenderRecovery | **174 027 661** | ✅ Tip |
| Execution | **`65 828 907`** / Tip **174 M** (~38 %) | 🔄 past Fail/Haber/Wright; ~19–33 blk/s cooled; **ETA Tip ~1¼–2¼ Mo** |
| MerkleExecute | **0** | ⏳ nach Exec Tip |
| History / Finish | — | ⏳ |
| P2P NAT/UPnP | FLOW-N02 / P2P-002 | ✅ Alt-Ports, `via_upnp=true`; Serve-RX 0 |
| P2P Dual-Stack | FLOW-N01 / P2P-006 | 📋 Default Dual-Stack noch offen |

Metrics: `files/opbnb-archive-sync-snapshot-20260820.json`.

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
| FLOW-X04 first mismatch | **idx=10** · `syncLightBlock(bytes,uint64)` → `0x67` · gas public **717672** / local **259171** · status=1 logCount=1 beide |
| Root cause / Fix | Overlay `BEFORE_HERTZ` vs op-geth Hertz `validatorSetChanged` · → `COMETBFT_LIGHT_BLOCK_VALIDATION` |
| Verify | **`re-execute` ✅** 08-15 ~14:13 CEST (54..55, kein Dump) · maxperf `dist/bin/op-reth-bnb` |
| Fixture / Harness | `files/receipts-21591154-public.json` · `files/harness-receipt-diff-21591154/` (+ `tx10.json`) |

#### op-geth vs Reth — State-/Receipt-Root in der Pipeline (08-15)

Referenz: `bnb-chain_op-geth.git` FullSync `InsertChain` → `ValidateState`; Trail: Execution → … → MerkleExecute. Path-Dep Receipt-Build: `optimism.git/rust/alloy-op-evm` (`block/mod.rs` deposit fields + `strip_deposit_nonce`).

| | **bnb op-geth FullSync** | **reth-trail staged** |
| --- | --- | --- |
| Wann Receipt-Root | Sofort in `ValidateState`: `DeriveSha` / `EncodeIndex` (`core/block_validator.go`, `types/receipt.go`) | Sofort in **Execution**: `calculate_receipt_root_optimism` (`proof.rs`; Regolith∧¬Canyon strippt `deposit_nonce`) |
| Wann State-Root | Sofort: `statedb.IntermediateRoot` vs `header.Root` (gleiches `ValidateState`, außer `skipRoot`) | **Später** in **MerkleExecute** über hashed tables vs Header — Execution prüft **kein** `stateRoot` |
| Modell | In-Memory StateDB + MPT pro Block | Bundle/Changesets → Plain(+hashed v2) → Hashing → Merkle |
| Fail @ `21591154` | würde Receipt in `ValidateState` failen | failt in Execution → Merkle nie erreicht |
| Live Fail #3 Cap | — | Merkle state-root @ dirty Cap (OPS-001), nicht IntermediateRoot-Formel |

**Semantik die State-Root später drehen kann (nicht Ursache von `21591154`):** L1-fee → `OptimismL1FeeRecipient` (pre-Wright immer); **PIPE-009/X02 ✅:** Wright `skip_l1_data_fee` + op-revm `gas_price==0` ≡ op-geth; Deposit gas/tip; Fermat/Haber precompiles; Snow nur via CL L1-price → L1 attributes (nicht Receipt-RLP). L1-Fee-**RPC-Felder** gehören nicht in den Receipt-Trie.

**Folgerung:** Staging-Unterschied erklärt nicht `got≠expected` Receipts. DoD bleibt FLOW-X04 Receipt-Content-Diff; danach MerkleExecute ≈ op-geth-`IntermediateRoot`-Gate.

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

**Ops (verbindlich bis live Exec ≫ `21591154`; PIPE-014 offline ✅):**

1. **Stabilster Park vor Bad-Block:** Process **stop** (nicht Reload), bevor Execution den Fail-Block anfasst — gilt weiter bei neuem Receipt-/Root-Fail.
2. **`--debug.max-block <H>`** nur für **Clean-Rebuild** wenn alle relevanten Stage-Checkpoints **≤ H** (sonst Skip → PORT-OPS-001). Optional `--debug.terminate`.
3. **`--debug.skip-fcu`** ist **kein** Höhen-Stop.
4. Offline Harness: Bodies/Sender→`21591154`, Exec→`21591153`, dann `re-execute --from 21591154 --to 21591155 --dump-receipts-on-fail` → `diff_receipts.py` (post-Fix: kein Dump erwartet).
5. Exec `--from` = **ChangeSets-SF tip**, nicht Bodies-Cap (sonst `missing static file data`).
6. Journal: `journalctl -D /var/lib/machines/<archive-ct>/var/log/journal/`.
7. Headers-Unwind: Journal ohne Batch-Progress ≠ Hang — Fortschritt an `reth_static_files_jar_provider_calls_total{…init-cursor}` / CPU messen.
8. Point4/RPC: Live-Node hat **nur IPC** (`--ipcpath /tmp/<archive-ct>.ipc`); HTTP erst mit `--http`. Raw JSON-RPC über Unix-Socket.

#### Health / Anomalien (~10:28 08-20)

| Check | Befund |
| --- | --- |
| Fail-Block `21591154` live? | ✅ **durch** — Exec ≫ Fail seit 08-16; PIPE-014 Hertz bestätigt |
| Bodies / Sender | ✅ Tip **174 027 661**; body validation **0** |
| Execution | 🔄 **`65 828 907`** / Tip **174 M** (~38 %); past Haber/Wright; ~19–33 blk/s; **ETA Tip ~1¼–2¼ Mo** |
| X02 / PIPE-009 | ✅ Code ≡ op-geth (Unit); Haber Point-4 ✅; Wright height passed |
| Headers Tip | ✅ **174 027 661** (parked; public ~176.4 M) |
| P2P UPnP / Announce | ✅ **P2P-002**; 📋 **P2P-006** offen |
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
| Bodies Catch-up (Tip) | 08-15 ~19→~22:42 | ~3 h | Tip **174 027 661** ✅ |
| SenderRecovery (3.) | 08-15 ~22:42→08-16 ~03:38 | ~5 h | Tip **174 027 661** ✅ |
| Execution (past Fail → Tip) | 08-16 ~03:38→ | 🔄 | Floor **`21591153`** → Tip **174 M**; past `21591154` ✅; ETA ~3 Wo |

#### Network usage (CT `<archive-ct>:9100`, `node_network_*`)

| Phase | RX (typ.) | TX (typ.) | Notes |
| --- | --- | --- | --- |
| Headers Falling (08-11) | ~**25–60 Mbit/s** | — | P2P Header-Batches |
| Bodies | ~**140–200 Mbit/s** | niedrig | Peak ~200 Mbit/s |
| SenderRecovery | ~**0.5–0.8 Mbit/s** | ~**0.7–0.9 Mbit/s** | CPU-lokal |
| Execution (bis Fail) | ~**0.6 Mbit/s** | ~**0.7 Mbit/s** | peers~12–13 |

Kein `reth_network_*_bytes` auf `:6060` — Bandbreite über CT-Exporter `:9100`.

### Session 14 — opBNB Peer-Connectivity-Degradation + migrate-v2 Dev-Host-Validierung (2026-09-03)

**Symptom:** Live-Archive-Node fiel von historisch 8–17 auf konstant **5 connected_peers**.
Bootnode-Fix (`crates/net/peers/src/optimism.rs`) aus früherer Session gegen offizielle
opBNB-ENRs (`bnb-chain/opbnb#105`) re-verifiziert — Pubkeys stimmen exakt überein, Fix bleibt
korrekt. Kein offizieller statischer opBNB-Peer-Discovery-Mechanismus existiert (Design-Lücke,
seit 2024 unadressiert laut `bnb-chain/opbnb#105`/`#310`).

**ForkHash-Korrektur:** Der reale opBNB-Netz-ForkHash ist **`45eac6aa`** (ENR-Key `"eth"`), *nicht*
`716d4a3a` — letzterer ist `NetworkStackId::OPEL` (`"opel"`, `crates/net/discv5/src/network_stack_id.rs`),
unser **eigener**, vom aktuellen Sync-Stand abhängiger Pre-Canyon-ForkHash. EIP-2124
(`alloy-eip2124` `ForkFilter::validate()`) akzeptiert Peers mit `45eac6aa` korrekt via Regel 3
(remote = bekannter zukünftiger Fork) — kein Bug.

**Peer-Injection-Tool:** `.cursor/local/opbnb-peer-inject.py` (gitignored) erstellt — liest
`admin_peers`/schreibt `admin_addTrustedPeer` roh über den IPC-Unix-Socket (kein HTTP-Framing
verfügbar). 11 verifizierte ForkHash-`45eac6aa`-Peers aus `reth.log`-Traces extrahiert; 6 davon
neu. Live-Injection zeigte: 4× `Disconnected(TooManyPeers)` (Gegenstelle gesund, aber
Slot-limitiert), 2× `ECIES UnreadableStream`.

**Reachability-Isolationstest** (`op-reth-bnb p2p body`, Scratch-Datadir, `--trusted-only
--disable-discovery`): Von diesem Dev-Host aus scheiterten alle 6 Kandidaten bereits am
ECIES-Handshake (nie `TooManyPeers`) — härterer Fehlermodus als beim Live-Node. Kontrolltest mit
einem der 5 **verbundenen** Peers (`167.235.95.170:30305`) gelang sofort (`Session established`
+ `Successfully downloaded body`) → bestätigt: Tool/Setup/Netzwerkpfad des Dev-Hosts sind
grundsätzlich funktionsfähig; das Scheitern ist spezifisch auf die 6 Kandidatenhosts beschränkt
(Überlastung/Reputation), kein generelles Konnektivitätsproblem.

**Automatisierung:** `opbnb-peer-inject.service` (oneshot) + `opbnb-peer-inject.timer`
(`OnBootSec=5min`, `OnUnitActiveSec=20min`, `Persistent=true`) auf dem Dev-Host als
root-systemd-Units angelegt und aktiviert (`systemctl enable --now opbnb-peer-inject.timer`) —
retriggert periodisch `admin_addTrustedPeer` für die 6 kapazitätslimitierten Kandidaten, da deren
Slots sich jederzeit freimachen können. Nur der Timer ist enabled (`static` Service, kein eigenes
`[Install]`, läuft ausschließlich getriggert).

**migrate-v2 End-zu-Ende-Validierung (Dev-Host, isoliert):** Frischer `op-reth-bnb`-Node mit
`--storage.v2 false` + `--debug.tip <hash Block 300>` + `--debug.terminate` bis Block 300
gesynct (V1-Layout bestätigt: `Loaded storage settings settings=StorageSettings { storage_v2:
false }`). Anschließend `db migrate-v2` ausgeführt: ChangeSets → Static Files,
3 MDBX-Tabellen → RocksDB, `StorageSettings` → v2, Recompute-Tabellen geleert, MDBX kompaktiert —
**keine Fehler**. Neustart nach Migration bestätigte `storage_v2: true`; Pipeline baute alle
geleerten Daten (SenderRecovery, MerkleExecute@100%, Hashing) korrekt neu auf.
`stage-checkpoints get` zeigt alle 13 Stages konsistent @ Block 300. Kein Repro des
`71185159`/`71185160`-Vorfalls (fehlende Receipt-SF, canonical-root-Mismatch) bei diesem kleinen,
sauberen (nicht unterbrochenen) Testlauf — Crash-Resume-Semantik von `migrate-v2` bleibt
weiterhin ungetestet (siehe PORT-FLOW-S04).

### Session 15 — PORT-P2P-006 / FLOW-N01 Dual-Stack live-verified (2026-09-03)

**Fund:** Die zwei zuvor gemergten Commits `4bbdd60fd6` ("true dual-stack discv5 + NAT announce
when --addr is unset") und `45db221aeb` ("reconcile discv5's own UDP listen port in dual-stack
NAT announce") lösen `PORT-P2P-006`/`FLOW-N01` bereits vollständig — der Plan-Status war nur
veraltet (noch `📋 todo`).

**Live-Verifikation (Dev-Host, isolierter Datadir, kein `--addr`):** Kernbefunde bestätigt:
- `discv5::service: Discv5 Service started mode=DualStack`
- Tatsächliche UDP-Bindings auf **beiden** Familien: `0.0.0.0:9200` **und** `[::]:9200`
- NAT-Announce deckt beide Familien getrennt ab: `Announced dialable enode` (IPv6, primär) +
  `Announced additional discv5 dual-stack NAT endpoint` (IPv4, via UPnP)
- RLPx-TCP-Bind folgt ebenfalls `listen_ip=::` (IPv6-Wildcard, OS-Dual-Stack), discv4 bleibt wie
  spezifiziert single-stack (kein zweiter discv4-Socket)

Damit sind alle drei DoD-Kriterien erfüllt: (a) kein `--addr` → Dual-Stack + dialbare Announce je
Familie, (b) `--addr` weiterhin single-family, (c) keine undialbare Familie wird announct.

**Live-Node-Status unverändert:** Der Live-Archive-Node läuft weiterhin mit explizitem
`--addr 0.0.0.0` (systemd `ExecStart`) — bindet also bewusst single-family (IPv4-only), der neue
Dual-Stack-Pfad ist dort nicht aktiv. Der Fix wurde ausschließlich in einem isolierten Dev-Host-Test
verifiziert, nicht am produktiven Node.

> **Korrektur (Session 16):** Diese Annahme war falsch — der Live-Node läuft tatsächlich **ohne**
> `--addr` (siehe reales `ExecStart` unten) und damit **bereits im Dual-Stack-Pfad**. Das führte
> live zu genau dem in Session 16 gefundenen und gefixten Folgebug.

### Session 16 — PORT-P2P-006 Folgebug: UPnP-Familie im Dual-Stack-Pfad falsch behandelt (2026-09-03)

**Live-Journal-Fund (Live-Node, `BlockChain.service`, tatsächlicher `ExecStart` ohne `--addr`):**
```
NAT mapped alternative UPnP port(s) external_ip=193.81.225.224 ... local_ip=10.0.0.85
WARN UPnP mapped IP family does not match --addr listen family; ignoring mapping listen_ip=:: mapped_ip=193.81.225.224
Resolved public IP via HTTP (no UPnP port mapping) ip=2001:871:25c:8ad:6a05:caff:fea6:103 listen_ip=::
Announced dialable enode after NAT resolution enode=...@[2001:871:25c:8ad:6a05:caff:fea6:103]:30303 via_upnp=false
WARN UPnP NAT mapping failed; falling back to HTTP public IP without port mapping err=UPnP gateway search failed: No response within timeout
Resolved public IP via HTTP (no UPnP port mapping) ip=193.81.225.224 listen_ip=0.0.0.0
Announced additional discv5 dual-stack NAT endpoint ip=193.81.225.224 tcp_port=30303 udp_port=30303 via_upnp=false
```

**Root Cause:** `resolve_nat_endpoint()` (`crates/net/nat/src/lib.rs`) is called once per family in
dual-stack mode with `listen_ip=::` (primary/IPv6) and `listen_ip=0.0.0.0` (secondary/IPv4). The
old family-match check (`endpoint.ip.is_ipv4() != want_ipv4`) discarded the perfectly good IPv4
UPnP mapping obtained during the *primary* (`::`) call — because UPnP/IGD only maps IPv4 (no
consumer router exposes an IPv6 IGD; globally routable IPv6 isn't behind NAT to begin with, at
most a stateful firewall pinhole). This forced the *second* (IPv4) call to redo an independent SSDP
gateway search from scratch, which is inherently flaky (multicast discovery) and failed with
`No response within timeout` here — falling back to announcing the raw, **unmapped** listen port
(`30303`) as if it were externally reachable, when in fact no port-forward exists for it.

Also flagged (same live log): `Announced dialable enode after NAT resolution` /
`Updated discv5 ENR with NAT endpoint` for the plain global-IPv6 HTTP-resolved address is a
misnomer — no NAT/UPnP was actually involved for that IPv6 leg, it's just public IP resolution.

**Fix:**
- `crates/net/nat/src/lib.rs`: `resolve_nat_endpoint()` now only attempts UPnP/IGD when the target
  family is IPv4 (`want_ipv4`); IPv6 targets always skip straight to HTTP/interface resolution.
  Removes the family-mismatch-discard branch entirely (structurally impossible now) and its
  misleading warning.
- `crates/net/network/src/manager.rs` / `crates/net/discv5/src/lib.rs`: log wording no longer
  unconditionally claims "NAT resolution"/"NAT endpoint" — the pre-existing `via_upnp` field
  already conveys whether real NAT/UPnP mapping happened.

**Verification (dev-host, `dist/bin/op-reth-bnb` built via `make maxperf-op`, no `--addr`):**
```
INFO net::nat: Resolved public IP via HTTP (no UPnP port mapping) ip=<global-ipv6> listen_ip=::
INFO net: Announced dialable enode enode=...@[<global-ipv6>]:30303 via_upnp=false
INFO net::nat: NAT mapped alternative UPnP port(s) external_ip=193.81.225.224 ... tcp_ext=54099 udp_ext=33431
INFO net::nat: NAT mapped additional UPnP UDP port external_ip=193.81.225.224 udp_ext=9200
INFO net: Announced additional discv5 dual-stack endpoint ip=193.81.225.224 tcp_port=54099 udp_port=33431 via_upnp=true
```
No family-mismatch warning, no wasted/failed second SSDP search, IPv4 leg now correctly UPnP-mapped
(`via_upnp=true`) instead of falling back to an unmapped announce. `cargo test -p reth-net-nat`
green (5 passed, 2 ignored network tests).

**Not yet done:** live-node binary not redeployed (deploy/restart is user-controlled); this fix is
built (`dist/bin/op-reth-bnb`, maxperf) and dev-host-verified only, pending the user's own restart
of `BlockChain.service`.

**Update — live-deployed and confirmed (same day):** User rebuilt+restarted `BlockChain.service`
with the fixed binary. Live journal confirms the fix end-to-end:
```
INFO Resolved public IP via HTTP (no UPnP port mapping) ip=<global-ipv6> listen_ip=::
INFO Updated discv5 ENR with dialable endpoint ip=<global-ipv6> tcp_port=30303 udp_port=9200
INFO Announced dialable enode enode=...@[<global-ipv6>]:30303 via_upnp=false
INFO NAT mapped alternative UPnP port(s) external_ip=193.81.225.224 tcp_ext=65285 udp_ext=64591
INFO NAT mapped additional UPnP UDP port external_ip=193.81.225.224 udp_ext=62383
INFO Updated discv5 ENR with dialable endpoint ip=193.81.225.224 tcp_port=65285 udp_port=62383
INFO Announced additional discv5 dual-stack endpoint ip=193.81.225.224 tcp_port=65285 udp_port=64591 via_upnp=true
```
No family-mismatch warning, no failed second SSDP search — single clean UPnP mapping for IPv4.
`ss -tulpen` on the live host confirms both UDP sockets bound (`0.0.0.0:9200` + `[::]:9200`,
`0.0.0.0:30303`) and the single dual-stack TCP listener (`*:30303 v6only:0`). 5 known peers
reconnected normally, sync (Bodies stage) resumed without disruption. `PORT-P2P-006`/`FLOW-N01`
dual-stack fix is now confirmed correct **both** in an isolated dev-host test **and** in live
production.
