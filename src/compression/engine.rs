use super::dct::{dct_8x8_fast, idct_8x8_fast, zigzag_scan, zigzag_unscan};
use super::entropy::{EntropyDecoder, EntropyEncoder};
use super::predictor::{apply_optimal_predictor, reverse_predictor};
use super::quantizer::Quantizer;
use crate::error::WkResult;
use crate::format::header::CompressionMode;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub mode: CompressionMode,
    pub quality: u8,
    pub use_optimal_predictor: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: 85,
            use_optimal_predictor: true,
        }
    }
}

impl CompressionConfig {
    pub fn lossless() -> Self {
        Self {
            mode: CompressionMode::Lossless,
            quality: 100,
            use_optimal_predictor: true,
        }
    }

    pub fn lossy(quality: u8) -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: quality.clamp(1, 100),
            use_optimal_predictor: false,
        }
    }
}

pub struct CompressionEngine {
    config: CompressionConfig,
}

impl CompressionEngine {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn compress_lossless(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        let filtered = apply_optimal_predictor(data, width, height, channels);
        let mut encoder = EntropyEncoder::new();
        Ok(encoder.encode_with_huffman(&filtered))
    }

    pub fn decompress_lossless(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        let decoder = EntropyDecoder::new();
        let filtered = decoder.decode_huffman(data)?;
        reverse_predictor(&filtered, width, height, channels)
    }

    pub fn compress_lossy(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        let quantizer = Quantizer::new(self.config.quality);
        let block_width = (width + 7) / 8;
        let block_height = (height + 7) / 8;

        let mut all_coeffs: Vec<i16> = Vec::new();

        for ch in 0..channels {
            let is_chroma = ch > 0 && channels >= 3;
            let channel_data: Vec<u8> = (0..height)
                .flat_map(|y| (0..width).map(move |x| data[(y * width + x) * channels + ch]))
                .collect();

            let padded_w = block_width * 8;
            let padded_h = block_height * 8;
            let mut padded = vec![0u8; padded_w * padded_h];

            for y in 0..height {
                for x in 0..width {
                    padded[y * padded_w + x] = channel_data[y * width + x];
                }
                for x in width..padded_w {
                    padded[y * padded_w + x] = channel_data[y * width + width - 1];
                }
            }
            for y in height..padded_h {
                for x in 0..padded_w {
                    padded[y * padded_w + x] = padded[(height - 1) * padded_w + x];
                }
            }

            let blocks: Vec<[i16; 64]> = (0..block_height)
                .flat_map(|by| (0..block_width).map(move |bx| (bx, by)))
                .collect::<Vec<_>>()
                .par_iter()
                .map(|&(bx, by)| {
                    let mut block = [0i16; 64];
                    for y in 0..8 {
                        for x in 0..8 {
                            let px = bx * 8 + x;
                            let py = by * 8 + y;
                            block[y * 8 + x] = padded[py * padded_w + px] as i16 - 128;
                        }
                    }

                    let dct = dct_8x8_fast(&block);
                    let quantized = quantizer.quantize(&dct, is_chroma);
                    zigzag_scan(&quantized)
                })
                .collect();

            for block in blocks {
                all_coeffs.extend(block.iter());
            }
        }

        let mut encoder = EntropyEncoder::new();
        let compressed = encoder.encode_rle_huffman(&all_coeffs);

        let mut output = Vec::new();
        output.extend(
            quantizer
                .luma_table()
                .table
                .iter()
                .flat_map(|&v| v.to_le_bytes()),
        );
        output.extend(
            quantizer
                .chroma_table()
                .table
                .iter()
                .flat_map(|&v| v.to_le_bytes()),
        );
        output.extend(compressed);

        Ok(output)
    }

    pub fn decompress_lossy(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        if data.len() < 256 {
            return Err(crate::error::WkError::DecodingError(
                "Data too short".into(),
            ));
        }

        let mut luma_table = [0u16; 64];
        let mut chroma_table = [0u16; 64];

        for i in 0..64 {
            luma_table[i] = u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        }
        for i in 0..64 {
            chroma_table[i] = u16::from_le_bytes([data[128 + i * 2], data[128 + i * 2 + 1]]);
        }

        let compressed = &data[256..];
        let decoder = EntropyDecoder::new();
        let coeffs = decoder.decode_rle_huffman(compressed)?;

        let block_width = (width + 7) / 8;
        let block_height = (height + 7) / 8;
        let blocks_per_channel = block_width * block_height;

        let mut output = vec![0u8; width * height * channels];
        let padded_w = block_width * 8;
        let padded_h = block_height * 8;

        for ch in 0..channels {
            let is_chroma = ch > 0 && channels >= 3;
            let table = if is_chroma {
                &chroma_table
            } else {
                &luma_table
            };
            let channel_offset = ch * blocks_per_channel * 64;

            let mut padded = vec![0u8; padded_w * padded_h];

            for by in 0..block_height {
                for bx in 0..block_width {
                    let block_idx = by * block_width + bx;
                    let coeff_start = channel_offset + block_idx * 64;

                    if coeff_start + 64 > coeffs.len() {
                        continue;
                    }

                    let mut scanned = [0i16; 64];
                    scanned.copy_from_slice(&coeffs[coeff_start..coeff_start + 64]);

                    let zigzagged = zigzag_unscan(&scanned);

                    let mut dequantized = [0i16; 64];
                    for i in 0..64 {
                        dequantized[i] = (zigzagged[i] as i32 * table[i] as i32) as i16;
                    }

                    let block = idct_8x8_fast(&dequantized);

                    for y in 0..8 {
                        for x in 0..8 {
                            let px = bx * 8 + x;
                            let py = by * 8 + y;
                            if px < padded_w && py < padded_h {
                                let val = (block[y * 8 + x] + 128).clamp(0, 255) as u8;
                                padded[py * padded_w + px] = val;
                            }
                        }
                    }
                }
            }

            for y in 0..height {
                for x in 0..width {
                    output[(y * width + x) * channels + ch] = padded[y * padded_w + x];
                }
            }
        }

        Ok(output)
    }

    pub fn compress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        match self.config.mode {
            CompressionMode::Lossless => self.compress_lossless(data, width, height, channels),
            CompressionMode::Lossy => self.compress_lossy(data, width, height, channels),
            CompressionMode::Mixed => self.compress_lossy(data, width, height, channels),
        }
    }

    pub fn decompress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        mode: CompressionMode,
    ) -> WkResult<Vec<u8>> {
        match mode {
            CompressionMode::Lossless => self.decompress_lossless(data, width, height, channels),
            CompressionMode::Lossy => self.decompress_lossy(data, width, height, channels),
            CompressionMode::Mixed => self.decompress_lossy(data, width, height, channels),
        }
    }
}
