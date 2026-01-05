use super::{inline_bytes::InlineBytes, tag::Tag, varint::encode_base128};

pub(crate) struct TableDirectoryEntry {
    pub tag: Tag,
    pub orig_length: u32,
    pub transform_version: u8,
    pub transform_length: Option<u32>,
}

impl From<&TableDirectoryEntry> for InlineBytes<15> {
    fn from(entry: &TableDirectoryEntry) -> Self {
        let mut data = [0u8; 15];
        let mut len = 0usize;

        let flags = entry.tag.to_flags(entry.transform_version);
        data[len] = flags;
        len += 1;

        if entry.tag.known_index().is_none() {
            data[len..len + 4].copy_from_slice(&entry.tag.0);
            len += 4;
        }

        let orig_len_bytes = encode_base128(entry.orig_length);
        let orig_slice = orig_len_bytes.as_slice();
        data[len..len + orig_slice.len()].copy_from_slice(orig_slice);
        len += orig_slice.len();

        let is_glyf_or_loca = entry.tag.is_glyf() || entry.tag.is_loca();
        if is_glyf_or_loca
            && entry.transform_version == 0
            && let Some(tlen) = entry.transform_length
        {
            let tlen_bytes = encode_base128(tlen);
            let tlen_slice = tlen_bytes.as_slice();
            data[len..len + tlen_slice.len()].copy_from_slice(tlen_slice);
            len += tlen_slice.len();
        }

        InlineBytes::new(data, len as u8)
    }
}
