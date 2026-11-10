#!/usr/bin/env python3
"""Parallel + JSON-RPC batch RPC bench: local IPC vs public HTTPS."""

from __future__ import annotations

import http.client
import json
import os
import socket
import ssl
import statistics
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from urllib.parse import urlparse

IPC = os.environ["ARCHIVE_IPC"]
OUT = os.path.join(
    os.path.dirname(__file__), "opbnb-rpc-parallel-batch-2026-11-10.json"
)
PUBS = {
    "bnbchain": "https://opbnb-mainnet-rpc.bnbchain.org",
    "publicnode": "https://opbnb-rpc.publicnode.com",
}
WBNB = "0x4200000000000000000000000000000000000006"
TOTAL = "0x18160ddd"


def ipc_connect() -> socket.socket:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(120)
    s.connect(IPC)
    return s


def ipc_tip() -> int:
    s = ipc_connect()
    try:
        s.sendall(b'{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}\n')
        buf = b""
        while b"\n" not in buf:
            chunk = s.recv(1 << 16)
            if not chunk:
                break
            buf += chunk
        return int(json.loads(buf.split(b"\n", 1)[0])["result"], 16)
    finally:
        s.close()


def ipc_single(method: str, params: list):
    req = (
        json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}) + "\n"
    ).encode()
    s = ipc_connect()
    t0 = time.perf_counter()
    try:
        s.sendall(req)
        buf = b""
        while b"\n" not in buf:
            chunk = s.recv(1 << 20)
            if not chunk:
                break
            buf += chunk
        dt = (time.perf_counter() - t0) * 1000
        resp = json.loads(buf.split(b"\n", 1)[0])
        return dt, "result" in resp, resp.get("error")
    finally:
        s.close()


def ipc_batch_array(calls: list[tuple[str, list]]):
    payload = [
        {"jsonrpc": "2.0", "id": i + 1, "method": m, "params": p}
        for i, (m, p) in enumerate(calls)
    ]
    raw = (json.dumps(payload) + "\n").encode()
    s = ipc_connect()
    t0 = time.perf_counter()
    try:
        s.sendall(raw)
        buf = b""
        while True:
            chunk = s.recv(1 << 20)
            if not chunk:
                break
            buf += chunk
            if b"\n" in buf:
                break
        dt = (time.perf_counter() - t0) * 1000
        resp = json.loads(buf.split(b"\n", 1)[0])
        if not isinstance(resp, list):
            return dt, 0, len(calls), {"bad": type(resp).__name__}
        ok = sum(1 for r in resp if isinstance(r, dict) and "result" in r)
        return dt, ok, len(resp) - ok, None
    finally:
        s.close()


