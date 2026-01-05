use crate::{known_tags::find_tag_index, variable_int::encode_base128};

pub struct TableDirectoryEntry {
    pub tag: [u8; 4],
    pub orig_length: u32,
    pub transform_version: u8,
    pub transform_length: Option<u32>,
}

impl TableDirectoryEntry {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let tag_index = find_tag_index(&self.tag);
        let flags = match tag_index {
            Some(idx) => idx | (self.transform_version << 6),
            None => 63 | (self.transform_version << 6),
        };
        result.push(flags);

        if tag_index.is_none() {
            result.extend_from_slice(&self.tag);
        }

        result.extend(encode_base128(self.orig_length));

        let is_glyf_or_loca = tag_index == Some(10) || tag_index == Some(11);
        if is_glyf_or_loca
            && self.transform_version == 0
            && let Some(tlen) = self.transform_length
        {
            result.extend(encode_base128(tlen));
        }

        result
    }
}

pub fn build_directory(tables: &[(&[u8; 4], u32)]) -> Vec<TableDirectoryEntry> {
    tables
        .iter()
        .map(|(tag, length)| {
            let tag_index = find_tag_index(tag);
            let is_glyf_or_loca = tag_index == Some(10) || tag_index == Some(11);
            let transform_version = if is_glyf_or_loca { 3 } else { 0 };

            TableDirectoryEntry {
                tag: **tag,
                orig_length: *length,
                transform_version,
                transform_length: None,
            }
        })
        .collect()
}
