#!/usr/bin/env python3
"""Render opBNB RPC bench comparison SVGs (no matplotlib).

Reads headline numbers from the 2026-11-10 JSON benches / README tables and
writes crisp SVGs under docs/repo/charts/.
"""

from __future__ import annotations

from pathlib import Path

OUT = Path(__file__).resolve().parent

# Palette — cool slate + teal local accent (not purple / cream-terracotta)
BG = "#0f1419"
PANEL = "#171d25"
GRID = "#2a3441"
TEXT = "#e8eef4"
MUTED = "#8b9aab"
LOCAL = "#2dd4bf"
OFFICIAL = "#f59e0b"
PUBLICNODE = "#64748b"
FAIL = "#f87171"
ACCENT_LINE = "#334155"


def esc(s: str) -> str:
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def svg_header(w: int, h: int, title: str) -> list[str]:
    return [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" '
        f'viewBox="0 0 {w} {h}" role="img" aria-label="{esc(title)}">',
        f"<title>{esc(title)}</title>",
        f'<rect width="{w}" height="{h}" fill="{BG}" rx="12"/>',
        f'<rect x="16" y="16" width="{w - 32}" height="{h - 32}" fill="{PANEL}" rx="10"/>',
        f'<text x="40" y="52" fill="{TEXT}" font-family="ui-sans-serif, system-ui, '
        f'Segoe UI, Helvetica, Arial, sans-serif" font-size="20" font-weight="700">'
        f"{esc(title)}</text>",
    ]


def svg_footer() -> list[str]:
    return ["</svg>"]


def legend(x: int, y: int, items: list[tuple[str, str]]) -> list[str]:
    out: list[str] = []
    cx = x
    for label, color in items:
        out.append(f'<rect x="{cx}" y="{y - 10}" width="12" height="12" rx="2" fill="{color}"/>')
        out.append(
            f'<text x="{cx + 18}" y="{y}" fill="{MUTED}" font-family="ui-sans-serif, system-ui, '
            f'Segoe UI, Helvetica, Arial, sans-serif" font-size="12">{esc(label)}</text>'
        )
        cx += 18 + 7 * len(label) + 28
    return out


def fmt_ms(v: float) -> str:
    if v < 1:
        return f"{v:.2f} ms"
    if v < 10:
        return f"{v:.1f} ms"
    return f"{v:.0f} ms"


def fmt_qps(v: float) -> str:
    if v <= 0:
        return "0 (fail)"
    if v >= 1000:
        return f"{v / 1000:.1f}k"
    return f"{v:.0f}"


