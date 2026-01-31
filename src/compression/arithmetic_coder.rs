use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

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

    pub fn write_exp_golomb(&mut self, val: u32) {
        if val == 0 {
            self.write_bit(true);
        } else {
            let val1 = val + 1;
            let bits = 32 - val1.leading_zeros();
            for _ in 0..bits - 1 {
                self.write_bit(false);
            }
            self.write_bits(val1, bits as u8);
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

    pub fn read_exp_golomb(&mut self) -> u32 {
        let mut zeros = 0u32;
        while !self.read_bit() {
            zeros += 1;
            if zeros > 16 {
                return 0;
            }
        }
        if zeros == 0 {
            return 0;
        }
        let rest = self.read_bits(zeros as u8);
        ((1 << zeros) | rest) - 1
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
    pub writer: BitWriter,
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
    pub reader: BitReader,
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

    if coeffs.iter().take(n).all(|&c| c == 0) {
        writer.write_bits(0, 6);
        writer.write_bit(true);
        return writer.finish();
    }

    writer.write_bits((last_nz + 1) as u32, 6);
    writer.write_bit(false);

    let mut i = 0;
    while i <= last_nz {
        let c = coeffs[i];
        if c == 0 {
            let mut run = 0;
            while i + run <= last_nz && coeffs[i + run] == 0 {
                run += 1;
            }
            run = run.min(32);
            writer.write_bit(false);
            writer.write_exp_golomb(run as u32 - 1);
            i += run;
        } else {
            writer.write_bit(true);
            let abs_val = c.unsigned_abs() as u32;
            writer.write_exp_golomb(abs_val - 1);
            writer.write_bit(c < 0);
            i += 1;
        }
    }

    writer.finish()
}

pub fn decode_block(data: &[u8], size: usize) -> Vec<i16> {
    let mut coeffs = vec![0i16; size];
    let mut reader = BitReader::new(data.to_vec());
    let n = size.min(64);

    let count = reader.read_bits(6) as usize;
    let is_zero = reader.read_bit();

    if count == 0 && is_zero {
        return coeffs;
    }
    let last_nz = count.saturating_sub(1);

    let mut i = 0;
    while i <= last_nz && i < n {
        let is_nonzero = reader.read_bit();
        if !is_nonzero {
            let run = reader.read_exp_golomb() as usize + 1;
            i += run.min(last_nz + 1 - i);
        } else {
            let abs_val = reader.read_exp_golomb() + 1;
            let sign = reader.read_bit();
            coeffs[i] = if sign {
                -(abs_val as i16)
            } else {
                abs_val as i16
            };
            i += 1;
        }
    }

    coeffs
}

pub fn compress_coefficients(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    let _ = encoder.write_all(data);
    encoder.finish().unwrap_or_default()
}

pub fn decompress_coefficients(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    let _ = decoder.read_to_end(&mut result);
    result
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
    fn test_exp_golomb() {
        for val in [0, 1, 2, 3, 7, 15, 31, 100, 255] {
            let mut w = BitWriter::new();
            w.write_exp_golomb(val);
            let data = w.finish();
            let mut r = BitReader::new(data);
            assert_eq!(r.read_exp_golomb(), val, "Failed for {}", val);
        }
    }

    #[test]
    fn test_block_roundtrip() {
        let coeffs: Vec<i16> = vec![
            100, -50, 25, -12, 6, -3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let encoded = encode_block(&coeffs);
        let decoded = decode_block(&encoded, 64);
        for i in 0..7 {
            assert_eq!(coeffs[i], decoded[i], "Mismatch at {}", i);
        }
    }

    #[test]
    fn test_multi_block() {
        let blocks: Vec<Vec<i16>> = vec![
            vec![
                100, -50, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            vec![
                80, 0, -40, 20, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
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
            let decoded = decode_coefficients(&mut decoder, &mut dec_ctx, 64);
            for i in 0..8 {
                assert_eq!(orig[i], decoded[i], "Block mismatch at {}", i);
            }
        }
    }
}
