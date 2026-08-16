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

### Permitted use — private individuals only (non-commercial)

**The results of this experiment** (this repository as packaged, including author-original port/docs/skills,
effort logs, and any binaries or artifacts derived from this tree for the experiment) are made available
**only for private natural persons** for personal, educational, or hobby use.

**Not permitted** without a separate written license from the project author:

- any **commercial** use (paid services, SaaS, consulting deliverables, product bundling, token/ops revenue, …);
- any use **by or for companies / enterprises / organizations** (including internal R&D, staging, or production
  infrastructure), whether or not money changes hands;
- redistribution of this experimental packaging **for** commercial or organizational purposes.

See **`NOTICE-PERSONAL-USE.md`** for the full additional terms and the relationship to upstream licenses.

**Why not GPL / public domain for this restriction?** GPL, MIT, Apache-2.0, and public-domain dedications
all **allow commercial use**. A non-commercial / private-individuals-only policy cannot honestly be expressed
by relicensing the whole tree as GPL or “open domain.” Upstream Reth code remains under its existing
**Apache-2.0 OR MIT** terms (`LICENSE-APACHE` / `LICENSE-MIT`); those rights are **not** revoked here. The
additional personal-use terms apply to **this experiment’s packaging and author-original material** as stated
in `NOTICE-PERSONAL-USE.md`. For production BSC/opBNB, use official maintained clients.

**No warranty, no liability, use at your own risk.** This software is provided "AS IS", without warranty of
any kind, express or implied, including but not limited to fitness for a particular purpose, merchantability,
or non-infringement. The author(s) and contributors accept **no responsibility or liability whatsoever** for
any damages, financial losses, chain-consensus incidents, data loss, or other harm arising from the use,
misuse, or inability to use this software — whether run as a node, a library, or in any other capacity. This
code has **not** been security-audited and should not be trusted with real funds or run against mainnet
without independent review. If you need a supported client for BNB Smart Chain or opBNB, use one of the
actively maintained official clients instead (see upstream project links further below).

**DE (Kurzfassung):** Die Ergebnisse dieses Experiments stehen **nur Privatpersonen** für persönliche /
nicht-kommerzielle Nutzung zur Verfügung. **Kommerzielle Nutzung und jeder Einsatz in/durch Unternehmen
oder Organisationen sind nicht gestattet.** Details: `NOTICE-PERSONAL-USE.md`. Upstream-Reth bleibt
Apache-2.0/MIT; GPL/Public Domain würden kommerzielle Nutzung erlauben und werden deshalb für diese
Zusatzbeschränkung **nicht** verwendet.

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
    --log.file.directory ./datadir/logs
```

New databases use storage V2 by default (`--storage.v2`; opt out with `--storage.v2=false`).
Legacy BSC flags `--enable-prefetch` / `--optimize.enable-execution-cache` are **obsolete** on this
v2.4.1 rebase — use engine prewarming/cache controls instead (e.g. `--engine.disable-prewarming` to
opt out; see `bsc-reth node --help` under Engine).

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
    --log.file.directory /data/logs
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
    --log.file.directory ./datadir/logs
```

Built-in chain names: `opbnb` / `opbnb-mainnet` (mainnet), `opbnb-testnet`, `opbnb-qa`
(so `opbnb-${network}` with `network=mainnet|testnet` is correct).
New databases default to storage V2 (`--storage.v2`; `--storage.v2=false` for legacy v1).
Do **not** pass the old BSC flags `--enable-prefetch` / `--optimize.enable-execution-cache` — they are
not wired on this rebase; use `--engine.*` prewarming/cache flags instead.

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
    --log.file.directory /data/logs
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

**Availability:** experiment results are for **private individuals only** (personal / non-commercial).
**Commercial use and any company/enterprise use are not permitted.** See the Project Notice and
`NOTICE-PERSONAL-USE.md`. Upstream Reth remains Apache-2.0 OR MIT; this project does **not** relicense
the tree as GPL or public domain (those allow commercial use and would not express the NC intent).

