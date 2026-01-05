#!/usr/bin/env python3
"""
Validate WOFF2 file against original TTF.

Usage:
    uv run --with fonttools --with brotli scripts/validate_woff2.py <ttf_file> <woff2_file>
"""

import sys
from pathlib import Path


def validate(ttf_path: str, woff2_path: str) -> bool:
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.ttLib import TTFont

    print(f"Validating {woff2_path} against {ttf_path}")
    print()

    original = TTFont(ttf_path)
    woff2 = TTFont(woff2_path)

    # Metadata comparison
    print("Metadata:")
    all_match = True

    checks = [
        ("numGlyphs", original["maxp"].numGlyphs, woff2["maxp"].numGlyphs),
        ("unitsPerEm", original["head"].unitsPerEm, woff2["head"].unitsPerEm),
        ("fontRevision", original["head"].fontRevision, woff2["head"].fontRevision),
        ("xMin", original["head"].xMin, woff2["head"].xMin),
        ("yMin", original["head"].yMin, woff2["head"].yMin),
        ("xMax", original["head"].xMax, woff2["head"].xMax),
        ("yMax", original["head"].yMax, woff2["head"].yMax),
        ("sTypoAscender", original["OS/2"].sTypoAscender, woff2["OS/2"].sTypoAscender),
        (
            "sTypoDescender",
            original["OS/2"].sTypoDescender,
            woff2["OS/2"].sTypoDescender,
        ),
        (
            "familyName",
            original["name"].getDebugName(1),
            woff2["name"].getDebugName(1),
        ),
    ]

    for name, orig_val, woff2_val in checks:
        match = "✓" if orig_val == woff2_val else "✗"
        if orig_val != woff2_val:
            all_match = False
        print(f"  {name}: {match} (orig={orig_val}, woff2={woff2_val})")

    # Glyph shape comparison
    print()
    print("Glyph shapes:")
    glyph_set_orig = original.getGlyphSet()
    glyph_set_woff2 = woff2.getGlyphSet()
    glyphs = original.getGlyphOrder()
    matched = 0
    mismatches = []

    for name in glyphs:
        pen1 = RecordingPen()
        pen2 = RecordingPen()
        glyph_set_orig[name].draw(pen1)
        glyph_set_woff2[name].draw(pen2)
        if pen1.value == pen2.value:
            matched += 1
        else:
            mismatches.append(name)

    total = len(glyphs)
    pct = matched / total * 100
    status = "✓" if matched == total else "✗"
    print(f"  {status} {matched}/{total} glyphs match ({pct:.2f}%)")

    if mismatches:
        print(f"  Mismatches: {mismatches[:10]}{'...' if len(mismatches) > 10 else ''}")
        all_match = False

    print()
    if all_match and matched == total:
        print("✓ Validation PASSED")
        return True
    else:
        print("✗ Validation FAILED")
        return False


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <ttf_file> <woff2_file>")
        sys.exit(1)

    ttf_path = sys.argv[1]
    woff2_path = sys.argv[2]

    if not Path(ttf_path).exists():
        print(f"Error: {ttf_path} not found")
        sys.exit(1)

    if not Path(woff2_path).exists():
        print(f"Error: {woff2_path} not found")
        sys.exit(1)

    success = validate(ttf_path, woff2_path)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
