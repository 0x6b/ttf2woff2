use crate::{tag::Tag, variable_int::encode_base128};

pub struct TableDirectoryEntry {
    pub tag: Tag,
    pub orig_length: u32,
    pub transform_version: u8,
    pub transform_length: Option<u32>,
}

impl TableDirectoryEntry {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let flags = self.tag.to_flags(self.transform_version);
        result.push(flags);

        if self.tag.known_index().is_none() {
            result.extend_from_slice(&self.tag.0);
        }

        result.extend_from_slice(encode_base128(self.orig_length).as_slice());

        let is_glyf_or_loca = self.tag.is_glyf() || self.tag.is_loca();
        if is_glyf_or_loca
            && self.transform_version == 0
            && let Some(tlen) = self.transform_length
        {
            result.extend_from_slice(encode_base128(tlen).as_slice());
        }

        result
    }
}
