use crate::compression::rle_decompress;
use crate::error::{WkError, WkResult};
use crate::header::{ColorType, WkHeader};
use crate::metadata::WkMetadata;
use byteorder::{LittleEndian, ReadBytesExt};
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use std::io::Read;

pub struct WkDecoder;

impl WkDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode<R: Read>(&self, reader: &mut R) -> WkResult<(DynamicImage, WkMetadata)> {
        let header = WkHeader::read(reader)?;

        let metadata = if header.metadata_size > 0 {
            let mut metadata_bytes = vec![0u8; header.metadata_size as usize];
            reader.read_exact(&mut metadata_bytes)?;
            WkMetadata::decode(&metadata_bytes)?
        } else {
            WkMetadata::default()
        };

        let compressed_size = reader.read_u32::<LittleEndian>()? as usize;

        let mut compressed_data = vec![0u8; compressed_size];
        reader.read_exact(&mut compressed_data)?;

        let expected_size =
            (header.width * header.height * header.color_type.channels() as u32) as usize;
        let raw_data = if header.compression == 1 {
            rle_decompress(&compressed_data, expected_size)?
        } else {
            if compressed_data.len() != expected_size {
                return Err(WkError::InvalidFormat(format!(
                    "Data size mismatch: expected {}, got {}",
                    expected_size,
                    compressed_data.len()
                )));
            }
            compressed_data
        };

        let image = match header.color_type {
            ColorType::RGB => {
                let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
                    header.width,
                    header.height,
                    raw_data,
                )
                .ok_or_else(|| WkError::DecodingError("Failed to create RGB image".to_string()))?;
                DynamicImage::ImageRgb8(img)
            }
            ColorType::RGBA => {
                let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                    header.width,
                    header.height,
                    raw_data,
                )
                .ok_or_else(|| WkError::DecodingError("Failed to create RGBA image".to_string()))?;
                DynamicImage::ImageRgba8(img)
            }
            ColorType::Grayscale => {
                let img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(
                    header.width,
                    header.height,
                    raw_data,
                )
                .ok_or_else(|| {
                    WkError::DecodingError("Failed to create grayscale image".to_string())
                })?;
                DynamicImage::ImageLuma8(img)
            }
            ColorType::GrayscaleAlpha => {
                let img = ImageBuffer::<LumaA<u8>, Vec<u8>>::from_raw(
                    header.width,
                    header.height,
                    raw_data,
                )
                .ok_or_else(|| {
                    WkError::DecodingError("Failed to create grayscale+alpha image".to_string())
                })?;
                DynamicImage::ImageLumaA8(img)
            }
        };

        Ok((image, metadata))
    }
}

impl Default for WkDecoder {
    fn default() -> Self {
        Self::new()
    }
}
