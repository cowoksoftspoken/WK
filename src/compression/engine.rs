use super::adaptive_quant::{AdaptiveQuantizer, QuantTable};
use super::cabac::{
    compress_coefficients, decode_coefficients, decompress_coefficients, encode_coefficients,
    ArithmeticDecoder, ArithmeticEncoder, CABACContext,
};
use super::dct::{dct_8x8_fast, idct_8x8_fast, zigzag_scan, zigzag_unscan};
use super::entropy::{EntropyDecoder, EntropyEncoder};
use super::intra_prediction::{IntraMode, IntraPredictor};
use super::predictor::{apply_optimal_predictor, reverse_predictor};
use super::quantizer::Quantizer;
use super::simd::{dct_8x8_simd, detect_simd, idct_8x8_simd, SimdLevel};
use crate::error::WkResult;
use crate::format::header::CompressionMode;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub mode: CompressionMode,
    pub quality: u8,
    pub use_optimal_predictor: bool,
    pub use_cabac: bool,
    pub use_intra_prediction: bool,
    pub use_adaptive_quant: bool,
    pub use_simd: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: 85,
            use_optimal_predictor: true,
            use_cabac: true,
            use_intra_prediction: true,
            use_adaptive_quant: true,
            use_simd: true,
        }
    }
}

impl CompressionConfig {
    pub fn lossless() -> Self {
        Self {
            mode: CompressionMode::Lossless,
            quality: 100,
            use_optimal_predictor: true,
            use_cabac: false,
            use_intra_prediction: false,
            use_adaptive_quant: false,
            use_simd: true,
        }
    }

    pub fn lossy(quality: u8) -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: quality.clamp(1, 100),
            use_optimal_predictor: false,
            use_cabac: true,
            use_intra_prediction: true,
            use_adaptive_quant: true,
            use_simd: true,
        }
    }

    pub fn fast_lossy(quality: u8) -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: quality.clamp(1, 100),
            use_optimal_predictor: false,
            use_cabac: false,
            use_intra_prediction: false,
            use_adaptive_quant: false,
            use_simd: true,
        }
    }

    pub fn lossy_v3(quality: u8) -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: quality.clamp(1, 100),
            use_optimal_predictor: false,
            use_cabac: true,
            use_intra_prediction: true,
            use_adaptive_quant: true,
            use_simd: true,
        }
    }
}

pub struct CompressionEngine {
    config: CompressionConfig,
    simd_level: SimdLevel,
}

