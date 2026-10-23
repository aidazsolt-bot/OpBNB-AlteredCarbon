---
name: opbnb-headers-flow
description: >-
  Diagnose opBNB Headers Tip-DoD / reverse-download stalls (FLOW-H01…H07, E03).
  Use when Headers shows target=None, flushed≪downloaded, Cap working_tip,
  Bound Tip HeaderSeed, or Tip catch-up after Finish/Skip EL.
---

# opBNB Headers FLOW (Tip-DoD / Cap / Seed)

Companion to `reth-opbnb-port`. External overview: `.cursor/skills/reth` (hairyf).
Authoritative gates live in `plan.md` PORT-FLOW-H* / E03.

## Symptom → likely gate

| Live signal | Gate | Meaning |
| --- | --- | --- |
| No `FCU tip buffered+seeded, starting backfill` | **E03** | Backfill Start used `finalized` (canonical) instead of buffered FCU **head** |
| `Bound Tip` then `Capping … working_tip=N` on a **seeded** tip | **H07 bug** | Must not Cap seeded Tip — Falling from HeaderSeed (fixed 2026-10-23) |
| `Received headers total=1` then silence after Cap | **H07** | Classic Cap-to-Number stall; expect Falling batches instead |
| Status `stage=Headers target=None` for Tip(hash) | often **cosmetic** | Pipeline tip is hash; numeric target unset until Cap/Number |
| `ChannelClosed` / `MissingDownloadBuffer` | stream ended early | Downloader `Ready(None)` after `clear()` |

## Invariants (do not violate)

1. **Tip number only from HeaderSeed (NewPayload)** — never sole `GetBlockHeaders(hash,1)` (E03).
2. **Cap must not clear `sync_target_request`** on Tip churn while already capped (H07).
3. **Cap Number tip must prime Falling** (`next_request_block_number > local`) — regression: only tip flushed then stall (unit `caps_working_tip_to_max_peer_best`).
4. **Empty → backoff, no ban** (H02/H07). Coalesced `empty_retry` can look like `in_flight=0` while Pending.
5. **Checkpoint / Writing headers** only after ETL fill (H05) — `downloaded` alone ≠ progress.

## Live checklist (≤60s)

```bash
source .cursor/local/opbnb-archive-paths.env
curl -s "http://${ARCHIVE_CT}:6060/metrics" | rg \
  'headers_total_(downloaded|flushed)|headers_(in_flight|queued|buffered)|headers_.*_errors|connected_peers |checkpoint\{stage="Headers"\}'
rg -n 'Bound Tip|Capping header|Received headers|Writing headers|FCU tip buffered|ChannelClosed|DetachedHead' \
  "$ARCHIVE_LOG" | tail -40
```

Healthy Tip-DoD: `Bound Tip` → (optional Cap) → repeated `Received headers` with falling numbers → `Writing headers` → Headers checkpoint↑.

## Root-cause order

1. Did E03 Start(head)? Log `starting backfill` + `backfill_hash` == FCU head.
2. Did Cap fire? Compare eventual CL tip vs `working_tip` / peer Status tips.
3. After Cap: Falling primed? (`next_request` activity, not only `total=1`).
4. P2P: Empty storm vs hung `sync_target_request` vs buffer full + can't validate.
5. Wrong Cap tip: `GetHeaders(Number(peer_best))` may not be an ancestor of seeded CL tip — validation/buffer stall.

## Code map (v2.8 tree)

- Engine Start(head): `engine/tree` `handle_missing_block`
- Seed: `HeaderSeed` + `seed_header_from_block`
- Cap / Falling: `downloaders/headers/reverse_headers.rs` (`apply_working_tip_cap`, `on_block_number_update`, `poll_next`)
- Peer eligibility: `network/fetch/mod.rs` `HeadersAtLeast` (eth/68 soft, eth/69 range)

## DoD

Fix only with PIPE+FLOW id, unit covering the stall mode, and live: Headers checkpoint advances toward public tip (or documented peer-coverage limit).
