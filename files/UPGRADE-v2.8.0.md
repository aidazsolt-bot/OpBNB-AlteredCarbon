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

