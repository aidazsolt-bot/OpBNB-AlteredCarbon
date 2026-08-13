#!/usr/bin/env python3
"""Summarize public fixture for PORT-EXEC-001 / block 21591154."""

from __future__ import annotations

import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

DEFAULT = Path(__file__).resolve().parents[1] / "receipts-21591154-public.json"


def main() -> None:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT
    doc = json.loads(path.read_text())
    b = doc["block"]
    rcpts = doc["receipts"]
    ts = int(b["timestamp"], 16)
    print("number", int(b["number"], 16))
    print("hash", b["hash"])
    print("timestamp", ts, datetime.fromtimestamp(ts, timezone.utc).isoformat())
    print("receiptsRoot", b["receiptsRoot"])
    print("stateRoot", b["stateRoot"])
    print("txs", len(b.get("transactions") or rcpts))
    print("receipts", len(rcpts))
    types = Counter(r.get("type", "?") for r in rcpts)
    print("receipt types", dict(types))
    fails = [i for i, r in enumerate(rcpts) if int(r.get("status", "0x1"), 16) == 0]
    print("failed idxs", fails)
    snow, canyon = 1713160800, 1718870400
    print("Snow active", ts >= snow, "Canyon active", ts >= canyon)


if __name__ == "__main__":
    main()
