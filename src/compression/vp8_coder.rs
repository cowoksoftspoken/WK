pub struct RangeEncoder {
    low: u64,
    range: u64,
    buffer: Vec<u8>,
    pending: u8,
    pending_count: u32,
}

impl RangeEncoder {
    pub fn new() -> Self {
        Self {
            low: 0,
            range: 0xFFFFFFFF,
            buffer: Vec::with_capacity(64),
            pending: 0,
            pending_count: 0,
        }
    }

    fn emit(&mut self, byte: u8) {
        if self.pending_count == 0 {
            self.pending = byte;
            self.pending_count = 1;
        } else if byte == 0xFF {
            self.pending_count += 1;
        } else {
            self.buffer.push(self.pending);
            for _ in 1..self.pending_count {
                self.buffer.push(0xFF);
            }
            self.pending = byte;
            self.pending_count = 1;
        }
    }

    fn emit_with_carry(&mut self, byte: u8, carry: bool) {
        if carry {
            self.pending += 1;
            for _ in 1..self.pending_count {
                self.buffer.push(self.pending);
                self.pending = 0;
            }
            self.pending_count = 1;
        }
        self.emit(byte);
    }

    pub fn encode(&mut self, bit: bool, prob: u32) {
        let prob = prob.clamp(1, 255) as u64;
        let bound = (self.range * prob) >> 8;

        if bit {
            self.low += bound;
            self.range -= bound;
        } else {
            self.range = bound;
        }

        while self.range < 0x1000000 {
            let carry = (self.low >> 32) != 0;
            let byte = ((self.low >> 24) & 0xFF) as u8;
            self.emit_with_carry(byte, carry);
            self.low = (self.low << 8) & 0xFFFFFFFF;
            self.range <<= 8;
        }
    }

    pub fn encode_bit(&mut self, bit: bool) {
        self.encode(bit, 128);
    }

    pub fn encode_value(&mut self, value: u32, bits: u8) {
        for i in (0..bits).rev() {
            self.encode_bit(((value >> i) & 1) != 0);
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        for _ in 0..5 {
            let carry = (self.low >> 32) != 0;
            let byte = ((self.low >> 24) & 0xFF) as u8;
            self.emit_with_carry(byte, carry);
            self.low = (self.low << 8) & 0xFFFFFFFF;
        }

        self.buffer.push(self.pending);
        for _ in 1..self.pending_count {
            self.buffer.push(0xFF);
        }

        while self.buffer.len() > 1 && self.buffer.last() == Some(&0) {
            self.buffer.pop();
        }

        self.buffer
    }
}

impl Default for RangeEncoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RangeDecoder {
    data: Vec<u8>,
    pos: usize,
    range: u64,
    code: u64,
}

impl RangeDecoder {
    pub fn new(data: Vec<u8>) -> Self {
        let mut d = Self {
            data,
            pos: 0,
            range: 0xFFFFFFFF,
            code: 0,
        };
        for _ in 0..4 {
            d.code = (d.code << 8) | d.read_byte() as u64;
        }
        d
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

    pub fn decode(&mut self, prob: u32) -> bool {
        let prob = prob.clamp(1, 255) as u64;
        let bound = (self.range * prob) >> 8;

        let bit = self.code >= bound;

        if bit {
            self.code -= bound;
            self.range -= bound;
        } else {
            self.range = bound;
        }

        while self.range < 0x1000000 {
            self.code = (self.code << 8) | self.read_byte() as u64;
            self.range <<= 8;
        }

        bit
    }

    pub fn decode_bit(&mut self) -> bool {
        self.decode(128)
    }

    pub fn decode_value(&mut self, bits: u8) -> u32 {
        let mut v = 0u32;
        for _ in 0..bits {
            v = (v << 1) | (self.decode_bit() as u32);
        }
        v
    }
}

pub type VP8BoolWriter = RangeEncoder;
pub type VP8BoolReader = RangeDecoder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_true() {
        let mut enc = RangeEncoder::new();
        enc.encode_bit(true);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert!(dec.decode_bit());
    }

    #[test]
    fn test_single_false() {
        let mut enc = RangeEncoder::new();
        enc.encode_bit(false);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert!(!dec.decode_bit());
    }

    #[test]
    fn test_alternating() {
        let mut enc = RangeEncoder::new();
        enc.encode_bit(true);
        enc.encode_bit(false);
        enc.encode_bit(true);
        enc.encode_bit(false);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert!(dec.decode_bit());
        assert!(!dec.decode_bit());
        assert!(dec.decode_bit());
        assert!(!dec.decode_bit());
    }

    #[test]
    fn test_8bit_value() {
        let mut enc = RangeEncoder::new();
        enc.encode_value(0xAB, 8);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert_eq!(dec.decode_value(8), 0xAB);
    }

    #[test]
    fn test_16bit_value() {
        let mut enc = RangeEncoder::new();
        enc.encode_value(0x1234, 16);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert_eq!(dec.decode_value(16), 0x1234);
    }

    #[test]
    fn test_many_bits() {
        let mut enc = RangeEncoder::new();
        for i in 0..64 {
            enc.encode_bit(i % 2 == 0);
        }
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        for i in 0..64 {
            assert_eq!(dec.decode_bit(), i % 2 == 0, "Mismatch at bit {}", i);
        }
    }

    #[test]
    fn test_various_probs() {
        let mut enc = RangeEncoder::new();
        enc.encode(true, 200);
        enc.encode(false, 50);
        enc.encode(true, 10);
        enc.encode(false, 240);
        let data = enc.finish();

        let mut dec = RangeDecoder::new(data);
        assert!(dec.decode(200));
        assert!(!dec.decode(50));
        assert!(dec.decode(10));
        assert!(!dec.decode(240));
    }
}
