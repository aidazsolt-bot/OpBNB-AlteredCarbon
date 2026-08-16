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
MILESTONES=(
  "96405ff4a4|docs: Session 9 — storage v2 + Phase-5 nextest/EF milestone"
  "d93bd7ea02|feat(storage): StorageChangeSets static-file segment (STOR-006)"
  "d1d3646e88|feat(opbnb): Fermat/Haber/Wright execution-layer hardfork wiring"
  "e8eb6ccb74|fix(net): reachable headers tip seed (PORT-P2P-003)"
  "8f2fe9222e|docs: live archive sync past Fermat Point-4 gate"
  "e080ef0edc|fix(op): Hertz precompile overlay for PIPE-014 receipt-root"
  "b9792ab58a|feat(net): UPnP NAT mapping for --nat any (PORT-P2P-002)"
  "c01f13fb16|docs: personal-use SECURITY notice; drop upstream contributing guides"
  "6ee161cfdf|chore: anonymized alteredcarbon milestone publish scripts"
  "284525276c|chore: decommission Docker images, compose, and related CI/docs"
)

export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-SerLevArrisZT}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-SerLevArrisZT@dev.null}"
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME"
export GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"

cd "$REPO"
git fetch "$REMOTE" 2>/dev/null || true

rm -rf "$WT"
git worktree add --detach "$WT" HEAD
cd "$WT"
git checkout --orphan "$BRANCH"
git rm -rf . >/dev/null 2>&1 || true
# clean untracked from orphan switch
git clean -fdx >/dev/null 2>&1 || true

for entry in "${MILESTONES[@]}"; do
  sha="${entry%%|*}"
  msg="${entry#*|}"
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
  # Detect deletions vs previous commit
  if git diff --cached --quiet && git rev-parse --verify HEAD >/dev/null 2>&1; then
    echo "WARNING: no content change vs previous milestone at $sha" >&2
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
rg -n 'BSCRethArchiveNode|DataDomainNVMER40|/root/|/usr/src/Erigon|/var/lib/machines|10\.0\.0\.|nspawn' \
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
