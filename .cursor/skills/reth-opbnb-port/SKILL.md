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

## Erster Schritt (hätte von Anfang an so laufen müssen)

**Bevor** Compile-Loops, Tip-Debug oder Einzel-Fixes: die **Pipeline-Verify-Matrix `PORT-PIPE-00x` in `plan.md`** gegen `bnb-chain/op-geth` aufbauen bzw. abarbeiten.

1. op-geth-Regeln für opBNB EL extrahieren (Milli, Wright baseFee/L1-Fee, Precompiles, EIP-1559-Params, …).
2. Pro Stage/Gate eine `PORT-PIPE-00x`-Zeile: Soll (op-geth Datei:Zeile) → Reth-Stand → Verify-Kriterium.
3. Live oder Fixture **in Reihenfolge** PIPE-001 → … verifizieren; erst bei Fail den konkreten Fix (dann ggf. `PORT-CONS-*` / `PORT-ENGINE-*`).

Das war der fehlende erste Schritt im Experiment: ohne Matrix wurde an Symptomen (Peers, Timestamps, Grafana) entlang vibecodiert, statt systematisch Pipeline↔op-geth zu sichern. **Skill-Pflicht:** bei Port-/Sync-Aufgaben zuerst `plan.md` Abschnitt *Pipeline-Verify-Matrix (PORT-PIPE)* lesen und Status der offenen PIPE-IDs nennen.

## Wann laden

- opBNB/OP-Stack-Port, Rebase auf neueres `reth`, Diff gegen `bnb-chain/op-geth` / `bnb-chain/opbnb`
- Live-Sync hängt (Headers/Bodies/Execution/Merkle, Engine FCU, Peers, Grafana Stages)
- Eintrag/Update von `PORT-*` / `PORT-PIPE-*` in `plan.md`

## Kernprinzip (nicht verhandelbar)

1. **PORT-PIPE zuerst.** Matrix in `plan.md` ist der Einstieg — nicht optionaler Nachtrag.
2. **Referenz zuerst, Code danach.** Jede vermutete Lücke braucht einen Beleg in op-geth (oder opbnb CL) mit Datei:Zeile — nicht nur „sollte so sein“.
3. **Compile/nextest grün beweist keine Chain-Korrektheit.** Live- oder Fixture-Verify Stage für Stage (`PORT-PIPE-00x`).
4. **Symptom → Layer → Regel → Diff → Fix → Verify.** Nicht random refactors.
5. **Human-owned:** Catch-up/Full-Sync und lange Läufe startet der Operator; Agent höchstens Boot-Smoke / gezielte Log-Analyse.

## Pflicht-Referenzen (dieser Workspace)

| Quelle | Pfad / Hinweis |
|--------|----------------|
| Arbeitsprotokoll + Bugliste | `plan.md` |
| Fork-Zweck / Methodik / Effort | `README.md` → *About This Fork* |
| Reth-Fork | Workspace-Root (Branch `rebase/reth-v2.4.1`) |
| op-geth (EL-Konsens) | `/usr/src/Erigon/Binance/bnb-chain_op-geth.git` |
| opbnb (CL/Derivation) | `/usr/src/Erigon/Binance/bnb-chain_opbnb.git` |
| Upstream-Architektur | `paradigmxyz/reth` v2.4.1 Patterns (`engine-tree`, `DefaultStages`) |

## Vorgehensweise (Checkliste)

### A. PORT-PIPE-Matrix (immer zuerst)

1. `plan.md` → *Pipeline-Verify-Matrix (PORT-PIPE)* öffnen.
2. Offene `PORT-PIPE-00x` (⏳/🐛) auflisten; nächste Verify-ID nennen.
3. Bei neuer Lücke: neue PIPE-Zeile oder Verweis auf CONS/ENGINE/STOR — keine parallele Schattenliste.

### B. Bestandsaufnahme (Symptom)

1. Welches Symptom? (Log-Zeile, Grafana No data vs 0, Stage-Name, Blockhöhe)
2. Welche Pipeline-Stage / Engine-Pfad? (Headers downloader ≠ Execution overlay)
3. Mappt auf welche `PORT-PIPE-00x` / existierende `PORT-*`?

### C. op-geth-Regel extrahieren

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

### D. Reth-Wiring prüfen

1. Pipeline: stock `DefaultStages` + injiziertes `OpBeaconConsensus` / `OpEvmConfig` — selten fehlt eine Stage, oft fehlt die **Regel im Consensus/EVM**.
2. Headers: Downloader → `HeaderValidator::validate_header_against_parent` (nicht nur Engine NewPayload).
3. Execution: Overlays (`opbnb_precompiles`, Wright flags) müssen am **historischen** Blockzeitpunkt greifen.
4. Engine: Tip-FCU ohne Backfill = Pipeline idle (Grafana „No data“); großer Gap → Backfill auf Head (OpStack).

### E. Fix-Disziplin

- Minimaler Diff; eine logische Lücke pro Commit wenn möglich.
- Unit-Test an **live beobachteten** Werten (z. B. equal-second Milli-Headers), plus Negativtest (OP-Mainnet behält Sekunden-Regel).
- `plan.md` Bugliste + kurzer Session-Eintrag; README nur bei Meilenstein/Methodik.
- Keine Secrets/`trusted_nodes` in committed Config; keine `files/*.log` committen.

### F. Live-Verify = Abarbeitung von PORT-PIPE-001…

Reihenfolge und Kriterien stehen in `plan.md` (PORT-PIPE-001 … 013). Kurz:

1. PIPE-001 Engine Backfill → Headers startet
2. PIPE-002…004 Headers (Milli, Wright baseFee, EIP-1559)
3. PIPE-005 Bodies → PIPE-006 SenderRecovery
4. PIPE-007…010 Execution (Fermat/Haber/Wright L1-Fee / L1-Attr)
5. PIPE-011 MerkleExecute → PIPE-012 History/TxLookup
6. PIPE-013 nur Testnet PreContract

Bei Failure: Stage + Höhe + op-geth-Regel + Reth-Pfad; PIPE-Status in `plan.md` aktualisieren.

## Anti-Patterns (aus dem Experiment)

- Live-Sync debuggen **ohne** vorherige `PORT-PIPE`-Matrix gegen op-geth
- „Workspace kompiliert“ als Done für Protokoll-Port
- Eth-Default-Validierung ohne op-geth-Diff (klassisch: Sekunden-Timestamp)
- Tip-Chase/`Download(single_block)` statt Pipeline bei Genesis→Tip-Gap
- Holocene/Isthmus-Code portieren, obwohl opBNB-Hardfork-Liste sie nicht hat
- Pipeline-Stages umbauen, obwohl nur Consensus/EVM-Overlay fehlt
- Blindes Vibecoding ohne Referenz-Repo und ohne Bugliste

## Ausgabe an den Nutzer

- Kurz: Symptom → Ursache (mit Referenz) → Fix-Status → nächster Verify-Schritt
- Keine Garantie Produktionsreife; Catch-up/Full-Sync = Human
