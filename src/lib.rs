pub mod animation;
pub mod compression;
pub mod converter;
pub mod decoder;
pub mod encoder;
pub mod error;
pub mod format;
pub mod metadata;

pub use compression::{CompressionConfig, CompressionEngine};
pub use converter::WkConverter;
pub use decoder::{DecodedImage, WkDecoder};
pub use encoder::WkEncoder;
pub use error::{WkError, WkResult};
pub use format::header::{ColorType, CompressionMode, WkHeader};
pub use format::{Chunk, ChunkType};
pub use metadata::{CustomMetadata, ExifData, ExifTag, IccProfile, WkMetadata, XmpData};

pub const VERSION: &str = "2.0.0";
pub const MAGIC: &[u8; 8] = b"WK2.0\x00\x00\x00";

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage, RgbaImage};

    #[test]
    fn test_lossless_roundtrip() {
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(32, 32, |x, y| {
            image::Rgb([(x * 8) as u8, (y * 8) as u8, 128])
        }));

        let encoder = WkEncoder::lossless();
        let encoded = encoder.encode_to_vec(&img).unwrap();

        let decoder = WkDecoder::new();
        let decoded = decoder.decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.image.width(), 32);
        assert_eq!(decoded.image.height(), 32);
    }

    #[test]
    fn test_lossy_roundtrip() {
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(64, 64, |x, y| {
            image::Rgb([(x * 4) as u8, (y * 4) as u8, 200])
        }));

        let encoder = WkEncoder::lossy(85);
        let encoded = encoder.encode_to_vec(&img).unwrap();

        let decoder = WkDecoder::new();
        let decoded = decoder.decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.image.width(), 64);
        assert_eq!(decoded.image.height(), 64);
    }

    #[test]
    fn test_rgba_support() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(16, 16, |x, y| {
            image::Rgba([(x * 16) as u8, (y * 16) as u8, 100, ((x + y) * 8) as u8])
        }));

        let encoder = WkEncoder::lossless();
        let encoded = encoder.encode_to_vec(&img).unwrap();

        let decoder = WkDecoder::new();
        let decoded = decoder.decode(encoded.as_slice()).unwrap();

        assert!(decoded.header.has_alpha);
    }

    #[test]
    fn test_metadata() {
        let mut metadata = WkMetadata::new();
        metadata.custom.author = Some("Test Author".into());
        metadata.custom.set("key", "value");

        let img =
            DynamicImage::ImageRgb8(RgbImage::from_fn(8, 8, |_, _| image::Rgb([100, 100, 100])));

        let encoder = WkEncoder::lossy(90).with_metadata(metadata);
        let encoded = encoder.encode_to_vec(&img).unwrap();

        let decoder = WkDecoder::new();
        let decoded = decoder.decode(encoded.as_slice()).unwrap();

        assert_eq!(
            decoded.metadata.custom.author.as_deref(),
            Some("Test Author")
        );
    }

    #[test]
    fn test_exif_metadata() {
        use metadata::exif::ExifBuilder;

        let exif = ExifBuilder::new()
            .make("Canon")
            .model("EOS R5")
            .iso(800)
            .aperture(2.8)
            .build();

        let metadata = WkMetadata::new().with_exif(exif);

        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(8, 8, |_, _| image::Rgb([50, 50, 50])));
        let encoder = WkEncoder::lossy(85).with_metadata(metadata);
        let encoded = encoder.encode_to_vec(&img).unwrap();

        let decoder = WkDecoder::new();
        let decoded = decoder.decode(encoded.as_slice()).unwrap();

        let exif = decoded.metadata.exif.unwrap();
        assert_eq!(exif.camera_make(), Some("Canon"));
        assert_eq!(exif.camera_model(), Some("EOS R5"));
        assert_eq!(exif.iso(), Some(800));
    }

    #[test]
    fn test_compression_ratio() {
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(128, 128, |_, _| {
            image::Rgb([100, 100, 100])
        }));

        let lossless_enc = WkEncoder::lossless().encode_to_vec(&img).unwrap();
        let lossy_enc = WkEncoder::lossy(50).encode_to_vec(&img).unwrap();
        let raw_size = 128 * 128 * 3;

        assert!(lossless_enc.len() < raw_size);
        assert!(lossy_enc.len() < lossless_enc.len());
    }
}
