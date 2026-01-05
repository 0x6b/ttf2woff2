use brotli::enc::{BrotliCompress, BrotliEncoderParams};

use crate::{
    BrotliQuality, Error,
    directory::TableDirectoryEntry,
    header::{WOFF2_SIGNATURE, Woff2Header},
    sfnt::{Sfnt, SfntTable},
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

struct Encoder<'a> {
    data: &'a [u8],
    sfnt: Sfnt,
    quality: BrotliQuality,
}

impl<'a> Encoder<'a> {
    fn new(data: &'a [u8], quality: BrotliQuality) -> Result<Self, Error> {
        let sfnt = time_section!("SFNT parsing", Sfnt::parse(data)?);
        Ok(Self { data, sfnt, quality })
    }

    fn encode(self, transform_glyf_loca: bool) -> Result<Vec<u8>, Error> {
        #[cfg(feature = "timing")]
        let total_start = std::time::Instant::now();

        let mut sorted_tables: Vec<_> = self.sfnt.tables.iter().collect();
        sorted_tables.sort_by_key(|t| t.tag);

        let (major_version, minor_version) = self.extract_version(&sorted_tables);
        let transformed_glyf =
            self.transform_glyf_if_needed(&sorted_tables, transform_glyf_loca)?;
        let transformed_glyf_len = transformed_glyf.as_ref().map(|v| v.len() as u32);

        let directory_entries = self.build_directory_entries(&sorted_tables, transformed_glyf_len);
        let uncompressed_data =
            self.build_uncompressed_data(&sorted_tables, transformed_glyf.as_ref());
        let compressed_data = self.compress(&uncompressed_data)?;

        let result = self.build_output(
            &sorted_tables,
            &directory_entries,
            &compressed_data,
            major_version,
            minor_version,
        );

        #[cfg(feature = "timing")]
        eprintln!("[TIMING] Total encode time: {:?}", total_start.elapsed());

        Ok(result)
    }

    fn extract_version(&self, sorted_tables: &[&SfntTable]) -> (u16, u16) {
        sorted_tables
            .iter()
            .find(|t| t.tag.is_head())
            .and_then(|head| {
                let start = head.offset as usize;
                let data = self.data.get(start + 4..start + 8)?;
                Some((
                    u16::from_be_bytes([data[0], data[1]]),
                    u16::from_be_bytes([data[2], data[3]]),
                ))
            })
            .unwrap_or((0, 0))
    }

    fn transform_glyf_if_needed(
        &self,
        sorted_tables: &[&SfntTable],
        transform_glyf_loca: bool,
    ) -> Result<Option<Vec<u8>>, Error> {
        if !transform_glyf_loca {
            return Ok(None);
        }

        let glyf_table = sorted_tables.iter().find(|t| t.tag.is_glyf());
        let loca_table = sorted_tables.iter().find(|t| t.tag.is_loca());
        let head_table = sorted_tables.iter().find(|t| t.tag.is_head());
        let maxp_table = sorted_tables.iter().find(|t| t.tag.is_maxp());

        if let (Some(glyf), Some(loca), Some(head), Some(maxp)) =
            (glyf_table, loca_table, head_table, maxp_table)
        {
            let glyf_data = &self.data[glyf.offset as usize..(glyf.offset + glyf.length) as usize];
            let loca_data = &self.data[loca.offset as usize..(loca.offset + loca.length) as usize];
            let head_data = &self.data[head.offset as usize..(head.offset + head.length) as usize];
            let maxp_data = &self.data[maxp.offset as usize..(maxp.offset + maxp.length) as usize];

            let transformed = time_section!(
                "glyf/loca transform",
                transform_glyf(glyf_data, loca_data, head_data, maxp_data)?
            );
            return Ok(Some(transformed));
        }

        Ok(None)
    }

    fn build_directory_entries(
        &self,
        sorted_tables: &[&SfntTable],
        transformed_glyf_len: Option<u32>,
    ) -> Vec<TableDirectoryEntry> {
        sorted_tables
            .iter()
            .map(|t| {
                let is_glyf = t.tag.is_glyf();
                let is_loca = t.tag.is_loca();

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
            .collect()
    }

    fn build_uncompressed_data(
        &self,
        sorted_tables: &[&SfntTable],
        transformed_glyf: Option<&Vec<u8>>,
    ) -> Vec<u8> {
        let mut uncompressed_data = Vec::new();
        for table in sorted_tables {
            let is_glyf = table.tag.is_glyf();
            let is_loca = table.tag.is_loca();

            if let Some(tglyf) = transformed_glyf {
                if is_glyf {
                    uncompressed_data.extend(tglyf);
                    continue;
                } else if is_loca {
                    continue;
                }
            }
            let start = table.offset as usize;
            let end = start + table.length as usize;
            uncompressed_data.extend_from_slice(&self.data[start..end]);
        }
        uncompressed_data
    }

    fn compress(&self, uncompressed_data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut compressed_data = Vec::with_capacity(uncompressed_data.len());
        let params = BrotliEncoderParams {
            quality: self.quality.into(),
            mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT,
            size_hint: uncompressed_data.len(),
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
                .map_err(|e| Error::Compression(e.to_string()))?
        );

        #[cfg(feature = "timing")]
        eprintln!(
            "[TIMING] Compressed data size: {} bytes ({:.2} MB), ratio: {:.1}%",
            compressed_data.len(),
            compressed_data.len() as f64 / (1024.0 * 1024.0),
            (compressed_data.len() as f64 / uncompressed_data.len() as f64) * 100.0
        );

        Ok(compressed_data)
    }

    fn build_output(
        &self,
        sorted_tables: &[&SfntTable],
        directory_entries: &[TableDirectoryEntry],
        compressed_data: &[u8],
        major_version: u16,
        minor_version: u16,
    ) -> Vec<u8> {
        let total_sfnt_size = 12
            + 16 * self.sfnt.tables.len() as u32
            + sorted_tables.iter().map(|t| (t.length + 3) & !3).sum::<u32>();

        let mut directory_bytes = Vec::new();
        for entry in directory_entries {
            directory_bytes.extend(entry.to_bytes());
        }

        let total_length = 48 + directory_bytes.len() as u32 + compressed_data.len() as u32;

        let header = Woff2Header {
            signature: WOFF2_SIGNATURE,
            flavor: self.sfnt.flavor,
            length: total_length,
            num_tables: self.sfnt.tables.len() as u16,
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
        result.extend(compressed_data);
        result
    }
}

pub fn encode(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    Encoder::new(ttf_data, quality)?.encode(true)
}

pub fn encode_no_transform(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    Encoder::new(ttf_data, quality)?.encode(false)
}
