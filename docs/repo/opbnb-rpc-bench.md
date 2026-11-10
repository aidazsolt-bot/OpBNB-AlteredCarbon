# opBNB local archive RPC vs public endpoints (2026-11-10)

Measured from the operator archive host (EU) against a tip-following **op-reth** archive
(`~204.93 M`, IPC NDJSON) and two public HTTPS RPCs with **warm keep-alive**:

| Endpoint | Role |
| --- | --- |
| Local IPC | This fork’s archive node |
| `https://opbnb-mainnet-rpc.bnbchain.org` | Official public |
| `https://opbnb-rpc.publicnode.com` | PublicNode |

Raw numbers:

- Sequential local-vs-public: [`opbnb-rpc-local-vs-public-2026-11-10.json`](./opbnb-rpc-local-vs-public-2026-11-10.json)
- IPC tip suite: [`opbnb-ipc-rpc-bench-2026-11-10.json`](./opbnb-ipc-rpc-bench-2026-11-10.json)
- Parallel + batch load: [`opbnb-rpc-parallel-batch-2026-11-10.json`](./opbnb-rpc-parallel-batch-2026-11-10.json)
- Repro script: [`bench_rpc_parallel_batch.py`](./bench_rpc_parallel_batch.py)

## Headline (sequential latency)

| Workload | Local p50 | Official public p50 | PublicNode p50 | Speedup vs official |
| --- | ---: | ---: | ---: | ---: |
| `eth_getBlockByNumber(latest)` | **0.09 ms** | 131 ms | 52 ms | **~1500×** |
| `eth_call` (WBNB `totalSupply`, tip) | **0.18 ms** | 121 ms | 31 ms | **~680×** |
| Historical `eth_getBlockByNumber` (1k … tip) | **~0.08–0.14 ms** | ~110–124 ms | ~23–26 ms | **~800–1400×** |
| Historical `eth_getBalance` / `eth_call` | **~0.15–0.48 ms** | **fail** (`missing trie node`) | **fail** (state not available) | archive-only |
| `eth_getLogs` WBNB Transfer (2k blocks, tip/mid/Fermat) | **~2.3–3.6 ms** | ~112–121 ms | ~28–40 ms | **~30–50×** |
| `eth_getLogs` 10k near tip | **9.1 ms** | 127 ms | 42 ms | **~14×** |

Point-4 header spot-checks (hash / `stateRoot` / `transactionsRoot`) vs official public:
**MATCH** at heights `1000`, `100000`, `9397477` (Fermat), and tip−1000.

## Parallel + JSON-RPC batch (throughput)

Same host (~204.93 M tip). Local = many concurrent IPC connections / batch arrays.
Public = milder concurrency (c≤32, smaller batches) to limit abuse; still enough to show the gap.

| Mode | Local | Official public | PublicNode |
| --- | ---: | ---: | ---: |
| Parallel `getBlock(latest)` c=8 | **~4.9 k QPS** | ~21 QPS | ~12 QPS |
| Parallel `getBlock(latest)` c=32 | **~5.7 k QPS** | ~67 QPS | ~61 QPS |
| Parallel `getBlock(latest)` c=128 (n=800) | **~5.0 k QPS** | — | — |
| Parallel `eth_call` tip c=32 | **~5.9 k QPS** | ~34 QPS | ~7 QPS (many errors) |
| Parallel `eth_call` tip c=128 | **~6.2 k QPS** | — | — |
| Parallel hist `getBalance` c=128 | **~6.1 k QPS** | **0** (100% fail) | **0** |
| Batch tip `getBlock` bs=10 c=8 | **~30 k call/s** | ~241 call/s | ~6 call/s + mass errors |
| Batch tip `getBlock` bs=100 c=8 | **~65 k call/s** | ~275 call/s | collapses |
| Batch mixed tip/hist bs=100 c=8 | **~70 k call/s** | — | — |
| Burst parallel getBlock c=256 n=4000 | **~5.0 k QPS** | — | — |
| Burst batch tip getBlock bs=200 ×200 c=64 | **~60 k call/s** | — | — |
| Burst batch mixed bs=200 ×200 c=32 | **~64 k call/s** | — | — |

Rough multiples at c=32 tip `getBlock`: local ≈ **85×** official public QPS, ≈ **90×** PublicNode.
Batch tip `getBlock` bs=100: local ≈ **230×** official call/s (PublicNode effectively unusable under that batch load from this host).

## Takeaways

1. **Tip + header reads:** local IPC is sub-millisecond; public warm HTTPS is tens–hundreds of ms
   (WAN + shared fleet). Even the faster public (~30–50 ms) is still **hundreds of times** slower.
2. **Archive state:** public endpoints rejected historical `eth_getBalance` / `eth_call` with
   `missing trie node` / state unavailable. The local archive answered in **&lt;0.5 ms** sequentially
   and sustained **~6 k QPS** in parallel.
3. **Logs:** still a clear win locally (low-ms vs 30–120 ms), with a smaller multiple because
   work is heavier.
4. **Load:** JSON-RPC batches push local into the **~30–70 k call/s** band; keyless public stays
   in the low hundreds (or errors / rate-limits). Parallel singles plateau ~**5–6 k QPS** locally.
5. Not a claim of “faster than every hosted archive product” — only that a **local archive IPC**
   beats these **keyless public** URLs for latency, depth, and sustained request rate on this host.

## Method (short)

- Local: Unix-socket JSON-RPC (NDJSON; batch = JSON array). Parallel = thread pool of connections.
- Public: HTTPS keep-alive; sequential headline uses warm reuse; parallel/batch uses limited concurrency.
- Contract probe: WBNB `0x4200…0006`, `totalSupply` selector `0x18160ddd`, ERC-20 `Transfer` topic.
