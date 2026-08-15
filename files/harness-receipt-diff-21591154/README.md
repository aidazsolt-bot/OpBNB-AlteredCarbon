# FLOW-X04 / PIPE-014 — Receipt diff harness (`21591154`)

Single-block / fail-dump workflow for **PORT-EXEC-001** (`receipt root mismatch` @ opBNB
mainnet block **21591154**).

## Goal

Find the **first** transaction index where executed receipts diverge from public
`eth_getBlockReceipts`, then fix that path (not Wright/`PIPE-009` — block is pre-Wright).

## Artifacts

| File | Role |
| --- | --- |
| `../receipts-21591154-public.json` | Public block + receipts (expected) |
| `diff_receipts.py` | Diff local dump vs public fixture → first mismatch |
| `mutate_receipt_root.py` | Public receipts → trie root; mutate fields toward node `got` `0x61c1…` |
| `summarize_public.py` | Quick fork/tx/receipt summary of the fixture |
| `local-executed-receipts.json` | Produced by `op-reth re-execute --dump-receipts-on-fail` (gitignored pattern under files/) |

## CLI range semantics (easy to get wrong)

| Command | `--from` / `--to` | Loop in code | Effect for fail-block `21591154` |
| --- | --- | --- | --- |
| `stage run …` | **inclusive** `from..=to` | Execution: `start..=max_block` | `--to 21591153` = last safe Exec commit; **`--to 21591154` = Fail** |
| `re-execute` | **half-open** `from..to` | `for block in start..end` with `end=--to` | `--from 21591154 --to 21591155` = **only** `21591154` |

`re-execute` loads state at **`from - 1`**. So parent for the dump is **`21591153`**.

`--to 21591154` on `re-execute` alone would execute **zero** blocks (`54..54`).

**Mid-pipeline state (Finish=0, Headers ≫ Execution):** `history_by_block_number(exec_tip)` must
use **Latest** (PlainState tip = Execution checkpoint). Comparing only to Finish/`best_block_number()`
skipped that path → empty accounts / `nonce … expected 0`. Fixed in
`DatabaseProvider::try_into_history_at_block` (+ `re-execute` no longer clamps `--to` to Finish=0).
History indices (`IndexAccountHistory`) are still required for parents **below** Exec tip.

## Preconditions (datadir)

Do **not** let live pipeline Execution **commit** `21591154` before the PIPE-014 fix —
same fail → another FLOW-X05 unwind (seen **twice**: 2026-08-13 and 2026-08-14).

Needed before dump:

1. Headers covering `21591154`
2. Bodies + SenderRecovery through **`21591154`** (body of the bad block)
3. Execution / Account+Storage **ChangeSets SF** through parent **`21591153`**

### Bodies Cap ≠ Execution / ChangeSets SF tip

After Fail #3 (`unwind_to=0` + Kill) and Cap-Rebuild, **Bodies/Sender** can sit at Cap
`21579110` while **Execution** (and `account/storage-change-sets` static files) are still far
behind. ChangeSets SF track **Execution commits**, not Bodies.

**2026-08-15:** Opening `stage run` triggered three-way heal on
`…_change-sets_20000000_20499999`:

```
header_claims=365615  sidecar_has=379384  → valid_blocks=365615
tip = 20_000_000 + 365_615 − 1 = 20_365_614
```

Heal only truncated the **uncommitted sidecar** ahead of the header (~14 k blocks). It did
**not** chop from Cap down to ~20.36 M. The ~1.2 M gap vs Cap was simply **Exec not finished**
to Cap yet. `stage run execution --from 21579110` then failed with
`missing static file data for block number: 20365615`.

Resume Exec from the **SF tip**, not from Bodies Cap:

```bash
# WRONG if ChangeSets tip is still ~20365614:
#   stage run … --from 21579110 --to 21591153 … execution

# RIGHT (example after heal @ 20365614):
op-reth-bnb stage run --chain opbnb-mainnet \
  --from 20365614 --to 21591153 \
  --skip-unwind --commit --checkpoints execution
```

`--skip-unwind` avoids FLOW-X05 if `to` is set too high; still prefer never Exec-commit
`21591154` until fixed.

### Offline stage run (Cap → parent) — correct heights

Cap reference after clean rebuild: **`21579110`**. Bad block: **`21591154`**. Parent: **`21591153`**.

