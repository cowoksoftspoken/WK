use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    SRGB,
    AdobeRGB,
    ProPhotoRGB,
    DisplayP3,
    Rec709,
    Rec2020,
    CMYK,
    Grayscale,
    Lab,
    Custom,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::SRGB
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

impl Default for RenderingIntent {
    fn default() -> Self {
        Self::Perceptual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IccProfile {
    pub color_space: ColorSpace,
    pub rendering_intent: RenderingIntent,
    pub profile_name: String,
    pub description: String,
    pub raw_data: Option<Vec<u8>>,
}

impl Default for IccProfile {
    fn default() -> Self {
        Self::srgb()
    }
}

impl IccProfile {
    pub fn srgb() -> Self {
        Self {
            color_space: ColorSpace::SRGB,
            rendering_intent: RenderingIntent::Perceptual,
            profile_name: "sRGB IEC61966-2.1".into(),
            description: "sRGB color space profile".into(),
            raw_data: None,
        }
    }

    pub fn adobe_rgb() -> Self {
        Self {
            color_space: ColorSpace::AdobeRGB,
            rendering_intent: RenderingIntent::RelativeColorimetric,
            profile_name: "Adobe RGB (1998)".into(),
            description: "Adobe RGB color space profile".into(),
            raw_data: None,
        }
    }

    pub fn display_p3() -> Self {
        Self {
            color_space: ColorSpace::DisplayP3,
            rendering_intent: RenderingIntent::Perceptual,
            profile_name: "Display P3".into(),
            description: "Apple Display P3 color space".into(),
            raw_data: None,
        }
    }

    pub fn prophoto_rgb() -> Self {
        Self {
            color_space: ColorSpace::ProPhotoRGB,
            rendering_intent: RenderingIntent::RelativeColorimetric,
            profile_name: "ProPhoto RGB".into(),
            description: "ProPhoto RGB color space for wide gamut".into(),
            raw_data: None,
        }
    }

    pub fn rec2020() -> Self {
        Self {
            color_space: ColorSpace::Rec2020,
            rendering_intent: RenderingIntent::Perceptual,
            profile_name: "ITU-R BT.2020".into(),
            description: "Rec. 2020 HDR color space".into(),
            raw_data: None,
        }
    }

    pub fn from_raw(data: Vec<u8>) -> Self {
        Self {
            color_space: ColorSpace::Custom,
            rendering_intent: RenderingIntent::Perceptual,
            profile_name: "Custom ICC Profile".into(),
            description: "Embedded ICC profile".into(),
            raw_data: Some(data),
        }
    }

    pub fn is_wide_gamut(&self) -> bool {
        matches!(
            self.color_space,
            ColorSpace::AdobeRGB
                | ColorSpace::ProPhotoRGB
                | ColorSpace::DisplayP3
                | ColorSpace::Rec2020
        )
    }

    pub fn is_hdr(&self) -> bool {
        matches!(self.color_space, ColorSpace::Rec2020)
    }
}
