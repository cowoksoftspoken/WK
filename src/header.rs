use crate::error::{WkError, WkResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const WK_MAGIC: [u8; 4] = [b'W', b'K', 0x01, 0x00];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    RGB = 0,
    RGBA = 1,
    Grayscale = 2,
    GrayscaleAlpha = 3,
}

impl ColorType {
    pub fn from_u8(value: u8) -> WkResult<Self> {
        match value {
            0 => Ok(ColorType::RGB),
            1 => Ok(ColorType::RGBA),
            2 => Ok(ColorType::Grayscale),
            3 => Ok(ColorType::GrayscaleAlpha),
            _ => Err(WkError::InvalidFormat(format!(
                "Unknown color type: {}",
                value
            ))),
        }
    }

    pub fn channels(&self) -> u8 {
        match self {
            ColorType::RGB => 3,
            ColorType::RGBA => 4,
            ColorType::Grayscale => 1,
            ColorType::GrayscaleAlpha => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WkHeader {
    pub width: u32,
    pub height: u32,
    pub color_type: ColorType,
    pub compression: u8,
    pub metadata_size: u32,
}

impl WkHeader {
    pub fn new(width: u32, height: u32, color_type: ColorType) -> Self {
        Self {
            width,
            height,
            color_type,
            compression: 1,
            metadata_size: 0,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> WkResult<()> {
        writer.write_all(&WK_MAGIC)?;
        writer.write_u32::<LittleEndian>(self.width)?;
        writer.write_u32::<LittleEndian>(self.height)?;
        writer.write_u8(self.color_type as u8)?;
        writer.write_u8(self.compression)?;
        writer.write_u32::<LittleEndian>(self.metadata_size)?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> WkResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != WK_MAGIC {
            return Err(WkError::InvalidFormat(
                "Invalid magic number. Not a WK file.".to_string(),
            ));
        }

        let width = reader.read_u32::<LittleEndian>()?;
        let height = reader.read_u32::<LittleEndian>()?;
        let color_type = ColorType::from_u8(reader.read_u8()?)?;
        let compression = reader.read_u8()?;
        let metadata_size = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            width,
            height,
            color_type,
            compression,
            metadata_size,
        })
    }

    pub fn size(&self) -> usize {
        4 + 4 + 4 + 1 + 1 + 4
    }
}
