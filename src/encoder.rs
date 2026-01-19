use crate::compression::{CompressionConfig, CompressionEngine};
use crate::error::WkResult;
use crate::format::header::{ColorType, CompressionMode, WkHeader};
use crate::format::{Chunk, ChunkType, ChunkWriter};
use crate::metadata::WkMetadata;
use image::DynamicImage;
use std::io::Write;

pub struct WkEncoder {
    config: CompressionConfig,
    metadata: WkMetadata,
}

impl WkEncoder {
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
            metadata: WkMetadata::new(),
        }
    }

    pub fn lossless() -> Self {
        Self {
            config: CompressionConfig::lossless(),
            metadata: WkMetadata::new(),
        }
    }

    pub fn lossy(quality: u8) -> Self {
        Self {
            config: CompressionConfig::lossy(quality),
            metadata: WkMetadata::new(),
        }
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.config.quality = quality.clamp(1, 100);
        if quality == 100 {
            self.config.mode = CompressionMode::Lossless;
        }
        self
    }

    pub fn with_metadata(mut self, metadata: WkMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_compression_mode(mut self, mode: CompressionMode) -> Self {
        self.config.mode = mode;
        self
    }

    fn image_to_raw(image: &DynamicImage) -> (ColorType, Vec<u8>) {
        match image {
            DynamicImage::ImageLuma8(img) => (ColorType::Grayscale, img.as_raw().clone()),
            DynamicImage::ImageLumaA8(img) => (ColorType::GrayscaleAlpha, img.as_raw().clone()),
            DynamicImage::ImageRgb8(img) => (ColorType::Rgb, img.as_raw().clone()),
            DynamicImage::ImageRgba8(img) => (ColorType::Rgba, img.as_raw().clone()),
            _ => {
                let rgba = image.to_rgba8();
                (ColorType::Rgba, rgba.as_raw().clone())
            }
        }
    }

    pub fn encode<W: Write>(&self, image: &DynamicImage, writer: W) -> WkResult<()> {
        let width = image.width();
        let height = image.height();
        let (color_type, raw_data) = Self::image_to_raw(image);

        let header = WkHeader {
            width,
            height,
            color_type,
            compression_mode: self.config.mode,
            quality: self.config.quality,
            has_alpha: color_type.has_alpha(),
            has_animation: false,
            bit_depth: 8,
        };

        let engine = CompressionEngine::new(self.config.clone());
        let compressed = engine.compress(
            &raw_data,
            width as usize,
            height as usize,
            color_type.channels() as usize,
        )?;

        let mut chunk_writer = ChunkWriter::new(writer);

        let header_chunk = Chunk::new(ChunkType::ImageHeader, header.encode());
        chunk_writer.write_chunk(&header_chunk)?;

        if let Some(ref icc) = self.metadata.icc_profile {
            let icc_data = bincode::serialize(icc)
                .map_err(|e| crate::error::WkError::MetadataError(e.to_string()))?;
            let icc_chunk = Chunk::new(ChunkType::IccProfile, icc_data);
            chunk_writer.write_chunk(&icc_chunk)?;
        }

        if let Some(ref exif) = self.metadata.exif {
            let exif_data = bincode::serialize(exif)
                .map_err(|e| crate::error::WkError::MetadataError(e.to_string()))?;
            let exif_chunk = Chunk::new(ChunkType::Exif, exif_data);
            chunk_writer.write_chunk(&exif_chunk)?;
        }

        if let Some(ref xmp) = self.metadata.xmp {
            let xmp_data = bincode::serialize(xmp)
                .map_err(|e| crate::error::WkError::MetadataError(e.to_string()))?;
            let xmp_chunk = Chunk::new(ChunkType::Xmp, xmp_data);
            chunk_writer.write_chunk(&xmp_chunk)?;
        }

        let custom_data = self.metadata.custom.clone();
        if !custom_data.fields.is_empty() || custom_data.author.is_some() {
            let custom_bytes = bincode::serialize(&custom_data)
                .map_err(|e| crate::error::WkError::MetadataError(e.to_string()))?;
            let custom_chunk = Chunk::new(ChunkType::Custom, custom_bytes);
            chunk_writer.write_chunk(&custom_chunk)?;
        }

        let data_type = match self.config.mode {
            CompressionMode::Lossless => ChunkType::ImageData,
            _ => ChunkType::ImageDataLossy,
        };
        let data_chunk = Chunk::new(data_type, compressed);
        chunk_writer.write_chunk(&data_chunk)?;

        chunk_writer.finish()?;

        Ok(())
    }

    pub fn encode_to_vec(&self, image: &DynamicImage) -> WkResult<Vec<u8>> {
        let mut buffer = Vec::new();
        self.encode(image, &mut buffer)?;
        Ok(buffer)
    }
}

impl Default for WkEncoder {
    fn default() -> Self {
        Self::new()
    }
}
