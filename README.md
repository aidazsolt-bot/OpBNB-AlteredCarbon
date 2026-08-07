# ‼️ Project Notice — Read Before Use ‼️

This repository is an **independent, personal experiment**. It is **not affiliated with, endorsed by, or
sponsored by Binance, BNB Chain, or any related entity**, and it is **not an official BNB Chain product**.
Any resemblance to naming used by upstream projects (e.g. "bnb-chain/reth") is retained only for historical/
attribution purposes because this repository was originally forked from that project; it does not imply any
ongoing relationship, support, or endorsement.

**Purpose of this project:** this fork exists purely to evaluate how far modern AI coding assistants ("vibe
coding") can go in modernizing and reviving a real-world, moderately complex, previously-abandoned blockchain
client codebase — rebasing it onto a current upstream ([paradigmxyz/reth](https://github.com/paradigmxyz/reth))
and porting forward protocol changes from an actively maintained downstream fork. It is a technology/process
evaluation, not a production initiative.

**No warranty, no liability, use at your own risk.** This software is provided "AS IS", without warranty of
any kind, express or implied, including but not limited to fitness for a particular purpose, merchantability,
or non-infringement. The author(s) and contributors accept **no responsibility or liability whatsoever** for
any damages, financial losses, chain-consensus incidents, data loss, or other harm arising from the use,
misuse, or inability to use this software — whether run as a node, a library, or in any other capacity. This
code has **not** been security-audited and should not be trusted with real funds or run against mainnet
without independent review. If you need a supported client for BNB Smart Chain or opBNB, use one of the
actively maintained official clients instead (see upstream project links further below).

-------

# reth-bsc-trail

[![CI status](https://github.com/paradigmxyz/reth/workflows/unit/badge.svg)][gh-ci]
[![cargo-deny status](https://github.com/paradigmxyz/reth/workflows/deny/badge.svg)][gh-deny]

[gh-ci]: https://github.com/bnb-chain/reth/actions/workflows/unit.yml

[gh-deny]: https://github.com/bnb-chain/reth/actions/workflows/deny.yml

This is an experimental, community/hobbyist fork of a blockchain client based on
[Reth](https://github.com/paradigmxyz/reth/), historically providing support for
[BNB Smart Chain (BSC)](https://github.com/bnb-chain/bsc) and [opBNB](https://github.com/bnb-chain/op-geth)
network protocols. See the project notice above for the current status and intent of this repository.

## Build from Source

For prerequisites and detailed build instructions please read
the [Installation Instructions](https://paradigmxyz.github.io/reth/installation/source.html).

With Rust and the dependencies installed, you're ready to build this fork. First, clone the repository:

```shell
git clone https://github.com/bnb-chain/reth.git
cd reth
```

In the realm of BSC, you have the option to execute the following commands to compile bsc-reth:

```shell
make build-bsc
```

Alternatively, you can install reth using the following command:

```shell
make install-bsc
```

When it comes to opBNB, you can run the following commands to compile op-reth:

```shell
make build-op
```

Or, opt for installing op-reth with the command:

```shell
make install-op
```

## Before setting up the node

### Optimizing `vm.min_free_kbytes` for MDBX Storage in Reth

#### Why Adjust `vm.min_free_kbytes`?

Reth uses **MDBX** as its underlying storage engine, which relies on **memory-mapped I/O (mmap)** for high-performance operations. However, MDBX can consume a significant amount of memory, and in scenarios where applications allocate memory aggressively, the system may run into **memory pressure**.

By increasing `vm.min_free_kbytes`, you can **prevent the Linux OOM (Out-Of-Memory) killer** from terminating essential processes when free memory runs low. This ensures smoother performance and better stability.

#### Recommended Setting

We recommend setting `vm.min_free_kbytes` to at least **4GB (4194304 kbytes)** to ensure system stability when using MDBX.

#### **Linux**

To apply the setting temporarily (until reboot):

```sh
sudo sysctl -w vm.min_free_kbytes=4194304
```

To make it persist across reboots, add the following line to /etc/sysctl.conf:

```sh
echo "vm.min_free_kbytes=4194304" | sudo tee -a /etc/sysctl.conf
```

Then apply the changes:

```sh
sudo sysctl -p
```

### Verifying the Configuration

To verify that the setting has been applied correctly, run:

```sh
cat /proc/sys/vm/min_free_kbytes
```

## Run Reth for BSC

### Hardware Requirements

* CPU with 16+ cores
* 128GB RAM
* High-performance NVMe SSD with at least 4TB of free space for full node and 8TB of free space for archive node
* A broadband internet connection with upload/download speeds of 25 MB/s

### Steps to Run bsc-reth

The command below is for an archive node. To run a full node, simply add the `--full` tag.

```shell
# for mainnet
export network=bsc

# for testnet
# export network=bsc-testnet

./target/release/bsc-reth node \
    --datadir=./datadir \
    --chain=${network} \
    --http \
    --http.api="eth, net, txpool, web3, rpc" \
    --log.file.directory ./datadir/logs \
    --enable-prefetch \
    --optimize.enable-execution-cache
```

You can run `bsc-reth --help` for command explanations.

For running bsc-reth with docker, please use the following command:

```shell
# for mainnet
export network=bsc

# for testnet
# export network=bsc-testnet

# check this for version of the docker image, https://github.com/bnb-chain/reth/pkgs/container/bsc-reth
export version=latest

# the directory where reth data will be stored
export data_dir=/xxx/xxx

docker run -d -p 8545:8545 -p 30303:30303 -p 30303:30303/udp -v ${data_dir}:/data \
    --name bsc-reth ghcr.io/bnb-chain/bsc-reth:${version} node \
    --datadir=/data \
    --chain=${network} \
    --http \
    --http.api="eth, net, txpool, web3, rpc" \
    --log.file.directory /data/logs \
    --enable-prefetch \
    --optimize.enable-execution-cache
```

### Snapshots

There are snapshots available from the community, you can use a snapshot to reduce the sync time for catching up.

* [fuzzland snapshot](https://github.com/fuzzland/snapshots)
* [bnb-chain snapshot](https://github.com/bnb-chain/reth-snapshots)

## Run Reth for opBNB

The op-reth can function as both a full node and an archive node. Due to its unique storage advantages, it is primarily
utilized for running archive nodes.

### Hardware Requirements

* CPU with 16+ cores
* 128GB RAM
* High-performance NVMe SSD with at least 3TB of free space
* A broadband internet connection with upload/download speeds of 25 MB/s

### Steps to Run op-reth

The op-reth is an [execution client](https://ethereum.org/en/developers/docs/nodes-and-clients/#execution-clients) for
opBNB. You need to run op-node along with op-reth to synchronize with the opBNB network.

Here is the quick command for running the op-node. For more details, refer to
the [opbnb repository](https://github.com/bnb-chain/opbnb).

```shell
git clone https://github.com/bnb-chain/opbnb
cd opbnb
make op-node

# for mainnet
export network=mainnet
export L1_RPC=https://bsc-dataseed.bnbchain.org
export P2P_BOOTNODES="enr:-J24QA9sgVxbZ0KoJ7-1gx_szfc7Oexzz7xL2iHS7VMHGj2QQaLc_IQZmFthywENgJWXbApj7tw7BiouKDOZD4noWEWGAYppffmvgmlkgnY0gmlwhDbjSM6Hb3BzdGFja4PMAQCJc2VjcDI1NmsxoQKetGQX7sXd4u8hZr6uayTZgHRDvGm36YaryqZkgnidS4N0Y3CCIyuDdWRwgiMs,enr:-J24QPSZMaGw3NhO6Ll25cawknKcOFLPjUnpy72HCkwqaHBKaaR9ylr-ejx20INZ69BLLj334aEqjNHKJeWhiAdVcn-GAYv28FmZgmlkgnY0gmlwhDTDWQOHb3BzdGFja4PMAQCJc2VjcDI1NmsxoQJ-_5GZKjs7jaB4TILdgC8EwnwyL3Qip89wmjnyjvDDwoN0Y3CCIyuDdWRwgiMs"

# for testnet
# it's better to replace the L1_RPC with your own BSC Testnet RPC Endpoint for stability
# export network=testnet
# export L1_RPC=https://bsc-testnet.bnbchain.org
# export P2P_BOOTNODES="enr:-J24QGQBeMsXOaCCaLWtNFSfb2Gv50DjGOKToH2HUTAIn9yXImowlRoMDNuPNhSBZNQGCCE8eAl5O3dsONuuQp5Qix2GAYjB7KHSgmlkgnY0gmlwhDREiqaHb3BzdGFja4PrKwCJc2VjcDI1NmsxoQL4I9wpEVDcUb8bLWu6V8iPoN5w8E8q-GrS5WUCygYUQ4N0Y3CCIyuDdWRwgiMr,enr:-J24QJKXHEkIhy0tmIk2EscMZ2aRrivNsZf_YhgIU51g4ZKHWY0BxW6VedRJ1jxmneW9v7JjldPOPpLkaNSo6cXGFxqGAYpK96oCgmlkgnY0gmlwhANzx96Hb3BzdGFja4PrKwCJc2VjcDI1NmsxoQMOCzUFffz04eyDrmkbaSCrMEvLvn5O4RZaZ5k1GV4wa4N0Y3CCIyuDdWRwgiMr"

./op-node/bin/op-node \
  --l1.trustrpc \
  --sequencer.l1-confs=15 \
  --verifier.l1-confs=15 \
  --l1.http-poll-interval 60s \
  --l1.epoch-poll-interval 180s \
  --l1.rpc-max-batch-size 20 \
  --rollup.config=./assets/${network}/rollup.json \
  --rpc.addr=0.0.0.0 \
  --rpc.port=8546 \
  --p2p.sync.req-resp \
  --p2p.listen.ip=0.0.0.0 \
  --p2p.listen.tcp=9003 \
  --p2p.listen.udp=9003 \
  --snapshotlog.file=./snapshot.log \
  --p2p.bootnodes=$P2P_BOOTNODES \
  --metrics.enabled \
  --metrics.addr=0.0.0.0 \
  --metrics.port=7300 \
  --pprof.enabled \
  --rpc.enable-admin \
  --l1=${L1_RPC} \
  --l2=http://localhost:8551 \
  --l2.jwt-secret=./jwt.txt \
  --syncmode=execution-layer
```

**It's important to mention that op-node and op-reth both need the same jwt.txt file.**
To do this, switch to the op-reth workdir and paste the jwt.txt file created during op-node execution into the current
workspace.

Here is a quick command for running op-reth. The command below is for an archive node, to run a full node, simply add
the `--full` tag.

```shell
# for mainnet
export network=mainnet
export L2_RPC=https://opbnb-mainnet-rpc.bnbchain.org

# for testnet
# export network=testnet
# export L2_RPC=https://opbnb-testnet-rpc.bnbchain.org

./target/release/op-reth node \
    --datadir=./datadir \
    --chain=opbnb-${network} \
    --rollup.sequencer-http=${L2_RPC} \
    --authrpc.addr="0.0.0.0" \
    --authrpc.port=8551 \
    --authrpc.jwtsecret=./jwt.txt \
    --http \
    --http.api="eth, net, txpool, web3, rpc" \
    --log.file.directory ./datadir/logs \
    --enable-prefetch \
    --optimize.enable-execution-cache
```

You can run `op-reth --help` for command explanations. More details on running opbnb nodes can be
found [here](https://docs.bnbchain.org/opbnb-docs/docs/tutorials/running-a-local-node/).

For running op-reth with docker, please use the following command:

```shell
# for mainnet
export network=mainnet
export L2_RPC=https://opbnb-mainnet-rpc.bnbchain.org

# for testnet
# export network=testnet
# export L2_RPC=https://opbnb-testnet-rpc.bnbchain.org

# check this for version of the docker image, https://github.com/bnb-chain/reth/pkgs/container/op-reth
export version=latest

# the directory where reth data will be stored
export data_dir=/xxx/xxx

# the directory where the jwt.txt file is stored
export jwt_dir=/xxx/xxx

docker run -d -p 8545:8545 -p 30303:30303 -p 30303:30303/udp -v ${data_dir}:/data -v ${jwt_dir}:/jwt \
    --name op-reth ghcr.io/bnb-chain/op-reth:${version} node \
    --datadir=/data \
    --chain=opbnb-${network} \
    --rollup.sequencer-http=${L2_RPC} \
    --authrpc.addr="0.0.0.0" \
    --authrpc.port=8551 \
    --authrpc.jwtsecret=/jwt/jwt.txt \
    --http \
    --http.api="eth, net, txpool, web3, rpc" \
    --log.file.directory /data/logs \
    --enable-prefetch \
    --optimize.enable-execution-cache
```

## Contribution

This is a personal experimental fork, not an actively maintained community project — there is no dedicated
support channel or roadmap. Thank you for considering helping out with the source code! Contributions
(forks, fixes, PRs) are welcome, but please understand this is best-effort with no guaranteed review turnaround.

Please see the [Developers' Guide](https://github.com/bnb-chain/reth/tree/develop/docs)
for more details on configuring your environment, managing project dependencies, and
testing procedures.

## About This Fork: Purpose, Method, and Effort Log

### What this is

This repository was resurrected from an archived, unmaintained state (last upstream release `v1.1.1`,
suspended per the notice at the top of this file) as an experiment in AI-assisted ("vibecoding") software
maintenance: rebasing a moderately large, protocol-critical Rust codebase onto a much newer upstream
release of [paradigmxyz/reth](https://github.com/paradigmxyz/reth) (targeting `v2.4.1`), resolving the
resulting merge conflicts, restructuring code around upstream's architectural changes (e.g. the `revm`
41.x execution-engine rewrite, the extraction of `crates/optimism` into a separate downstream project,
and the `blockchain-tree` → `engine-tree` migration), and porting forward opBNB-specific protocol/hardfork
changes from [bnb-chain/opbnb](https://github.com/bnb-chain/opbnb). The explicit goal was to evaluate how
far current-generation AI coding assistants can carry this class of work — not to produce a production-ready
or officially supported client.

### Method

Work was performed interactively with an AI coding agent (GitHub Copilot CLI) across multiple sessions,
using a mix of direct agent-driven edits and delegated background sub-agents supervising/verifying each
other's changes (given the scale of the merge — 200+ conflicting files across the initial rebase alone).
Progress was checkpointed via small, incrementally verified git commits rather than large unreviewed
batches, specifically to keep the change history auditable and revertible given the semi-autonomous
nature of the work.

### Effort log (approximate, based on available session telemetry)

| Metric | Value |
| --- | --- |
| Elapsed wall-clock time (this rebase effort, across sessions) | Multiple sessions over several days |
| LLM models used | Claude Sonnet 5 (primary), GPT-5.4, Claude Sonnet 4.6, GPT-5.3-Codex (delegated/specialized passes) |
| Approx. input tokens consumed (current session `a95758da`, snapshot 2026-08-07 04:12 UTC) | ~356.9M (Claude Sonnet 5) + ~135.4M (GPT-5.4) + ~88.4M (Claude Sonnet 4.6) + ~3.3M (GPT-5.3-Codex), total ~584.1M |
| Approx. output tokens generated (current session `a95758da`, snapshot 2026-08-07 04:12 UTC) | ~1.162M (Claude Sonnet 5) + ~297.7K (GPT-5.4) + ~260.3K (Claude Sonnet 4.6) + ~3.5K (GPT-5.3-Codex), total ~1.725M |
| Approx. interaction volume (current session `a95758da`) | 17 CLI turns, 5,188 model usage events |
| Commits produced during the v2.4.1 rebase | See `git log` on the `rebase/reth-v2.4.1` branch for the full, itemized commit history and messages, which double as a technical changelog of what was ported, what was fixed, and why |

These figures are a session telemetry snapshot and are illustrative of the scale of context/inference
required for this kind of large structural migration; total cumulative consumption across all sessions in
this effort is higher. They are provided for transparency about the practical cost of AI-assisted
maintenance at this scale, not as a benchmark claim — no rigorous token-efficiency optimization was
attempted.

> **TODO:** Update this effort log once the port has been validated against live BSC/opBNB testnet (or
> mainnet) sync, including final cumulative token/time figures and the outcome of live testing.

### Side-evaluation: `kona-node` as an alternative to `op-node` for opBNB

While blocked on the `crates/primitives` legacy-API remediation (see commit history), a live test of
[`kona-node`](https://github.com/op-rs/kona) (a modern Rust rollup-node implementation, evaluated from a
local checkout under `optimism.git/rust/kona`) was run against opBNB mainnet config, using an
in-progress `reth` (this fork) instance as the L2 engine and public BSC RPC endpoints as the L1 source.

**Outcome:** `kona-node` crashed on startup with
`Failed to load genesis time from beacon client: Backend("HTTP request failed: error decoding response body")`
(`kona/crates/providers/providers-alloy/src/blobs.rs:61`, `OnlineBlobProvider::init()`). Root cause: BSC does
not expose a standard Ethereum Beacon Chain API (`bsc-dataseed.bnbchain.org` is an execution-layer JSON-RPC
endpoint, not a beacon API), and `kona-node`'s `OnlineBlobProvider::init()` unconditionally fetches beacon
genesis time/slot-interval via `.expect(...)`, with no non-beacon fallback in that code path. This is not a
BSC/opBNB-specific defect in `kona-node` itself; `op-node` (Go) has the same underlying dependency on an L1
Beacon API for blob-derivation post-Ecotone, currently disabled in `bnb-chain/opbnb`'s fork
(`op-node/node/config.go` — the Ecotone Beacon-API-required check is commented out).

**Mitigation found:** `kona-node` already ships a flag for exactly this situation:
`--l1.slot-duration-override <seconds>` (`kona/bin/node/src/flags/engine/providers.rs`,
`kona/crates/node/service/src/service/builder.rs`), which bypasses the initial beacon-spec fetch by supplying
a fixed L1 slot duration instead of querying it from a beacon client. This was identified but not yet
re-tested end-to-end (blocked on the same `reth-bsc-trail` compile blockers as the main port); a live retest
with this flag is a recommended next validation step.

**Assessment — should we switch to `kona-node`?**
- `kona-node` is architecturally more modern (Rust, matches this project's `reth` stack, actively developed,
  and already spec-aware up to the Isthmus/Jovian/Karst/Lagoon hardforks per its own startup log), which is
  attractive for a project whose stated goal is evaluating modern tooling.
- However, it is materially less proven in production than `op-node` for exotic L1s like BSC that lack a
  genuine Beacon Chain API — the blob-provider assumption is baked in fairly deep and needs the
  slot-duration-override workaround (or a proper "no-beacon" derivation mode) to function at all here.
- **Recommendation:** keep `op-node` (via `bnb-chain/opbnb`) as the primary/default rollup-node for opBNB in
  this fork for now, but track `kona-node` as a promising secondary/experimental option once the
  slot-duration-override path is validated live. Do not switch defaults until a full derivation-to-tip sync
  has been demonstrated against opBNB with `kona-node`.
- Regarding upstream `reth`/`op-reth` rollup defaults: the mainline `reth-optimism-*` crates already bundle
  the standard OP-stack hardfork schedule (Regolith through Isthmus and beyond) as of the `v2.4.1` base this
  fork rebases onto — no separate/manual hardfork-default wiring is needed there; opBNB-specific deviations
  (Snow/Volta/Fourier and similar) are what still needs explicit porting in this fork's BSC/opBNB crates.

> **TODO:** Re-run this `kona-node` evaluation live with `--l1.slot-duration-override` once
> `reth-bsc-trail` compiles again, and record whether it reaches a synced L2 tip against opBNB
> mainnet/testnet, including timing and any further blockers found.

### Status disclaimer (repeated for emphasis)

As stated at the top of this document: this is an unaudited, experimental fork with **no warranty and no
liability accepted by the author(s)** for any use of this software. It is not affiliated with or endorsed
by Binance or BNB Chain. Treat it as a research/engineering case study, not as infrastructure to depend on.