### Method

Work was performed interactively with AI coding agents across multiple sessions — primarily
**GitHub Copilot CLI** (session `a95758da`, 2026-08-06–07) and follow-on **Cursor Composer**
sessions (2026-08-09–11) — using a mix of direct agent-driven edits and delegated background
sub-agents supervising/verifying each other's changes (given the scale of the merge — 200+
conflicting files across the initial rebase alone). Progress was checkpointed via small,
incrementally verified git commits where possible, specifically to keep the change history
auditable and revertible given the semi-autonomous nature of the work. Large mid-session compile
loops may leave uncommitted working-tree diffs until explicitly reviewed (see `plan.md`).

### Method finding: AI needs an explicit porting skill / approach hints

A central result of this experiment: **generic AI coding agents were not able to perform a
meaningful protocol port on their own.** They can drive large compile/fix loops and surface
plausible diffs, but without **operator-supplied approach hints** (reference-first against
`bnb-chain/op-geth`, stage-by-stage live verify, dual **`PORT-PIPE` + `PORT-FLOW`** matrices,
Engine vs Pipeline vs Consensus vs Downloader layering) they repeatedly stop at “builds green”
or chase the wrong layer (e.g. Eth second-resolution timestamps, tip-chase instead of backfill,
Cap/Falling stalls treated as “live follow-ups” instead of missing dataflow analysis).

Those hints were therefore given explicitly during live-sync debugging (Session 10). From that
procedure the agent was instructed to author a reusable Cursor skill; Session 10 cont. then
hardened `plan.md` with a mandatory **Migrations-Gate** (PIPE = consensus rule, FLOW = state
machine / wire / persistence) so Bodies/Execution cannot repeat the Headers dataflow gap:

- **`.cursor/skills/reth-opbnb-port/SKILL.md`** — *Reth / opBNB Portierungs-Spezialist*
- **`.cursor/skills/rust-best-practices/SKILL.md`** — *Experienced Rust / best practices*
- **`.cursor/rules/reth-opbnb-port-mandatory.mdc`** — `alwaysApply: true` (load both skills first every session)
- **`.cursor/hooks.json`** `sessionStart` — injects both skills into agent context

Subsequent port/sync work on this fork **must** load both skills at session start (rule + hook enforce it)
and follow `plan.md` (**`PORT-PIPE-*` and `PORT-FLOW-*`**, DoD before live) instead of unaided vibecoding.

### Effort log (approximate, based on available session telemetry)

