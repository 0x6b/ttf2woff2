use crate::{
    encoding::{InlineBytes, encode_base128},
    tag::Tag,
};

pub struct TableDirectoryEntry {
    pub tag: Tag,
    pub orig_length: u32,
    pub transform_version: u8,
    pub transform_length: Option<u32>,
}

impl TableDirectoryEntry {
    /// Maximum size: 1 (flags) + 4 (tag) + 5 (orig_length) + 5 (transform_length) = 15 bytes
    pub fn to_bytes(&self) -> InlineBytes<15> {
        let mut data = [0u8; 15];
        let mut len = 0usize;

        let flags = self.tag.to_flags(self.transform_version);
        data[len] = flags;
        len += 1;

        if self.tag.known_index().is_none() {
            data[len..len + 4].copy_from_slice(&self.tag.0);
            len += 4;
        }

        let orig_len_bytes = encode_base128(self.orig_length);
        let orig_slice = orig_len_bytes.as_slice();
        data[len..len + orig_slice.len()].copy_from_slice(orig_slice);
        len += orig_slice.len();

        let is_glyf_or_loca = self.tag.is_glyf() || self.tag.is_loca();
        if is_glyf_or_loca
            && self.transform_version == 0
            && let Some(tlen) = self.transform_length
        {
            let tlen_bytes = encode_base128(tlen);
            let tlen_slice = tlen_bytes.as_slice();
            data[len..len + tlen_slice.len()].copy_from_slice(tlen_slice);
            len += tlen_slice.len();
        }

        InlineBytes::new(data, len as u8)
    }
}
