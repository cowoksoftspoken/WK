pub struct BitWriter {
    bytes: Vec<u8>,
    current: u8,
    bits: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current: 0,
            bits: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        self.current = (self.current << 1) | (bit as u8);
        self.bits += 1;
        if self.bits == 8 {
            self.bytes.push(self.current);
            self.current = 0;
            self.bits = 0;
        }
    }

    pub fn write_bits(&mut self, value: u32, count: u8) {
        for i in (0..count).rev() {
            self.write_bit((value >> i) & 1 != 0);
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        if self.bits > 0 {
            self.current <<= 8 - self.bits;
            self.bytes.push(self.current);
        }
        self.bytes
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BitReader {
    bytes: Vec<u8>,
    byte_pos: usize,
    bit_pos: u8,
}

impl BitReader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    pub fn read_bit(&mut self) -> bool {
        if self.byte_pos >= self.bytes.len() {
            return false;
        }
        let bit = (self.bytes[self.byte_pos] >> (7 - self.bit_pos)) & 1 != 0;
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        bit
    }

    pub fn read_bits(&mut self, count: u8) -> u32 {
        let mut value = 0u32;
        for _ in 0..count {
            value = (value << 1) | (self.read_bit() as u32);
        }
        value
    }
}

#[derive(Clone)]
pub struct ProbabilityModel;
impl ProbabilityModel {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ProbabilityModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ArithmeticEncoder {
    writer: BitWriter,
}

impl ArithmeticEncoder {
    pub fn new() -> Self {
        Self {
            writer: BitWriter::new(),
        }
    }

    pub fn encode_bypass(&mut self, bit: bool) {
        self.writer.write_bit(bit);
    }

    pub fn finish(self) -> Vec<u8> {
        self.writer.finish()
    }
}

impl Default for ArithmeticEncoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ArithmeticDecoder {
    reader: BitReader,
}

impl ArithmeticDecoder {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            reader: BitReader::new(data),
        }
    }

    pub fn decode_bypass(&mut self) -> bool {
        self.reader.read_bit()
    }
}

pub struct CABACContext;
impl CABACContext {
    pub fn new(_block_size: usize) -> Self {
        Self
    }
    pub fn reset(&mut self) {}
}
impl Default for CABACContext {
    fn default() -> Self {
        Self::new(8)
    }
}

pub fn encode_block(coeffs: &[i16]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    let n = coeffs.len().min(64);

    let mut last_nz = 0;
    for (i, &c) in coeffs.iter().enumerate().take(n) {
        if c != 0 {
            last_nz = i;
        }
    }

    writer.write_bits(last_nz as u32, 6);

    for &c in coeffs.iter().take(last_nz + 1) {
        let sig = c != 0;
        writer.write_bit(sig);

        if sig {
            let abs_val = c.unsigned_abs();
            let sign = c < 0;

            if abs_val <= 15 {
                writer.write_bit(false);
                writer.write_bits(abs_val as u32 - 1, 4);
            } else {
                writer.write_bit(true);
                writer.write_bits(abs_val as u32 - 1, 16);
            }

            writer.write_bit(sign);
        }
    }

    writer.finish()
}

pub fn decode_block(data: &[u8], size: usize) -> Vec<i16> {
    let mut coeffs = vec![0i16; size];
    let mut reader = BitReader::new(data.to_vec());
    let n = size.min(64);

    let last_nz = reader.read_bits(6) as usize;
    if last_nz >= n {
        return coeffs;
    }

    for i in 0..=last_nz {
        let sig = reader.read_bit();
        if sig {
            let is_large = reader.read_bit();
            let abs_val = if !is_large {
                reader.read_bits(4) + 1
            } else {
                reader.read_bits(16) + 1
            };
            let sign = reader.read_bit();
            coeffs[i] = if sign {
                -(abs_val as i16)
            } else {
                abs_val as i16
            };
        }
    }

    coeffs
}

pub fn encode_coefficients(
    encoder: &mut ArithmeticEncoder,
    _ctx: &mut CABACContext,
    coeffs: &[i16],
) {
    let block_data = encode_block(coeffs);
    encoder.writer.write_bits(block_data.len() as u32, 16);
    for &b in &block_data {
        encoder.writer.write_bits(b as u32, 8);
    }
}

pub fn decode_coefficients(
    decoder: &mut ArithmeticDecoder,
    _ctx: &mut CABACContext,
    size: usize,
) -> Vec<i16> {
    let block_len = decoder.reader.read_bits(16) as usize;
    let mut block_data = Vec::with_capacity(block_len);
    for _ in 0..block_len {
        block_data.push(decoder.reader.read_bits(8) as u8);
    }
    decode_block(&block_data, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_roundtrip() {
        let coeffs: Vec<i16> = vec![100, -50, 25, -12, 6, 0, 0, 0];
        let encoded = encode_block(&coeffs);
        let decoded = decode_block(&encoded, 8);
        assert_eq!(&coeffs[..], &decoded[..8]);
    }

    #[test]
    fn test_multi_block() {
        let blocks: Vec<Vec<i16>> = vec![
            vec![100, -50, 25, 0, 0, 0, 0, 0],
            vec![80, -40, 20, -10, 0, 0, 0, 0],
            vec![60, -30, 15, -8, 4, 0, 0, 0],
        ];

        let mut encoder = ArithmeticEncoder::new();
        let mut ctx = CABACContext::new(8);
        for block in &blocks {
            encode_coefficients(&mut encoder, &mut ctx, block);
        }
        let encoded = encoder.finish();

        let mut decoder = ArithmeticDecoder::new(encoded);
        let mut dec_ctx = CABACContext::new(8);
        for orig in &blocks {
            let decoded = decode_coefficients(&mut decoder, &mut dec_ctx, 8);
            assert_eq!(&orig[..], &decoded[..8]);
        }
    }
}