impl CompressionEngine {
    pub fn new(config: CompressionConfig) -> Self {
        let simd_level = if config.use_simd {
            detect_simd()
        } else {
            SimdLevel::None
        };
        Self { config, simd_level }
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

    pub fn compress_lossy_v3(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        let adaptive_quant = AdaptiveQuantizer::new(self.config.quality);
        let predictor = IntraPredictor::new(8);
        let block_width = (width + 7) / 8;
        let block_height = (height + 7) / 8;
        let padded_w = block_width * 8;
        let padded_h = block_height * 8;

        let mut output = Vec::new();
        output.push(if self.config.use_cabac { 1 } else { 0 });
        output.push(if self.config.use_intra_prediction {
            1
        } else {
            0
        });
        output.push(if self.config.use_adaptive_quant { 1 } else { 0 });

        let base_table = QuantTable::aggressive(self.config.quality, false);
        for &v in &base_table.table {
            output.extend(&v.to_le_bytes());
        }
        let chroma_table = QuantTable::aggressive(self.config.quality, true);
        for &v in &chroma_table.table {
            output.extend(&v.to_le_bytes());
        }

        let mut all_data: Vec<u8> = Vec::new();

        for ch in 0..channels {
            let is_chroma = ch > 0 && channels >= 3;
            let channel_data: Vec<u8> = (0..height)
                .flat_map(|y| (0..width).map(move |x| data[(y * width + x) * channels + ch]))
                .collect();

            let mut padded = vec![128u8; padded_w * padded_h];
            for y in 0..height {
                for x in 0..width {
                    padded[y * padded_w + x] = channel_data[y * width + x];
                }
                let last = channel_data[y * width + width - 1];
                for x in width..padded_w {
                    padded[y * padded_w + x] = last;
                }
            }
            for y in height..padded_h {
                for x in 0..padded_w {
                    padded[y * padded_w + x] = padded[(height - 1) * padded_w + x];
                }
            }

            let mut intra_modes = Vec::new();
            let mut block_qps = Vec::new();
            let mut coeffs_data = Vec::new();

            for by in 0..block_height {
                for bx in 0..block_width {
                    let mut block = [0u8; 64];
                    for y in 0..8 {
                        for x in 0..8 {
                            block[y * 8 + x] = padded[(by * 8 + y) * padded_w + bx * 8 + x];
                        }
                    }

                    let (top, left, top_left) = self.get_neighbors(&padded, padded_w, bx, by);

                    let (mode, residual) = if self.config.use_intra_prediction && !is_chroma {
                        let is_first_row = by == 0;
                        let is_first_col = bx == 0;
                        let (best_mode, _) = predictor.select_best_mode_edge(
                            &block,
                            &top,
                            &left,
                            top_left,
                            is_first_row,
                            is_first_col,
                        );
                        let pred = predictor.predict(best_mode, &top, &left, top_left);
                        let res: Vec<i16> = block
                            .iter()
                            .zip(pred.iter())
                            .map(|(&b, &p)| b as i16 - p as i16)
                            .collect();
                        (best_mode, res)
                    } else {
                        let res: Vec<i16> = block.iter().map(|&b| b as i16 - 128).collect();
                        (IntraMode::DC, res)
                    };
                    intra_modes.push(mode.to_u8());

                    let qp = if self.config.use_adaptive_quant {
                        let stats = adaptive_quant.analyze_block(&block, 8);
                        adaptive_quant.compute_qp(&stats)
                    } else {
                        self.config.quality
                    };
                    block_qps.push(qp);

                    let mut block_i16 = [0i16; 64];
                    for i in 0..64 {
                        block_i16[i] = residual[i];
                    }

                    let dct = if self.simd_level != SimdLevel::None {
                        dct_8x8_simd(&block_i16)
                    } else {
                        dct_8x8_fast(&block_i16)
                    };

                    let table = adaptive_quant.get_table(qp, is_chroma);
                    let quantized = adaptive_quant.quantize(&dct, &table);
                    let scanned = zigzag_scan(&quantized);

                    coeffs_data.push(scanned);
                }
            }

            let mode_bytes: Vec<u8> = intra_modes.iter().map(|&m| m).collect();
            all_data.extend(&(mode_bytes.len() as u32).to_le_bytes());
            all_data.extend(&mode_bytes);

            let qp_bytes: Vec<u8> = block_qps.clone();
            all_data.extend(&(qp_bytes.len() as u32).to_le_bytes());
            all_data.extend(&qp_bytes);

            if self.config.use_cabac {
                let mut cabac_encoder = ArithmeticEncoder::new();
                let mut ctx = CABACContext::new(8);
                for coeffs in &coeffs_data {
                    encode_coefficients(&mut cabac_encoder, &mut ctx, coeffs);
                }
                let encoded = cabac_encoder.finish();
                all_data.extend(&(encoded.len() as u32).to_le_bytes());
                all_data.extend(&encoded);
            } else {
                let flat: Vec<i16> = coeffs_data.iter().flatten().copied().collect();
                let mut encoder = EntropyEncoder::new();
                let encoded = encoder.encode_rle_huffman(&flat);
                all_data.extend(&(encoded.len() as u32).to_le_bytes());
                all_data.extend(&encoded);
            }
        }

        let compressed = compress_coefficients(&all_data);
        output.extend(&(compressed.len() as u32).to_le_bytes());
        output.extend(compressed);
        Ok(output)
    }

    pub fn decompress_lossy_v3(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        if data.len() < 259 {
            return Err(crate::error::WkError::DecodingError(
                "Data too short".into(),
            ));
        }

        let use_cabac = data[0] != 0;
        let use_intra = data[1] != 0;
        let use_adaptive = data[2] != 0;

        let mut base_table = [0u16; 64];
        let mut chroma_table = [0u16; 64];
        for i in 0..64 {
            base_table[i] = u16::from_le_bytes([data[3 + i * 2], data[3 + i * 2 + 1]]);
        }
        for i in 0..64 {
            chroma_table[i] = u16::from_le_bytes([data[131 + i * 2], data[131 + i * 2 + 1]]);
        }

        let compressed_len =
            u32::from_le_bytes([data[259], data[260], data[261], data[262]]) as usize;
        let compressed_data = &data[263..263 + compressed_len.min(data.len().saturating_sub(263))];
        let all_data = decompress_coefficients(compressed_data);

        let mut pos = 0usize;
        let block_width = (width + 7) / 8;
        let block_height = (height + 7) / 8;
        let padded_w = block_width * 8;
        let padded_h = block_height * 8;
        let blocks_per_channel = block_width * block_height;

        let predictor = IntraPredictor::new(8);
        let mut output = vec![0u8; width * height * channels];

        for ch in 0..channels {
            let is_chroma = ch > 0 && channels >= 3;

            let modes_len = u32::from_le_bytes([
                all_data[pos],
                all_data[pos + 1],
                all_data[pos + 2],
                all_data[pos + 3],
            ]) as usize;
            pos += 4;
            let modes: Vec<u8> = all_data[pos..pos + modes_len].to_vec();
            pos += modes_len;

            let qps_len = u32::from_le_bytes([
                all_data[pos],
                all_data[pos + 1],
                all_data[pos + 2],
                all_data[pos + 3],
            ]) as usize;
            pos += 4;
            let qps: Vec<u8> = all_data[pos..pos + qps_len].to_vec();
            pos += qps_len;

            let coeffs_len = u32::from_le_bytes([
                all_data[pos],
                all_data[pos + 1],
                all_data[pos + 2],
                all_data[pos + 3],
            ]) as usize;
            pos += 4;
            let coeffs_data = &all_data[pos..pos + coeffs_len];
            pos += coeffs_len;

            let all_coeffs: Vec<Vec<i16>> = if use_cabac {
                let mut decoder = ArithmeticDecoder::new(coeffs_data.to_vec());
                let mut ctx = CABACContext::new(8);
                (0..blocks_per_channel)
                    .map(|_| decode_coefficients(&mut decoder, &mut ctx, 64))
                    .collect()
            } else {
                let decoder = EntropyDecoder::new();
                let flat = decoder.decode_rle_huffman(coeffs_data)?;
                flat.chunks(64).map(|c| c.to_vec()).collect()
            };

            let mut padded = vec![128u8; padded_w * padded_h];

            for by in 0..block_height {
                for bx in 0..block_width {
                    let block_idx = by * block_width + bx;
                    if block_idx >= all_coeffs.len() {
                        continue;
                    }

                    let mut scanned = [0i16; 64];
                    for (i, &v) in all_coeffs[block_idx].iter().enumerate().take(64) {
                        scanned[i] = v;
                    }
                    let zigzagged = zigzag_unscan(&scanned);

                    let qp = qps.get(block_idx).copied().unwrap_or(85);
                    let table = if use_adaptive {
                        QuantTable::for_quality(qp, is_chroma)
                    } else {
                        QuantTable {
                            table: if is_chroma { chroma_table } else { base_table },
                        }
                    };

                    let mut dequantized = [0i16; 64];
                    for i in 0..64 {
                        dequantized[i] = (zigzagged[i] as i32 * table.table[i] as i32) as i16;
                    }

                    let block = if self.simd_level != SimdLevel::None {
                        idct_8x8_simd(&dequantized)
                    } else {
                        idct_8x8_fast(&dequantized)
                    };

                    let mode = IntraMode::from_u8(modes.get(block_idx).copied().unwrap_or(0))
                        .unwrap_or(IntraMode::DC);
                    let (top, left, top_left) = self.get_neighbors(&padded, padded_w, bx, by);

                    let pred_block = if use_intra && !is_chroma {
                        predictor.predict(mode, &top, &left, top_left)
                    } else {
                        vec![128u8; 64]
                    };

                    for y in 0..8 {
                        for x in 0..8 {
                            let px = bx * 8 + x;
                            let py = by * 8 + y;
                            let residual = block[y * 8 + x];
                            let pred_val = pred_block[y * 8 + x] as i16;
                            let val = (pred_val + residual).clamp(0, 255) as u8;
                            padded[py * padded_w + px] = val;
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

    fn get_neighbors(
        &self,
        padded: &[u8],
        stride: usize,
        bx: usize,
        by: usize,
    ) -> (Vec<u8>, Vec<u8>, u8) {
        let mut top = vec![128u8; 8];
        let mut left = vec![128u8; 8];
        let top_left = if bx > 0 && by > 0 {
            padded[(by * 8 - 1) * stride + bx * 8 - 1]
        } else {
            128
        };
        if by > 0 {
            for x in 0..8 {
                top[x] = padded[(by * 8 - 1) * stride + bx * 8 + x];
            }
        }
        if bx > 0 {
            for y in 0..8 {
                left[y] = padded[(by * 8 + y) * stride + bx * 8 - 1];
            }
        }
        (top, left, top_left)
    }

    pub fn compress_lossy(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        if self.config.use_cabac
            || self.config.use_intra_prediction
            || self.config.use_adaptive_quant
        {
            self.compress_lossy_v3(data, width, height, channels)
        } else {
            self.compress_lossy_legacy(data, width, height, channels)
        }
    }

    pub fn decompress_lossy(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
    ) -> WkResult<Vec<u8>> {
        if !data.is_empty() && data[0] <= 1 && data.len() > 259 {
            self.decompress_lossy_v3(data, width, height, channels)
        } else {
            self.decompress_lossy_legacy(data, width, height, channels)
        }
    }

    fn compress_lossy_legacy(
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
                            block[y * 8 + x] =
                                padded[(by * 8 + y) * padded_w + bx * 8 + x] as i16 - 128;
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

    fn decompress_lossy_legacy(
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

        let decoder = EntropyDecoder::new();
        let coeffs = decoder.decode_rle_huffman(&data[256..])?;

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
                                padded[py * padded_w + px] =
                                    (block[y * 8 + x] + 128).clamp(0, 255) as u8;
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
            CompressionMode::Lossy | CompressionMode::Mixed => {
                self.compress_lossy(data, width, height, channels)
            }
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
            CompressionMode::Lossy | CompressionMode::Mixed => {
                self.decompress_lossy(data, width, height, channels)
            }
        }
    }
}
