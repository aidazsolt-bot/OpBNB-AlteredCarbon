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
3. **Phase 3 — BSC-Crate (`crates/bsc`) aktualisieren** ✅ Compile-Meilenstein: `reth-bsc-node --features bsc` grün (2026-08-09)
4. **Phase 4 — Optimism/opBNB-Crate + Snow/Volta/Fourier-Hardforks** 🔄 Hardfork-Enum/Schedules ✅; forks/chainspec/primitives/consensus compile; op-evm/node noch architektonisch (revm-41 → `op-revm`)
5. **Phase 5 — Build/Lint/Test/EF-Tests** 🔄 angelaufen (`cargo check --workspace --no-default-features` ✅ 0 errors 2026-08-09; Clippy/nextest/EF noch offen)
6. **Phase 6 — Doku & Freigabe** 🔄 teilweise (Disclaimer/Effort-Log in README bereits drin, wird nach Live-Tests aktualisiert)

## Todo-Status (Stand 2026-08-09)

| ID | Titel | Status |
| --- | --- | --- |
| inventory-diff | Bestandsaufnahme & Diff-Baseline erstellen | ✅ done |
| core-rebase | Kern-Crates auf reth v2.4.1 rebasen | ✅ done |
| bsc-crate-update | BSC-Crate (crates/bsc) aktualisieren | ✅ done (compile: bsc-node grün; uncommitted) |
| opbnb-hardforks | Optimism/opBNB-Crate + Snow/Volta/Fourier | 🔄 Hardforks ✅; op-evm/node pending |
| build-test-validate | Build, Lint, Tests, EF-Tests | ⏳ pending (workspace check grün) |
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

| Session | Zeitraum (UTC) | Modelle | Input-Tokens | Output-Tokens | Turns / Events | Wichtigste Ergebnisse |
| --- | --- | --- | --- | --- | --- | --- |
| Frühere Sessions (kumulativ, vor `a95758da`) | mehrere Tage | Claude Sonnet 5 (primär), GPT-5.4 | ~58,7M (Sonnet 5) + ~38,5M (GPT-5.4) | ~231K (Sonnet 5) + ~78K (GPT-5.4) | ~800 | Merge/Rebase auf v2.4.1, Konflikte, Blockchain-Tree→Engine-Tree, Kona-Node-Eval, README-Disclaimer |
| Copilot CLI `a95758da` (Snapshot **2026-08-09**, final DB) | 2026-08-06 09:50 – 2026-08-07 18:05 | Sonnet 5, GPT-5.4, Sonnet 4.6, GPT-5.3-Codex, GPT-5.4-mini | ~356,9M + ~135,4M + ~88,4M + ~63,1M + ~6,4M = **~650,1M** (+ ~636M cache-read) | ~1,163K + ~298K + ~260K + ~124K + ~16K = **~1,861K** | 32 Turns / 5.803 Events / ~8,1h request-wall | Compile-Loop bis node-core/stages/rpc-Typen; Phase-2/3 Vorarbeit |
| Cursor Composer YOLO (Session 6, Chat `42f88fe7…`, Snapshot **2026-08-09 12:05 UTC**) | 06:45 – ~12:05 UTC (**~5,34 h** Wall) | **composer-2.5-fast** (4.986 `modelName`-Hits) + **cursor-grok-4.5-high-fast** (178); Parent `default` | **Kein lokaler Billed-Token-Ledger.** Content-Proxy: Transcripts ~2,34M chars ≈ **~0,58M Tokens** (÷4); Cleartext-Chat-JSON ≈ **~0,33M Tokens** (Untergrenze, Tool/Context unterzählt). Erwartete billed/context-Wiederholung deutlich höher | (Proxy, s. Input-Spalte) | **15 Agents** (1 Parent + 14 Subs); 2.582 Assistant-Msgs; 5.861 Tool-Blobs; ~11.722 Tool-Calls; 74.482 `ai_code_hashes` | **`reth-bsc-node --features bsc` + workspace `--no-default-features` grün**; Phase-4 op-forks/chainspec/primitives/consensus; Details: `files/cursor-session-metrics.json` |

> Hinweis: Copilot-Token-Zahlen sind kumulative Modellaufrufe inkl. Tool-Nutzung/Kontext-Wiederholung pro
> Turn. Cursor speichert hier **keinen** äquivalenten `assistant_usage_events`-Zähler (Chat-Blobs teils
> verschlüsselt) — daher Activity-Counts + Content-Size-Proxies. Kein Effizienz-Benchmark.
> Quellen: Copilot `<copilot-session-store>`; Cursor `~/.cursor/chats/3ad71c6c…/` +
> `agent-transcripts/` + `~/.cursor/ai-tracking/ai-code-tracking.db`.

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

### Session 6 Docs-Update (2026-08-09 ~10:45–12:45 UTC):
- README Effort-Log: Copilot `a95758da` final **~650.1M in / ~1.861M out / 5803 events**.
- Cursor-Metriken ergänzt (nicht mehr nur „unmetered“): Wall **~5.34 h**, 15 Agents, Models **composer-2.5-fast** / **cursor-grok-4.5-high-fast**, Activity (Msgs/Tool-Calls), Content-Token-Proxies (~0.58M / ~0.33M), **74.482** AI-code hashes; Snapshot `files/cursor-session-metrics.json`.
- plan.md Aufwandsprotokoll-Tabelle synchronisiert.
- Phase 4 Start: `reth-optimism-{forks,chainspec,primitives,consensus}` compile-fähig.
- Phase 5: Workspace `--no-default-features` **0 errors**; WIP weiterhin uncommitted.
