#!/usr/bin/env bash
# Build anonymized milestone history for alteredcarbon (orphan branch).
# Does NOT modify local rebase/reth-v2.4.1 working tree.
#
# Usage:
#   scripts/publish/build_alteredcarbon_milestones.sh           # build only
#   scripts/publish/build_alteredcarbon_milestones.sh --push    # build + force-with-lease push
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
SCRUB="$REPO/scripts/publish/scrub_public_tree.py"
WT="$REPO/files/_wt-alteredcarbon-milestones"
BRANCH="publish/alteredcarbon-milestones"
REMOTE="${REMOTE:-alteredcarbon}"
PUSH=0
[[ "${1:-}" == "--push" ]] && PUSH=1

# Milestone: local SHA → public subject (no infra / hosts / paths).
# Author kept as configured identity (SerLevArrisZT ok per operator).
# Early roots (trail/port) may be orphan objects retained from prior alteredcarbon cuts;
# remaining SHAs are ancestors of rebase/reth-v2.4.1.
# AI / effort milestones keep token+cost transparency; scrub strips host/IP/path leaks from trees.
MILESTONES=(
  "916fd0ea21|initial trail import from https://github.com/bnb-chain/reth-bsc-trail"
  "9260a53e3d|chore: merge tag v2.4.1 into rebase/reth-v2.4.1"
  "e637df195b|initial port import, remove obsolete upstream docu"
  "fa51958da7|docs: add plan.md tracking phases, blockers, effort/token log"
  "76635fb207|docs: Copilot CLI LLM token/cost snapshot (illustrative ~USD 1.5–2k)"
  "bba88dd910|feat(bsc,optimism): Phase-3 bsc-node green + Phase-4 hardforks/workspace"
  "cff41d1a2f|feat(optimism): get reth-optimism-cli and op-reth bin compiling"
  "3236c86952|docs: Session 8 AI effort/cost + maxperf-op Makefile"
  "96405ff4a4|docs: Session 9 AI milestone — storage v2 + Phase-5 nextest/EF"
  "d93bd7ea02|feat(storage): StorageChangeSets static-file segment (STOR-006)"
  "d1d3646e88|feat(opbnb): Fermat/Haber/Wright execution-layer hardfork wiring"
  "4a791f5d36|fix(opbnb): milli-timestamp consensus + engine backfill (live sync)"
  "6eeb07a7a4|docs: AI needs operator-led porting skill (PIPE+FLOW methodology)"
  "e8eb6ccb74|fix(net): reachable headers tip seed (PORT-P2P-003)"
  "ae662153f4|chore: checkpoint — started syncing, ok for testing"
  "d8bdbbea7a|docs: experiment verdict — AI as mechanic under PIPE+FLOW"
  "8f2fe9222e|docs: live archive sync past Fermat Point-4 gate"
  "9353bcdb52|docs: Session 12 effort metrics — receipt-root / SF gap / FLOW-X04"
  "e080ef0edc|fix(op): Hertz precompile overlay for PIPE-014 receipt-root"
  "b9792ab58a|feat(net): UPnP NAT mapping for --nat any (PORT-P2P-002)"
  "14745efe56|docs: Session 12 roadmap + cumulative AI effort refresh"
  "c01f13fb16|docs: personal-use SECURITY notice; drop upstream contributing guides"
  "6ee161cfdf|chore: anonymized alteredcarbon milestone publish scripts"
  "284525276c|chore: decommission Docker images, compose, and related CI/docs"
  "dfec68cb1a|chore: disable GitHub Actions CI for public mirror"
  "27b3faff57|docs: no usable public snapshots; sync from network"
  "2080c5d0c6|docs: archive sync wall-clock + stage ETAs in README"
  "9431930830|docs: AI milestones + operator/senior-admin effort ledger"
  "84b01b6207|chore: drop upstream Reth brand images; keep project logo"
  "0db36e17c4|docs: fix README logo embed for GitHub (absolute URL, mode 644)"
  "1638796774|docs: README logo via relative JPEG (avoid raw.githubusercontent 429)"
  "4e2ec3f822|chore: Point-4 gate tooling + ignore files/*.log noise"
  "f0e886f276|docs: fix clone/docs URLs + drop stale CI badges; GitHub About"
  "db54e73e9c|docs: drop cookbook sysctl numbers; TuneD + irqbalance per host/stage"
  "85d50231c3|docs: hardware ballpark only — no fixed core/RAM/NVMe shopping list"
  "35f938985c|docs: clarify one-node vs multi-node fleet hardware (consumer NVMe OK)"
  "HEAD|docs: opBNB archive sync status + ETA refresh (2026-08-20)"
)


export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-SerLevArrisZT}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-SerLevArrisZT@dev.null}"
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME"
export GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"

cd "$REPO"
git fetch "$REMOTE" 2>/dev/null || true
TIP_SHA="$(git rev-parse HEAD)"

rm -rf "$WT"
git worktree prune
git worktree add --detach "$WT" HEAD
cd "$WT"
git branch -D "$BRANCH" >/dev/null 2>&1 || true
git checkout --orphan "$BRANCH"
git rm -rf . >/dev/null 2>&1 || true
# clean untracked from orphan switch
git clean -fdx >/dev/null 2>&1 || true

for entry in "${MILESTONES[@]}"; do
  sha="${entry%%|*}"
  msg="${entry#*|}"
  if [[ "$sha" == "HEAD" ]]; then
    sha="$TIP_SHA"
  fi
  if ! git -C "$REPO" cat-file -e "${sha}^{commit}"; then
    echo "missing commit $sha" >&2
    exit 1
  fi
  echo "=== milestone $sha ==="
  echo "    $msg"
  # Wipe prior tree (keep .git); replace with milestone archive (tracked files only).
  find "$WT" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
  git -C "$REPO" archive "$sha" | tar -x -C "$WT"
  python3 "$SCRUB" "$WT"
  git add -A
  # Skip empty milestones (same tree as previous) so set -e does not abort before --push.
  if git rev-parse --verify HEAD >/dev/null 2>&1 && git diff --cached --quiet; then
    echo "WARNING: no content change vs previous milestone at $sha — skip" >&2
    continue
  fi
  GIT_AUTHOR_DATE="$(git -C "$REPO" log -1 --format=%aI "$sha")"
  GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE"
  export GIT_AUTHOR_DATE GIT_COMMITTER_DATE
  git commit -m "$msg"
done

TIP="$(git rev-parse HEAD)"
echo
echo "Built $BRANCH at $TIP ($(git rev-list --count HEAD) commits)"
echo "Verify scrub leftovers (should be empty / only false positives):"
rg -n '<archive-ct>|<datadir-vol>|/root/|/usr/src/Erigon|/var/lib/machines|10\.0\.0\.|container-host' \
  --glob '!Cargo.lock' --glob '!**/libmdbx/**' . 2>/dev/null | head -40 || true

if [[ "$PUSH" -eq 1 ]]; then
  echo
  echo "Pushing $BRANCH → $REMOTE/main (force-with-lease)"
  git push --force-with-lease "$REMOTE" "HEAD:main"
  echo "Done. $REMOTE/main → $(git rev-parse --short HEAD)"
else
  echo
  echo "Build only. Push with: $0 --push"
  echo "Or: git push --force-with-lease $REMOTE $BRANCH:main"
fi
