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
| `summarize_public.py` | Quick fork/tx/receipt summary of the fixture |
| `local-executed-receipts.json` | Produced by `op-reth re-execute --dump-receipts-on-fail` (gitignored pattern under files/) |

## Preconditions (datadir)

After the Unwind-Sturm (FLOW-X05), Execution/Bodies/Sender sit at **`21579118`**. To re-exec
`21591154` you need:

1. Headers still (or again) covering `21591154`
2. Bodies + SenderRecovery for `21579119..=21591154`
3. Historical state at parent **`21591153`** (i.e. Execution advanced to at least `21591153`,
   or `re-execute --from 21579119 --to 21591154` from committed state @ `21579118`)

Do **not** let the live pipeline Execution commit **`21591154`** before the PIPE-014 fix —
same fail → another FLOW-X05 unwind (seen **twice**: 2026-08-13 and 2026-08-14).

**Live height cap (preferred):**

```bash
# Pipeline stops at H; process exits when stages done
op-reth-bnb node … --debug.max-block 21579110 --debug.terminate
# For parent state before harness later: --debug.max-block 21591153
```

**Not a height cap:** `--debug.skip-fcu <N>` only skips N engine FCUs; pipeline still runs.
**Does not exist:** `--debug.skip-stages`.

**Journal (nspawn machine):**

```bash
journalctl -D /var/lib/machines/BSCRethArchiveNode/var/log/journal/ --since '1 hour ago'
```

## Produce local dump

```bash
# from repo root; use Session-12 rebuild-only binary (not yet installed over live)
# adjust datadir; never /tmp scratch
./target/maxperf/op-reth re-execute \
  --chain opbnb \
  --datadir /media/DataDomainNVMER40/BSCRethArchiveNode/.local/share/reth/opbnb-mainnet \
  --from 21591154 \
  --to 21591155 \
  --dump-receipts-on-fail files/harness-receipt-diff-21591154/local-executed-receipts.json
```

Alternative (replay from unwind floor if parent state not yet executed to `21591153`):

```bash
./target/maxperf/op-reth re-execute \
  --chain opbnb \
  --datadir /media/DataDomainNVMER40/BSCRethArchiveNode/.local/share/reth/opbnb-mainnet \
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
