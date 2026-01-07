#!/usr/bin/env python3
# /// script
# dependencies = ["fonttools", "brotli"]
# ///
"""Generate golden WOFF2 files using fonttools for comparison tests."""

import sys
from pathlib import Path

from fontTools.ttLib import TTFont

FIXTURES_DIR = Path(__file__).parent.parent / "tests" / "fixtures"
GOLDEN_DIR = FIXTURES_DIR / "golden"

FONTS = [
    "WarpnineSans-Regular",
    "NotoSansJP-Medium",
    "Recursive_VF_1.085",
]


def generate_golden_woff2(font_name: str) -> None:
    ttf_path = FIXTURES_DIR / f"{font_name}.ttf"
    woff2_path = GOLDEN_DIR / f"{font_name}.woff2"

    print(f"Generating {woff2_path.name}...")

    font = TTFont(ttf_path)
    font.flavor = "woff2"
    font.save(woff2_path)

    ttf_size = ttf_path.stat().st_size
    woff2_size = woff2_path.stat().st_size
    compression = (1.0 - woff2_size / ttf_size) * 100

    print(f"  {ttf_size:,} -> {woff2_size:,} bytes ({compression:.1f}% compression)")


def main() -> int:
    GOLDEN_DIR.mkdir(exist_ok=True)

    for font_name in FONTS:
        generate_golden_woff2(font_name)

    print(f"\nGenerated {len(FONTS)} golden files in {GOLDEN_DIR}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
