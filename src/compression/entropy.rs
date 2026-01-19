use crate::error::{WkError, WkResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HuffmanNode {
    pub symbol: Option<u8>,
    pub freq: u32,
    pub left: Option<Box<HuffmanNode>>,
    pub right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    pub fn leaf(symbol: u8, freq: u32) -> Self {
        Self {
            symbol: Some(symbol),
            freq,
            left: None,
            right: None,
        }
    }

    pub fn internal(left: HuffmanNode, right: HuffmanNode) -> Self {
        let freq = left.freq + right.freq;
        Self {
            symbol: None,
            freq,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HuffmanCode {
    pub bits: u32,
    pub len: u8,
}

pub struct HuffmanTable {
    codes: HashMap<u8, HuffmanCode>,
    decode_tree: Option<HuffmanNode>,
}

impl HuffmanTable {
    pub fn build(freq: &[u32; 256]) -> Self {
        let mut nodes: Vec<HuffmanNode> = freq
            .iter()
            .enumerate()
            .filter(|(_, &f)| f > 0)
            .map(|(i, &f)| HuffmanNode::leaf(i as u8, f))
            .collect();

        if nodes.is_empty() {
            return Self {
                codes: HashMap::new(),
                decode_tree: None,
            };
        }

        if nodes.len() == 1 {
            let node = nodes.pop().unwrap();
            let symbol = node.symbol.unwrap();
            let mut codes = HashMap::new();
            codes.insert(symbol, HuffmanCode { bits: 0, len: 1 });
            return Self {
                codes,
                decode_tree: Some(node),
            };
        }

        while nodes.len() > 1 {
            nodes.sort_by(|a, b| b.freq.cmp(&a.freq));
            let right = nodes.pop().unwrap();
            let left = nodes.pop().unwrap();
            nodes.push(HuffmanNode::internal(left, right));
        }

        let root = nodes.pop().unwrap();
        let mut codes = HashMap::new();
        Self::build_codes(&root, 0, 0, &mut codes);

        Self {
            codes,
            decode_tree: Some(root),
        }
    }

    fn build_codes(node: &HuffmanNode, bits: u32, len: u8, codes: &mut HashMap<u8, HuffmanCode>) {
        if let Some(symbol) = node.symbol {
            codes.insert(
                symbol,
                HuffmanCode {
                    bits,
                    len: len.max(1),
                },
            );
            return;
        }
        if let Some(ref left) = node.left {
            Self::build_codes(left, bits << 1, len + 1, codes);
        }
        if let Some(ref right) = node.right {
            Self::build_codes(right, (bits << 1) | 1, len + 1, codes);
        }
    }

    pub fn get(&self, symbol: u8) -> Option<&HuffmanCode> {
        self.codes.get(&symbol)
    }
}

pub struct EntropyEncoder {
    buffer: Vec<u8>,
    bit_buffer: u32,
    bit_count: u8,
}

impl EntropyEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    fn write_bits(&mut self, bits: u32, len: u8) {
        self.bit_buffer = (self.bit_buffer << len) | (bits & ((1 << len) - 1));
        self.bit_count += len;

        while self.bit_count >= 8 {
            self.bit_count -= 8;
            let byte = (self.bit_buffer >> self.bit_count) as u8;
            self.buffer.push(byte);
        }
    }

    fn flush(&mut self) {
        if self.bit_count > 0 {
            let byte = (self.bit_buffer << (8 - self.bit_count)) as u8;
            self.buffer.push(byte);
            self.bit_buffer = 0;
            self.bit_count = 0;
        }
    }

    pub fn encode_with_huffman(&mut self, data: &[u8]) -> Vec<u8> {
        let mut freq = [0u32; 256];
        for &b in data {
            freq[b as usize] += 1;
        }

        let table = HuffmanTable::build(&freq);

        let mut output = Vec::new();
        for i in 0..256 {
            output.write_u32::<LittleEndian>(freq[i]).unwrap();
        }

        self.buffer.clear();
        self.bit_buffer = 0;
        self.bit_count = 0;

        for &b in data {
            if let Some(code) = table.get(b) {
                self.write_bits(code.bits, code.len);
            }
        }
        self.flush();

        output.write_u32::<LittleEndian>(data.len() as u32).unwrap();
        output
            .write_u32::<LittleEndian>(self.buffer.len() as u32)
            .unwrap();
        output.extend(&self.buffer);

        output
    }

    pub fn encode_rle_huffman(&mut self, data: &[i16]) -> Vec<u8> {
        let mut rle = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0 {
                let mut count = 0u8;
                while i < data.len() && data[i] == 0 && count < 255 {
                    count += 1;
                    i += 1;
                }
                rle.push(0u8);
                rle.push(count);
            } else {
                let val = data[i];
                let magnitude = val.unsigned_abs();
                let sign = if val < 0 { 1u8 } else { 0u8 };

                if magnitude <= 127 {
                    rle.push(1);
                    rle.push((magnitude as u8) | (sign << 7));
                } else {
                    rle.push(2);
                    rle.push((magnitude & 0xFF) as u8);
                    rle.push(((magnitude >> 8) as u8) | (sign << 7));
                }
                i += 1;
            }
        }

        self.encode_with_huffman(&rle)
    }
}

impl Default for EntropyEncoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntropyDecoder;

impl EntropyDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode_huffman(&self, data: &[u8]) -> WkResult<Vec<u8>> {
        if data.len() < 1024 + 8 {
            return Err(WkError::DecodingError("Huffman data too short".into()));
        }

        let mut cursor = std::io::Cursor::new(data);
        let mut freq = [0u32; 256];
        for f in &mut freq {
            *f = cursor.read_u32::<LittleEndian>()?;
        }

        let original_len = cursor.read_u32::<LittleEndian>()? as usize;
        let compressed_len = cursor.read_u32::<LittleEndian>()? as usize;

        let pos = cursor.position() as usize;
        if pos + compressed_len > data.len() {
            return Err(WkError::DecodingError("Truncated huffman data".into()));
        }
        let compressed = &data[pos..pos + compressed_len];

        let table = HuffmanTable::build(&freq);

        if table.decode_tree.is_none() {
            return Ok(Vec::new());
        }

        let root = table.decode_tree.as_ref().unwrap();
        let mut output = Vec::with_capacity(original_len);
        let mut current = root;

        'outer: for &byte in compressed {
            for bit_pos in (0..8).rev() {
                let bit = (byte >> bit_pos) & 1;

                if bit == 0 {
                    if let Some(ref left) = current.left {
                        current = left;
                    }
                } else if let Some(ref right) = current.right {
                    current = right;
                }

                if let Some(symbol) = current.symbol {
                    output.push(symbol);
                    current = root;
                    if output.len() >= original_len {
                        break 'outer;
                    }
                }
            }
        }

        Ok(output)
    }

    pub fn decode_rle_huffman(&self, data: &[u8]) -> WkResult<Vec<i16>> {
        let rle = self.decode_huffman(data)?;
        let mut output = Vec::new();
        let mut i = 0;

        while i < rle.len() {
            match rle[i] {
                0 => {
                    if i + 1 >= rle.len() {
                        break;
                    }
                    let count = rle[i + 1] as usize;
                    output.extend(std::iter::repeat(0i16).take(count));
                    i += 2;
                }
                1 => {
                    if i + 1 >= rle.len() {
                        break;
                    }
                    let b = rle[i + 1];
                    let magnitude = (b & 0x7F) as i16;
                    let sign = (b >> 7) & 1;
                    let val = if sign == 1 { -magnitude } else { magnitude };
                    output.push(val);
                    i += 2;
                }
                2 => {
                    if i + 2 >= rle.len() {
                        break;
                    }
                    let low = rle[i + 1] as u16;
                    let high = (rle[i + 2] & 0x7F) as u16;
                    let magnitude = (low | (high << 8)) as i16;
                    let sign = (rle[i + 2] >> 7) & 1;
                    let val = if sign == 1 { -magnitude } else { magnitude };
                    output.push(val);
                    i += 3;
                }
                _ => i += 1,
            }
        }

        Ok(output)
    }
}

impl Default for EntropyDecoder {
    fn default() -> Self {
        Self::new()
    }
}
