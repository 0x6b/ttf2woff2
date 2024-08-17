# ttf2woff2

A Rust library and CLI for compressing a TTF font to WOFF2 format. The output is compatible with [google/woff2](https://github.com/google/woff2/blob/master/src/woff2_compress.cc) (via the `woff2_compress` command).

You may use [Brooooooklyn/woff-build](https://github.com/Brooooooklyn/woff-build) instead, which has a more user-friendly interface. This library is more for my personal use and learning purposes.

## Prerequisites

- Linux (tested on Ubuntu 22.04.4 LTS): `sudo apt install -y libbrotli-dev, g++`
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

## License

- The [Noto Sans Japanese](https://fonts.google.com/noto/specimen/Noto+Sans+JP) font for [testing](tests) in the repository is licensed under its own license. See [OFL.txt](OFL.txt) for details.
- Other files are licensed under the MIT. See [LICENSE](LICENSE) for details.

## Acknowledgements

- [lemonrock/woff2-sys: Rust FFI bindings to Google's woff2 library](https://github.com/lemonrock/woff2-sys)
- [thibault-cne/woff2: An FFI biding to the google woff2 library in Rust.](https://github.com/thibault-cne/woff2)

## Reference

- [Reduce web font size | Articles | web.dev](https://web.dev/articles/reduce-webfont-size?hl=en)
