#!/usr/bin/env python3
"""Contrast check for the status bar palette.

Reads the constants straight out of ``src/palette.rs`` so the numbers here
cannot drift from the ones the plugin draws with. This is a script rather than
a ``#[test]`` because the crate is a WASM plugin: ``zellij_tile``'s host shims
have no symbols to link on the host, so ``cargo test`` cannot build the binary
at all.

    ./scripts/check-palette-contrast.py

Exits non-zero if anything falls below its floor.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

PALETTE = Path(__file__).resolve().parent.parent / "src" / "palette.rs"
# WCAG AA for normal text. The theme notes want accents on the canvas to clear
# ~5.5:1; the bar draws them on raised surfaces, where 4.5:1 is the real floor.
MIN = 4.5

Color = tuple[int, int, int]


def parse() -> tuple[dict[str, Color], dict[str, tuple[str, str]]]:
    src = PALETTE.read_text()
    colors: dict[str, Color] = {}
    for name, r, g, b in re.findall(
        r"pub const (\w+): Color = \((\d+), (\d+), (\d+)\);", src
    ):
        colors[name] = (int(r), int(g), int(b))
    # Aliases: `pub const BAR_BG: Color = BASE10;`
    for name, target in re.findall(r"pub const (\w+): Color = (\w+);", src):
        colors[name] = colors[target]

    accents: dict[str, tuple[str, str]] = {}
    for name, kind, args in re.findall(
        r"pub const (\w+): Accent = Accent::(new|flat)\(([^)]*)\);", src
    ):
        parts = [a.strip() for a in args.split(",")]
        accents[name] = (parts[0], parts[1] if kind == "new" else parts[0])
    return colors, accents


def luminance(c: Color) -> float:
    def channel(v: int) -> float:
        v /= 255
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4

    r, g, b = c
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)


def contrast(a: Color, b: Color) -> float:
    x, y = luminance(a), luminance(b)
    return (max(x, y) + 0.05) / (min(x, y) + 0.05)


def main() -> int:
    colors, accents = parse()
    failures: list[str] = []

    def check(label: str, fg: Color, bg: Color, floor: float = MIN) -> None:
        ratio = contrast(fg, bg)
        mark = "ok " if ratio >= floor else "FAIL"
        print(f"  {mark} {label:<44} {ratio:5.2f}:1")
        if ratio < floor:
            failures.append(f"{label} = {ratio:.2f}:1 (need {floor})")

    surfaces = [
        ("inactive tab", colors["TAB_BG_INACTIVE"], False),
        ("active tab", colors["TAB_BG_ACTIVE"], True),
    ]

    print("Activity glyphs on both tab surfaces")
    for name, (base, sel) in sorted(accents.items()):
        if not name.startswith("ACT_"):
            continue
        for label, bg, selected in surfaces:
            check(f"{name.lower()} / {label}", colors[sel if selected else base], bg)

    print("\nTab and prefix text")
    for role, surf_names in [
        ("TEXT_PRIMARY", ["TAB_BG_ACTIVE", "PREFIX_BG", "BAR_BG"]),
        ("TEXT_SECONDARY", ["TAB_BG_INACTIVE"]),
        # Deliberately recessed: the dim name on a non-Claude tab. Held to a
        # lower floor so Claude tabs keep their edge in the peripheral glance.
        ("TEXT_TERTIARY", ["TAB_BG_INACTIVE"]),
    ]:
        floor = 4.0 if role == "TEXT_TERTIARY" else MIN
        for s in surf_names:
            check(f"{role.lower()} on {s.lower()}", colors[role], colors[s], floor)

    print("\nDark ink on accent surfaces")
    for surface in ["PREFIX_BG_SETTINGS", "FLASH_BG"]:
        check(f"text_on_accent on {surface.lower()}", colors["TEXT_ON_ACCENT"], colors[surface])
    for mode in sorted(n for n in colors if n.startswith("MODE_")):
        check(f"mode pill {mode.lower()}", colors["TEXT_ON_ACCENT"], colors[mode])

    print("\nSurfaces must be neutral grey (warmth lives in the text ramp)")
    for name in ["BAR_BG", "PREFIX_BG", "TAB_BG_INACTIVE", "TAB_BG_ACTIVE"]:
        r, g, b = colors[name]
        neutral = r == g == b
        print(f"  {'ok ' if neutral else 'FAIL'} {name.lower():<44} ({r}, {g}, {b})")
        if not neutral:
            failures.append(f"{name} is tinted")

    if failures:
        print(f"\n{len(failures)} failure(s):", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1
    print("\nAll checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
