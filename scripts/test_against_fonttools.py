#!/usr/bin/env python3
"""
Test Rust WOFF2 encoder against fonttools.

This script:
1. Encodes TTF to WOFF2 using fonttools
2. Compares with the Rust-generated WOFF2
3. Validates both produce identical glyph shapes

Usage:
    uv run --with fonttools --with brotli scripts/test_against_fonttools.py <ttf_file> <rust_woff2_file>
"""

import os
import sys
import tempfile
from pathlib import Path


def test_against_fonttools(ttf_path: str, rust_woff2_path: str) -> bool:
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.ttLib import TTFont

    print(f"Testing {rust_woff2_path} against fonttools")
    print()

    # Generate fonttools WOFF2
    with tempfile.NamedTemporaryFile(suffix=".woff2", delete=False) as tmp:
        ft_woff2_path = tmp.name

    try:
        print("Generating fonttools WOFF2...")
        ft_font = TTFont(ttf_path)
        ft_font.flavor = "woff2"
        ft_font.save(ft_woff2_path)

        # Load all three fonts
        original = TTFont(ttf_path)
        rust_woff2 = TTFont(rust_woff2_path)
        ft_woff2 = TTFont(ft_woff2_path)

        # File sizes
        ttf_size = os.path.getsize(ttf_path)
        rust_size = os.path.getsize(rust_woff2_path)
        ft_size = os.path.getsize(ft_woff2_path)

        print("File sizes:")
        print(f"  Original TTF:  {ttf_size:>12,} bytes")
        print(
            f"  Rust WOFF2:    {rust_size:>12,} bytes ({100 * (1 - rust_size / ttf_size):.1f}%)"
        )
        print(
            f"  fonttools:     {ft_size:>12,} bytes ({100 * (1 - ft_size / ttf_size):.1f}%)"
        )
        print(f"  Difference:    {rust_size - ft_size:>+12,} bytes")
        print()

        # Compare glyph shapes
        glyphs = original.getGlyphOrder()
        glyph_set_orig = original.getGlyphSet()
        glyph_set_rust = rust_woff2.getGlyphSet()
        glyph_set_ft = ft_woff2.getGlyphSet()

        rust_match = 0
        ft_match = 0
        rust_mismatches = []
        ft_mismatches = []

        for name in glyphs:
            pen_orig = RecordingPen()
            pen_rust = RecordingPen()
            pen_ft = RecordingPen()

            glyph_set_orig[name].draw(pen_orig)
            glyph_set_rust[name].draw(pen_rust)
            glyph_set_ft[name].draw(pen_ft)

            if pen_orig.value == pen_rust.value:
                rust_match += 1
            else:
                rust_mismatches.append(name)

            if pen_orig.value == pen_ft.value:
                ft_match += 1
            else:
                ft_mismatches.append(name)

        total = len(glyphs)
        print(f"Glyph shape comparison ({total} glyphs):")
        print(f"  Rust vs Original:      {rust_match}/{total} match")
        print(f"  fonttools vs Original: {ft_match}/{total} match")

        if rust_mismatches:
            print(
                f"  Rust mismatches: {rust_mismatches[:5]}{'...' if len(rust_mismatches) > 5 else ''}"
            )
        if ft_mismatches:
            print(
                f"  fonttools mismatches: {ft_mismatches[:5]}{'...' if len(ft_mismatches) > 5 else ''}"
            )

        print()

        # Final result
        all_pass = rust_match == total
        if all_pass:
            print("✓ PASSED: Rust encoder matches original TTF exactly")
            return True
        else:
            print("✗ FAILED: Rust encoder has mismatches")
            return False

    finally:
        os.unlink(ft_woff2_path)


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <ttf_file> <rust_woff2_file>")
        sys.exit(1)

    ttf_path = sys.argv[1]
    rust_woff2_path = sys.argv[2]

    if not Path(ttf_path).exists():
        print(f"Error: {ttf_path} not found")
        sys.exit(1)

    if not Path(rust_woff2_path).exists():
        print(f"Error: {rust_woff2_path} not found")
        sys.exit(1)

    success = test_against_fonttools(ttf_path, rust_woff2_path)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
