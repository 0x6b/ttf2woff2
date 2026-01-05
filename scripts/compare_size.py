#!/usr/bin/env python3
"""
Compare WOFF2 file sizes between Pure Rust encoder and fonttools.

Usage:
    uv run --with fonttools --with brotli scripts/compare_size.py <ttf_file> <rust_woff2_file>
"""

import os
import sys
import tempfile
from pathlib import Path


def compare(ttf_path: str, rust_woff2_path: str) -> None:
    from fontTools.ttLib import TTFont

    ttf_size = os.path.getsize(ttf_path)
    rust_size = os.path.getsize(rust_woff2_path)

    # Generate fonttools WOFF2 for comparison
    with tempfile.NamedTemporaryFile(suffix=".woff2", delete=False) as tmp:
        ft_path = tmp.name

    try:
        f = TTFont(ttf_path)
        f.flavor = "woff2"
        f.save(ft_path)
        ft_size = os.path.getsize(ft_path)
    finally:
        os.unlink(ft_path)

    name = Path(ttf_path).stem
    diff = rust_size - ft_size
    diff_pct = (diff / ft_size) * 100 if ft_size > 0 else 0
    rust_compression = (1 - rust_size / ttf_size) * 100
    ft_compression = (1 - ft_size / ttf_size) * 100

    print(f"File size comparison for {name}:")
    print()
    print(f"  Original TTF:  {ttf_size:>12,} bytes")
    print(
        f"  Pure Rust:     {rust_size:>12,} bytes ({rust_compression:.1f}% compression)"
    )
    print(f"  fonttools:     {ft_size:>12,} bytes ({ft_compression:.1f}% compression)")
    print()
    print(f"  Difference:    {diff:>+12,} bytes ({diff_pct:+.2f}%)")
    print()

    if abs(diff_pct) < 1:
        print("✓ Sizes match (within 1%)")
    elif diff < 0:
        print("✓ Pure Rust is smaller!")
    else:
        print(f"⚠ Pure Rust is {diff_pct:.1f}% larger")


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

    compare(ttf_path, rust_woff2_path)


if __name__ == "__main__":
    main()
