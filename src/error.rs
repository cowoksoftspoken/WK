use thiserror::Error;

#[derive(Error, Debug)]
pub enum WkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    #[error("CRC mismatch: expected {expected:#010x}, got {actual:#010x}")]
    CrcMismatch { expected: u32, actual: u32 },
    #[error("Invalid chunk: {0}")]
    InvalidChunk(String),
    #[error("Missing required chunk: {0}")]
    MissingChunk(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Metadata error: {0}")]
    MetadataError(String),
    #[error("Image processing error: {0}")]
    ImageError(String),
}

impl From<image::ImageError> for WkError {
    fn from(e: image::ImageError) -> Self {
        WkError::ImageError(e.to_string())
    }
}

pub type WkResult<T> = Result<T, WkError>;
