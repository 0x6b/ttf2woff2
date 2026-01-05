use brotli::enc::BrotliEncoderParams;

use crate::pure::{
    directory::TableDirectoryEntry,
    header::{WOFF2_SIGNATURE, Woff2Header},
    known_tags::find_tag_index,
    sfnt::Sfnt,
};

pub fn encode(ttf_data: &[u8], quality: u32) -> Result<Vec<u8>, String> {
    let sfnt = Sfnt::parse(ttf_data).map_err(|e| e.to_string())?;

    let mut sorted_tables: Vec<_> = sfnt.tables.iter().collect();
    sorted_tables.sort_by_key(|t| t.tag);

    let (major_version, minor_version) = sorted_tables
        .iter()
        .find(|t| &t.tag == b"head")
        .and_then(|head| {
            let start = head.offset as usize;
            let data = ttf_data.get(start + 4..start + 8)?;
            Some((u16::from_be_bytes([data[0], data[1]]), u16::from_be_bytes([data[2], data[3]])))
        })
        .unwrap_or((0, 0));

    let directory_entries: Vec<TableDirectoryEntry> = sorted_tables
        .iter()
        .map(|t| {
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
    for table in &sorted_tables {
        let start = table.offset as usize;
        let end = start + table.length as usize;
        uncompressed_data.extend_from_slice(&ttf_data[start..end]);
    }

    let mut compressed_data = Vec::new();
    let params = BrotliEncoderParams {
        quality: quality as i32,
        mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT,
        ..Default::default()
    };
    brotli::enc::BrotliCompress(&mut &uncompressed_data[..], &mut compressed_data, &params)
        .map_err(|e| format!("Brotli compression failed: {e}"))?;

    let total_sfnt_size = 12
        + 16 * sfnt.tables.len() as u32
        + sorted_tables.iter().map(|t| (t.length + 3) & !3).sum::<u32>();

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
        major_version,
        minor_version,
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
