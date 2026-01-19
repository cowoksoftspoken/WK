pub const JPEG_LUMINANCE_QUANT: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

pub const JPEG_CHROMINANCE_QUANT: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

#[derive(Debug, Clone)]
pub struct QuantizationTable {
    pub table: [u16; 64],
}

impl QuantizationTable {
    pub fn for_quality(quality: u8, is_chroma: bool) -> Self {
        let base = if is_chroma {
            JPEG_CHROMINANCE_QUANT
        } else {
            JPEG_LUMINANCE_QUANT
        };

        let quality = quality.clamp(1, 100) as u32;
        let scale = if quality < 50 {
            5000 / quality
        } else {
            200 - quality * 2
        };

        let mut table = [0u16; 64];
        for i in 0..64 {
            let val = (base[i] as u32 * scale + 50) / 100;
            table[i] = val.clamp(1, 255) as u16;
        }

        Self { table }
    }

    pub fn lossless() -> Self {
        Self { table: [1u16; 64] }
    }
}

pub struct Quantizer {
    luma_table: QuantizationTable,
    chroma_table: QuantizationTable,
}

impl Quantizer {
    pub fn new(quality: u8) -> Self {
        Self {
            luma_table: QuantizationTable::for_quality(quality, false),
            chroma_table: QuantizationTable::for_quality(quality, true),
        }
    }

    pub fn lossless() -> Self {
        Self {
            luma_table: QuantizationTable::lossless(),
            chroma_table: QuantizationTable::lossless(),
        }
    }

    pub fn quantize(&self, block: &[i16; 64], is_chroma: bool) -> [i16; 64] {
        let table = if is_chroma {
            &self.chroma_table
        } else {
            &self.luma_table
        };
        let mut output = [0i16; 64];
        for i in 0..64 {
            output[i] = (block[i] as i32 / table.table[i] as i32) as i16;
        }
        output
    }

    pub fn dequantize(&self, block: &[i16; 64], is_chroma: bool) -> [i16; 64] {
        let table = if is_chroma {
            &self.chroma_table
        } else {
            &self.luma_table
        };
        let mut output = [0i16; 64];
        for i in 0..64 {
            output[i] = (block[i] as i32 * table.table[i] as i32) as i16;
        }
        output
    }

    pub fn luma_table(&self) -> &QuantizationTable {
        &self.luma_table
    }

    pub fn chroma_table(&self) -> &QuantizationTable {
        &self.chroma_table
    }
}

pub fn adaptive_block_quantize(
    block: &[i16; 64],
    base_table: &QuantizationTable,
    activity_factor: f32,
) -> [i16; 64] {
    let mut output = [0i16; 64];
    let scale = 1.0 + (activity_factor - 0.5) * 0.5;

    for i in 0..64 {
        let q = (base_table.table[i] as f32 * scale) as i32;
        let q = q.max(1);
        output[i] = (block[i] as i32 / q) as i16;
    }

    output
}

pub fn calculate_block_activity(block: &[i16; 64]) -> f32 {
    let mut activity = 0i64;
    for i in 1..64 {
        activity += (block[i] as i64).abs();
    }
    let normalized = (activity as f64 / 63.0 / 128.0).clamp(0.0, 1.0);
    normalized as f32
}
