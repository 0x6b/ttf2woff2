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
    "WarpnineSans-Regular.ttf",
    "NotoSansJP-Medium.ttf",
    "Recursive_VF_1.085.ttf",
    "MinimalCFF-Regular.otf",
]


def generate_golden_woff2(font_file: str) -> None:
    font_path = FIXTURES_DIR / font_file
    woff2_path = GOLDEN_DIR / f"{Path(font_file).stem}.woff2"

    print(f"Generating {woff2_path.name}...")

    font = TTFont(font_path)
    font.flavor = "woff2"
    font.save(woff2_path)

    ttf_size = font_path.stat().st_size
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
