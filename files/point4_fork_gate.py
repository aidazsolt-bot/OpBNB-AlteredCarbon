#!/usr/bin/env python3
"""Point-4 fork-window gate: local IPC vs public opBNB mainnet RPC.

Compares hash / transactionsRoot / stateRoot (and receiptsRoot) at fork
activation ±N and a few nearby heights. Gates on Mimir Execution checkpoint
unless --force.

Forks (opBNB mainnet 204, resolved 2026-08-16 vs public RPC):
  Fermat  9_397_477   ts 1701151200
  Haber  27_118_477   ts 1718872200
  Wright 32_984_677   ts 1724738400

Usage (from repo root):
  python3 files/point4_fork_gate.py haber          # wait-ready check + run if Exec≥Haber
  python3 files/point4_fork_gate.py haber --dry-run # print plan + Exec gap only
  python3 files/point4_fork_gate.py wright
  python3 files/point4_fork_gate.py fermat --force  # re-verify past gate

Output JSON under files/point4-<fork>-YYYYMMDD-HHMMSS.json
"""
from __future__ import annotations

import argparse
import json
import os
import socket
import sys
import time
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

_REPO = Path(__file__).resolve().parents[1]
_LOCAL_ENV = _REPO / ".cursor" / "local" / "opbnb-archive-paths.env"
_OUT_DIR = _REPO / "files"

PUBLIC = "https://opbnb-mainnet-rpc.bnbchain.org"
MIMIR = "http://<metrics-api>/api/v1/query"

# First block with fork timestamp (mainnet).
FORKS: dict[str, dict[str, Any]] = {
    "fermat": {
        "height": 9_397_477,
        "ts": 1_701_151_200,
        "pipe": "PIPE-007",
        "flow": "FLOW-X01",
        "note": "Fermat precompiles 0x66/0x67",
    },
    "haber": {
        "height": 27_118_477,
        "ts": 1_718_872_200,
        "pipe": "PIPE-008",
        "flow": "FLOW-X01",
        "note": "Haber early p256 @ 0x100 (pre-Fjord)",
    },
    "wright": {
        "height": 32_984_677,
        "ts": 1_724_738_400,
        "pipe": "PIPE-009",
        "flow": "FLOW-X02",
        "note": "Wright L1-fee skip only when gasPrice==0",
    },
}


def _load_local_env(path: Path) -> None:
    if not path.is_file():
        return
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, val = line.partition("=")
        os.environ.setdefault(key.strip(), val.strip())


_load_local_env(_LOCAL_ENV)

_ARCHIVE_CT = os.environ.get("ARCHIVE_CT", "archive-ct")
IPC = os.environ.get("ARCHIVE_IPC", f"/tmp/{_ARCHIVE_CT}.ipc")
INSTANCE = f"{_ARCHIVE_CT}:6060"


def rpc_unix(method: str, params: list[Any] | None = None) -> Any:
    req = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params or []}
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(90)
    s.connect(IPC)
    s.sendall((json.dumps(req) + "\n").encode())
    buf = b""
    while True:
        chunk = s.recv(1 << 20)
        if not chunk:
            break
        buf += chunk
        try:
            data = json.loads(buf.decode())
            if "error" in data and data["error"]:
                raise RuntimeError(f"IPC {method}: {data['error']}")
            return data.get("result")
        except json.JSONDecodeError:
            continue
    data = json.loads(buf.decode())
    if "error" in data and data["error"]:
        raise RuntimeError(f"IPC {method}: {data['error']}")
    return data.get("result")


