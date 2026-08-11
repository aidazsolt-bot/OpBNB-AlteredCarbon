---
name: reth-opbnb-port
description: >-
  Portierungs-Spezialist für Reth/opBNB-Rebases und Protokoll-Nachführung gegen
  bnb-chain/op-geth und Upstream reth. Nutzen bei Merge/Rebase-Konflikten,
  Hardfork-/Consensus-/EVM-Lücken, Pipeline-/Engine-Sync-Debug, PORT-*-Bugliste
  und Live-Verify gegen eine referenzierende op-geth-Instanz.
---

# Reth / opBNB Portierungs-Spezialist

> Projekt-Skill: `.cursor/skills/reth-opbnb-port/`. Entstanden aus Session-10-Hinweisen,
> weil generische Coding-Agents ohne diese Vorgehensweise keine sinnvolle Protokoll-Portierung
> leisten (Compile-Grün ≠ Sync-Korrekt).

## Wann laden

- opBNB/OP-Stack-Port, Rebase auf neueres `reth`, Diff gegen `bnb-chain/op-geth` / `bnb-chain/opbnb`
- Live-Sync hängt (Headers/Bodies/Execution/Merkle, Engine FCU, Peers, Grafana Stages)
- Eintrag/Update von `PORT-*` in `plan.md`

## Kernprinzip (nicht verhandelbar)

1. **Referenz zuerst, Code danach.** Jede vermutete Lücke braucht einen Beleg in op-geth (oder opbnb CL) mit Datei:Zeile — nicht nur „sollte so sein“.
2. **Compile/nextest grün beweist keine Chain-Korrektheit.** Live- oder Fixture-Verify Stage für Stage.
3. **Symptom → Layer → Regel → Diff → Fix → Verify.** Nicht random refactors.
4. **Human-owned:** Catch-up/Full-Sync und lange Läufe startet der Operator; Agent höchstens Boot-Smoke / gezielte Log-Analyse.

## Pflicht-Referenzen (dieser Workspace)

| Quelle | Pfad / Hinweis |
|--------|----------------|
| Arbeitsprotokoll + Bugliste | `plan.md` |
| Fork-Zweck / Methodik / Effort | `README.md` → *About This Fork* |
| Reth-Fork | Workspace-Root (Branch `rebase/reth-v2.4.1`) |
| op-geth (EL-Konsens) | `<src-root>/Binance/bnb-chain_op-geth.git` |
| opbnb (CL/Derivation) | `<src-root>/Binance/bnb-chain_opbnb.git` |
| Upstream-Architektur | `paradigmxyz/reth` v2.4.1 Patterns (`engine-tree`, `DefaultStages`) |

## Vorgehensweise (Checkliste)

### A. Bestandsaufnahme

1. Welches Symptom? (Log-Zeile, Grafana No data vs 0, Stage-Name, Blockhöhe)
2. Welche Pipeline-Stage / Engine-Pfad? (Headers downloader ≠ Execution overlay)
3. Gibt es schon `PORT-*`? Lesen, nicht doppelte IDs erfinden.

### B. op-geth-Regel extrahieren

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

CL-only (Snow, Volta/Fourier-Kadenz): EL muss resultierende Header **akzeptieren**, nicht die Kadenz selbst erzwingen.

### C. Reth-Wiring prüfen

1. Pipeline: stock `DefaultStages` + injiziertes `OpBeaconConsensus` / `OpEvmConfig` — selten fehlt eine Stage, oft fehlt die **Regel im Consensus/EVM**.
2. Headers: Downloader → `HeaderValidator::validate_header_against_parent` (nicht nur Engine NewPayload).
3. Execution: Overlays (`opbnb_precompiles`, Wright flags) müssen am **historischen** Blockzeitpunkt greifen.
4. Engine: Tip-FCU ohne Backfill = Pipeline idle (Grafana „No data“); großer Gap → Backfill auf Head (OpStack).

### D. Fix-Disziplin

- Minimaler Diff; eine logische Lücke pro Commit wenn möglich.
- Unit-Test an **live beobachteten** Werten (z. B. equal-second Milli-Headers), plus Negativtest (OP-Mainnet behält Sekunden-Regel).
- `plan.md` Bugliste + kurzer Session-Eintrag; README nur bei Meilenstein/Methodik.
- Keine Secrets/`trusted_nodes` in committed Config; keine `files/*.log` committen.

### E. Live-Verify-Reihenfolge

1. Engine Backfill startet (`Preparing stage Headers`)
2. Headers checkpoint ↑ ohne `TimestampIsInPast` / Peer-Ban-Sturm
3. Bodies
4. SenderRecovery (Deposits)
5. Execution an Fermat / Haber-Fenster / Wright+
6. MerkleExecute (erster harter State-Beweis)
7. History / TxLookup unter storage.v2

Bei Failure: Stage + Höhe + op-geth-Regel + Reth-Pfad in `PORT-*` dokumentieren.

## Anti-Patterns (aus dem Experiment)

- „Workspace kompiliert“ als Done für Protokoll-Port
- Eth-Default-Validierung ohne op-geth-Diff (klassisch: Sekunden-Timestamp)
- Tip-Chase/`Download(single_block)` statt Pipeline bei Genesis→Tip-Gap
- Holocene/Isthmus-Code portieren, obwohl opBNB-Hardfork-Liste sie nicht hat
- Pipeline-Stages umbauen, obwohl nur Consensus/EVM-Overlay fehlt
- Blindes Vibecoding ohne Referenz-Repo und ohne Bugliste

## Ausgabe an den Nutzer

- Kurz: Symptom → Ursache (mit Referenz) → Fix-Status → nächster Verify-Schritt
- Keine Garantie Produktionsreife; Catch-up/Full-Sync = Human
