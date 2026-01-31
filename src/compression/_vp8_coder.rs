use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub struct VP8BoolEncoder {
    low: u32,
    range: u32,
    count: i32,
    buffer: Vec<u8>,
}

impl VP8BoolEncoder {
    pub fn new() -> Self {
        Self {
            low: 0,
            range: 255,
            count: -24,
            buffer: Vec::with_capacity(1024),
        }
    }

    pub fn encode_bool(&mut self, bit: bool, prob: u8) {
        let prob = prob.max(1) as u32;
        let split = 1 + (((self.range - 1) * prob) >> 8);

        if bit {
            self.low += split;
            self.range -= split;
        } else {
            self.range = split;
        }

        while self.range < 128 {
            self.range <<= 1;

            if self.low & 0x80000000 != 0 {
                let mut i = self.buffer.len();
                while i > 0 {
                    i -= 1;
                    if self.buffer[i] == 0xFF {
                        self.buffer[i] = 0;
                    } else {
                        self.buffer[i] += 1;
                        break;
                    }
                }
            }

            self.low <<= 1;
            self.count += 1;

            if self.count >= 0 {
                self.buffer.push((self.low >> 24) as u8);
                self.low &= 0x00FFFFFF;
                self.count -= 8;
            }
        }
    }

    pub fn encode_value(&mut self, value: i32, bits: u8) {
        for i in (0..bits).rev() {
            let bit = ((value >> i) & 1) != 0;
            self.encode_bool(bit, 128);
        }
    }

    pub fn encode_signed(&mut self, value: i16) {
        let abs_val = value.unsigned_abs() as u32;

        if abs_val == 0 {
            self.encode_bool(false, 200);
            return;
        }

        self.encode_bool(true, 200);

        let mut v = abs_val;
        let mut bits = 0u32;
        while v > 0 {
            bits += 1;
            v >>= 1;
        }

        for i in 0..bits {
            self.encode_bool(true, 180);
            if i < bits - 1 {
                continue;
            }
            self.encode_bool(false, 180);
        }

        if bits > 1 {
            for i in (0..bits - 1).rev() {
                let bit = ((abs_val >> i) & 1) != 0;
                self.encode_bool(bit, 128);
            }
        }

        self.encode_bool(value < 0, 128);
    }

    pub fn finish(mut self) -> Vec<u8> {
        for _ in 0..4 {
            self.count += 8;
            if self.count >= 0 {
                self.buffer.push((self.low >> 24) as u8);
                self.low <<= 8;
            }
        }
        self.buffer
    }
}

impl Default for VP8BoolEncoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VP8BoolDecoder {
    data: Vec<u8>,
    pos: usize,
    value: u32,
    range: u32,
    bits: i32,
}

impl VP8BoolDecoder {
    pub fn new(data: Vec<u8>) -> Self {
        let mut decoder = Self {
            data,
            pos: 0,
            value: 0,
            range: 255,
            bits: 0,
        };
        decoder.init();
        decoder
    }

    fn init(&mut self) {
        for _ in 0..4 {
            self.value = (self.value << 8) | self.read_byte() as u32;
        }
        self.bits = 32;
    }

    fn read_byte(&mut self) -> u8 {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            b
        } else {
            0
        }
    }

    pub fn decode_bool(&mut self, prob: u8) -> bool {
        let prob = prob.max(1) as u32;
        let split = 1 + (((self.range - 1) * prob) >> 8);
        let big_split = split << 24;

        let bit = self.value >= big_split;

        if bit {
            self.value -= big_split;
            self.range -= split;
        } else {
            self.range = split;
        }

        while self.range < 128 {
            self.range <<= 1;
            self.value <<= 1;
            self.bits -= 1;

            if self.bits <= 24 {
                self.value |= (self.read_byte() as u32) << (24 - self.bits);
                self.bits += 8;
            }
        }

        bit
    }

    pub fn decode_value(&mut self, bits: u8) -> i32 {
        let mut value = 0i32;
        for _ in 0..bits {
            value = (value << 1) | (self.decode_bool(128) as i32);
        }
        value
    }

    pub fn decode_signed(&mut self) -> i16 {
        if !self.decode_bool(200) {
            return 0;
        }

        let mut bits = 1u32;
        while self.decode_bool(180) {
            bits += 1;
            if bits > 16 {
                return 0;
            }
        }

        let mut abs_val = 1u32 << (bits - 1);
        if bits > 1 {
            for i in (0..bits - 1).rev() {
                if self.decode_bool(128) {
                    abs_val |= 1 << i;
                }
            }
        }

        let negative = self.decode_bool(128);

        if negative {
            -(abs_val as i16)
        } else {
            abs_val as i16
        }
    }
}

