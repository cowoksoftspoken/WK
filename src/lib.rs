// src/lib.rs

//! WK Image Format Library
//!
//! Format gambar custom dengan fitur:
//! - RLE compression untuk efisiensi
//! - Metadata support (author, description, custom fields)
//! - Multiple color types (RGB, RGBA, Grayscale, GrayscaleAlpha)
//! - Konversi dari/ke PNG, JPEG, WebP, HEIC

pub mod compression; // Made public for testing
pub mod converter;
pub mod decoder;
pub mod encoder;
pub mod error;
pub mod header;
pub mod metadata;

pub use converter::WkConverter;
pub use decoder::WkDecoder;
pub use encoder::WkEncoder;
pub use error::{WkError, WkResult};
pub use header::{ColorType, WkHeader};
pub use metadata::WkMetadata;

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};

    #[test]
    fn test_header_write_read() {
        let header = WkHeader::new(100, 100, ColorType::RGB);
        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let read_header = WkHeader::read(&mut buffer.as_slice()).unwrap();
        assert_eq!(read_header.width, 100);
        assert_eq!(read_header.height, 100);
    }

    #[test]
    fn test_compression() {
        use crate::compression::{rle_compress, rle_decompress};

        let data = vec![1, 1, 1, 1, 2, 3, 4, 4, 4];
        let compressed = rle_compress(&data).unwrap();
        let decompressed = rle_decompress(&compressed, data.len()).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_encode_decode_rgb() {
        // Buat image test sederhana
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(10, 10, |x, y| {
            image::Rgb([(x * 25) as u8, (y * 25) as u8, 128])
        }));

        // Encode
        let mut buffer = Vec::new();
        let encoder = WkEncoder::new();
        let result = encoder.encode(&img, &mut buffer);

        if let Err(e) = &result {
            eprintln!("Encode error: {:?}", e);
        }
        assert!(result.is_ok(), "Encoding failed");

        // Decode
        let decoder = WkDecoder::new();
        let decode_result = decoder.decode(&mut buffer.as_slice());

        if let Err(e) = &decode_result {
            eprintln!("Decode error: {:?}", e);
            eprintln!("Buffer size: {}", buffer.len());
        }

        let (decoded_img, _metadata) = decode_result.unwrap();

        // Verifikasi dimensi
        assert_eq!(decoded_img.width(), 10);
        assert_eq!(decoded_img.height(), 10);
    }

    #[test]
    fn test_metadata() {
        let mut metadata = WkMetadata::new();
        metadata.author = Some("Test Author".to_string());
        metadata.add_custom_field("test_key".to_string(), "test_value".to_string());

        let encoded = metadata.encode().unwrap();
        let decoded = WkMetadata::decode(&encoded).unwrap();

        assert_eq!(decoded.author, Some("Test Author".to_string()));
        assert_eq!(decoded.custom_fields.get("test_key").unwrap(), "test_value");
    }
}
