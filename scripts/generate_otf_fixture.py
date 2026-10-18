#!/usr/bin/env python3
# /// script
# dependencies = ["fonttools", "brotli"]
# ///
"""Generate a minimal CFF-flavored OpenType (OTTO) fixture for tests.

Produces tests/fixtures/MinimalCFF-Regular.otf, a small synthetic font with a
handful of glyphs (lines, curves, and a contour with a counter) so the CFF
table contains non-trivial charstring data. Run scripts/generate_golden.py
afterwards to (re)generate the matching fonttools golden WOFF2.
"""

import sys
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.ttLib import TTFont

FIXTURES_DIR = Path(__file__).parent.parent / "tests" / "fixtures"

UNITS_PER_EM = 1000
ADVANCE_WIDTHS = {
    ".notdef": 600,
    "space": 250,
    "A": 600,
    "B": 650,
    "C": 700,
    "O": 750,
}
CHAR_MAP = {0x20: "space", 0x41: "A", 0x42: "B", 0x43: "C", 0x4F: "O"}


def draw_box(pen, x, y, w, h):
    pen.moveTo((x, y))
    pen.lineTo((x + w, y))
    pen.lineTo((x + w, y + h))
    pen.lineTo((x, y + h))
    pen.closePath()


def draw_triangle(pen, cx, base_y, half_w, h):
    pen.moveTo((cx - half_w, base_y))
    pen.lineTo((cx + half_w, base_y))
    pen.lineTo((cx, base_y + h))
    pen.closePath()


def draw_glyph(name, pen):
    if name == ".notdef":
        draw_box(pen, 50, 0, 500, 700)
    elif name == "space":
        pass  # empty charstring
    elif name == "A":
        draw_triangle(pen, 300, 0, 280, 700)
    elif name == "B":
        draw_box(pen, 40, 0, 570, 700)
    elif name == "C":
        draw_circle_arc(pen, 350, 350, 300)
    elif name == "O":
        draw_ring(pen, 375, 350, 340, 190)


def draw_circle_arc(pen, cx, cy, r):
    # A "C"-like open arc of three cubic segments, top-to-bottom.
    k = 0.5522847498 * r
    pen.moveTo((cx - r, cy))
    pen.curveTo((cx - r, cy + k), (cx - k, cy + r), (cx, cy + r))
    pen.curveTo((cx + k, cy + r), (cx + r, cy + k), (cx + r, cy))
    pen.curveTo((cx + r, cy - k), (cx + k, cy - r), (cx + 80, cy - r))
    pen.closePath()


def draw_ring(pen, cx, cy, r_outer, r_inner):
    # Outer contour counterclockwise, inner contour clockwise: nonzero winding
    # treats the inner circle as a counter.
    _circle_contour(pen, cx, cy, r_outer, clockwise=False)
    _circle_contour(pen, cx, cy, r_inner, clockwise=True)


def _circle_contour(pen, cx, cy, r, clockwise):
    k = 0.5522847498 * r
    if not clockwise:
        pen.moveTo((cx - r, cy))
        pen.curveTo((cx - r, cy + k), (cx - k, cy + r), (cx, cy + r))
        pen.curveTo((cx + k, cy + r), (cx + r, cy + k), (cx + r, cy))
        pen.curveTo((cx + r, cy - k), (cx + k, cy - r), (cx, cy - r))
        pen.curveTo((cx - k, cy - r), (cx - r, cy - k), (cx - r, cy))
    else:
        pen.moveTo((cx - r, cy))
        pen.curveTo((cx - r, cy - k), (cx - k, cy - r), (cx, cy - r))
        pen.curveTo((cx + k, cy - r), (cx + r, cy - k), (cx + r, cy))
        pen.curveTo((cx + r, cy + k), (cx + k, cy + r), (cx, cy + r))
        pen.curveTo((cx - k, cy + r), (cx - r, cy + k), (cx - r, cy))
    pen.closePath()


def left_side_bearing(charstring):
    bounds = charstring.calcBounds(None)
    return bounds[0] if bounds else 0


def build_font():
    fb = FontBuilder(UNITS_PER_EM, isTTF=False)
    glyph_order = [".notdef"] + [g for g in ADVANCE_WIDTHS if g != ".notdef"]
    fb.setupGlyphOrder(glyph_order)
    fb.setupCharacterMap(CHAR_MAP)

    charstrings = {}
    for name in glyph_order:
        pen = T2CharStringPen(ADVANCE_WIDTHS[name], None)
        draw_glyph(name, pen)
        charstrings[name] = pen.getCharString()

    fb.setupCFF(
        "MinimalCFF-Regular",
        {"FullName": "MinimalCFF Regular"},
        charstrings,
        {},
    )
    metrics = {name: (ADVANCE_WIDTHS[name], left_side_bearing(cs)) for name, cs in charstrings.items()}
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=800, descent=-200)
    fb.setupNameTable(
        {
            "familyName": {"en": "MinimalCFF"},
            "styleName": {"en": "Regular"},
            "uniqueFontIdentifier": "fonttools: MinimalCFF Regular",
            "fullName": "MinimalCFF Regular",
            "psName": "MinimalCFF-Regular",
            "version": "Version 0.001",
        }
    )
    fb.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
    fb.setupPost()
    return fb.font


def main() -> int:
    otf_path = FIXTURES_DIR / "MinimalCFF-Regular.otf"
    font = build_font()
    font.save(otf_path)

    reopened = TTFont(otf_path)
    assert reopened.sfntVersion == "OTTO", reopened.sfntVersion

    print(f"Generated {otf_path.name}: {otf_path.stat().st_size:,} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