pub fn vp8_encode_block(coeffs: &[i16]) -> Vec<u8> {
    let mut encoder = VP8BoolEncoder::new();
    let n = coeffs.len().min(64);
    let mut last_nz = 0;
    for (i, &c) in coeffs.iter().enumerate().take(n) {
        if c != 0 {
            last_nz = i;
        }
    }

    let all_zeros = coeffs.iter().take(n).all(|&c| c == 0);
    encoder.encode_bool(all_zeros, 128);

    if all_zeros {
        return encoder.finish();
    }

    encoder.encode_value((last_nz + 1) as i32, 6);

    for i in 0..=last_nz {
        encoder.encode_signed(coeffs[i]);
    }

    encoder.finish()
}

pub fn vp8_decode_block(data: &[u8], size: usize) -> Vec<i16> {
    let mut coeffs = vec![0i16; size];

    if data.is_empty() {
        return coeffs;
    }

    let mut decoder = VP8BoolDecoder::new(data.to_vec());

    if decoder.decode_bool(128) {
        return coeffs;
    }
    let count = decoder.decode_value(6) as usize;
    let last_nz = count.saturating_sub(1).min(size - 1);

    for i in 0..=last_nz {
        if i < size {
            coeffs[i] = decoder.decode_signed();
        }
    }

    coeffs
}

pub fn vp8_compress_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    let _ = encoder.write_all(data);
    encoder.finish().unwrap_or_default()
}

pub fn vp8_decompress_data(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    let _ = decoder.read_to_end(&mut result);
    result
}

pub fn vp8_encode_all_coeffs(blocks: &[Vec<i16>]) -> Vec<u8> {
    let mut encoder = VP8BoolEncoder::new();
    encoder.encode_value(blocks.len() as i32, 16);
    for block in blocks {
        let n = block.len().min(64);
        let mut last_nz = 0;
        for (i, &c) in block.iter().enumerate().take(n) {
            if c != 0 {
                last_nz = i;
            }
        }
        let all_zeros = block.iter().take(n).all(|&c| c == 0);
        encoder.encode_bool(all_zeros, 128);
        if !all_zeros {
            encoder.encode_value((last_nz + 1) as i32, 6);
            for i in 0..=last_nz {
                encoder.encode_signed(block[i]);
            }
        }
    }
    encoder.finish()
}

pub fn vp8_decode_all_coeffs(data: &[u8], block_size: usize) -> Vec<Vec<i16>> {
    if data.is_empty() {
        return vec![];
    }
    let mut decoder = VP8BoolDecoder::new(data.to_vec());
    let block_count = decoder.decode_value(16) as usize;
    let mut blocks = Vec::with_capacity(block_count);
    for _ in 0..block_count {
        let mut coeffs = vec![0i16; block_size];
        if decoder.decode_bool(128) {
            blocks.push(coeffs);
            continue;
        }
        let count = decoder.decode_value(6) as usize;
        let last_nz = count.saturating_sub(1).min(block_size - 1);
        for i in 0..=last_nz {
            if i < block_size {
                coeffs[i] = decoder.decode_signed();
            }
        }
        blocks.push(coeffs);
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vp8_bool_roundtrip() {
        let mut encoder = VP8BoolEncoder::new();
        encoder.encode_bool(true, 128);
        encoder.encode_bool(false, 128);
        encoder.encode_bool(true, 200);
        encoder.encode_bool(false, 50);
        let data = encoder.finish();

        let mut decoder = VP8BoolDecoder::new(data);
        assert!(decoder.decode_bool(128));
        assert!(!decoder.decode_bool(128));
        assert!(decoder.decode_bool(200));
        assert!(!decoder.decode_bool(50));
    }

    #[test]
    fn test_vp8_block_roundtrip() {
        let coeffs: Vec<i16> = vec![
            100, -50, 25, -12, 6, -3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let encoded = vp8_encode_block(&coeffs);
        let decoded = vp8_decode_block(&encoded, 64);

        for i in 0..7 {
            assert_eq!(coeffs[i], decoded[i], "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_vp8_signed_values() {
        let test_values: Vec<i16> = vec![-1000, -100, -10, -1, 0, 1, 10, 100, 1000];

        for &val in &test_values {
            let mut encoder = VP8BoolEncoder::new();
            encoder.encode_signed(val);
            let data = encoder.finish();

            let mut decoder = VP8BoolDecoder::new(data);
            let decoded = decoder.decode_signed();

            assert_eq!(val, decoded, "Failed for value {}", val);
        }
    }
}
