mod inline_bytes;
mod triplet;
mod varint;

pub use inline_bytes::InlineBytes;
pub use triplet::{TripletData, encode_triplet};
pub use varint::{EncodedInt, encode_255_u_int16, encode_base128};