def grouped_bars_log(
    *,
    path: Path,
    title: str,
    subtitle: str,
    categories: list[str],
    series: list[tuple[str, str, list[float | None]]],
    ylabel: str,
    value_fmt,
    width: int = 920,
    height: int = 420,
) -> None:
    """Grouped vertical bars with log10 y-scale. None = failed / unavailable."""
    import math

    lines = svg_header(width, height, title)
    lines.append(
        f'<text x="40" y="76" fill="{MUTED}" font-family="ui-sans-serif, system-ui, '
        f'Segoe UI, Helvetica, Arial, sans-serif" font-size="12">{esc(subtitle)}</text>'
    )
    lines += legend(40, 100, [(name, color) for name, color, _ in series])

    plot_l, plot_t, plot_r, plot_b = 90, 130, width - 40, height - 70
    plot_w = plot_r - plot_l
    plot_h = plot_b - plot_t

    vals = [v for _, _, row in series for v in row if v is not None and v > 0]
    vmin = min(vals) * 0.7
    vmax = max(vals) * 1.15
    log_min = math.log10(vmin)
    log_max = math.log10(vmax)

    def y_of(v: float) -> float:
        return plot_b - (math.log10(v) - log_min) / (log_max - log_min) * plot_h

    # grid / ticks
    tick_candidates = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000]
    ticks = [t for t in tick_candidates if vmin <= t <= vmax]
    for t in ticks:
        y = y_of(t)
        lines.append(
            f'<line x1="{plot_l}" y1="{y:.1f}" x2="{plot_r}" y2="{y:.1f}" '
            f'stroke="{GRID}" stroke-width="1"/>'
        )
        label = value_fmt(t)
        lines.append(
            f'<text x="{plot_l - 10}" y="{y + 4:.1f}" fill="{MUTED}" text-anchor="end" '
            f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
            f'font-size="11">{esc(label)}</text>'
        )

    lines.append(
        f'<text x="28" y="{(plot_t + plot_b) / 2:.0f}" fill="{MUTED}" text-anchor="middle" '
        f'transform="rotate(-90 28 {(plot_t + plot_b) / 2:.0f})" '
        f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
        f'font-size="12">{esc(ylabel)}</text>'
    )

    n_cat = len(categories)
    n_ser = len(series)
    group_w = plot_w / n_cat
    bar_gap = 4
    bar_w = min(36.0, (group_w - 24) / n_ser - bar_gap)

    for i, cat in enumerate(categories):
        gx = plot_l + i * group_w
        cx = gx + group_w / 2
        lines.append(
            f'<text x="{cx:.1f}" y="{plot_b + 22}" fill="{TEXT}" text-anchor="middle" '
            f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
            f'font-size="11">{esc(cat)}</text>'
        )
        for j, (_name, color, row) in enumerate(series):
            v = row[i]
            bx = cx - (n_ser * (bar_w + bar_gap) - bar_gap) / 2 + j * (bar_w + bar_gap)
            if v is None or v <= 0:
                # fail marker
                mid_y = plot_b - 18
                lines.append(
                    f'<rect x="{bx:.1f}" y="{mid_y:.1f}" width="{bar_w:.1f}" height="18" '
                    f'rx="3" fill="{FAIL}" opacity="0.35"/>'
                )
                lines.append(
                    f'<text x="{bx + bar_w / 2:.1f}" y="{mid_y - 6:.1f}" fill="{FAIL}" '
                    f'text-anchor="middle" font-family="ui-sans-serif, system-ui, Segoe UI, '
                    f'Helvetica, Arial, sans-serif" font-size="10">fail</text>'
                )
                continue
            y = y_of(v)
            h = plot_b - y
            lines.append(
                f'<rect x="{bx:.1f}" y="{y:.1f}" width="{bar_w:.1f}" height="{h:.1f}" '
                f'rx="3" fill="{color}"/>'
            )
            lines.append(
                f'<text x="{bx + bar_w / 2:.1f}" y="{y - 6:.1f}" fill="{TEXT}" text-anchor="middle" '
                f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
                f'font-size="10" font-weight="600">{esc(value_fmt(v))}</text>'
            )

    lines.append(
        f'<line x1="{plot_l}" y1="{plot_b}" x2="{plot_r}" y2="{plot_b}" '
        f'stroke="{ACCENT_LINE}" stroke-width="1.5"/>'
    )
    lines += svg_footer()
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {path}")


def horizontal_speedup(
    *,
    path: Path,
    title: str,
    subtitle: str,
    rows: list[tuple[str, float, str]],
    width: int = 920,
    height: int = 360,
) -> None:
    lines = svg_header(width, height, title)
    lines.append(
        f'<text x="40" y="76" fill="{MUTED}" font-family="ui-sans-serif, system-ui, '
        f'Segoe UI, Helvetica, Arial, sans-serif" font-size="12">{esc(subtitle)}</text>'
    )

    plot_l, plot_t, plot_r, plot_b = 220, 110, width - 50, height - 50
    plot_w = plot_r - plot_l
    n = len(rows)
    row_h = (plot_b - plot_t) / n
    max_v = max(v for _, v, _ in rows) * 1.08

    for i, (label, value, note) in enumerate(rows):
        y = plot_t + i * row_h + row_h * 0.2
        bh = row_h * 0.55
        bw = (value / max_v) * plot_w
        lines.append(
            f'<text x="{plot_l - 12}" y="{y + bh * 0.72:.1f}" fill="{TEXT}" text-anchor="end" '
            f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
            f'font-size="12">{esc(label)}</text>'
        )
        lines.append(
            f'<rect x="{plot_l}" y="{y:.1f}" width="{bw:.1f}" height="{bh:.1f}" '
            f'rx="4" fill="{LOCAL}"/>'
        )
        lines.append(
            f'<text x="{plot_l + bw + 8:.1f}" y="{y + bh * 0.72:.1f}" fill="{TEXT}" '
            f'font-family="ui-sans-serif, system-ui, Segoe UI, Helvetica, Arial, sans-serif" '
            f'font-size="12" font-weight="700">{esc(note)}</text>'
        )

    lines += svg_footer()
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {path}")


