//! A Pure Rust library for compressing TTF fonts to WOFF2 format.
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

pub use brotli_quality::BrotliQuality;
pub use encode::{encode, encode_no_transform};
pub use error::Error;

mod brotli_quality;
mod directory;
mod encode;
mod error;
mod header;
mod sfnt;
mod tag;
mod transform;
mod util;
mod variable_int;
