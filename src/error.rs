use thiserror::Error;

#[derive(Error, Debug)]
pub enum WkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid WK file: {0}")]
    InvalidFormat(String),

    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),
}

pub type WkResult<T> = Result<T, WkError>;
