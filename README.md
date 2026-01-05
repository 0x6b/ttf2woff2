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

## Performance

Benchmarks on NotoSansJP-Medium (17,808 glyphs, 5.7MB):

| Implementation | Time | Output Size |
|----------------|------|-------------|
| **Rust (quality 11)** | 3.0s | 2.32 MB |
| **Rust (quality 9)** | 0.33s | 2.42 MB |
| Python fonttools | 9.2s | 2.32 MB |

- Quality 11 (default): **3x faster** than fonttools
- Quality 9: **27x faster** than fonttools, with only 4% larger output

For faster conversion with minimal size impact, use `-q 9`.

## Compression Results

| Font | Original TTF | WOFF2 | Compression |
|------|-------------|-------|-------------|
| WarpnineSans-Regular | 275 KB | 80 KB | 70.7% |
| NotoSansJP-Medium | 5.7 MB | 2.3 MB | 59.5% |

## Validation

Tests generate WOFF2 files and validate against fonttools:

```bash
cargo test
```

Manual validation:

```bash
uv run --with fonttools --with brotli scripts/validate.py font.ttf font.woff2
```

## License

- The [Noto Sans Japanese](https://fonts.google.com/noto/specimen/Noto+Sans+JP) font in [tests/](tests) is licensed under [OFL](OFL.txt).
- Everything else is [MIT](LICENSE).

## References

- [W3C WOFF2 Specification](https://www.w3.org/TR/WOFF2/)
- [fonttools woff2.py](https://github.com/fonttools/fonttools/blob/main/Lib/fontTools/ttLib/woff2.py)
