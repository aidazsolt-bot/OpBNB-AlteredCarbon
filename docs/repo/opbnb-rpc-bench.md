# opBNB local archive RPC vs public endpoints (2026-11-10)

Measured from the operator archive host (EU) against a tip-following **op-reth** archive
(`~204.93 M`, IPC NDJSON) and two public HTTPS RPCs with **warm keep-alive**:

| Endpoint | Role |
| --- | --- |
| Local IPC | This fork’s archive node |
| `https://opbnb-mainnet-rpc.bnbchain.org` | Official public |
| `https://opbnb-rpc.publicnode.com` | PublicNode |

Raw numbers: [`opbnb-rpc-local-vs-public-2026-11-10.json`](./opbnb-rpc-local-vs-public-2026-11-10.json)
(+ IPC-only tip suite [`opbnb-ipc-rpc-bench-2026-11-10.json`](./opbnb-ipc-rpc-bench-2026-11-10.json)).

## Headline

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

## Takeaways

1. **Tip + header reads:** local IPC is sub-millisecond; public warm HTTPS is tens–hundreds of ms
   (WAN + shared fleet). Even the faster public (~30–50 ms) is still **hundreds of times** slower.
2. **Archive state:** public endpoints rejected historical `eth_getBalance` / `eth_call` with
   `missing trie node` / state unavailable. The local archive answered in **&lt;0.5 ms**.
3. **Logs:** still a clear win locally (low-ms vs 30–120 ms), with a smaller multiple because
   work is heavier.
4. Not a claim of “faster than every hosted archive product” — only that a **local archive IPC**
   beats these **keyless public** URLs for latency and historical state depth on this host.

## Method (short)

- Local: Unix-socket JSON-RPC (NDJSON line protocol), sequential samples after warmup.
- Public: HTTPS keep-alive reuse (warm); cold TCP+TLS not counted in the headline table.
- Contract probe: WBNB `0x4200…0006`, `totalSupply` selector `0x18160ddd`, ERC-20 `Transfer` topic.