class HttpPool:
    def __init__(self, url: str):
        self.u = urlparse(url)
        self.local = __import__("threading").local()
        self.ctx = ssl.create_default_context()

    def _conn(self):
        c = getattr(self.local, "conn", None)
        if c is None:
            c = http.client.HTTPSConnection(
                self.u.hostname, self.u.port or 443, timeout=60, context=self.ctx
            )
            self.local.conn = c
        return c

    def single(self, method: str, params: list):
        body = json.dumps(
            {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
        ).encode()
        t0 = time.perf_counter()
        try:
            c = self._conn()
            c.request(
                "POST",
                self.u.path or "/",
                body=body,
                headers={"Content-Type": "application/json", "Connection": "keep-alive"},
            )
            raw = c.getresponse().read()
            dt = (time.perf_counter() - t0) * 1000
            j = json.loads(raw)
            return dt, "result" in j, j.get("error")
        except Exception as e:
            try:
                self.local.conn.close()
            except Exception:
                pass
            self.local.conn = None
            return (time.perf_counter() - t0) * 1000, False, {"message": str(e)}

    def batch(self, calls: list[tuple[str, list]]):
        payload = [
            {"jsonrpc": "2.0", "id": i + 1, "method": m, "params": p}
            for i, (m, p) in enumerate(calls)
        ]
        body = json.dumps(payload).encode()
        t0 = time.perf_counter()
        try:
            c = self._conn()
            c.request(
                "POST",
                self.u.path or "/",
                body=body,
                headers={"Content-Type": "application/json", "Connection": "keep-alive"},
            )
            raw = c.getresponse().read()
            dt = (time.perf_counter() - t0) * 1000
            resp = json.loads(raw)
            if not isinstance(resp, list):
                return dt, 0, len(calls), {"non_list": str(resp)[:200]}
            ok = sum(1 for x in resp if isinstance(x, dict) and "result" in x)
            return dt, ok, len(resp) - ok, None
        except Exception as e:
            try:
                self.local.conn.close()
            except Exception:
                pass
            self.local.conn = None
            return (time.perf_counter() - t0) * 1000, 0, len(calls), {"message": str(e)}


def pct(vals, p):
    s = sorted(vals)
    return s[min(len(s) - 1, int(round((len(s) - 1) * p)))]


def summarize(lats, ok, err, wall, units_ok):
    return {
        "ok": ok,
        "errors": err,
        "wall_s": round(wall, 3),
        "qps": round(units_ok / wall, 1) if wall > 0 else 0,
        "lat_ms": None
        if not lats
        else {
            "p50": round(statistics.median(lats), 3),
            "p90": round(pct(lats, 0.9), 3),
            "p99": round(pct(lats, 0.99), 3),
            "mean": round(statistics.mean(lats), 3),
            "min": round(min(lats), 3),
            "max": round(max(lats), 3),
        },
    }


def parallel_single(name, worker, method, params, n, conc):
    lats, ok, err = [], 0, 0
    t0 = time.perf_counter()
    with ThreadPoolExecutor(max_workers=conc) as ex:
        futs = [ex.submit(worker, method, params) for _ in range(n)]
        for f in as_completed(futs):
            dt, good, _e = f.result()
            if good:
                ok += 1
                lats.append(dt)
            else:
                err += 1
    wall = time.perf_counter() - t0
    out = summarize(lats, ok, err, wall, ok)
    out.update(
        {
            "name": name,
            "mode": "parallel_single",
            "method": method,
            "n": n,
            "concurrency": conc,
        }
    )
    return out


def parallel_batches(name, batch_fn, make_calls, batch_size, n_batches, conc):
    lats, ok, err, meta_err = [], 0, 0, None
    t0 = time.perf_counter()

    def one(_):
        return batch_fn(make_calls(batch_size))

    with ThreadPoolExecutor(max_workers=conc) as ex:
        futs = [ex.submit(one, i) for i in range(n_batches)]
        for f in as_completed(futs):
            dt, o, e, me = f.result()
            lats.append(dt)
            ok += o
            err += e
            if me and meta_err is None:
                meta_err = me
    wall = time.perf_counter() - t0
    out = summarize(lats, ok, err, wall, ok)
    out.update(
        {
            "name": name,
            "mode": "parallel_batch",
            "batch_size": batch_size,
            "n_batches": n_batches,
            "concurrency": conc,
            "calls_total": n_batches * batch_size,
            "batch_qps": round(n_batches / wall, 1) if wall > 0 else 0,
            "meta_err": meta_err,
        }
    )
    return out


def main():
    tip = ipc_tip()
    print("tip", tip)
    results = []
    pools = {k: HttpPool(v) for k, v in PUBS.items()}

    def make_tip_getblock(n):
        return [("eth_getBlockByNumber", ["latest", False]) for _ in range(n)]

    def make_tip_call(n):
        return [("eth_call", [{"to": WBNB, "data": TOTAL}, "latest"]) for _ in range(n)]

    def make_mixed(n):
        out = []
        for i in range(n):
            r = i % 3
            if r == 0:
                out.append(("eth_getBlockByNumber", ["latest", False]))
            elif r == 1:
                out.append(("eth_call", [{"to": WBNB, "data": TOTAL}, "latest"]))
            else:
                out.append(
                    ("eth_getBlockByNumber", [hex(max(1, tip - 100_000)), False])
                )
        return out

    def run_par(label, method, params, heavy=True):
        cfgs = [(50, 8), (100, 16), (200, 32)]
        if heavy:
            cfgs += [(400, 64), (800, 128)]
        for n, c in cfgs:
            r = parallel_single(f"{label}_local_c{c}_n{n}", ipc_single, method, params, n, c)
            results.append(r)
            print(
                f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
            )
            if c <= 32:
                for pk, pool in pools.items():
                    r = parallel_single(
                        f"{label}_{pk}_c{c}_n{n}", pool.single, method, params, n, c
                    )
                    results.append(r)
                    print(
                        f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
                    )

    print("=== parallel tip getBlock ===")
    run_par("par_tip_getBlock", "eth_getBlockByNumber", ["latest", False])

    print("=== parallel tip eth_call ===")
    run_par(
        "par_tip_eth_call",
        "eth_call",
        [{"to": WBNB, "data": TOTAL}, "latest"],
    )

    print("=== parallel hist getBlock ===")
    run_par(
        "par_hist_getBlock",
        "eth_getBlockByNumber",
        [hex(max(1, tip - 100_000)), False],
    )

    print("=== parallel hist getBalance ===")
    for n, c in [(100, 16), (200, 32), (400, 64), (800, 128)]:
        r = parallel_single(
            f"par_hist_bal_local_c{c}_n{n}",
            ipc_single,
            "eth_getBalance",
            [WBNB, hex(100_000)],
            n,
            c,
        )
        results.append(r)
        print(
            f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
        )
    for n, c in [(40, 8), (80, 16)]:
        for pk, pool in pools.items():
            r = parallel_single(
                f"par_hist_bal_{pk}_c{c}_n{n}",
                pool.single,
                "eth_getBalance",
                [WBNB, hex(100_000)],
                n,
                c,
            )
            results.append(r)
            print(
                f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
            )

    print("=== JSON-RPC batches ===")
    batch_cfgs = [
        (10, 40, 8),
        (50, 40, 8),
        (100, 40, 8),
        (50, 80, 16),
        (100, 80, 16),
        (100, 100, 32),
        (200, 50, 16),
    ]
    for bs, nb, conc in batch_cfgs:
        r = parallel_batches(
            f"batch_tip_getBlock_local_bs{bs}_nb{nb}_c{conc}",
            ipc_batch_array,
            make_tip_getblock,
            bs,
            nb,
            conc,
        )
        results.append(r)
        print(
            f"{r['name']:58s} call_qps={r['qps']:>8} batch_qps={r['batch_qps']:>6} "
            f"batch_p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
        )
        r = parallel_batches(
            f"batch_mixed_local_bs{bs}_nb{nb}_c{conc}",
            ipc_batch_array,
            make_mixed,
            bs,
            nb,
            conc,
        )
        results.append(r)
        print(
            f"{r['name']:58s} call_qps={r['qps']:>8} batch_qps={r['batch_qps']:>6} "
            f"batch_p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
        )
        if conc <= 16 and bs <= 100 and nb <= 40:
            for pk, pool in pools.items():
                r = parallel_batches(
                    f"batch_tip_getBlock_{pk}_bs{bs}_nb{nb}_c{conc}",
                    pool.batch,
                    make_tip_getblock,
                    bs,
                    nb,
                    conc,
                )
                results.append(r)
                print(
                    f"{r['name']:58s} call_qps={r['qps']:>8} batch_qps={r['batch_qps']:>6} "
                    f"batch_p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']} meta={r.get('meta_err')}"
                )

    print("=== local burst ===")
    for n, c in [(2000, 128), (4000, 256)]:
        r = parallel_single(
            f"burst_tip_getBlock_local_c{c}_n{n}",
            ipc_single,
            "eth_getBlockByNumber",
            ["latest", False],
            n,
            c,
        )
        results.append(r)
        print(
            f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
        )
        r = parallel_single(
            f"burst_tip_call_local_c{c}_n{n}",
            ipc_single,
            "eth_call",
            [{"to": WBNB, "data": TOTAL}, "latest"],
            n,
            c,
        )
        results.append(r)
        print(
            f"{r['name']:52s} qps={r['qps']:>8} p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
        )

    r = parallel_batches(
        "burst_batch_mixed_local_bs200_nb200_c32",
        ipc_batch_array,
        make_mixed,
        200,
        200,
        32,
    )
    results.append(r)
    print(
        f"{r['name']:58s} call_qps={r['qps']:>8} batch_qps={r['batch_qps']:>6} "
        f"batch_p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
    )

    r = parallel_batches(
        "burst_batch_tip_getBlock_local_bs200_nb200_c64",
        ipc_batch_array,
        make_tip_getblock,
        200,
        200,
        64,
    )
    results.append(r)
    print(
        f"{r['name']:58s} call_qps={r['qps']:>8} batch_qps={r['batch_qps']:>6} "
        f"batch_p50={r['lat_ms'] and r['lat_ms']['p50']} err={r['errors']}"
    )

    report = {
        "when": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "tip": tip,
        "local": "IPC NDJSON + JSON-RPC batch arrays",
        "public": PUBS,
        "note": (
            "Parallel = concurrent single requests. Batch = JSON-RPC array per request, "
            "multiple in flight. Public limited to milder concurrency."
        ),
        "results": results,
    }
    with open(OUT, "w") as f:
        json.dump(report, f, indent=2)
    print("wrote", OUT, "n_results", len(results))


if __name__ == "__main__":
    main()
