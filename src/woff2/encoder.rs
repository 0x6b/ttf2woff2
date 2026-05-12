use std::num::NonZeroUsize;

use brotli::enc::{
    BrotliCompress, BrotliEncoderParams, StandardAlloc,
    backward_references::UnionHasher,
    multithreading::MultiThreadedSpawner,
    threading::{CompressMultiSlice, SendAlloc},
};

use super::{
    brotli_quality::BrotliQuality,
    directory::TableDirectoryEntry,
    header::{WOFF2_SIGNATURE, Woff2Header},
    inline_bytes::InlineBytes,
    sfnt::{Sfnt, SfntTable},
    transform::GlyfContext,
};
use crate::Error;

/// Options for WOFF2 encoding.
#[derive(Debug, Clone, Copy)]
pub struct EncodeOptions {
    /// Brotli compression quality (0-11). Higher values produce smaller output but take longer.
    pub quality: BrotliQuality,
    /// Apply the WOFF2 `glyf`/`loca` table transformation.
    ///
    /// When enabled, the `glyf` and `loca` tables are restructured per the
    /// [WOFF2 specification](https://www.w3.org/TR/WOFF2/#glyf_table_format)
    /// before Brotli compression, which typically reduces output size noticeably.
    /// The transformation is only applied when all of `glyf`, `loca`, `head`,
    /// and `maxp` tables are present; otherwise the tables are stored as-is.
    pub transform_glyf_loca: bool,
    /// Number of threads to use for the Brotli compression step.
    ///
    /// `None` (default) uses the single-threaded encoder, which is fully deterministic.
    ///
    /// `Some(n)` with `n >= 2` uses the multi-threaded Brotli encoder, which can
    /// roughly halve to quarter wall-time on large fonts at quality 10-11 on multi-core
    /// machines. The cost: output bytes depend on `n` (each thread compresses an
    /// independent slice with `catable=true`), and total compressed size grows by
    /// roughly 0.05-0.5 % vs single-threaded. The output remains a single valid
    /// Brotli stream that any spec-compliant WOFF2 decoder accepts.
    ///
    /// Has no effect when `n == 1`.
    ///
    /// On `wasm32-*` targets this option is silently ignored and compression
    /// always runs single-threaded, because `std::thread::spawn` is not
    /// available on WebAssembly. Setting `Some(n > 1)` from a WASM build is
    /// safe (no panic) but yields the same output as `None`.
    pub threads: Option<NonZeroUsize>,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            quality: BrotliQuality::default(),
            transform_glyf_loca: true,
            threads: None,
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
            if table.tag.is_glyf() {
                refs.glyf = Some(table);
            } else if table.tag.is_loca() {
                refs.loca = Some(table);
            } else if table.tag.is_head() {
                refs.head = Some(table);
            } else if table.tag.is_maxp() {
                refs.maxp = Some(table);
            }
        }
        refs
    }
}

impl<'a> Encoder<'a> {
    fn new(data: &'a [u8], options: EncodeOptions) -> Result<Self, Error> {
        let sfnt: Sfnt = data.try_into()?;
        Ok(Self { data, sfnt, options })
    }

    fn table_slice(&self, table: &SfntTable) -> &'a [u8] {
        let start = table.offset as usize;
        let end = start + table.length as usize;
        &self.data[start..end]
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

            let transformed = GlyfContext {
                glyf: glyf_data,
                loca: loca_data,
                head: head_data,
                maxp: maxp_data,
            }
            .transform()?;
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
                let (transform_version, transform_length) = match transformed_glyf_len {
                    Some(len) => (0, is_glyf.then_some(len).or(is_loca.then_some(0))),
                    None => (if is_glyf || is_loca { 3 } else { 0 }, None),
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
        let encoded: Vec<_> = entries.iter().map(InlineBytes::from).collect();
        let size = encoded.iter().map(InlineBytes::len).sum();
        (encoded, size)
    }

    fn build_uncompressed_data(
        &self,
        sorted_tables: &[&SfntTable],
        transformed_glyf: Option<&[u8]>,
    ) -> Vec<u8> {
        let total_len: usize = match transformed_glyf {
            Some(tglyf) => sorted_tables
                .iter()
                .map(|table| {
                    if table.tag.is_loca() {
                        0
                    } else if table.tag.is_glyf() {
                        tglyf.len()
                    } else {
                        table.length as usize
                    }
                })
                .sum(),
            None => sorted_tables.iter().map(|table| table.length as usize).sum(),
        };

        let mut data = Vec::with_capacity(total_len);
        if let Some(tglyf) = transformed_glyf {
            for table in sorted_tables {
                if table.tag.is_loca() {
                    continue;
                }
                if table.tag.is_glyf() {
                    data.extend_from_slice(tglyf);
                } else {
                    data.extend_from_slice(self.table_slice(table));
                }
            }
        } else {
            for table in sorted_tables {
                data.extend_from_slice(self.table_slice(table));
            }
        }
        data
    }

