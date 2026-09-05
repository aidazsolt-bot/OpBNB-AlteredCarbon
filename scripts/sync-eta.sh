#!/usr/bin/env bash
# Live opBNB archive sync status + ETA snapshot.
#
# Pulls current stage checkpoints from the Mimir/Prometheus endpoint used by
# `.cursor/rules/opbnb-live-sync-health.mdc` and prints a short, copy-pasteable
# report (stage checkpoints, active-stage throughput over several windows, and
# a rough ETA) for pasting into a `plan.md` session entry.
#
# Usage: scripts/sync-eta.sh [instance] [prometheus_url]
#   instance:        Prometheus `instance` label (default: BSCRethArchiveNode:6060)
#   prometheus_url:  base URL, no trailing slash (default: http://grafana/api/v1)
#
# Does not write to plan.md itself — the agent/operator reviews the output and
# adds it to the relevant session entry, per the "update plan.md on milestones,
# not every poll" rule.
set -euo pipefail

INSTANCE="${1:-BSCRethArchiveNode:6060}"
PROM_URL="${2:-http://grafana/api/v1}"

python3 - "$INSTANCE" "$PROM_URL" <<'PYEOF'
import json
import subprocess
import sys
import datetime

instance, base = sys.argv[1], sys.argv[2]

def query(promql):
    out = subprocess.run(
        ["curl", "-s", "-m", "10", f"{base}/query", "--data-urlencode", f"query={promql}"],
        capture_output=True, text=True, check=True,
    ).stdout
    return json.loads(out)

# Stages in pipeline order (reth-bsc-trail / op-reth stages).
STAGES = [
    "Headers", "Bodies", "SenderRecovery", "Execution", "MerkleExecute",
    "AccountHashing", "StorageHashing", "IndexAccountHistory",
    "IndexStorageHistory", "TransactionLookup", "Prune", "Finish",
]

now = datetime.datetime.now(datetime.timezone.utc)
print(f"=== opBNB sync ETA snapshot — {now.strftime('%Y-%m-%d %H:%M:%S UTC')} (instance={instance}) ===")

checkpoints = {}
res = query(f'reth_sync_checkpoint{{instance="{instance}"}}')
for r in res.get("data", {}).get("result", []):
    checkpoints[r["metric"]["stage"]] = float(r["value"][1])

tip = checkpoints.get("Headers", 0.0)
print(f"Headers/Bodies tip (chain head at last check): {tip:,.0f}")
print()
print(f"{'stage':<22}{'checkpoint':>16}{'% of tip':>10}")
active_stage = None
for s in STAGES:
    v = checkpoints.get(s, 0.0)
    pct = (v / tip * 100.0) if tip else 0.0
    print(f"{s:<22}{v:>16,.0f}{pct:>9.1f}%")
    if v > 0 and v < tip and active_stage is None:
        active_stage = s
if active_stage is None:
    # all listed stages are either 0 (not started) or == tip (done)
    for s in STAGES:
        if checkpoints.get(s, 0.0) == 0.0 and s not in ("Prune", "Finish"):
            active_stage = s
            break

print()
if active_stage is None:
    print("No stage currently between 0 and tip — pipeline idle/fully caught up or check labels.")
    sys.exit(0)

print(f"Active stage (heuristic, first 0<checkpoint<tip): {active_stage}")
remaining = tip - checkpoints.get(active_stage, 0.0)
print(f"Remaining to tip: {remaining:,.0f} blocks (tip is a moving target, grows ~ chain block time)")
def fmt_eta(seconds):
    days, rem = divmod(int(seconds), 86400)
    hours, rem = divmod(rem, 3600)
    minutes, _ = divmod(rem, 60)
    return f"{days}d {hours:02d}h {minutes:02d}m"

print()
print(f"{'window':<8} {'rate (blocks/s)':>16}   {'ETA (remaining-to-tip only)'}")
for window in ("15m", "30m", "1h", "3h", "6h", "12h", "24h"):
    r = query(f'rate(reth_sync_checkpoint{{instance="{instance}",stage="{active_stage}"}}[{window}])')
    result = r.get("data", {}).get("result", [])
    if not result:
        print(f"{window:<8} {'no data':>16}")
        continue
    rate = float(result[0]["value"][1])
    if rate <= 0:
        print(f"{window:<8} {rate:>16,.2f}   n/a (stalled?)")
        continue
    eta_s = remaining / rate
    print(f"{window:<8} {rate:>16,.2f}   {fmt_eta(eta_s)}")

print()
print("Note: ETA uses the *current* stage's own checkpoint rate only; it does not")
print("account for later stages (Merkle/Hashing/Prune) still to run after this one.")
PYEOF
