use brotli::enc::BrotliEncoderParams;

use crate::pure::{
    directory::TableDirectoryEntry,
    header::{WOFF2_SIGNATURE, Woff2Header},
    known_tags::find_tag_index,
    sfnt::Sfnt,
};

pub fn encode(ttf_data: &[u8], quality: u32) -> Result<Vec<u8>, String> {
    let sfnt = Sfnt::parse(ttf_data).map_err(|e| e.to_string())?;

    let mut sorted_tables: Vec<_> = sfnt
        .tables
        .iter()
        .map(|t| {
            let sort_key = match find_tag_index(&t.tag) {
                Some(idx) => (0, idx as u16, [0u8; 4]),
                None => (1, 0, t.tag),
            };
            (sort_key, t)
        })
        .collect();
    sorted_tables.sort_by_key(|(key, _)| *key);

    let directory_entries: Vec<TableDirectoryEntry> = sorted_tables
        .iter()
        .map(|(_, t)| {
            let tag_index = find_tag_index(&t.tag);
            let is_glyf_or_loca = tag_index == Some(10) || tag_index == Some(11);
            let transform_version = if is_glyf_or_loca { 3 } else { 0 };

            TableDirectoryEntry {
                tag: t.tag,
                orig_length: t.length,
                transform_version,
            }
        })
        .collect();

    let mut uncompressed_data = Vec::new();
    for (_, table) in &sorted_tables {
        let start = table.offset as usize;
        let end = start + table.length as usize;
        uncompressed_data.extend_from_slice(&ttf_data[start..end]);
    }

    let mut compressed_data = Vec::new();
    let params = BrotliEncoderParams { quality: quality as i32, ..Default::default() };
    brotli::enc::BrotliCompress(&mut &uncompressed_data[..], &mut compressed_data, &params)
        .map_err(|e| format!("Brotli compression failed: {e}"))?;

    let total_sfnt_size = 12
        + 16 * sfnt.tables.len() as u32
        + sorted_tables.iter().map(|(_, t)| t.length).sum::<u32>();

    let mut directory_bytes = Vec::new();
    for entry in &directory_entries {
        directory_bytes.extend(entry.to_bytes());
    }

    let total_length = 48 + directory_bytes.len() as u32 + compressed_data.len() as u32;

    let header = Woff2Header {
        signature: WOFF2_SIGNATURE,
        flavor: sfnt.flavor,
        length: total_length,
        num_tables: sfnt.tables.len() as u16,
        reserved: 0,
        total_sfnt_size,
        total_compressed_size: compressed_data.len() as u32,
        major_version: 1,
        minor_version: 0,
        meta_offset: 0,
        meta_length: 0,
        meta_orig_length: 0,
        priv_offset: 0,
        priv_length: 0,
    };

    let mut result = Vec::with_capacity(total_length as usize);
    result.extend_from_slice(&header.to_bytes());
    result.extend(&directory_bytes);
    result.extend(&compressed_data);

    Ok(result)
}
