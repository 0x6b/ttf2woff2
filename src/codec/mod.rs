mod directory;
mod encoder;
mod header;

pub use encoder::{EncodeOptions, encode, encode_no_transform};
pub(crate) use directory::TableDirectoryEntry;
pub(crate) use header::{Woff2Header, WOFF2_SIGNATURE};
