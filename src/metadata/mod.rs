pub mod custom;
pub mod exif;
pub mod icc;
pub mod xmp;

pub use custom::CustomMetadata;
pub use exif::{ExifData, ExifTag};
pub use icc::IccProfile;
pub use xmp::XmpData;

use crate::error::WkResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WkMetadata {
    pub exif: Option<ExifData>,
    pub icc_profile: Option<IccProfile>,
    pub xmp: Option<XmpData>,
    pub custom: CustomMetadata,
}

impl WkMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_exif(mut self, exif: ExifData) -> Self {
        self.exif = Some(exif);
        self
    }

    pub fn with_icc(mut self, icc: IccProfile) -> Self {
        self.icc_profile = Some(icc);
        self
    }

    pub fn with_xmp(mut self, xmp: XmpData) -> Self {
        self.xmp = Some(xmp);
        self
    }

    pub fn encode(&self) -> WkResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| crate::error::WkError::MetadataError(e.to_string()))
    }

    pub fn decode(data: &[u8]) -> WkResult<Self> {
        bincode::deserialize(data).map_err(|e| crate::error::WkError::MetadataError(e.to_string()))
    }
}
