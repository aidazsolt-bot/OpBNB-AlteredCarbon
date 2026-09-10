## CI

Public CI for this fork is the GitHub Actions workflow
[`.github/workflows/op-reth-build-smoke.yml`](../../.github/workflows/op-reth-build-smoke.yml):

1. **Build** `op-reth` (`maxperf`, jemalloc + asm-keccak + keccak-cache-global).
2. **Smoke sync** on `opbnb-mainnet` until
   `--debug.tip 0x6eb90fbf2063a1d99c3890878d8894dbe469ad764c8abad68729adc40446710f`
   (block **20000**), then exit via `--debug.terminate`.

Success is a clean process exit (`0`) after the tip is reached — not a fixed wall-clock
timeout. A 90-minute `timeout` wrapper is only a hang safety net (job budget: 120 minutes).

Triggers: `workflow_dispatch`, and `push` to `main` when `Cargo.toml` / `Cargo.lock` /
`Makefile` / `crates/**` / the workflow file change.

Upstream Reth lint/unit/book workflows are not mirrored here. For local checks:

```bash
cargo +nightly fmt --all --check
cargo check -p op-reth
cargo nextest run -p reth-optimism-chainspec   # example slice
```

For upstream CI definitions, see [paradigmxyz/reth](https://github.com/paradigmxyz/reth/tree/main/.github/workflows).
