#!/usr/bin/env python3
"""Contrast check for the status bar palettes (Claude Dark & Claude Light).

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
from typing import Any

PALETTE = Path(__file__).resolve().parent.parent / "src" / "palette.rs"
# WCAG AA for normal text. The theme notes want accents on the canvas to clear
# ~5.5:1; the bar draws them on raised surfaces, where 4.5:1 is the real floor.
MIN = 4.5

Color = tuple[int, int, int]


def luminance(c: Color) -> float:
    def channel(v: int) -> float:
        v /= 255
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4

    r, g, b = c
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)


def contrast(a: Color, b: Color) -> float:
    x, y = luminance(a), luminance(b)
    return (max(x, y) + 0.05) / (min(x, y) + 0.05)


def parse_palette_block(src: str, const_name: str) -> dict[str, Any]:
    pattern = rf"pub const {const_name}:\s*ThemePalette\s*=\s*ThemePalette\s*\{{([^}}]+)\}};"
    m = re.search(pattern, src, re.DOTALL)
    if not m:
        raise ValueError(f"Could not find {const_name} in {PALETTE}")
    block = m.group(1)
    fields: dict[str, Any] = {}
    for line in block.splitlines():
        line = line.strip()
        if not line or line.startswith("//"):
            continue
        c_match = re.match(r"(\w+):\s*\((\d+),\s*(\d+),\s*(\d+)\)", line)
        if c_match:
            name, r, g, b = c_match.groups()
            fields[name] = (int(r), int(g), int(b))
            continue
        a_match = re.match(
            r"(\w+):\s*Accent::new\(\((\d+),\s*(\d+),\s*(\d+)\),\s*\((\d+),\s*(\d+),\s*(\d+)\)\)",
            line,
        )
        if a_match:
            name, r1, g1, b1, r2, g2, b2 = a_match.groups()
            fields[name] = ((int(r1), int(g1), int(b1)), (int(r2), int(g2), int(b2)))
            continue
    return fields


def check_palette(palette_name: str, pal: dict[str, Any], is_dark: bool) -> list[str]:
    failures: list[str] = []

    def check(label: str, fg: Color, bg: Color, floor: float = MIN) -> None:
        ratio = contrast(fg, bg)
        mark = "ok " if ratio >= floor else "FAIL"
        print(f"  {mark} {label:<44} {ratio:5.2f}:1")
        if ratio < floor:
            failures.append(f"[{palette_name}] {label} = {ratio:.2f}:1 (need {floor})")

    print(f"=== {palette_name} ===")
    print("Activity glyphs on both tab surfaces")
    surfaces = [
        ("inactive tab", pal["tab_bg_inactive"], False),
        ("active tab", pal["tab_bg_active"], True),
    ]
    for name, val in sorted(pal.items()):
        if not name.startswith("act_"):
            continue
        base, sel = val
        for label, bg, selected in surfaces:
            floor = 4.0 if name == "act_idle" and not selected else MIN
            check(f"{name} / {label}", sel if selected else base, bg, floor)

    print("\nTab and prefix text")
    check("text_primary on prefix_bg", pal["text_primary"], pal["prefix_bg"])
    check("text_primary on bar_bg", pal["text_primary"], pal["bar_bg"])
    check("text_secondary on tab_bg_inactive", pal["text_secondary"], pal["tab_bg_inactive"])
    check("text_tertiary on tab_bg_inactive", pal["text_tertiary"], pal["tab_bg_inactive"], floor=4.0)
    check("text_on_accent on tab_bg_active", pal["text_on_accent"], pal["tab_bg_active"])

    print("\nAccent surfaces & mode pills")
    check("text_on_accent on prefix_bg_settings", pal["text_on_accent"], pal["prefix_bg_settings"])
    for mode in sorted(n for n in pal if n.startswith("mode_")):
        check(f"mode pill {mode}", pal["text_on_accent"], pal[mode])

    print("\nPermission flash combinations")
    flash_bg_base, flash_bg_sel = pal["flash_bg"]
    flash_tx_base, flash_tx_sel = pal["text_on_flash"]
    check("inactive flash (text on flash_bg)", flash_tx_base, flash_bg_base)
    check("active flash (text on flash_bg)", flash_tx_sel, flash_bg_sel)

    if is_dark:
        print("\nSurfaces must be neutral grey (Claude Dark)")
        for name in ["bar_bg", "prefix_bg", "tab_bg_inactive"]:
            r, g, b = pal[name]
            neutral = r == g == b
            print(f"  {'ok ' if neutral else 'FAIL'} {name:<44} ({r}, {g}, {b})")
            if not neutral:
                failures.append(f"[{palette_name}] {name} is tinted")

    return failures


def main() -> int:
    src = PALETTE.read_text()
    all_failures: list[str] = []

    for palette_name, is_dark in [("DARK_PALETTE", True), ("LIGHT_PALETTE", False)]:
        pal = parse_palette_block(src, palette_name)
        failures = check_palette(palette_name, pal, is_dark)
        all_failures.extend(failures)
        print()

    if all_failures:
        print(f"\n{len(all_failures)} failure(s):", file=sys.stderr)
        for f in all_failures:
            print(f"  - {f}", file=sys.stderr)
        return 1

    print("All checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
