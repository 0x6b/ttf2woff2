use brotli::enc::{BrotliCompress, BrotliEncoderParams};

use super::brotli_quality::BrotliQuality;
use super::directory::TableDirectoryEntry;
use super::header::{Woff2Header, WOFF2_SIGNATURE};
use super::inline_bytes::InlineBytes;
use super::sfnt::{Sfnt, SfntTable};
use super::transform::transform_glyf;
use crate::Error;

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

/// Options for WOFF2 encoding
#[derive(Debug, Clone, Copy)]
pub struct EncodeOptions {
    pub quality: BrotliQuality,
    pub transform_glyf_loca: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            quality: BrotliQuality::default(),
            transform_glyf_loca: true,
        }
    }
}

struct Encoder<'a> {
    data: &'a [u8],
    sfnt: Sfnt,
    options: EncodeOptions,
}

struct TableRefs<'a> {
    glyf: Option<&'a SfntTable>,
    loca: Option<&'a SfntTable>,
    head: Option<&'a SfntTable>,
    maxp: Option<&'a SfntTable>,
}

impl<'a> TableRefs<'a> {
    fn from_sorted(sorted_tables: &[&'a SfntTable]) -> Self {
        let mut refs = Self { glyf: None, loca: None, head: None, maxp: None };
        for &table in sorted_tables {
            match () {
                _ if table.tag.is_glyf() => refs.glyf = Some(table),
                _ if table.tag.is_loca() => refs.loca = Some(table),
                _ if table.tag.is_head() => refs.head = Some(table),
                _ if table.tag.is_maxp() => refs.maxp = Some(table),
                _ => {}
            }
        }
        refs
    }
}

impl<'a> Encoder<'a> {
    fn new(data: &'a [u8], options: EncodeOptions) -> Result<Self, Error> {
        let sfnt = time_section!("SFNT parsing", Sfnt::parse(data)?);
        Ok(Self { data, sfnt, options })
    }

    fn table_slice(&self, table: &SfntTable) -> &'a [u8] {
        let start = table.offset as usize;
        let end = start + table.length as usize;
        &self.data[start..end]
    }

    fn encode(self) -> Result<Vec<u8>, Error> {
        #[cfg(feature = "timing")]
        let total_start = std::time::Instant::now();

        let mut sorted_tables: Vec<_> = self.sfnt.tables.iter().collect();
        sorted_tables.sort_by_key(|t| t.tag);

        let table_refs = TableRefs::from_sorted(&sorted_tables);
        let (major_version, minor_version) = self.extract_version(&table_refs);
        let transformed_glyf = self.transform_glyf_if_needed(&table_refs)?;
        let transformed_glyf_len = transformed_glyf.as_ref().map(|v| v.len() as u32);

        let directory_entries = self.build_directory_entries(&sorted_tables, transformed_glyf_len);
        let (encoded_directory, directory_size) = self.encode_directory_entries(&directory_entries);
        let uncompressed_data =
            self.build_uncompressed_data(&sorted_tables, transformed_glyf.as_deref());
        let compressed_data = self.compress(&uncompressed_data)?;

        let result = self.build_output(
            &sorted_tables,
            &encoded_directory,
            directory_size,
            &compressed_data,
            major_version,
            minor_version,
        );

        #[cfg(feature = "timing")]
        eprintln!("[TIMING] Total encode time: {:?}", total_start.elapsed());

        Ok(result)
    }

    fn extract_version(&self, table_refs: &TableRefs) -> (u16, u16) {
        table_refs
            .head
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

    fn transform_glyf_if_needed(&self, table_refs: &TableRefs) -> Result<Option<Vec<u8>>, Error> {
        if !self.options.transform_glyf_loca {
            return Ok(None);
        }

        if let (Some(glyf), Some(loca), Some(head), Some(maxp)) =
            (table_refs.glyf, table_refs.loca, table_refs.head, table_refs.maxp)
        {
            let glyf_data = self.table_slice(glyf);
            let loca_data = self.table_slice(loca);
            let head_data = self.table_slice(head);
            let maxp_data = self.table_slice(maxp);

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
                let (transform_version, transform_length) =
                    match (transformed_glyf_len, is_glyf, is_loca) {
                        (Some(tglyf_len), true, _) => (0, Some(tglyf_len)),
                        (Some(_), _, true) => (0, Some(0)),
                        (Some(_), _, _) => (0, None),
                        (None, true, _) | (None, _, true) => (3, None),
                        (None, _, _) => (0, None),
                    };

                TableDirectoryEntry {
                    tag: t.tag,
                    orig_length: t.length,
                    transform_version,
                    transform_length,
                }
            })
            .collect()
    }

    fn encode_directory_entries(
        &self,
        entries: &[TableDirectoryEntry],
    ) -> (Vec<InlineBytes<15>>, usize) {
        let mut encoded = Vec::with_capacity(entries.len());
        let mut size = 0usize;
        for entry in entries {
            let bytes = entry.to_bytes();
            size += bytes.len();
            encoded.push(bytes);
        }
        (encoded, size)
    }

    fn build_uncompressed_data(
        &self,
        sorted_tables: &[&SfntTable],
        transformed_glyf: Option<&[u8]>,
    ) -> Vec<u8> {
        let total_len = match transformed_glyf {
            Some(tglyf) => {
                let mut total = 0usize;
                for table in sorted_tables {
                    if table.tag.is_glyf() {
                        total += tglyf.len();
                        continue;
                    }
                    if table.tag.is_loca() {
                        continue;
                    }
                    total += table.length as usize;
                }
                total
            }
            None => sorted_tables.iter().map(|table| table.length as usize).sum(),
        };

        let mut uncompressed_data = Vec::with_capacity(total_len);
        match transformed_glyf {
            Some(tglyf) => {
                for table in sorted_tables {
                    if table.tag.is_glyf() {
                        uncompressed_data.extend_from_slice(tglyf);
                        continue;
                    }
                    if table.tag.is_loca() {
                        continue;
                    }
                    uncompressed_data.extend_from_slice(self.table_slice(table));
                }
            }
            None => {
                for table in sorted_tables {
                    uncompressed_data.extend_from_slice(self.table_slice(table));
                }
            }
        }

        uncompressed_data
    }

    fn compress(&self, uncompressed_data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut compressed_data = Vec::with_capacity(uncompressed_data.len());
        let params = BrotliEncoderParams {
            quality: self.options.quality.into(),
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
        encoded_directory: &[InlineBytes<15>],
        directory_size: usize,
        compressed_data: &[u8],
        major_version: u16,
        minor_version: u16,
    ) -> Vec<u8> {
        let total_sfnt_size = 12
            + 16 * self.sfnt.tables.len() as u32
            + sorted_tables.iter().map(|t| (t.length + 3) & !3).sum::<u32>();

        let total_length = 48 + directory_size as u32 + compressed_data.len() as u32;

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
        for entry in encoded_directory {
            result.extend_from_slice(entry.as_slice());
        }
        result.extend_from_slice(compressed_data);
        result
    }
}

pub fn encode(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    let options = EncodeOptions { quality, transform_glyf_loca: true };
    Encoder::new(ttf_data, options)?.encode()
}

pub fn encode_no_transform(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    let options = EncodeOptions { quality, transform_glyf_loca: false };
    Encoder::new(ttf_data, options)?.encode()
}
