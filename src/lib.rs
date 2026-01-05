//! A Rust library compressing a TTF font to WOFF2 format. The output is compatible with
//! [google/woff2](https://github.com/google/woff2) (via its [`woff2_compress`](https://github.com/google/woff2/blob/master/src/woff2_compress.cc) command).
//!
//! You may use [Brooooooklyn/woff-build](https://github.com/Brooooooklyn/woff-build) instead, which has a more user-friendly interface. This library is more for my personal use and learning purposes.
pub use brotli_quality::BrotliQuality;
pub use converter::Converter;
pub use encode::encode;
pub use error::Error;

mod brotli_quality;
mod converter;
pub mod directory;
pub mod encode;
mod error;
pub mod header;
pub mod known_tags;
pub mod sfnt;
mod state;
pub mod transform;
pub mod variable_int;
