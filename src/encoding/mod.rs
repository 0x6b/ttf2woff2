mod inline_bytes;
mod triplet;
mod varint;

pub(crate) use inline_bytes::InlineBytes;
pub(crate) use triplet::encode_triplet;
pub(crate) use varint::{encode_255_u_int16, encode_base128};
