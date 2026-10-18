//! A Pure Rust library for compressing TTF and OTF fonts to WOFF2 format.
//!
//! Both TrueType (`glyf`-based) and CFF-flavored OpenType (OTTO) inputs are
//! supported. The WOFF2 specification defines no transformation for the `CFF`
//! table, so CFF data is stored as-is and only Brotli-compressed.
//!
//! ## Example
//!
//! ```no_run
//! use ttf2woff2::{encode, BrotliQuality};
//!
//! let ttf_data = std::fs::read("font.ttf").unwrap();
//! let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();
//! std::fs::write("font.woff2", &woff2_data).unwrap();
//! ```

pub use error::Error;
pub use woff2::{BrotliQuality, EncodeOptions, encode, encode_no_transform, encode_with_options};

mod error;
mod woff2;
