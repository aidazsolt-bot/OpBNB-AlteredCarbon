## CI

Public CI for this fork is the GitHub Actions workflow
[`.github/workflows/op-reth-build-smoke.yml`](../../.github/workflows/op-reth-build-smoke.yml):

1. **Log runner hardware** (`lscpu`, memory, `lsblk`/`df`, Azure IMDS `vmSize` when available).
2. **Build** `op-reth` (`maxperf`, jemalloc + asm-keccak + keccak-cache-global). Cache hits via
   `Swatinem/rust-cache` often cut wall time from ~20–25 m (cold Fat-LTO) to ~10–15 m
   (`save-if: false` — cache is restored for the build, then discarded before smoke).
3. **Free disk** for the smoke: copy `op-reth` to `$RUNNER_TEMP`, then delete workspace
   `target/` and `$CARGO_HOME/{registry,git}` (same root FS as the datadir; rust-cache
   `save-if: false` so the emptied tree is not re-uploaded).
4. **Tip-sync smoke** on `opbnb-mainnet`: `--debug.tip` = hash of block **20 000 000**
   (`0x32d00a406210786cb84c5336e32e6c1c28db605b58126892acb867fdd09c9f6b`) +
   `--debug.terminate` (no op-node; tip hash starts backfill). Override via
   `workflow_dispatch` input `tip_hash`.

Success is a clean exit after the tip backfill. A 330-minute `timeout` wrapper is only a hang
safety net (job budget: **360** minutes — GitHub-hosted max). **20 M** is ~2× the prior **10 M**
wall (~3 h); still watch the 6 h cap. Disk: ~45 G @10 M → roughly ~90 G @20 M if linear
(cleanup of cargo/`target` required).

### Observed GHA bench — tip **10 000 000** (2026-09-13)

**Not a product claim.** One successful maxperf smoke on GitHub-hosted runners
([run 34775664819](https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/actions/runs/34775664819/job/103773196382),
commit `3adbe256a9`). Workflow default tip was later raised to **20 000 000**.

| | |
| --- | --- |
| Job wall (build + sync) | ~**3 h 9 m** (`18:46`–`21:55` UTC) |
| Build `op-reth` (maxperf) | ~**12.5 m** |
| Sync step (Headers→Finish @10 M) | ~**2 h 56 m** |
| **Execution** stage only | **~1 h 52 m** (`19:53:37`–`21:46:01` UTC) |
| Execution wall blocks/s | **~1483** (10 M / 6743 s) |
| Logged gas throughput | `Executed block range` → `throughput="… Ggas/second"` (n=301) |
| Gas median / mean / max | **~1.03 / ~1.24 / ~4.27 Ggas/s** (~1030 / ~1240 / ~4270 Mgas/s) |
| Datadir after Finish | **`du -sh` = 45 G** (`$RUNNER_TEMP/opbnb-mainnet-maxperf`) |
| Root free after sync | **37 G** (`df`: 145 G / 108 G used) |

Later stages after Execution (MerkleExecute / TxLookup / Index* / Finish) added ~**9 m**.
Compare with the older tip-**2 M** notes below (different tip height; not directly comparable).

Triggers: `workflow_dispatch`, and `push` to `main` when `Cargo.toml` / `Cargo.lock` /
`Makefile` / `crates/**` / the workflow file change.

Upstream Reth lint/unit/book workflows are not mirrored here. For local checks:

```bash
cargo +nightly fmt --all --check
cargo check -p op-reth
cargo nextest run -p reth-optimism-chainspec   # example slice
```

For upstream CI definitions, see [paradigmxyz/reth](https://github.com/paradigmxyz/reth/tree/main/.github/workflows).

---

## Informal smoke timings (local vs GHA) — 2026-09-10

**Not a claim, not a product benchmark.** One-off operator notes while a live opBNB archive
Execution catch-up shared the same box. Tip for these runs was **2 000 000** (temporary CI tip
override; public workflow now defaults to **`--debug.tip` @ block 20 000 000**). Binary around `27fa672` (maxperf).

### Anonymized host class (local)

| | |
| --- | --- |
| CPU | 1× Intel Xeon (Cascade Lake-R class), **16c / 32t**, ~2.9 GHz base |
| Board | Single-socket server board (LGA3647 class) |
| RAM | **256 GiB** DDR4 ECC RDIMM (4×64 GiB), ~2933 MT/s configured |
| Disk | Multiple **consumer PCIe4 NVMe** (several ~8 TB-class + one ~4 TB-class), bind-mounted volumes; some role datadirs live under `<volume>/<role>/…` or `<role>.local`, not always at the volume root |
| Co-tenancy | **O(15–20)** concurrent EL/CL processes (reth / op-reth / chain forks / erigon / lighthouse / rollup consensus) sharing CPU/RAM/NVMe |
| IPC | Host shm sockets bind-mounted into containers as `/tmp/*.ipc` |

Serial numbers, asset tags, mount hostnames, and concrete role names are omitted on purpose.

### Stage wall times (genesis → tip 2 000 000)

| Stage | Local (SATA SSD + VDO) | Local (PCIe4 NVMe) | GHA (`Standard_D4ads_v5`, exclusive 4 vCPU) |
| --- | ---: | ---: | ---: |
| Headers | ~3.4 m | ~2.9 m | ~4.3 m |
| Bodies | ~9.5 m | ~11.0 m | ~10.6 m |
| SenderRecovery | ~8.8 s | ~6.2 s | ~24 s |
| Execution | ~3.8 m | ~3.2 m | ~2.4 m |
| **Wall (end-to-end)** | **~18.0 m** | **~18.3 m** | **~18.6 m** |

### Read-out

- **Disk class ≈ irrelevant** for this tip height: SATA+VDO vs PCIe4 NVMe did not move overall wall time.
- **Bodies** dominate and stay **peer-bound** (`connected_peers≈1` on local smokes).
- **Execution** is where exclusive GHA cores win (~2.4 m vs ~3.2–3.8 m) — expected under noisy-neighbor load (live archive alone was logging ~0.3–0.45 Ggas/s Execution concurrently).
- Buying a newer EPYC / more PCIe lanes is **not** justified by these numbers if the bottleneck stays peers + multi-tenant CPU.

GHA remains the cleaner apples-to-apples check for “does this binary sync?”; local smokes are useful for regression sniffing, not hardware ranking.
