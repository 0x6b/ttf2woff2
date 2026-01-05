#!/usr/bin/env python3
"""
Validate all WOFF2 files in tests/ directory against their TTF sources.

Usage:
    uv run --with fonttools --with brotli scripts/validate_all.py
"""

import os
import sys
from pathlib import Path


def validate_font(ttf_path: Path, woff2_path: Path) -> tuple[bool, dict]:
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.ttLib import TTFont

    original = TTFont(str(ttf_path))
    woff2 = TTFont(str(woff2_path))

    results = {
        "ttf_size": os.path.getsize(ttf_path),
        "woff2_size": os.path.getsize(woff2_path),
        "metadata_match": True,
        "glyphs_matched": 0,
        "glyphs_total": 0,
    }

    # Check key metadata
    checks = [
        (original["maxp"].numGlyphs, woff2["maxp"].numGlyphs),
        (original["head"].unitsPerEm, woff2["head"].unitsPerEm),
        (original["head"].xMin, woff2["head"].xMin),
        (original["head"].yMin, woff2["head"].yMin),
        (original["head"].xMax, woff2["head"].xMax),
        (original["head"].yMax, woff2["head"].yMax),
    ]

    for orig_val, woff2_val in checks:
        if orig_val != woff2_val:
            results["metadata_match"] = False
            break

    # Check glyph shapes
    glyph_set_orig = original.getGlyphSet()
    glyph_set_woff2 = woff2.getGlyphSet()
    glyphs = original.getGlyphOrder()
    matched = 0

    for name in glyphs:
        pen1 = RecordingPen()
        pen2 = RecordingPen()
        glyph_set_orig[name].draw(pen1)
        glyph_set_woff2[name].draw(pen2)
        if pen1.value == pen2.value:
            matched += 1

    results["glyphs_matched"] = matched
    results["glyphs_total"] = len(glyphs)

    success = results["metadata_match"] and matched == len(glyphs)
    return success, results


def main():
    tests_dir = Path(__file__).parent.parent / "tests"

    # Find all TTF files
    ttf_files = list(tests_dir.glob("*.ttf"))

    if not ttf_files:
        print("No TTF files found in tests/")
        sys.exit(1)

    print("Validating WOFF2 files:")
    print()

    all_passed = True
    for ttf_path in sorted(ttf_files):
        woff2_path = ttf_path.with_name(ttf_path.stem + "-pure.woff2")

        if not woff2_path.exists():
            print(f"  ⚠ {ttf_path.name}: No corresponding WOFF2 file")
            continue

        success, results = validate_font(ttf_path, woff2_path)
        compression = (1 - results["woff2_size"] / results["ttf_size"]) * 100

        status = "✓" if success else "✗"
        glyphs = f"{results['glyphs_matched']}/{results['glyphs_total']}"
        size = f"{results['woff2_size']:,} bytes ({compression:.1f}%)"

        print(f"  {status} {ttf_path.name}")
        print(f"      Glyphs: {glyphs}, Size: {size}")

        if not success:
            all_passed = False

    print()
    if all_passed:
        print("✓ All validations PASSED")
        sys.exit(0)
    else:
        print("✗ Some validations FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
