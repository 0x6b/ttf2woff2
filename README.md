# ttf2woff2

A Rust library and CLI for compressing a TTF font to WOFF2 format.

## Features

- **Pure Rust WOFF2 encoder** - No C/C++ dependencies required
- **glyf/loca table transformation** - Achieves compression comparable to Google's woff2
- **100% glyph fidelity** - All glyph shapes are preserved exactly
- Compatible with [fonttools](https://github.com/fonttools/fonttools) output

## Pure Rust Encoder

The `pure` module provides a Pure Rust implementation of WOFF2 encoding:

```rust
use ttf2woff2::pure::encode;

let ttf_data = std::fs::read("font.ttf")?;
let woff2_data = encode(&ttf_data, 11)?; // quality 0-11
std::fs::write("font.woff2", &woff2_data)?;
```

### Compression Results

| Font | Original TTF | Pure Rust WOFF2 | Compression |
|------|-------------|-----------------|-------------|
| WarpnineSans-Regular | 275,564 bytes | 80,605 bytes | 70.7% |
| NotoSansJP-Medium | 5,729,332 bytes | 2,322,431 bytes | 59.5% |

## Legacy C++ Encoder

The original C++ based encoder is still available but requires system dependencies.

### Prerequisites

- Linux (tested on Ubuntu 22.04.4 LTS): `sudo apt install -y libbrotli-dev g++`
- macOS (tested on Sonoma 14.4): `brew install brotli`

## Build

```console
$ cargo build --release
```

## Usage

```console
Usage: ttf2woff2 [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the input TTF file

Options:
  -o, --output <OUTPUT>    Path to the output WOFF2 file. Defaults to the name of the input file with a .woff2 extension
  -q, --quality <QUALITY>  Brotli quality, between 0 and 11 inclusive [default: 11]
  -h, --help               Print help
```

## Validation Scripts

Python scripts are provided to validate WOFF2 output against the original TTF:

```bash
# Validate a single WOFF2 file
uv run --with fonttools --with brotli scripts/validate_woff2.py font.ttf font.woff2

# Compare file sizes with fonttools
uv run --with fonttools --with brotli scripts/compare_size.py font.ttf font.woff2

# Validate all WOFF2 files in tests/
uv run --with fonttools --with brotli scripts/validate_all.py
```

## License

- The [Noto Sans Japanese](https://fonts.google.com/noto/specimen/Noto+Sans+JP) font for [testing](tests) in the repository is licensed under its own license. See [OFL.txt](OFL.txt) for details.
- Other files are licensed under the MIT. See [LICENSE](LICENSE) for details.

## Acknowledgements

- [lemonrock/woff2-sys: Rust FFI bindings to Google's woff2 library](https://github.com/lemonrock/woff2-sys)
- [thibault-cne/woff2: An FFI biding to the google woff2 library in Rust.](https://github.com/thibault-cne/woff2)

## Reference

- [Reduce web font size | Articles | web.dev](https://web.dev/articles/reduce-webfont-size?hl=en)