```bash
# Bodies: inclusive through bad block (body needed for re-execute)
# Use Cap as --from (not Cap+1): Cap+1 caused append expected #21579111 got #21579112
op-reth-bnb stage run --chain opbnb-mainnet \
  --from 21579110 --to 21591154 \
  --skip-unwind --commit --checkpoints bodies

# SenderRecovery: same inclusive range
op-reth-bnb stage run --chain opbnb-mainnet \
  --from 21579110 --to 21591154 \
  --skip-unwind --commit --checkpoints senderrecovery

# Execution: inclusive through PARENT only — stop at 21591153
# --from = current Execution / ChangeSets tip (probe via heal error or SF header), NOT Bodies Cap
op-reth-bnb stage run --chain opbnb-mainnet \
  --from <changeset_sf_tip> --to 21591153 \
  --skip-unwind --commit --checkpoints execution
```

`stage run` defaults `--log.file.max-files 0` (stdout only). Tee or set
`--log.file.max-files 5 --log.file.directory …` outside `/tmp`.

**Live height cap (pipeline):**

```bash
# Pipeline stops at H; process exits when stages done
op-reth-bnb node … --debug.max-block 21579110 --debug.terminate
# Parent state without committing fail block: --debug.max-block 21591153
```

**Not a height cap:** `--debug.skip-fcu <N>` only skips N engine FCUs.  
**Does not exist:** `--debug.skip-stages`.  
Dirty Cap when stages already **> H** → PORT-OPS-001 (skip → Merkle fail → unwind).

**Journal (container-host):**

```bash
journalctl -D <archive-journal> --since '1 hour ago'
```

## Produce local dump

Datadir exclusive (no parallel node writing the same DB). Prefer binary with dump flag
(`target/maxperf/op-reth` if `dist/bin` lacks it).

```bash
# from repo root; half-open: executes ONLY 21591154; state @ 21591153
./target/maxperf/op-reth re-execute \
  --chain opbnb-mainnet \
  --datadir /media/<datadir-vol>/<archive-ct>/.local/share/reth/opbnb-mainnet \
  --from 21591154 \
  --to 21591155 \
  --dump-receipts-on-fail files/harness-receipt-diff-21591154/local-executed-receipts.json
```

Alternative (replay from unwind floor if parent state not yet at `21591153`):

```bash
./target/maxperf/op-reth re-execute \
  --chain opbnb-mainnet \
  --datadir /media/<datadir-vol>/<archive-ct>/.local/share/reth/opbnb-mainnet \
  --from 21579119 \
  --to 21591155 \
  --dump-receipts-on-fail files/harness-receipt-diff-21591154/local-executed-receipts.json
```

On post-execution validation failure the dump contains per-tx
`status / gasUsed / cumulativeGasUsed / logCount / txHash`.

## Diff

```bash
python3 files/harness-receipt-diff-21591154/diff_receipts.py \
  --public files/receipts-21591154-public.json \
  --local files/harness-receipt-diff-21591154/local-executed-receipts.json
```

Exit code `1` = first mismatch printed (DoD for FLOW-X04). Exit `0` = all compared fields match
(then investigate receipt encoding / root assembly).

## Known constants

- expected receiptsRoot: `0x579924c85d951e538e7b9c5358a1acda6d1fb379af748b01274c60a283d5e50c`
- got (live): `0x61c1b64b0df2fc07a64c4d8fabde08bf8be235bdbfa6b8543c00b9683a9fbe6b`
- ts `1713344877` — Snow yes, Canyon/Haber/Wright no

## FLOW-X04 result (2026-08-15)

| Field | Value |
| --- | --- |
| First mismatch | **index 10** |
| txHash | `0x7f276cf9690ae2c09aee72b2333843765ce28301e55275e09d2cfe79ddd0ff47` |
| Call | `syncLightBlock(bytes,uint64)` → contract `0xf51ba131…` → precompile **`0x67`** |
| gasUsed | public **717672** vs local **259171** (Δ ≈ 458501); status/logs match |
| Root cause | Fermat overlay injected **`BEFORE_HERTZ`** (forces `validatorSetChanged=false`). op-geth always returns the **pre-update** flag (BSC Hertz semantics). When the set changes, IBC does extra SSTOREs → higher gas on public. |
| Fix | `opbnb_precompiles/mod.rs` → `COMETBFT_LIGHT_BLOCK_VALIDATION` (Hertz). |
| Verify | **2026-08-15 ~14:13 CEST:** `re-execute --from 21591154 --to 21591155` succeeded (no dump written = receiptsRoot match). Binary: maxperf / `dist/bin/op-reth-bnb`. |

## op-geth vs Reth (roots)

op-geth FullSync checks **receipt + state root in the same** `ValidateState` after `Process`.
Reth archive: **receipt** in Execution; **state root** later in MerkleExecute. That staging
difference is not the `21591154` fail — `got` is the trie of locally executed receipts.
Pre-Canyon deposit nonce strip is mirrored (`EncodeIndex` / `calculate_receipt_root_optimism`
/ `alloy-op-evm` `strip_deposit_nonce`). See `plan.md` § *op-geth vs Reth — State-/Receipt-Root*.
