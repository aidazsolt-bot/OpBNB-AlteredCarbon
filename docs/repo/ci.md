## CI

This fork does **not** run GitHub Actions CI.

Upstream Reth workflows (lint, unit, book, release, …) were removed so pushes to the public
mirror do not burn Actions minutes or show a permanent red “Check status” on `main`.

Validate locally as needed, e.g.:

```bash
cargo +nightly fmt --all --check
cargo check -p op-reth
cargo nextest run -p reth-optimism-chainspec   # example slice
```

For upstream CI definitions, see [paradigmxyz/reth](https://github.com/paradigmxyz/reth/tree/main/.github/workflows).
