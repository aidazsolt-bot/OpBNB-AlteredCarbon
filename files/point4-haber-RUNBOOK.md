# Point-4 Haber — Runbook (vorbereitet 2026-08-16)

## Gate
- Fork first block: **`27118477`** (ts `1718872200`)
- PIPE-008 / FLOW-X01 (early p256 @ `0x100` pre-Fjord)
- Run only when Mimir `Execution` ≥ `27118477` (Skript gated)

## Command
```bash
# readiness / ETA
python3 files/point4_fork_gate.py haber --dry-run --blk-per-s 140

# full gate (IPC vs https://opbnb-mainnet-rpc.bnbchain.org)
python3 files/point4_fork_gate.py haber
```

## Expect
- Exit `0` + `overall=MATCH`
- Writes `files/point4-haber-<stamp>.json` + `files/point4-haber-latest.json`
- Fields: `hash`, `transactionsRoot`, `stateRoot`, `receiptsRoot`

## After Haber MATCH
```bash
python3 files/point4_fork_gate.py wright --dry-run   # until Exec ≥ 32984677
python3 files/point4_fork_gate.py wright
```

## Notes
- Local: IPC only (`ARCHIVE_IPC` (default `/tmp/archive-ct.ipc`)) — no HTTP without `--http`
- MATCH ≠ “EVM perfect”; MerkleExecute + no unwind storms remain stronger gates
- Do **not** restart the live node for this check
