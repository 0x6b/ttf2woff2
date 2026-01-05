# ttf2woff2

A Pure Rust library and CLI for compressing TTF/OTF fonts to WOFF2 format.

## Features

- **Pure Rust** - No C/C++ dependencies
- **glyf/loca transformation** - Achieves compression comparable to Google's woff2
- **100% glyph fidelity** - All glyph shapes are preserved exactly
- Compatible with [fonttools](https://github.com/fonttools/fonttools) output

## Installation

```bash
cargo install ttf2woff2
```

## CLI Usage

```bash
ttf2woff2 font.ttf                     # Output: font.woff2
ttf2woff2 font.ttf -o output.woff2     # Custom output path
ttf2woff2 font.ttf -q 5                # Lower quality (faster, larger)
```

## Library Usage

```rust
use ttf2woff2::{encode, BrotliQuality};

let ttf_data = std::fs::read("font.ttf")?;
let woff2_data = encode(&ttf_data, BrotliQuality::default())?;
std::fs::write("font.woff2", &woff2_data)?;
```

## Compression Results

| Font | Original TTF | WOFF2 | Compression |
|------|-------------|-------|-------------|
| WarpnineSans-Regular | 275 KB | 80 KB | 70.7% |
| NotoSansJP-Medium | 5.7 MB | 2.3 MB | 59.5% |

## Validation Scripts

Python scripts are provided to validate WOFF2 output:

```bash
# Validate metadata and glyph shapes
uv run --with fonttools --with brotli scripts/validate_woff2.py font.ttf font.woff2

# Compare file sizes with fonttools
uv run --with fonttools --with brotli scripts/compare_size.py font.ttf font.woff2

# Validate all WOFF2 files in tests/
uv run --with fonttools --with brotli scripts/validate_all.py
```

## License

- The [Noto Sans Japanese](https://fonts.google.com/noto/specimen/Noto+Sans+JP) font in [tests/](tests) is licensed under [OFL](OFL.txt).
- Everything else is [MIT](LICENSE).

## References

- [W3C WOFF2 Specification](https://www.w3.org/TR/WOFF2/)
- [fonttools woff2.py](https://github.com/fonttools/fonttools/blob/main/Lib/fontTools/ttLib/woff2.py)
