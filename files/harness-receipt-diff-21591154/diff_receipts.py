#!/usr/bin/env python3
"""FLOW-X04: diff local re-execute receipt dump vs public eth_getBlockReceipts fixture."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def normalize_public(doc: dict) -> list[dict]:
    receipts = doc.get("receipts") or doc
    if isinstance(receipts, dict) and "receipts" in receipts:
        receipts = receipts["receipts"]
    out = []
    prev = 0
    for i, r in enumerate(receipts):
        cum = int(r["cumulativeGasUsed"], 16) if isinstance(r["cumulativeGasUsed"], str) else int(r["cumulativeGasUsed"])
        gas = int(r["gasUsed"], 16) if isinstance(r.get("gasUsed", "0x0"), str) else int(r.get("gasUsed", cum - prev))
        status_raw = r.get("status", "0x1")
        status = int(status_raw, 16) if isinstance(status_raw, str) else int(status_raw)
        out.append(
            {
                "i": i,
                "txHash": (r.get("transactionHash") or r.get("txHash") or "").lower(),
                "status": status,
                "gasUsed": gas,
                "cumulativeGasUsed": cum,
                "logCount": len(r.get("logs") or []),
            }
        )
        prev = cum
    return out


def normalize_local(doc: dict) -> list[dict]:
    rows = doc.get("receipts") or []
    out = []
    for r in rows:
        out.append(
            {
                "i": int(r["i"]),
                "txHash": (r.get("txHash") or "").lower(),
                "status": int(r["status"]),
                "gasUsed": int(r["gasUsed"]),
                "cumulativeGasUsed": int(r["cumulativeGasUsed"]),
                "logCount": int(r["logCount"]),
            }
        )
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--public", type=Path, required=True)
    ap.add_argument("--local", type=Path, required=True)
    args = ap.parse_args()

    pub = normalize_public(json.loads(args.public.read_text()))
    loc = normalize_local(json.loads(args.local.read_text()))

    n = min(len(pub), len(loc))
    if len(pub) != len(loc):
        print(f"WARN receipt count public={len(pub)} local={len(loc)} (comparing first {n})")

    keys = ("status", "gasUsed", "cumulativeGasUsed", "logCount")
    for i in range(n):
        p, l = pub[i], loc[i]
        diffs = {k: (p[k], l[k]) for k in keys if p[k] != l[k]}
        if diffs:
            print(f"FIRST_MISMATCH index={i}")
            print(f"  txHash public={p['txHash']} local={l['txHash']}")
            for k, (a, b) in diffs.items():
                print(f"  {k}: public={a} local={b}")
            return 1

    print(f"OK compared {n} receipts (status/gasUsed/cumGas/logCount)")
    if len(pub) != len(loc):
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
