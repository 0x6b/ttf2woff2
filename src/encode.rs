use brotli::enc::{BrotliCompress, BrotliEncoderParams};

use crate::{
    BrotliQuality,
    directory::TableDirectoryEntry,
    header::{WOFF2_SIGNATURE, Woff2Header},
    known_tags::find_tag_index,
    sfnt::Sfnt,
    transform::glyf::transform_glyf,
};

#[cfg(feature = "timing")]
macro_rules! time_section {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        eprintln!("[TIMING] {}: {:?}", $name, start.elapsed());
        result
    }};
}

#[cfg(not(feature = "timing"))]
macro_rules! time_section {
    ($name:expr, $block:expr) => {
        $block
    };
}

pub fn encode(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, String> {
    encode_with_options(ttf_data, quality, true)
}

pub fn encode_no_transform(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, String> {
    encode_with_options(ttf_data, quality, false)
}

fn encode_with_options(
    ttf_data: &[u8],
    quality: BrotliQuality,
    transform_glyf_loca: bool,
) -> Result<Vec<u8>, String> {
    #[cfg(feature = "timing")]
    let total_start = std::time::Instant::now();

    let sfnt = time_section!("SFNT parsing", Sfnt::parse(ttf_data).map_err(|e| e.to_string())?);

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

    let mut transformed_glyf: Option<Vec<u8>> = None;
    if transform_glyf_loca {
        let glyf_table = sorted_tables.iter().find(|t| &t.tag == b"glyf");
        let loca_table = sorted_tables.iter().find(|t| &t.tag == b"loca");
        let head_table = sorted_tables.iter().find(|t| &t.tag == b"head");
        let maxp_table = sorted_tables.iter().find(|t| &t.tag == b"maxp");

        if let (Some(glyf), Some(loca), Some(head), Some(maxp)) =
            (glyf_table, loca_table, head_table, maxp_table)
        {
            let glyf_data = &ttf_data[glyf.offset as usize..(glyf.offset + glyf.length) as usize];
            let loca_data = &ttf_data[loca.offset as usize..(loca.offset + loca.length) as usize];
            let head_data = &ttf_data[head.offset as usize..(head.offset + head.length) as usize];
            let maxp_data = &ttf_data[maxp.offset as usize..(maxp.offset + maxp.length) as usize];

            transformed_glyf = time_section!(
                "glyf/loca transform",
                Some(transform_glyf(glyf_data, loca_data, head_data, maxp_data)?)
            );
        }
    }

    let transformed_glyf_len = transformed_glyf.as_ref().map(|v| v.len() as u32);

    let directory_entries: Vec<TableDirectoryEntry> = sorted_tables
        .iter()
        .map(|t| {
            let tag_index = find_tag_index(&t.tag);
            let is_glyf = tag_index == Some(10);
            let is_loca = tag_index == Some(11);

            let (transform_version, orig_length, transform_length) =
                if let Some(tglyf_len) = transformed_glyf_len {
                    if is_glyf {
                        (0, t.length, Some(tglyf_len))
                    } else if is_loca {
                        (0, t.length, Some(0))
                    } else {
                        (0, t.length, None)
                    }
                } else if is_glyf || is_loca {
                    (3, t.length, None)
                } else {
                    (0, t.length, None)
                };

            TableDirectoryEntry {
                tag: t.tag,
                orig_length,
                transform_version,
                transform_length,
            }
        })
        .collect();

    let mut uncompressed_data = Vec::new();
    for table in &sorted_tables {
        let tag_index = find_tag_index(&table.tag);
        let is_glyf = tag_index == Some(10);
        let is_loca = tag_index == Some(11);

        if let Some(ref tglyf) = transformed_glyf {
            if is_glyf {
                uncompressed_data.extend(tglyf);
                continue;
            } else if is_loca {
                continue;
            }
        }
        let start = table.offset as usize;
        let end = start + table.length as usize;
        uncompressed_data.extend_from_slice(&ttf_data[start..end]);
    }

    let mut compressed_data = Vec::new();
    let params = BrotliEncoderParams {
        quality: quality.into(),
        mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT,
        ..Default::default()
    };

    #[cfg(feature = "timing")]
    eprintln!(
        "[TIMING] Uncompressed data size: {} bytes ({:.2} MB)",
        uncompressed_data.len(),
        uncompressed_data.len() as f64 / (1024.0 * 1024.0)
    );

    time_section!(
        "Brotli compression",
        BrotliCompress(&mut &uncompressed_data[..], &mut compressed_data, &params)
            .map_err(|e| format!("Brotli compression failed: {e}"))?
    );

    #[cfg(feature = "timing")]
    eprintln!(
        "[TIMING] Compressed data size: {} bytes ({:.2} MB), ratio: {:.1}%",
        compressed_data.len(),
        compressed_data.len() as f64 / (1024.0 * 1024.0),
        (compressed_data.len() as f64 / uncompressed_data.len() as f64) * 100.0
    );

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

    #[cfg(feature = "timing")]
    eprintln!("[TIMING] Total encode time: {:?}", total_start.elapsed());

    Ok(result)
}
