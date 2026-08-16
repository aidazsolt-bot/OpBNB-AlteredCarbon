#!/usr/bin/env python3
"""Scrub a working tree for public GitHub publish (alteredcarbon milestones).

Removes / replaces infra, host, path, and address identifiers. Keeps:
  - AI tool names (Cursor, Copilot, Claude, GPT, …)
  - Upstream / chain software (reth, op-geth, opBNB, revm, alloy, …)
  - Generic bind examples (0.0.0.0, 127.0.0.1)
  - Protocol IDs (PORT-*, block numbers, fork timestamps)

Usage:
  python3 scripts/publish/scrub_public_tree.py /path/to/workdir
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

SKIP_DIR_NAMES = {
    ".git",
    "target",
    "node_modules",
    "libmdbx",
}
SKIP_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".pdf",
    ".wasm",
    ".bin",
    ".tar",
    ".gz",
    ".zst",
    ".lock",
}

# Literal replacements (order matters: longer first).
LITERALS: list[tuple[str, str]] = [
    ("<agent-transcripts>/", "<agent-transcripts>/"),
    ("<copilot-session-store>", "<copilot-session-store>"),
    ("<repo>", "<repo>"),
    ("<external-analysis>/", "<external-analysis>/"),
    ("<src-root>/", "<src-root>/"),
    ("<archive-journal>/", "<archive-journal>/"),
    ("<machine-root>/", "<machine-root>/"),
    ("<archive-ct>", "<archive-ct>"),
    ("<datadir-vol>", "<datadir-vol>"),
    ("<host-user>", "<host-user>"),
]

KEEP_IPV4 = {
    "0.0.0.0",
    "127.0.0.1",
    "255.255.255.255",
    # Upstream CLI / docker doc examples (not site-local hosts)
    "10.0.0.0",
    "192.168.0.0",
    "172.16.0.0",
}

IPV4_RE = re.compile(
    r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b"
)

# Prefer known IPv6 prefixes / localhost; avoid matching Rust `foo::bar` paths.
IPV6_RES = [
    re.compile(r"\b(?:::1)\b"),
    re.compile(r"\b(?:fe80|fc00|fd[0-9a-f]{2}|2001|2002|3ffe):[0-9a-fA-F:]+", re.I),
]

TEXTISH_SUFFIXES = {
    "",
    ".md",
    ".mdc",
    ".mdx",
    ".txt",
    ".json",
    ".jsonl",
    ".toml",
    ".yml",
    ".yaml",
    ".rs",
    ".py",
    ".sh",
    ".ts",
    ".tsx",
    ".js",
    ".css",
    ".html",
    ".svg",
    ".env",
    ".example",
    ".gitignore",
    ".dockerignore",
}


def is_text_file(path: Path) -> bool:
    if path.suffix.lower() in SKIP_SUFFIXES:
        return False
    if path.suffix.lower() in TEXTISH_SUFFIXES or path.suffix == "":
        # skip obvious binaries by name
        if path.name.endswith(".bin"):
            return False
        return True
    return path.suffix.lower() in {".mk", ".makefile"}


def scrub_ipv4(text: str) -> str:
    def repl(m: re.Match[str]) -> str:
        ip = m.group(0)
        if ip in KEEP_IPV4:
            return ip
        # Cargo / semver false positives like 10.0.1 alone are not matched (need 4 octets).
        return "<ipv4>"

    return IPV4_RE.sub(repl, text)


def scrub_ipv6(text: str) -> str:
    for rx in IPV6_RES:
        text = rx.sub("<ipv6>", text)
    return text


def scrub_text(text: str) -> str:
    for old, new in LITERALS:
        text = text.replace(old, new)
    # journalctl machines path variants
    text = re.sub(
        r"journalctl\s+-D\s+(?:<machine-root>/\S+|<machine-root>\S*)",
        "journalctl -D <archive-journal>",
        text,
    )
    text = re.sub(
        r"`?<archive-journal>/`?",
        "<archive-journal>",
        text,
    )
    text = re.sub(
        r"http://<metrics-api>/api/v1/query",
        "http://<metrics-api>/api/v1/query",
        text,
    )
    text = re.sub(r"\bnspawn\b", "container-host", text, flags=re.I)
    # Cursor/agent paths under /root (not generic docker /root/.local/share/reth)
    text = re.sub(r"/root/\.cursor/[^\s\"']+", "<cursor-path>", text)
    text = re.sub(r"/root/\.copilot/[^\s\"']+", "<copilot-path>", text)
    text = re.sub(r"<home-path>"']+/[^\s\"']*", "<home-path>", text)
    text = scrub_ipv4(text)
    text = scrub_ipv6(text)
    return text


def scrub_tree(root: Path) -> int:
    changed = 0
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if any(p in SKIP_DIR_NAMES for p in path.parts):
            continue
        if not is_text_file(path):
            continue
        try:
            raw = path.read_bytes()
        except OSError:
            continue
        if b"\0" in raw[:8000]:
            continue
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            try:
                text = raw.decode("latin-1")
            except UnicodeDecodeError:
                continue
        new = scrub_text(text)
        if new != text:
            path.write_text(new, encoding="utf-8")
            changed += 1
    return changed


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: scrub_public_tree.py <workdir>", file=sys.stderr)
        return 2
    root = Path(sys.argv[1]).resolve()
    if not root.is_dir():
        print(f"not a directory: {root}", file=sys.stderr)
        return 1
    n = scrub_tree(root)
    print(f"scrubbed {n} files under {root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