def rpc_http(url: str, method: str, params: list[Any] | None = None) -> Any:
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or []}).encode()
    req = urllib.request.Request(
        url, data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        data = json.load(r)
    if data.get("error"):
        raise RuntimeError(f"HTTP {method}: {data['error']}")
    return data.get("result")


def mimir_execution() -> int:
    q = f'reth_sync_checkpoint{{instance="{INSTANCE}",stage="Execution"}}'
    url = MIMIR + "?" + urllib.parse.urlencode({"query": q})
    with urllib.request.urlopen(url, timeout=15) as r:
        data = json.load(r)
    rows = data.get("data", {}).get("result", [])
    if not rows:
        raise RuntimeError("no Execution checkpoint from Mimir")
    return int(float(rows[0]["value"][1]))


def pick_heights(fork_h: int, exec_ckpt: int) -> list[int]:
    """Activation ±1, +100, +1000, and near Exec tip if past fork."""
    raw = [
        fork_h - 1,
        fork_h,
        fork_h + 1,
        fork_h + 100,
        fork_h + 1000,
    ]
    if exec_ckpt > fork_h + 1000:
        raw.append(min(exec_ckpt, fork_h + 10_000))
        raw.append(max(fork_h, exec_ckpt - 1000))
    seen: set[int] = set()
    out: list[int] = []
    for h in raw:
        if h < 1 or h > exec_ckpt or h in seen:
            continue
        seen.add(h)
        out.append(h)
    return out


def compare_block(height: int) -> dict[str, Any]:
    hx = hex(height)
    local = rpc_unix("eth_getBlockByNumber", [hx, False])
    public = rpc_http(PUBLIC, "eth_getBlockByNumber", [hx, False])
    if not local:
        return {"n": height, "ok": False, "err": "local missing (not in DB / RPC)"}
    if not public:
        return {"n": height, "ok": False, "err": "public missing"}
    fields = ("hash", "transactionsRoot", "stateRoot", "receiptsRoot")
    row: dict[str, Any] = {
        "n": height,
        "ts": int(local.get("timestamp") or "0x0", 16),
        "ok": True,
    }
    diffs: dict[str, Any] = {}
    for f in fields:
        lv, pv = local.get(f), public.get(f)
        row[f] = lv
        if lv != pv:
            diffs[f] = {"local": lv, "public": pv}
            row["ok"] = False
    if diffs:
        row["diffs"] = diffs
    return row


def eta_hours(delta: int, blk_per_s: float) -> float | None:
    if delta <= 0:
        return 0.0
    if blk_per_s <= 0:
        return None
    return delta / blk_per_s / 3600.0


def main() -> int:
    ap = argparse.ArgumentParser(description="Point-4 fork gate (IPC vs public)")
    ap.add_argument("fork", choices=sorted(FORKS.keys()))
    ap.add_argument("--dry-run", action="store_true", help="print readiness only")
    ap.add_argument("--force", action="store_true", help="run even if Exec < fork height")
    ap.add_argument(
        "--blk-per-s",
        type=float,
        default=0.0,
        help="optional rate for ETA (else inferred from recent journal not available → skip)",
    )
    args = ap.parse_args()

    meta = FORKS[args.fork]
    fork_h = int(meta["height"])
    exec_ckpt = mimir_execution()
    delta = fork_h - exec_ckpt
    ready = exec_ckpt >= fork_h

    print(f"=== Point-4 gate: {args.fork} ===")
    print(f"fork height={fork_h} ts={meta['ts']}  {meta['pipe']}/{meta['flow']}")
    print(f"note: {meta['note']}")
    print(f"IPC={IPC}")
    print(f"Execution checkpoint={exec_ckpt}  delta_to_fork={delta}  ready={ready}")

    if args.dry_run or (not ready and not args.force):
        if args.blk_per_s > 0 and delta > 0:
            print(f"ETA ≈ {eta_hours(delta, args.blk_per_s):.1f} h @ {args.blk_per_s} blk/s")
        if not ready and not args.force:
            print("NOT READY — re-run when Exec ≥ fork (or pass --force).")
            return 2
        if args.dry_run:
            heights = pick_heights(fork_h, max(exec_ckpt, fork_h + 1000))
            print(f"planned heights (once ready): {heights}")
            return 0

    heights = pick_heights(fork_h, exec_ckpt)
    if not heights:
        print("FAIL: no heights ≤ Execution checkpoint")
        return 1

    print(f"sampling heights: {heights}")
    rows: list[dict[str, Any]] = []
    for h in heights:
        try:
            row = compare_block(h)
        except Exception as e:  # noqa: BLE001 — report per-height, continue
            row = {"n": h, "ok": False, "err": str(e)}
        rows.append(row)
        status = "MATCH" if row.get("ok") else "MISMATCH"
        print(f"  {h}: {status}" + (f" err={row.get('err')}" if row.get("err") else ""))
        if row.get("diffs"):
            for f, d in row["diffs"].items():
                print(f"    {f}: local={d['local']} public={d['public']}")
        time.sleep(0.15)

    overall = "MATCH" if all(r.get("ok") for r in rows) else "MISMATCH"
    stamp = datetime.now(timezone.utc).astimezone().strftime("%Y%m%d-%H%M%S")
    out = {
        "when": datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds"),
        "fork": args.fork,
        "fork_height": fork_h,
        "fork_ts": meta["ts"],
        "pipe": meta["pipe"],
        "flow": meta["flow"],
        "exec_ckpt": exec_ckpt,
        "ipc": IPC,
        "public": PUBLIC,
        "overall": overall,
        "heights": rows,
        "semantics": (
            "Matching header stateRoot via eth_getBlockByNumber is necessary but not "
            "sufficient for full EVM correctness; MerkleExecute + no unwind storms are stronger."
        ),
    }
    out_path = _OUT_DIR / f"point4-{args.fork}-{stamp}.json"
    out_path.write_text(json.dumps(out, indent=2) + "\n")
    # stable symlink-ish latest name for agents
    latest = _OUT_DIR / f"point4-{args.fork}-latest.json"
    latest.write_text(json.dumps(out, indent=2) + "\n")
    print(f"overall={overall}")
    print(f"wrote {out_path}")
    print(f"wrote {latest}")
    return 0 if overall == "MATCH" else 1


if __name__ == "__main__":
    sys.exit(main())
