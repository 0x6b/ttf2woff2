# ttf2woff2

A Pure Rust library and CLI for compressing TTF and OTF fonts to WOFF2 format.

## Features

- Pure Rust - No C/C++ or Python dependencies
- OTF/CFF input support - CFF tables are stored untransformed (the WOFF2 specification defines no CFF transformation) and only Brotli-compressed
- Optional glyf/loca transformation - Disabled by default for decoder compatibility
- Preserves original glyf/loca table data when transformation is disabled (the default)
- Compatible with [fonttools](https://github.com/fonttools/fonttools) output

## CLI Usage

```console
$ cargo install ttf2woff2
$ ttf2woff2 --help
A Pure Rust library and CLI for compressing TTF and OTF fonts to WOFF2 format.

Usage: ttf2woff2 [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the input TTF or OTF file

Options:
  -o, --output <OUTPUT>    Path to the output WOFF2 file (defaults to input with .woff2 extension)
  -q, --quality <QUALITY>  Brotli compression quality (0-11) [default: 9]
  -t, --threads <THREADS>  Number of threads for Brotli compression: 1=single-threaded (deterministic), 0=all cores, N=N threads [default: 1]
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

The `--threads` (`-t`) flag accepts:

| Value         | Meaning                                                 |
| ------------- | ------------------------------------------------------- |
| `1` (default) | Single-threaded Brotli — fully deterministic output.    |
| `0`           | Use all cores (`std::thread::available_parallelism()`). |
| `N` (>= 2)    | Use exactly `N` threads.                                |

Multi-threaded Brotli is much faster on large fonts at quality 10-11 but the output bytes depend on the thread count and total size grows by typically < 0.5 %. Output remains a valid Brotli stream that any spec-compliant WOFF2 decoder accepts.

## Library Usage

Add to your `Cargo.toml` with `default-features = false` to exclude the CLI.

```toml
[dependencies]
ttf2woff2 = { version = "0.14", default-features = false }
```

```rust
use ttf2woff2::{encode, BrotliQuality};

let ttf_data = std::fs::read("font.ttf")?;
let woff2_data = encode(&ttf_data, BrotliQuality::default())?;
std::fs::write("font.woff2", &woff2_data)?;
```

### Optional glyf/loca transformation

The CLI, `encode`, and `EncodeOptions::default()` leave the `glyf`/`loca`
transformation **disabled**. Tables are stored as-is before Brotli compression.
This prioritizes decoder compatibility over the smallest possible file size.

To opt in through the Rust API:

```rust
use ttf2woff2::{encode_with_options, EncodeOptions};

let options = EncodeOptions {
    transform_glyf_loca: true,
    ..EncodeOptions::default()
};
let woff2_data = encode_with_options(&ttf_data, options)?;
```

Enabling the transformation can reduce output size, but has a known compatibility
issue: glyph reconstruction can increase `glyf` beyond the short `loca` offset
limit without switching to long `loca`. Such output can be rejected by
Chromium/Google's WOFF2 decoder, even when fontTools can read the WOFF2 directly
([issue #30](https://github.com/0x6b/ttf2woff2/issues/30)). This is not inherently
limited to variable fonts. If you opt in, validate the output with your target
decoders and browsers; a successful encode or fontTools-only check is insufficient.

For Source Sans 3 variable (646,340-byte TTF), at Brotli quality 9:

| Transformation | WOFF2 size | Reduction from TTF | Chromium 155 |
| -------------- | ---------: | -----------------: | ------------ |
| Disabled       | 204,076 bytes | 68.43% | Loads |
| Enabled        | 191,080 bytes | 70.44% | Rejected |

Disabling it costs 12,996 bytes (+6.8% WOFF2 size) in this example. The size
tradeoff depends on the font. `encode_no_transform` remains available and has
the same behavior as `encode`.

## Node.js / WASM Usage

A WASM build is available on npm as [@0x6b/ttf2woff2-wasm](https://www.npmjs.com/package/@0x6b/ttf2woff2-wasm):

```console
$ npm install @0x6b/ttf2woff2-wasm
```

```js
const ttf2woff2 = require("@0x6b/ttf2woff2-wasm");
const fs = require("fs");

const ttf = fs.readFileSync("font.ttf");
const woff2 = ttf2woff2(ttf);
fs.writeFileSync("font.woff2", woff2);
```

> [!NOTE]
> Multi-threaded Brotli compression (`EncodeOptions::threads`) is not available on WebAssembly. The option is silently ignored.

## Performance

Benchmarks on NotoSansJP-Medium (17,808 glyphs, 5,729,332 bytes) on Apple M4 Pro
(12 cores), with glyf/loca transformation enabled (`transform_glyf_loca: true`):

| Implementation   | Brotli Quality | Threads | Time (s) | Output Size (bytes) |
| ---------------- | -------------: | ------: | -------: | ------------------: |
| Rust             |             11 |       1 |    3.251 |           2,322,432 |
| Rust             |             11 |    auto |    0.957 |           2,327,752 |
| Rust             |              9 |       1 |    0.354 |           2,424,432 |
| Rust             |              9 |    auto |    0.143 |           2,434,116 |
| Python fonttools |             11 |       1 |    9.863 |           2,322,828 |

## Validation

Tests generate WOFF2 files and validate against fonttools:

```console
$ cargo test
```

Manual validation (need [`uv`](https://docs.astral.sh/uv/) installed):

```console
$ uv run scripts/validate.py <font.ttf> <font.woff2>
```

Regenerate pre-generated fonttools output for faster tests:

```console
$ uv run scripts/generate_golden.py
```

## License

- The [Noto Sans Japanese](https://fonts.google.com/noto/specimen/Noto+Sans+JP) font in [tests/fixtures/](tests/fixtures) is licensed under [OFL](https://fonts.google.com/noto/specimen/Noto+Sans+JP/license).
- The [Recursive](https://github.com/arrowtype/recursive) font in [tests/fixtures/](tests/fixtures) is licensed under [OFL](https://github.com/arrowtype/recursive/blob/main/OFL.txt).
- The [WarpnineSans](https://github.com/0x6b/warpnine-fonts) font in [tests/fixtures/](tests/fixtures) is licensed under [OFL](https://github.com/0x6b/warpnine-fonts/blob/main/OFL).
- The MinimalCFF-Regular test font in [tests/fixtures/](tests/fixtures) is generated by [scripts/generate_otf_fixture.py](scripts/generate_otf_fixture.py) and carries no third-party license.
- Everything else is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

## Alternatives

If you need byte-for-byte compatibility with Google's woff2 converter, decompression support, or WOFF1 support, consider these alternatives:

- [woofwoof](https://github.com/bearcove/woofwoof) - Wraps Google's C++ woff2 library with pure Rust brotli. Supports both compression and decompression.
- [bodoni/woff](https://github.com/bodoni/woff) - Wraps Google's C++ woff2 and C brotli. Supports WOFF1 and WOFF2.

## Acknowledgments

This project started as an FFI wrapper around Google's [woff2](https://github.com/google/woff2) and [brotli](https://github.com/google/brotli) C/C++ libraries, then evolved into a pure Rust implementation ([v0.10.0](https://github.com/0x6b/ttf2woff2/tree/v0.10.0)) with assistance from AI coding assistants ([Claude Code](https://www.anthropic.com/claude-code), [Codex](https://openai.com/index/codex/), and [Amp](https://ampcode.com/)). While the code has been tested and validated against [fonttools](https://github.com/fonttools/fonttools), users should verify output for production use.

## References

- [W3C WOFF2 Specification](https://www.w3.org/TR/WOFF2/)
- [fonttools woff2.py](https://github.com/fonttools/fonttools/blob/main/Lib/fontTools/ttLib/woff2.py)
