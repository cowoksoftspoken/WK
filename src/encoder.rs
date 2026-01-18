use crate::compression::rle_compress;
use crate::error::WkResult;
use crate::header::{ColorType, WkHeader};
use crate::metadata::WkMetadata;
use image::DynamicImage;
use std::io::Write;

pub struct WkEncoder {
    metadata: WkMetadata,
    use_compression: bool,
}

impl WkEncoder {
    pub fn new() -> Self {
        Self {
            metadata: WkMetadata::new(),
            use_compression: true,
        }
    }

    pub fn with_metadata(mut self, metadata: WkMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn compression(mut self, enabled: bool) -> Self {
        self.use_compression = enabled;
        self
    }

    pub fn encode<W: Write>(&self, image: &DynamicImage, writer: &mut W) -> WkResult<()> {
        let (width, height) = (image.width(), image.height());

        let (color_type, raw_data) = match image {
            DynamicImage::ImageRgb8(img) => (ColorType::RGB, img.as_raw().clone()),
            DynamicImage::ImageRgba8(img) => (ColorType::RGBA, img.as_raw().clone()),
            DynamicImage::ImageLuma8(img) => (ColorType::Grayscale, img.as_raw().clone()),
            DynamicImage::ImageLumaA8(img) => (ColorType::GrayscaleAlpha, img.as_raw().clone()),
            _ => {
                let rgba = image.to_rgba8();
                (ColorType::RGBA, rgba.as_raw().clone())
            }
        };

        let metadata_bytes = self.metadata.encode()?;

        let mut header = WkHeader::new(width, height, color_type);
        header.compression = if self.use_compression { 1 } else { 0 };
        header.metadata_size = metadata_bytes.len() as u32;

        header.write(writer)?;

        writer.write_all(&metadata_bytes)?;

        let image_data = if self.use_compression {
            rle_compress(&raw_data)?
        } else {
            raw_data
        };

        use byteorder::{LittleEndian, WriteBytesExt};
        writer.write_u32::<LittleEndian>(image_data.len() as u32)?;

        writer.write_all(&image_data)?;

        Ok(())
    }
}

impl Default for WkEncoder {
    fn default() -> Self {
        Self::new()
    }
}
