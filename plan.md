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

## Phasenübersicht (Soll)

1. **Phase 1 — Bestandsaufnahme & Diff-Baseline** ✅ erledigt
2. **Phase 2 — Kern-Crates auf v2.4.1 rebasen** ✅ Merge/Konflikte erledigt, Detailarbeit läuft (s.u.)
3. **Phase 3 — BSC-Crate (`crates/bsc`) aktualisieren** 🔄 in Arbeit (aktueller Fokus)
4. **Phase 4 — Optimism/opBNB-Crate + Snow/Volta/Fourier-Hardforks** ⏳ ausstehend
5. **Phase 5 — Build/Lint/Test/EF-Tests** ⏳ ausstehend
6. **Phase 6 — Doku & Freigabe** 🔄 teilweise (Disclaimer/Effort-Log in README bereits drin, wird nach Live-Tests aktualisiert)

## Todo-Status (aus Session-DB, Stand 2026-08-06 18:06)

| ID | Titel | Status |
| --- | --- | --- |
| inventory-diff | Bestandsaufnahme & Diff-Baseline erstellen | ✅ done |
| core-rebase | Kern-Crates auf reth v2.4.1 rebasen | ✅ done |
| bsc-crate-update | BSC-Crate (crates/bsc) aktualisieren | 🔄 in_progress |
| opbnb-hardforks | Optimism/opBNB-Crate + Snow/Volta/Fourier | ⏳ pending |
| build-test-validate | Build, Lint, Tests, EF-Tests | ⏳ pending |
| docs-release | Doku aktualisieren, Freigabe vorbereiten | ⏳ pending |

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

| Session | Zeitraum (UTC) | Modelle | Input-Tokens | Output-Tokens | Turns | Wichtigste Ergebnisse |
| --- | --- | --- | --- | --- | --- | --- |
| Frühere Sessions (kumulativ, s. README-Stand vor dieser Aktualisierung) | mehrere Tage, mehrere Sitzungen | Claude Sonnet 5 (primär), GPT-5.4 (Sub-Agenten) | ~58,7M (Sonnet 5) + ~38,5M (GPT-5.4) | ~231K (Sonnet 5) + ~78K (GPT-5.4) | ~800 | Merge/Rebase auf v2.4.1 abgeschlossen, Konflikte aufgelöst, Blockchain-Tree→Engine-Tree-Fund, Kona-Node-Evaluierung, README-Disclaimer |
| Aktuelle Session `a95758da` | 2026-08-06 09:50 – 18:06 (laufend) | Claude Sonnet 5 (primär), GPT-5.4 (Sub-Agenten) | ~158,2M (Sonnet 5) + ~40,2M (GPT-5.4) | ~576K (Sonnet 5) + ~94K (GPT-5.4) | 14 Turns / 1273+321 Modell-Aufrufe | `crates/primitives`+`primitives-traits` komplett neu geschrieben (größter Einzel-Blocker der gesamten Portierung), `reth-bsc-chainspec`/`reth-chainspec` API-Drift behoben, mehrere Folgefixes (`execution-types`, `eth-wire`, etc.) |

> Hinweis: Token-Zahlen sind kumulative Modellaufrufe inkl. Tool-Nutzung/Kontext-Wiederholung pro
> Turn, kein Maß für "sinnvolle" Ausgabe — dienen der Transparenz über den praktischen Ressourcen-
> Aufwand dieser Art von KI-gestützter Modernisierung, nicht als Effizienz-Benchmark.

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
