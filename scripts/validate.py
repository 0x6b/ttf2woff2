#!/usr/bin/env python3
"""
Validate WOFF2 files against original TTF and fonttools.

Usage:
    uv run --with fonttools --with brotli scripts/validate.py <ttf_file> <woff2_file>
"""

import os
import struct
import sys
import tempfile
from pathlib import Path

from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont


def read_woff2_header(path):
    with open(path, "rb") as f:
        data = f.read(48)
    (
        sig,
        flavor,
        length,
        num_tables,
        reserved,
        total_sfnt_size,
        total_compressed_size,
        maj,
        min_,
        meta_off,
        meta_len,
        meta_orig_len,
        priv_off,
        priv_len,
    ) = struct.unpack(">4sIIHHIIHHIIIII", data)
    return {
        "signature": sig,
        "flavor": hex(flavor),
        "length": length,
        "numTables": num_tables,
        "totalSfntSize": total_sfnt_size,
        "totalCompressedSize": total_compressed_size,
        "majorVersion": maj,
        "minorVersion": min_,
    }


def compare_glyphs(font1, font2):
    glyph_set1 = font1.getGlyphSet()
    glyph_set2 = font2.getGlyphSet()
    glyphs = font1.getGlyphOrder()
    matched = 0
    mismatches = []
    for name in glyphs:
        pen1 = RecordingPen()
        pen2 = RecordingPen()
        glyph_set1[name].draw(pen1)
        glyph_set2[name].draw(pen2)
        if pen1.value == pen2.value:
            matched += 1
        else:
            mismatches.append(name)
    return matched, len(glyphs), mismatches


def validate(ttf_path: str, woff2_path: str) -> bool:
    print(f"Validating: {woff2_path}")
    print(f"Against:    {ttf_path}")
    print()

    # File sizes
    ttf_size = os.path.getsize(ttf_path)
    woff2_size = os.path.getsize(woff2_path)
    compression = (1 - woff2_size / ttf_size) * 100

    print("=" * 60)
    print("FILE SIZES")
    print("=" * 60)
    print(f"  Original TTF:  {ttf_size:>12,} bytes")
    print(f"  WOFF2:         {woff2_size:>12,} bytes ({compression:.1f}% compression)")
    print()

    # WOFF2 header
    print("=" * 60)
    print("WOFF2 HEADER")
    print("=" * 60)
    header = read_woff2_header(woff2_path)
    for key, value in header.items():
        print(f"  {key}: {value}")
    print()

    # Load fonts
    original = TTFont(ttf_path)
    woff2 = TTFont(woff2_path)

    # Metadata comparison
    print("=" * 60)
    print("METADATA COMPARISON")
    print("=" * 60)
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
    metadata_ok = True
    for name, orig_val, woff2_val in checks:
        match = "✓" if orig_val == woff2_val else "✗"
        if orig_val != woff2_val:
            metadata_ok = False
        print(f"  {match} {name}: orig={orig_val}, woff2={woff2_val}")
    print()

    # Glyph shape comparison
    print("=" * 60)
    print("GLYPH SHAPE COMPARISON")
    print("=" * 60)
    matched, total, mismatches = compare_glyphs(original, woff2)
    glyphs_ok = matched == total
    status = "✓" if glyphs_ok else "✗"
    print(f"  {status} {matched}/{total} glyphs match ({matched/total*100:.2f}%)")
    if mismatches:
        print(f"  Mismatches: {mismatches[:10]}{'...' if len(mismatches) > 10 else ''}")
    print()

    # Compare with fonttools
    print("=" * 60)
    print("FONTTOOLS COMPARISON")
    print("=" * 60)
    with tempfile.NamedTemporaryFile(suffix=".woff2", delete=False) as tmp:
        ft_path = tmp.name
    try:
        print("  Generating fonttools WOFF2...")
        ft_font = TTFont(ttf_path)
        ft_font.flavor = "woff2"
        ft_font.save(ft_path)
        ft_size = os.path.getsize(ft_path)
        ft_woff2 = TTFont(ft_path)

        diff = woff2_size - ft_size
        diff_pct = (diff / ft_size) * 100 if ft_size > 0 else 0
        print(f"  fonttools size: {ft_size:,} bytes")
        print(f"  Size difference: {diff:+,} bytes ({diff_pct:+.2f}%)")

        ft_matched, ft_total, ft_mismatches = compare_glyphs(original, ft_woff2)
        print(f"  fonttools vs original: {ft_matched}/{ft_total} glyphs match")
    finally:
        os.unlink(ft_path)
    print()

    # Final result
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    all_ok = metadata_ok and glyphs_ok
    if all_ok:
        print("✓ PASSED: All validations successful")
    else:
        print("✗ FAILED: Some validations failed")
        if not metadata_ok:
            print("  - Metadata mismatch")
        if not glyphs_ok:
            print("  - Glyph shape mismatch")

    return all_ok


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <ttf_file> <woff2_file>")
        print()
        print("Validates WOFF2 file against original TTF:")
        print("  - Compares metadata (numGlyphs, unitsPerEm, bbox, etc.)")
        print("  - Compares all glyph shapes")
        print("  - Compares file size with fonttools output")
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
