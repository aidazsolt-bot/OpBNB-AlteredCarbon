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
