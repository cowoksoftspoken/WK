use crate::compression::{CompressionConfig, CompressionEngine};
use crate::error::{WkError, WkResult};
use crate::format::header::{ColorType, WkHeader};
use crate::format::{ChunkReader, ChunkType};
use crate::metadata::{CustomMetadata, ExifData, IccProfile, WkMetadata, XmpData};
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use std::io::Read;

pub struct DecodedImage {
    pub image: DynamicImage,
    pub metadata: WkMetadata,
    pub header: WkHeader,
}

pub struct WkDecoder;

impl WkDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode<R: Read>(&self, reader: R) -> WkResult<DecodedImage> {
        let mut chunk_reader = ChunkReader::new(reader);
        let chunks = chunk_reader.read_all_chunks()?;

        let header_chunk = chunks
            .iter()
            .find(|c| matches!(c.chunk_type, ChunkType::ImageHeader))
            .ok_or_else(|| WkError::MissingChunk("IHDR".into()))?;

        let header = WkHeader::decode(&header_chunk.data)?;

        let mut metadata = WkMetadata::new();

        for chunk in &chunks {
            match chunk.chunk_type {
                ChunkType::IccProfile => {
                    if let Ok(icc) = bincode::deserialize::<IccProfile>(&chunk.data) {
                        metadata.icc_profile = Some(icc);
                    }
                }
                ChunkType::Exif => {
                    if let Ok(exif) = bincode::deserialize::<ExifData>(&chunk.data) {
                        metadata.exif = Some(exif);
                    }
                }
                ChunkType::Xmp => {
                    if let Ok(xmp) = bincode::deserialize::<XmpData>(&chunk.data) {
                        metadata.xmp = Some(xmp);
                    }
                }
                ChunkType::Custom => {
                    if let Ok(custom) = bincode::deserialize::<CustomMetadata>(&chunk.data) {
                        metadata.custom = custom;
                    }
                }
                _ => {}
            }
        }

        let data_chunk = chunks
            .iter()
            .find(|c| {
                matches!(
                    c.chunk_type,
                    ChunkType::ImageData | ChunkType::ImageDataLossy
                )
            })
            .ok_or_else(|| WkError::MissingChunk("IDAT/IDLS".into()))?;

        let is_lossy = matches!(data_chunk.chunk_type, ChunkType::ImageDataLossy);

        let config = if is_lossy {
            CompressionConfig::lossy(header.quality)
        } else {
            CompressionConfig::lossless()
        };

        let engine = CompressionEngine::new(config);
        let raw_data = engine.decompress(
            &data_chunk.data,
            header.width as usize,
            header.height as usize,
            header.color_type.channels() as usize,
            header.compression_mode,
        )?;

        let image = self.raw_to_image(&raw_data, &header)?;

        Ok(DecodedImage {
            image,
            metadata,
            header,
        })
    }

    fn raw_to_image(&self, data: &[u8], header: &WkHeader) -> WkResult<DynamicImage> {
        let w = header.width;
        let h = header.height;

        let image = match header.color_type {
            ColorType::Grayscale => {
                let img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(w, h, data.to_vec())
                    .ok_or_else(|| {
                        WkError::DecodingError("Failed to create grayscale image".into())
                    })?;
                DynamicImage::ImageLuma8(img)
            }
            ColorType::GrayscaleAlpha => {
                let img = ImageBuffer::<LumaA<u8>, Vec<u8>>::from_raw(w, h, data.to_vec())
                    .ok_or_else(|| {
                        WkError::DecodingError("Failed to create grayscale+alpha image".into())
                    })?;
                DynamicImage::ImageLumaA8(img)
            }
            ColorType::Rgb | ColorType::Yuv420 | ColorType::Yuv444 => {
                let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(w, h, data.to_vec())
                    .ok_or_else(|| WkError::DecodingError("Failed to create RGB image".into()))?;
                DynamicImage::ImageRgb8(img)
            }
            ColorType::Rgba => {
                let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, data.to_vec())
                    .ok_or_else(|| WkError::DecodingError("Failed to create RGBA image".into()))?;
                DynamicImage::ImageRgba8(img)
            }
        };

        Ok(image)
    }

    pub fn decode_header<R: Read>(&self, reader: R) -> WkResult<WkHeader> {
        let mut chunk_reader = ChunkReader::new(reader);
        chunk_reader.verify_magic()?;

        let chunk = chunk_reader.read_chunk()?;
        if !matches!(chunk.chunk_type, ChunkType::ImageHeader) {
            return Err(WkError::InvalidFormat("First chunk must be IHDR".into()));
        }

        WkHeader::decode(&chunk.data)
    }
}

impl Default for WkDecoder {
    fn default() -> Self {
        Self::new()
    }
}