| Metric | Value |
| --- | --- |
| Elapsed wall-clock time (this rebase effort, across sessions) | Multiple sessions over several days (Copilot: ~2026-08-06 09:50 – 2026-08-07 18:05 UTC; Cursor Session 6: **2026-08-09**, ~5.34 h; Session 8: **~2026-08-09**, ~2.1 h; Session 9: **~2026-08-10**, ~1.9 h; Session 10 live sync: **2026-08-11**, chat `84eb0b61…`, **~4.8 h** Wall; **Session 12** chat `ea987bef…`: calendar **~88 h** 08-12→16, interactive clusters **~4.5 h** early + **~4 h** 08-15 evening / 08-16 morning) |
| LLM models used (Copilot session `a95758da`) | Claude Sonnet 5 (primary), GPT-5.4, Claude Sonnet 4.6, GPT-5.3-Codex, GPT-5.4-mini |
| LLM models used (Cursor Session 6, chat `42f88fe7…`) | **composer-2.5-fast** + **cursor-grok-4.5-high-fast**; parent `default` |
| LLM models used (Cursor Session 8, chat `d6ebb428…`) | Parent Auto/Composer router; ~816 tool calls in agent transcript |
| LLM models used (Cursor Session 9, chat `6a6455c9…`) | Parent Auto/Composer router + Task subagents (inherit); ~250 tool_use in resume transcript |
| LLM models used (Cursor Session 10, chat `84eb0b61…`) | Parent Auto/Composer; live opBNB archive sync — CONS/ENGINE + **P2P-003/004/005** + **Migrations-Gate PIPE+FLOW** |
| LLM models used (Cursor Session 12, chat `ea987bef…`) | Parent Auto/Composer (+ occasional Task); **84** user / **367** assistant; **567** tool_use |
| Approx. input tokens (Copilot `a95758da`) | **~650.1M** (+ ~636.2M cache-read) |
| Approx. output tokens (Copilot `a95758da`) | **~1.861M** |
| Approx. model wall time (Copilot `a95758da`) | ~8.1 hours / 5,803 usage events / 32 turns |
| Cursor Session 6 activity | **15 agents**; 2,582 assistant msgs; ~11,722 tool-calls; **74,482** `ai_code_hashes`; transcript proxy **~0.58M tokens** |
| Cursor Session 8 activity (op-evm→cli/bin→smoke) | Transcript **~0.45M chars → ~0.11M tokens** (÷4 proxy); **11,288** `ai_code_hashes`; 350 assistant / 18 user msgs in jsonl |
| Cursor Session 9 activity (STOR-006 + Phase-5 nextest/EF) | Resume **~0.11M chars → ~28K tokens** + prior SCS chat **~0.28M chars → ~69K** (÷4 proxy, combined **~97K**); 12 user / 118 assistant; 250 tools resume |
| Cursor Session 12 activity (EXEC-001 → UPnP / past Fail / X02 / A02) | Snapshot **08-16:** Transcript **~1.58 MB** → proxy **~396K tokens** (filesize÷4); calendar **~88 h**; interactive **~4.5 h** early + **~4 h** 08-15 evening/08-16 morning; billed **n/a** — `files/cursor-session12-metrics.json` + `…-20260816.json` |
| Session 10 maxperf rebuilds (test/deploy cost) | 3 successful fat-LTO builds @ ~20–23 min each (`CARGO_BUILD_JOBS=1`); plus failed tipresolve SIGKILL; unit tests fetch 43 + reverse_headers 11 |
| Illustrative API-equivalent cost (Copilot only, **not an invoice**) | Order-of-magnitude **~USD 1.5–2k** if the ~650M in / ~1.9M out were billed at public Sonnet/GPT list bands without cache discount. Cursor billed usage is **not** available on disk — use the Cursor account dashboard. Session 12 content proxy undercounts context resend; subscription pricing ≠ raw API. |
| Compile / runnable milestone (2026-08-10, Session 9) | StorageChangeSets SF (**PORT-STOR-006**); stages nextest **106/106**; EF **v17.0** → **62/62**. Catch-up/full sync = **human-owned** (see `plan.md`). |
| Live sync milestone (2026-08-11, Session 10) | **PORT-CONS-001**; **PORT-ENGINE-001/003**; **PORT-P2P-003/004/005** (reachable tip + Cap idempotent + Falling-Prime — Downloader-Dataflow, not live follow-ups): Falling from peer head ~173.37M @ ~22k hdr/s. Checkpoint 0 until ETL write (Upstream TempDir). |
| Live sync progress (2026-08-12 ~17:03 CEST) | Headers+Bodies+**Sender** = Tip **173 369 140**. **Execution ~10 M (~5.8 %)**, Fermat **`9397477` Point4 MATCH** (IPC). Block-ETA ~**24–25 h** (entities-lag → 2–4 d). CL Tip ~173.7 M (op-node Tip-Feed; L1-re-org warns = Dataseed noise). Next: Haber / FLOW-X02. Details: `plan.md` § Live Sync Progress. |
| Live sync + Session 12 (2026-08-13 ~16:00 CEST, chat `ea987bef…`) | **PORT-EXEC-001** receipt-root @ **`21591154`** → Unwind FLOW-X05 → Headers Tip **~174.0 M** again; Bodies rebuild. Harness + `re-execute --dump-receipts-on-fail`; maxperf rebuild-only `target/maxperf/op-reth` (~22 min). **Ops:** Exec ≤`21591153` then offline FLOW-X04. Upstream: stay on **2.4.1** (bnb/op not on 2.5). |
| Live sync Session 12 cont. (2026-08-14 ~13:35 CEST) | **2. Fail** same `21591154` (~5 min Exec). Cap: **`--debug.max-block`** (+`terminate`); `skip-fcu`≠block stop. Journal via machine journal path. MerkleExecute @ `21579110`. |
| Live sync Session 12 cont. (2026-08-14 ~18:01 CEST) | Dirty Cap → Merkle fail @`21579110` → unwind_to=0; Kill rettet Headers Tip **174 M**. Reload/Stop Panic `SelectNextSome` (ENGINE-004 parked). Bodies clean **0→21579110**. **Ops:** Process-Stop ≫ max-block; Cap only if checkpoints ≤ H (OPS-001). |
| Live sync Session 12 cont. (2026-08-14 ~21:26 CEST) | Bodies+Sender Cap ✅; Exec ~**6.5 M**→`21579110`. Point4 via IPC `/tmp/BSCRethArchiveNode.ipc` MATCH (no HTTP without `--http`). PORT-OPS-001/ENGINE-004 in `plan.md`. |
| Live sync Session 12 cont. (2026-08-15 ~10:54 CEST) | Offline X04 + SF-Gap + **Effort-Metriken**: Bodies/Sender→`21591154`; SF tip `20365614`≠Cap; Exec→`21591153`; CLI half-open `54..55`. Agent: **~4.5–6 h** interactive / proxy **~72K–216K** tok; `files/cursor-session12-metrics.json`. |
| Live sync Session 12 cont. (2026-08-15 ~11:47 CEST) | Docs: op-geth `ValidateState` (receipt+state eager) vs Reth Execution+MerkleExecute staged; `21591154` = receipt content (PIPE-014), not state-root formula. |
| Live sync Session 12 cont. (2026-08-15 evening → 08-16 ~08:30 CEST) | **P2P-002** UPnP live; Bodies+Sender Tip **174 M**; Exec past **`21591154`** (~22.7 M↑); **X02/PIPE-009** ≡ op-geth (Unit); CLEANUP-A02 partial. ETA Haber ~16–19 h / Wright ~1.5–2 d / Tip ~3–4 Wo. Metrics: `files/cursor-session12-metrics-20260816.json`. |
| Commits | See `git log` on `rebase/reth-v2.4.1` |
| Metrics snapshots | `files/cursor-session-metrics.json` (Session 6), `files/cursor-session8-metrics.json` (Session 8), `files/cursor-session9-metrics.json` (Session 9), **`files/cursor-session12-metrics.json`** + **`files/cursor-session12-metrics-20260816.json`** |
| Maxperf binary (local, **not committed**) | `make maxperf-op` → `target/maxperf/op-reth` + install **`dist/bin/op-reth-bnb`** (avoids overwriting a generic `op-reth` on PATH); default CLI chain `opbnb` |

These figures are session telemetry snapshots and are illustrative of the scale of context/inference
required for this kind of large structural migration; earlier pre-`a95758da` sessions add further
consumption (see historical rows in `plan.md`). They are provided for transparency about the practical
cost of AI-assisted maintenance at this scale, not as a benchmark claim — no rigorous token-efficiency
optimization was attempted. Copilot token counts include tool/context repetition per turn; Cursor
figures mix activity counts with content-size token **proxies** where a billed meter is unavailable.

> **TODO:** After human catch-up/full sync validation on BSC/opBNB, refresh final cumulative token/time
> figures (replace Cursor proxies with account billing export if available) and live-test outcome.

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