def main() -> None:
    # 1) Sequential latency (p50 ms) — log scale
    grouped_bars_log(
        path=OUT / "opbnb-rpc-latency-p50.svg",
        title="opBNB RPC latency (p50) — local IPC vs public",
        subtitle="2026-11-10 · tip ~204.93M · warm keep-alive on public HTTPS · log scale",
        categories=[
            "getBlock latest",
            "eth_call tip",
            "hist getBlock",
            "hist getBalance",
            "getLogs 2k",
        ],
        series=[
            ("Local IPC", LOCAL, [0.09, 0.18, 0.11, 0.30, 3.0]),
            ("Official public", OFFICIAL, [131.0, 121.0, 115.0, None, 116.0]),
            ("PublicNode", PUBLICNODE, [52.0, 31.0, 24.0, None, 34.0]),
        ],
        ylabel="p50 latency",
        value_fmt=fmt_ms,
        height=440,
    )

    # 2) Parallel QPS
    grouped_bars_log(
        path=OUT / "opbnb-rpc-parallel-qps.svg",
        title="Parallel RPC throughput (QPS)",
        subtitle="Concurrent single requests · local IPC c=32/128 · public capped ≤32",
        categories=[
            "getBlock c=8",
            "getBlock c=32",
            "eth_call c=32",
            "hist bal c=128",
        ],
        series=[
            ("Local IPC", LOCAL, [4915.0, 5700.0, 5900.0, 6100.0]),
            ("Official public", OFFICIAL, [20.5, 67.0, 34.0, None]),
            ("PublicNode", PUBLICNODE, [12.0, 61.0, 7.0, None]),
        ],
        ylabel="requests / s",
        value_fmt=fmt_qps,
        height=420,
    )

    # 3) Batch call/s
    grouped_bars_log(
        path=OUT / "opbnb-rpc-batch-throughput.svg",
        title="JSON-RPC batch throughput (calls / s)",
        subtitle="Array batches over IPC / HTTPS · local ~30–70k call/s band",
        categories=[
            "getBlock bs=10",
            "getBlock bs=100",
            "mixed bs=100",
            "burst mixed×200",
        ],
        series=[
            ("Local IPC", LOCAL, [30000.0, 65000.0, 70000.0, 64000.0]),
            ("Official public", OFFICIAL, [241.0, 275.0, None, None]),
            ("PublicNode", PUBLICNODE, [6.0, None, None, None]),
        ],
        ylabel="calls / s",
        value_fmt=fmt_qps,
        height=420,
    )

    # 4) Speedup multiples vs official
    horizontal_speedup(
        path=OUT / "opbnb-rpc-speedup.svg",
        title="Local IPC speedup vs official public RPC",
        subtitle="Same host · sequential p50 (latency) and parallel/batch throughput multiples",
        rows=[
            ("getBlock latest p50", 1500, "~1500× faster"),
            ("eth_call tip p50", 680, "~680×"),
            ("hist getBlock p50", 1000, "~800–1400×"),
            ("parallel getBlock QPS", 85, "~85× QPS"),
            ("batch getBlock call/s", 230, "~230× call/s"),
            ("hist state (archive)", 1, "public: missing trie"),
        ],
        height=380,
    )


if __name__ == "__main__":
    main()
