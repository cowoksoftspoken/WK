use crate::decoder::{DecodedImage, WkDecoder};
use crate::encoder::WkEncoder;
use crate::error::WkResult;
use crate::metadata::WkMetadata;
use image::DynamicImage;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub struct WkConverter {
    quality: u8,
    lossless: bool,
    metadata: Option<WkMetadata>,
}

impl WkConverter {
    pub fn new() -> Self {
        Self {
            quality: 85,
            lossless: false,
            metadata: None,
        }
    }

    pub fn lossless() -> Self {
        Self {
            quality: 100,
            lossless: true,
            metadata: None,
        }
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.clamp(1, 100);
        self.lossless = quality == 100;
        self
    }

    pub fn with_metadata(mut self, metadata: WkMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn to_wk<P: AsRef<Path>, Q: AsRef<Path>>(&self, input: P, output: Q) -> WkResult<()> {
        let img = image::open(input)?;
        self.image_to_wk(&img, output)
    }

    pub fn image_to_wk<P: AsRef<Path>>(&self, image: &DynamicImage, output: P) -> WkResult<()> {
        let file = File::create(output)?;
        let writer = BufWriter::new(file);

        let encoder = if self.lossless {
            WkEncoder::lossless()
        } else {
            WkEncoder::lossy(self.quality)
        };

        let encoder = if let Some(ref meta) = self.metadata {
            encoder.with_metadata(meta.clone())
        } else {
            encoder
        };

        encoder.encode(image, writer)
    }

    pub fn from_wk<P: AsRef<Path>, Q: AsRef<Path>>(&self, input: P, output: Q) -> WkResult<()> {
        let decoded = self.wk_to_image(input)?;
        decoded.image.save(output)?;
        Ok(())
    }

    pub fn wk_to_image<P: AsRef<Path>>(&self, input: P) -> WkResult<DecodedImage> {
        let file = File::open(input)?;
        let reader = BufReader::new(file);
        let decoder = WkDecoder::new();
        decoder.decode(reader)
    }

    pub fn wk_to_dynamic_image<P: AsRef<Path>>(&self, input: P) -> WkResult<DynamicImage> {
        Ok(self.wk_to_image(input)?.image)
    }
}

impl Default for WkConverter {
    fn default() -> Self {
        Self::new()
    }
}
