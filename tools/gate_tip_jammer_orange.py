#!/usr/bin/env python3
"""Refuse empty hangar jammer tip stills (Soft Prison orange≈0.01 miss)."""
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("tip_capture gate: Pillow missing; pip install pillow", file=sys.stderr)
    sys.exit(1)

FLOORS = {
    "20_jammer_dish_follow_16x9.png": 0.05,
    "22_jammer_dish_overview_16x9.png": 0.03,
    "23_jammer_dish_seize_label_16x9.png": 0.05,
}


def orange_ratio(path: Path) -> float:
    im = Image.open(path).convert("RGB")
    w, h = im.size
    pix = im.load()
    orange = 0
    sampled = 0
    for y in range(0, h, 2):
        for x in range(0, w, 2):
            r, g, b = pix[x, y]
            sampled += 1
            if r > 140 and g > 40 and g < 200 and b < 120 and r > g and r > b + 30:
                orange += 1
    return (orange / sampled) if sampled else 0.0


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: gate_tip_jammer_orange.py OUT_DIR", file=sys.stderr)
        return 2
    out = Path(sys.argv[1])
    failed = False
    for name, floor in FLOORS.items():
        path = out / name
        if not path.is_file():
            print(f"tip_capture gate: missing {path}", file=sys.stderr)
            failed = True
            continue
        ratio = orange_ratio(path)
        print(f"tip_capture gate: {name} orange={ratio:.4f} floor={floor:.4f}")
        if ratio < floor:
            print(f"tip_capture gate: FAIL {name} empty hangar / no dish", file=sys.stderr)
            failed = True
    if failed:
        return 1
    print("tip_capture gate: PASS jammer orange footprint")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
