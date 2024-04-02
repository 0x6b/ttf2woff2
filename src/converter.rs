use std::ops::Deref;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use cpp::cpp;
use log::info;
use tokio::fs::{read, write};

use crate::{
    brotli_quality::BrotliQuality,
    state::{Loaded, State, Uninitialized},
    Error,
};

cpp! {{
    #include <woff2/encode.h>

    using std::string;
    using woff2::MaxWOFF2CompressedSize;
    using woff2::ConvertTTFToWOFF2;
    using woff2::WOFF2Params;
}}

pub struct Converter<S>
where
    S: State,
{
    state: S,
}

impl<S> Deref for Converter<S>
where
    S: State,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Converter<Uninitialized> {
    pub async fn try_new() -> Result<Converter<Loaded>> {
        let Uninitialized { input, output, quality } = Uninitialized::parse();

        if input.extension() != Some("ttf") {
            return Err(Error::InvalidFileName(input.to_string()).into());
        }

        let output = match output {
            None => {
                let mut output = input.clone();
                output.set_extension("woff2");
                output
            }
            Some(o) => o,
        };

        Self::from(input, output, quality).await
    }

    pub async fn from(
        input: Utf8PathBuf,
        output: Utf8PathBuf,
        quality: BrotliQuality,
    ) -> Result<Converter<Loaded>> {
        if !&output.exists() {
            info!("Creating a new output file");
            write(&output, &[]).await?;
        }

        info!("{} → {}", &input.canonicalize_utf8()?, &output.canonicalize_utf8()?);
        let data = read(&input).await?;

        Ok(Converter { state: Loaded { data, output, quality } })
    }
}

impl Converter<Loaded> {
    pub async fn to_woff2(&self) -> Result<(usize, usize)> {
        let data = self.convert_ttf_to_woff2().map_err(Error::from)?;
        write(&self.output, &data).await?;

        Ok((self.data.len(), data.len()))
    }

    fn convert_ttf_to_woff2(&self) -> Result<Vec<u8>> {
        let capacity = self.data.len() + 1024;

        let data = self.data.as_ptr();
        let length = self.data.len();

        let mut woff_font_bytes = Vec::with_capacity(capacity);
        let result = woff_font_bytes.as_mut_ptr();

        let mut woff_font_bytes_length = std::mem::MaybeUninit::<usize>::new(capacity);
        let result_length = woff_font_bytes_length.as_mut_ptr();

        let bytes: &[u8; 0] = &[];
        let extended_metadata = bytes.as_ptr();
        let extended_metadata_length = 0usize;

        let brotli_quality = self.quality.as_i32();
        let allow_transforms = true;

        let success = unsafe {
            cpp!([
                data as "const uint8_t *",
                length as "size_t",
                result as "uint8_t *",
                result_length as "size_t *",
                extended_metadata as "const char *",
                extended_metadata_length as "size_t",
                brotli_quality as "int",
                allow_transforms as "bool"
            ] -> bool as "bool" {
                string copyOfExtendedMetadata(extended_metadata, extended_metadata_length);

                struct WOFF2Params params;
                params.extended_metadata = copyOfExtendedMetadata;
                params.brotli_quality = brotli_quality;
                params.allow_transforms = allow_transforms;

                return ConvertTTFToWOFF2(data, length, result, result_length, params);
            })
        };

        if success {
            unsafe { woff_font_bytes.set_len(*woff_font_bytes_length.as_ptr()) };
            Ok(woff_font_bytes)
        } else {
            Err(Error::ConversionFailed.into())
        }
    }
}
