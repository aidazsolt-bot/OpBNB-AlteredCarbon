# Upgrade workspace: reth v2.4.1 port → v2.8.0

## Layout

| | |
|---|---|
| Live v2.4.1 tree (untouched) | `/usr/src/Erigon/Binance/reth-bsc-trail.git` |
| Upgrade dest | `/usr/src/Erigon/Binance/reth-bsc-trail-v2.8.0.git` |
| Branch | `rebase/reth-v2.8.0` |
| Upstream tag | `v2.8.0` (`05ba48120a`) |
| Cargo target-dir | `/usr/src/Erigon/Cargo/target-reth-v2.8.0` |

## Merge topology

1. Branch from `rebase/reth-v2.4.1` (keeps upstream ancestry).
2. Sync working tree from `main` (current opBNB ports).
3. Carry FLOW-E03 engine WIP from live tree.
4. `git merge refs/tags/v2.8.0`.

## Conflict policy (this merge)

- **Theirs (upstream):** engine-tree (BAL / new payload API), discv5 dial-family fix, peers throttle, most crates, CI kept deleted.
- **Ours (port):** `reverse_headers` (FLOW-H07), `nat` (FLOW-N01/N02 DualStack/UPnP), `AGENTS.md`, workspace port deps (cometbft/bls/op-reth paths).
- **BSC examples / paradigm CI:** remain deleted.

## Must re-apply onto upstream engine-tree (dropped for API jump)

- `HeaderSeed` wiring + `seed_header_from_block`
- `backfill_target_hash` → finalized (FLOW-E03)
- `advance_op_safe_finalized_if_stale`
- Engine launch `header_seed` arg
- TrieDB path in Execution stage (if still required)
- `NatEndpoint` import usage sites if compile breaks after nat=ours

## Next

1. `cargo check -p reth-optimism-cli` / `make build-op` in this dest.
2. Re-port FLOW-E03/H06/H07 engine hooks onto v2.8.0 tree API.
3. Live archive stays on v2.4.1 tree until Tip-DoD + green check here.


## Structural note (v2.8.0)

Upstream `paradigmxyz/reth` **v2.8.0 removed `crates/optimism` from the mono-repo**
(OP Stack lives elsewhere / was extracted). This dest **kept** our in-tree
`crates/optimism/**` as workspace members and re-added port `workspace.dependencies`
(cometbft/bls/op-alloy paths).

Also expect further layout drift vs v2.4.1 (`crates/primitives`, alloy/revm bumps,
engine BAL APIs). First `cargo metadata` errors are expected until those are
reconciled.


## Compile probe (2026-10-23)

- `cargo metadata --no-deps` ✅ after restoring optimism members + port workspace deps.
- `cargo check -p reth-net-nat` ❌ feature conflict:
  `reth-execution-types` wants `reth-ethereum-primitives/serde-bincode-compat`
  but in-tree `crates/ethereum/primitives` only has:
  `arbitrary, default, reth-codec, rpc, serde, std, test-utils`.
  → next: align ethereum-primitives (and other leftover v2.4.1 crates) with v2.8.0
  feature set, then re-apply HeaderSeed/E03 onto upstream engine-tree.

## Progress 2026-10-23 ~10:20 CEST

### Done
- Bulk-restored non-port crates from `v2.8.0` tag (359 files) — cleared leftover v2.4.1 sources.
- Kept/overlaid port: NAT DualStack/UPnP, FLOW-H07 `reverse_headers`, p2p HeaderSeed/`max_peer_best_number`, fetch client.
- Optimism manifests: stripped obsolete `optimism`/`rocksdb`/`serde-bincode-compat`/`op` feature forwards (upstream extracted OP; crates.io `reth-primitives-traits` 0.8.2 has no `op`).
- **Compile green:** `reth-net-nat`, `reth-network-p2p`, `reth-downloaders`.

### In flight
- `cargo check -p reth-engine-tree` (upstream engine; HeaderSeed/E03 not yet re-applied).
- Full `op-reth` / optimism crate compile.

### Next
1. Finish engine-tree check; re-apply FLOW-E03 (`HeaderSeed`, `backfill_target_hash`→finalized, `advance_op_safe_finalized_if_stale`) onto v2.8.0 engine API.
2. `cargo check -p op-reth` (or `reth-optimism-node`).
3. Regen `Cargo.lock` when resolution stabilizes.
