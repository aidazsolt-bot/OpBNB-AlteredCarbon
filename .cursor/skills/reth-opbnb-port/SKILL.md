---
name: reth-opbnb-port
description: >-
  MUST load at every session start in this repo (alwaysApply rule + sessionStart
  hook). Portierungs-Spezialist für Reth/opBNB-Rebases und Protokoll-Nachführung
  gegen bnb-chain/op-geth und Upstream reth. Nutzen bei jeder Aufgabe in diesem
  Workspace; besonders Merge/Rebase, Hardfork/Consensus/EVM, Pipeline/Engine-Sync,
  PORT-*/PORT-PIPE-*/PORT-FLOW-*, Live-Verify gegen op-geth.
---

# Reth / opBNB Portierungs-Spezialist

> Projekt-Skill: `.cursor/skills/reth-opbnb-port/`. Entstanden aus Session-10-Hinweisen,
> weil generische Coding-Agents ohne diese Vorgehensweise keine sinnvolle Protokoll-Portierung
> leisten (Compile-Grün ≠ Sync-Korrekt).
>
> **Session-Start (zwingend):** Rule `.cursor/rules/reth-opbnb-port-mandatory.mdc`
> (`alwaysApply: true`) + Hook `sessionStart` → dieses File **und**
> `.cursor/skills/rust-best-practices/SKILL.md` laden/befolgen, bevor sonstige Arbeit beginnt.

## Erster Schritt (nicht verhandelbar)

**Bevor** Compile-Loops, Tip-Debug, Einzel-Fixes oder Live-Restarts:

1. `plan.md` → Abschnitt **Migrations-Gate (verbindlich) — Zwei Matrizen** lesen.
2. Für die nächste Stage **beide** Matrizen nennen:
   - `PORT-PIPE-00x` (op-geth-Konsensregel)
   - `PORT-FLOW-*` (Zustandsautomat / Wire / Persistenz)
3. Fehlt FLOW für die Stage → **zuerst** FLOW-Zeilen + Invarianten anlegen, **dann** Code/Live.
4. DoD aus `plan.md` (Referenz, Matrix, Invariante, Transition-Test, Live-Kriterium, plan-Update).

`PORT-PIPE` allein reicht **nicht**. Cap-Idempotenz / Falling-Prime (FLOW-H03/H04 = P2P-004/005)
waren Analyse-Soll im Downloader-Dataflow — keine „Live-Folgebugs“.

## Wann laden

- **Immer bei Session-Start in diesem Repo** (nicht optional; Rule + Hook erzwingen das), zusammen mit `rust-best-practices`
- Zusätzlich explizit: opBNB/OP-Stack-Port, Rebase, Diff gegen `bnb-chain/op-geth` / `bnb-chain/opbnb`
- Live-Sync hängt (Headers/Bodies/Execution/Merkle, Engine FCU, Peers, Grafana Stages)
- Eintrag/Update von `PORT-*` / `PORT-PIPE-*` / `PORT-FLOW-*` in `plan.md`

## Kernprinzip (nicht verhandelbar)

1. **Zwei Matrizen.** PIPE = Regel; FLOW = Automat. Beide in `plan.md` — nicht optionaler Nachtrag.
2. **Referenz zuerst, Code danach.** op-geth Datei:Zeile **oder** Reth-Callgraph mit benannten Zuständen.
3. **Compile/nextest grün beweist keine Chain-Korrektheit.** Live erst nach FLOW+PIPE-DoD.
4. **Symptom → Layer → Regel/FLOW → Diff → Fix → Verify.** Nicht random refactors.
5. **Human-owned:** Catch-up/Full-Sync und lange Läufe startet der Operator; Agent höchstens Boot-Smoke / gezielte Log-/Mimir-Analyse.
6. **Kein Live als Analyseersatz.** Stall/Ban/`total=1` → fehlende FLOW-Zeile nachziehen, nicht „Folgebug“ taufen.
7. **Live Archive Sync aktiv:** Rule `.cursor/rules/opbnb-live-sync-health.mdc` — periodische Health-Checks (Mimir+Logs; Hash-Stichprobe; **Point 4 stateRoot sobald Execution > 0**).

## Pflicht-Referenzen (dieser Workspace)

| Quelle | Pfad / Hinweis |
|--------|----------------|
| Arbeitsprotokoll + Matrizen | `plan.md` (*Migrations-Gate*, PIPE, FLOW, Bugliste) |
| Fork-Zweck / Methodik / Effort | `README.md` → *About This Fork* |
| Reth-Fork | Workspace-Root (Branch `rebase/reth-v2.4.1`) |
| op-geth (EL-Konsens) | `/usr/src/Erigon/Binance/bnb-chain_op-geth.git` |
| opbnb (CL/Derivation) | `/usr/src/Erigon/Binance/bnb-chain_opbnb.git` |
| Upstream-Architektur | `paradigmxyz/reth` v2.4.1 Patterns (`engine-tree`, `DefaultStages`, downloaders) |

## Vorgehensweise (Checkliste)

### A. Matrizen öffnen (immer zuerst)

1. `plan.md` → *Migrations-Gate* + *PORT-FLOW* + *PORT-PIPE*.
2. Offene `PORT-PIPE-00x` **und** `PORT-FLOW-*` für die nächste Stage auflisten.
3. Bodies/Execution/… mit Status `🔬` → Analyse **vor** Live (Gate in PIPE-Tabelle: „gesperrt bis FLOW-*“).
4. Neue Lücke: PIPE- und/oder FLOW-Zeile — keine parallele Schattenliste.

### B. Dataflow skizzieren (FLOW, vor Code)

Pflichtfelder:

