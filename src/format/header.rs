use crate::error::{WkError, WkResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorType {
    Grayscale = 0,
    GrayscaleAlpha = 1,
    Rgb = 2,
    Rgba = 3,
    Yuv420 = 4,
    Yuv444 = 5,
}

impl ColorType {
    pub fn from_u8(v: u8) -> WkResult<Self> {
        match v {
            0 => Ok(Self::Grayscale),
            1 => Ok(Self::GrayscaleAlpha),
            2 => Ok(Self::Rgb),
            3 => Ok(Self::Rgba),
            4 => Ok(Self::Yuv420),
            5 => Ok(Self::Yuv444),
            _ => Err(WkError::InvalidFormat(format!("Unknown color type: {}", v))),
        }
    }

    pub fn channels(&self) -> u8 {
        match self {
            Self::Grayscale => 1,
            Self::GrayscaleAlpha => 2,
            Self::Rgb | Self::Yuv420 | Self::Yuv444 => 3,
            Self::Rgba => 4,
        }
    }

    pub fn has_alpha(&self) -> bool {
        matches!(self, Self::GrayscaleAlpha | Self::Rgba)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMode {
    Lossless = 0,
    Lossy = 1,
    Mixed = 2,
}

impl CompressionMode {
    pub fn from_u8(v: u8) -> WkResult<Self> {
        match v {
            0 => Ok(Self::Lossless),
            1 => Ok(Self::Lossy),
            2 => Ok(Self::Mixed),
            _ => Err(WkError::InvalidFormat(format!(
                "Unknown compression mode: {}",
                v
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WkHeader {
    pub width: u32,
    pub height: u32,
    pub color_type: ColorType,
    pub compression_mode: CompressionMode,
    pub quality: u8,
    pub has_alpha: bool,
    pub has_animation: bool,
    pub bit_depth: u8,
}

impl WkHeader {
    pub fn new(width: u32, height: u32, color_type: ColorType) -> Self {
        Self {
            width,
            height,
            color_type,
            compression_mode: CompressionMode::Lossy,
            quality: 85,
            has_alpha: color_type.has_alpha(),
            has_animation: false,
            bit_depth: 8,
        }
    }

    pub fn lossless(width: u32, height: u32, color_type: ColorType) -> Self {
        Self {
            width,
            height,
            color_type,
            compression_mode: CompressionMode::Lossless,
            quality: 100,
            has_alpha: color_type.has_alpha(),
            has_animation: false,
            bit_depth: 8,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16);
        buf.write_u32::<LittleEndian>(self.width).unwrap();
        buf.write_u32::<LittleEndian>(self.height).unwrap();
        buf.write_u8(self.color_type as u8).unwrap();
        buf.write_u8(self.compression_mode as u8).unwrap();
        buf.write_u8(self.quality).unwrap();

        let flags = (self.has_alpha as u8) | ((self.has_animation as u8) << 1);
        buf.write_u8(flags).unwrap();
        buf.write_u8(self.bit_depth).unwrap();
        buf.write_u8(0).unwrap(); // reserved
        buf.write_u16::<LittleEndian>(0).unwrap(); // reserved
        buf
    }

    pub fn decode(data: &[u8]) -> WkResult<Self> {
        if data.len() < 16 {
            return Err(WkError::InvalidFormat("Header too short".into()));
        }
        let mut cursor = std::io::Cursor::new(data);

        let width = cursor.read_u32::<LittleEndian>()?;
        let height = cursor.read_u32::<LittleEndian>()?;
        let color_type = ColorType::from_u8(cursor.read_u8()?)?;
        let compression_mode = CompressionMode::from_u8(cursor.read_u8()?)?;
        let quality = cursor.read_u8()?;
        let flags = cursor.read_u8()?;
        let bit_depth = cursor.read_u8()?;

        Ok(Self {
            width,
            height,
            color_type,
            compression_mode,
            quality,
            has_alpha: (flags & 0x01) != 0,
            has_animation: (flags & 0x02) != 0,
            bit_depth,
        })
    }

    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn raw_size(&self) -> usize {
        self.pixel_count() * self.color_type.channels() as usize
    }
}