    fn compress(&self, uncompressed_data: &[u8]) -> Result<Vec<u8>, Error> {
        let params = BrotliEncoderParams {
            quality: self.options.quality.into(),
            mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT,
            size_hint: uncompressed_data.len(),
            ..Default::default()
        };

        // WASM cannot spawn OS threads (`std::thread::spawn` panics on
        // `wasm32-unknown-unknown`), so silently force single-threaded there
        // regardless of the requested thread count.
        let num_threads = if cfg!(target_family = "wasm") {
            1
        } else {
            self.options.threads.map(NonZeroUsize::get).unwrap_or(1)
        };
        if num_threads <= 1 {
            let mut compressed_data = Vec::with_capacity(uncompressed_data.len());
            BrotliCompress(&mut &uncompressed_data[..], &mut compressed_data, &params)
                .map_err(|e| Error::Compression(e.to_string()))?;
            return Ok(compressed_data);
        }

        // Multi-threaded path. Worst-case output bound: input length + per-thread overhead.
        // (Brotli rarely expands input; the slack covers per-metablock headers.)
        let mut output = vec![0u8; uncompressed_data.len() + 1024 * num_threads + 1024];
        let mut allocs: Vec<SendAlloc<_, _, StandardAlloc, _>> = (0..num_threads)
            .map(|_| SendAlloc::new(StandardAlloc::default(), UnionHasher::Uninit))
            .collect();
        let mut spawner = MultiThreadedSpawner::default();
        let written = CompressMultiSlice(
            &params,
            uncompressed_data,
            &mut output[..],
            &mut allocs[..],
            &mut spawner,
        )
        .map_err(|e| Error::Compression(format!("{e:?}")))?;
        output.truncate(written);
        Ok(output)
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
        fn align4(value: u32) -> u32 {
            (value + 3) & !3
        }

        let total_sfnt_size = 12
            + 16 * self.sfnt.tables.len() as u32
            + sorted_tables.iter().map(|t| align4(t.length)).sum::<u32>();

        let unpadded_length = 48 + directory_size as u32 + compressed_data.len() as u32;
        // WOFF2 file must be padded to 4-byte boundary
        let total_length = align4(unpadded_length);

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
        result.extend_from_slice(&<[u8; 48]>::from(&header));
        for entry in encoded_directory {
            result.extend_from_slice(entry.as_slice());
        }
        result.extend_from_slice(compressed_data);
        // Add padding to reach 4-byte alignment
        result.resize(total_length as usize, 0u8);
        result
    }
}

impl TryFrom<Encoder<'_>> for Vec<u8> {
    type Error = Error;

    fn try_from(encoder: Encoder<'_>) -> Result<Self, Self::Error> {
        let mut sorted_tables: Vec<_> = encoder.sfnt.tables.iter().collect();
        sorted_tables.sort_by_key(|t| t.tag);

        // WOFF2 spec requires loca to immediately follow glyf in the table directory
        if let Some(glyf_pos) = sorted_tables.iter().position(|t| t.tag.is_glyf())
            && let Some(loca_pos) = sorted_tables.iter().position(|t| t.tag.is_loca())
            && loca_pos != glyf_pos + 1
        {
            let loca = sorted_tables.remove(loca_pos);
            let new_glyf_pos = sorted_tables.iter().position(|t| t.tag.is_glyf()).unwrap();
            sorted_tables.insert(new_glyf_pos + 1, loca);
        }

        let table_refs = TableRefs::from_sorted(&sorted_tables);
        let (major_version, minor_version) = encoder.extract_version(&table_refs);
        let transformed_glyf = encoder.transform_glyf_if_needed(&table_refs)?;
        let transformed_glyf_len = transformed_glyf.as_ref().map(|v| v.len() as u32);

        let directory_entries =
            encoder.build_directory_entries(&sorted_tables, transformed_glyf_len);
        let (encoded_directory, directory_size) =
            encoder.encode_directory_entries(&directory_entries);
        let uncompressed_data =
            encoder.build_uncompressed_data(&sorted_tables, transformed_glyf.as_deref());
        let compressed_data = encoder.compress(&uncompressed_data)?;

        let result = encoder.build_output(
            &sorted_tables,
            &encoded_directory,
            directory_size,
            &compressed_data,
            major_version,
            minor_version,
        );

        Ok(result)
    }
}

/// Encode a TTF font as WOFF2 with the `glyf`/`loca` transformation enabled.
///
/// This is the recommended entry point and produces the smallest output for
/// TrueType fonts. Only TrueType (TTF) input is supported; OpenType/CFF fonts
/// return [`Error::UnsupportedFormat`].
///
/// `quality` selects the Brotli compression quality (0-11); use
/// [`BrotliQuality::default`] (11) for smallest output, or a lower value for
/// faster encoding.
///
/// # Errors
///
/// Returns an [`Error`] if the input is not a valid TTF font, a table extends
/// beyond the input bounds, glyph data is malformed, or Brotli compression fails.
pub fn encode(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    let options = EncodeOptions { quality, ..EncodeOptions::default() };
    Encoder::new(ttf_data, options)?.try_into()
}

/// Encode a TTF font as WOFF2 with full control over [`EncodeOptions`].
///
/// Use this when you need to override defaults — e.g. to enable multi-threaded
/// Brotli compression via [`EncodeOptions::threads`], or to disable the
/// `glyf`/`loca` transformation.
///
/// See [`encode`] for argument and error semantics.
pub fn encode_with_options(ttf_data: &[u8], options: EncodeOptions) -> Result<Vec<u8>, Error> {
    Encoder::new(ttf_data, options)?.try_into()
}

/// Encode a TTF font as WOFF2 without applying the `glyf`/`loca` transformation.
///
/// Tables are stored as-is before Brotli compression. Output is typically larger
/// than [`encode`], but encoding skips the transformation step. Useful for
/// debugging or when bit-exact preservation of the original table layout is needed.
///
/// See [`encode`] for argument and error semantics.
pub fn encode_no_transform(ttf_data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>, Error> {
    let options = EncodeOptions {
        quality,
        transform_glyf_loca: false,
        ..EncodeOptions::default()
    };
    Encoder::new(ttf_data, options)?.try_into()
}
