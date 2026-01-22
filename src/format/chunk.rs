use crate::error::{WkError, WkResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const WK_MAGIC: &[u8; 8] = b"WK3.0\x00\x00\x00";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChunkType {
    ImageHeader = 0x01,
    IccProfile = 0x02,
    Exif = 0x03,
    Xmp = 0x04,
    Thumbnail = 0x05,
    Animation = 0x06,
    ImageData = 0x10,
    ImageDataLossy = 0x11,
    FrameData = 0x12,
    Custom = 0xFE,
    End = 0xFF,
}

impl ChunkType {
    pub fn from_u8(v: u8) -> WkResult<Self> {
        match v {
            0x01 => Ok(Self::ImageHeader),
            0x02 => Ok(Self::IccProfile),
            0x03 => Ok(Self::Exif),
            0x04 => Ok(Self::Xmp),
            0x05 => Ok(Self::Thumbnail),
            0x06 => Ok(Self::Animation),
            0x10 => Ok(Self::ImageData),
            0x11 => Ok(Self::ImageDataLossy),
            0x12 => Ok(Self::FrameData),
            0xFE => Ok(Self::Custom),
            0xFF => Ok(Self::End),
            _ => Err(WkError::InvalidChunk(format!(
                "Unknown chunk type: {:#04x}",
                v
            ))),
        }
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            Self::ImageHeader => *b"IHDR",
            Self::IccProfile => *b"ICCP",
            Self::Exif => *b"EXIF",
            Self::Xmp => *b"XMP\x00",
            Self::Thumbnail => *b"THUM",
            Self::Animation => *b"ANIM",
            Self::ImageData => *b"IDAT",
            Self::ImageDataLossy => *b"IDLS",
            Self::FrameData => *b"FRMD",
            Self::Custom => *b"CUST",
            Self::End => *b"IEND",
        }
    }

    pub fn from_bytes(bytes: &[u8; 4]) -> WkResult<Self> {
        match bytes {
            b"IHDR" => Ok(Self::ImageHeader),
            b"ICCP" => Ok(Self::IccProfile),
            b"EXIF" => Ok(Self::Exif),
            b"XMP\x00" => Ok(Self::Xmp),
            b"THUM" => Ok(Self::Thumbnail),
            b"ANIM" => Ok(Self::Animation),
            b"IDAT" => Ok(Self::ImageData),
            b"IDLS" => Ok(Self::ImageDataLossy),
            b"FRMD" => Ok(Self::FrameData),
            b"CUST" => Ok(Self::Custom),
            b"IEND" => Ok(Self::End),
            _ => Err(WkError::InvalidChunk(format!(
                "Unknown chunk: {:?}",
                String::from_utf8_lossy(bytes)
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk_type: ChunkType,
    pub data: Vec<u8>,
    pub crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = Self::compute_crc(&chunk_type, &data);
        Self {
            chunk_type,
            data,
            crc,
        }
    }

    fn compute_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&chunk_type.as_bytes());
        hasher.update(data);
        hasher.finalize()
    }

    pub fn verify_crc(&self) -> bool {
        let computed = Self::compute_crc(&self.chunk_type, &self.data);
        computed == self.crc
    }
}

pub struct ChunkReader<R: Read> {
    reader: R,
    magic_verified: bool,
}

impl<R: Read> ChunkReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            magic_verified: false,
        }
    }

    pub fn verify_magic(&mut self) -> WkResult<()> {
        let mut magic = [0u8; 8];
        self.reader.read_exact(&mut magic)?;
        if &magic != WK_MAGIC {
            return Err(WkError::InvalidFormat(
                "Invalid magic number. Not a WK v3.0 file.".into(),
            ));
        }
        self.magic_verified = true;
        Ok(())
    }

    pub fn read_chunk(&mut self) -> WkResult<Chunk> {
        if !self.magic_verified {
            self.verify_magic()?;
        }

        let mut type_bytes = [0u8; 4];
        self.reader.read_exact(&mut type_bytes)?;
        let chunk_type = ChunkType::from_bytes(&type_bytes)?;

        let size = self.reader.read_u32::<LittleEndian>()? as usize;

        let mut data = vec![0u8; size];
        if size > 0 {
            self.reader.read_exact(&mut data)?;
        }

        let crc = self.reader.read_u32::<LittleEndian>()?;

        let chunk = Chunk {
            chunk_type,
            data,
            crc,
        };

        if !chunk.verify_crc() {
            return Err(WkError::CrcMismatch {
                expected: chunk.crc,
                actual: Chunk::compute_crc(&chunk.chunk_type, &chunk.data),
            });
        }

        Ok(chunk)
    }

    pub fn read_all_chunks(&mut self) -> WkResult<Vec<Chunk>> {
        let mut chunks = Vec::new();
        loop {
            let chunk = self.read_chunk()?;
            let is_end = matches!(chunk.chunk_type, ChunkType::End);
            chunks.push(chunk);
            if is_end {
                break;
            }
        }
        Ok(chunks)
    }
}

pub struct ChunkWriter<W: Write> {
    writer: W,
    magic_written: bool,
}

impl<W: Write> ChunkWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            magic_written: false,
        }
    }

    pub fn write_magic(&mut self) -> WkResult<()> {
        self.writer.write_all(WK_MAGIC)?;
        self.magic_written = true;
        Ok(())
    }

    pub fn write_chunk(&mut self, chunk: &Chunk) -> WkResult<()> {
        if !self.magic_written {
            self.write_magic()?;
        }

        self.writer.write_all(&chunk.chunk_type.as_bytes())?;
        self.writer
            .write_u32::<LittleEndian>(chunk.data.len() as u32)?;
        self.writer.write_all(&chunk.data)?;
        self.writer.write_u32::<LittleEndian>(chunk.crc)?;

        Ok(())
    }

    pub fn write_end(&mut self) -> WkResult<()> {
        let end_chunk = Chunk::new(ChunkType::End, Vec::new());
        self.write_chunk(&end_chunk)
    }

    pub fn finish(mut self) -> WkResult<W> {
        self.write_end()?;
        Ok(self.writer)
    }
}