```
Trigger → Guard → State-Update → Request/IO → Response-Klassen → Penalty? → Next-State → Persistenz/Metrik
```

Headers-Referenzsoll (bereits FLOW-H01…H05):

```
FCU Tip → Backfill → HeaderSeed(CL)
  → Cap(working=max_peer_best) idempotent vs eventual_CL
  → Number(N) primt Falling → GetHeaders reverse
  → Empty=backoff/no ban | Headers=ETL(TempDir)
  → Writing headers → Checkpoint
```

Vor Bodies-Live mindestens FLOW-B01…B04 ausfüllen (Peer/Range, Empty-Politik, Buffer→Checkpoint, Headers-Kopplung).

### C. Bestandsaufnahme (Symptom)

1. Welches Symptom? (Log-Zeile, Mimir, Grafana No data vs 0, Stage, Blockhöhe)
2. Pipeline-Stage / Engine- / Downloader-Pfad?
3. Mappt auf welche `PORT-PIPE-00x` **und** `PORT-FLOW-*`?

### D. op-geth-Regel extrahieren (PIPE)

Für Header/Import/State typische opBNB-Delta vs vanilla OP:

| Regel | op-geth Anker (Orientierung) |
|-------|------------------------------|
| MilliTimestamp in `mixHash` | `core/types/block.go` `MilliTimestamp`; Parent-Check `consensus/beacon/consensus.go` |
| Wright `baseFee == 0` | `consensus/misc/eip1559` + `IsWright` |
| Wright L1-Fee nur bei `gasPrice==0` | `core/state_transition.go` |
| Fermat/Haber Precompiles | `core/vm/contracts.go` / `evm.go` |
| EIP-1559 elast=2, denom=8 | `params/config.go` OptimismConfig opBNB |
| PreContractFork | testnet only; mainnet nil |
| ForkId-Filter | Fermat/Snow/Volta/Fourier aus EL-forkid |
| Beacon tip by number | `eth/downloader/beaconsync.go`; Skeleton mock bans hash-head fetch |

CL-only (Snow, Volta/Fourier-Kadenz): EL muss resultierende Header **akzeptieren**, nicht die Kadenz selbst erzwingen.

### E. Reth-Wiring prüfen

1. Pipeline: stock `DefaultStages` + injiziertes `OpBeaconConsensus` / `OpEvmConfig` — selten fehlt eine Stage, oft fehlt die **Regel** oder der **Downloader-Automat**.
2. Headers: `ReverseHeadersDownloader` Zustände (Tip/Cap/Falling) + `HeaderValidator` (Milli).
3. Engine: Tip-FCU ohne Backfill = idle; Tip-Seed vs P2P-hash; poll-order.
4. Execution: Overlays am **historischen** Blockzeitpunkt.
5. Persistenz: SF-Segment-Routing, ETL-TempDir, Checkpoint-Semantik (FLOW-H05/S*).

### F. Fix-Disziplin

- Minimaler Diff; eine logische Lücke pro Commit wenn möglich.
- Transition-Test für FLOW-Knicke (Cap→Falling, Tip-Seed Empty, Backfill-Schwelle).
- Unit-Test an live beobachteten Werten wo PIPE (equal-second Milli + Negativtest OP-Mainnet).
- `plan.md` Bugliste + FLOW/PIPE-Status + kurzer Session-Eintrag; README bei Meilenstein/Methodik.
- Keine Secrets/`trusted_nodes` in committed Config; keine `files/*.log` committen.
- **Niemals `/tmp`** — Scratch/Logs/Datadir/IPC/JWT nur unter `files/` (siehe `.cursor/rules/no-tmp-writes.mdc`).

### G. Live-Verify = PIPE **und** FLOW grün für die Stage

Reihenfolge und Gates stehen in `plan.md`. Kurz:

1. FLOW-E* + PIPE-001 → Backfill / Headers startet
2. FLOW-H01…H04 + PIPE-002…004 → Falling stabil
3. FLOW-H05 → `Writing headers` / Checkpoint > 0
4. FLOW-B* **analysiert** + PIPE-005 → Bodies
5. FLOW-R01 + PIPE-006 → SenderRecovery
6. FLOW-X* + PIPE-007…010 → Execution (X02 = Wright L1-Fee Diff!)
7. FLOW-S* + PIPE-011…012 → Merkle/History

Bei Failure: Stage + Höhe + PIPE-Regel + FLOW-Übergang + Reth-Pfad; Status in `plan.md` aktualisieren.

## Anti-Patterns (aus dem Experiment)

- Live-Sync debuggen **ohne** PIPE **und** FLOW für die Stage
- Nur Konsensregel portieren, Downloader-/Engine-Automat ignorieren
- Cap/Seed/Tracker als „Live-Folgebug“ nach Restart entdecken
- Checkpoint 0 als „Headers broken“ ohne FLOW-H05 (ETL-TempDir)
- eth/68 Tip-Hash mit `best_number` verwechseln
- Ban auf Empty Headers/Bodies
- „Workspace kompiliert“ / nextest grün als Protokoll-Done
- Holocene/Isthmus portieren, obwohl opBNB-Hardfork-Liste sie nicht hat
- Pipeline-Stages umbauen, obwohl nur Consensus/EVM-Overlay fehlt
- Blindes Vibecoding ohne Referenz-Repo und ohne Bugliste

## Ausgabe an den Nutzer

- Kurz: Symptom → PIPE + FLOW (mit Referenz) → Fix-Status → nächstes Gate
- Offene `🔬` FLOW-IDs nennen, bevor Live vorgeschlagen wird
- Keine Garantie Produktionsreife; Catch-up/Full-Sync = Human
