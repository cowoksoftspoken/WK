use crate::decoder::WkDecoder;
use crate::encoder::WkEncoder;
use crate::error::{WkError, WkResult};
use crate::metadata::WkMetadata;
use image::DynamicImage;
use std::path::Path;

pub struct WkConverter;

impl WkConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn to_wk<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> WkResult<()> {
        let img = image::open(&input_path).map_err(|e| WkError::ImageError(e))?;

        let mut metadata = WkMetadata::new();
        if let Some(filename) = input_path.as_ref().file_name() {
            metadata.description = Some(format!("Converted from {}", filename.to_string_lossy()));
        }

        let encoder = WkEncoder::new().with_metadata(metadata);
        let mut file = std::fs::File::create(output_path)?;
        encoder.encode(&img, &mut file)?;

        Ok(())
    }

    pub fn from_wk<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> WkResult<()> {
        let mut file = std::fs::File::open(input_path)?;
        let decoder = WkDecoder::new();
        let (img, _metadata) = decoder.decode(&mut file)?;

        img.save(output_path).map_err(|e| WkError::ImageError(e))?;

        Ok(())
    }

    pub fn image_to_wk<P: AsRef<Path>>(
        &self,
        img: &DynamicImage,
        output_path: P,
        metadata: Option<WkMetadata>,
    ) -> WkResult<()> {
        let encoder = if let Some(meta) = metadata {
            WkEncoder::new().with_metadata(meta)
        } else {
            WkEncoder::new()
        };

        let mut file = std::fs::File::create(output_path)?;
        encoder.encode(img, &mut file)?;

        Ok(())
    }

    pub fn wk_to_image<P: AsRef<Path>>(
        &self,
        input_path: P,
    ) -> WkResult<(DynamicImage, WkMetadata)> {
        let mut file = std::fs::File::open(input_path)?;
        let decoder = WkDecoder::new();
        decoder.decode(&mut file)
    }
}

impl Default for WkConverter {
    fn default() -> Self {
        Self::new()
    }
}
