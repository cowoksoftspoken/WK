pub const JPEG_LUMA: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

pub const JPEG_CHROMA: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

pub const CSF_WEIGHTS: [f32; 64] = [
    1.0, 0.98, 0.93, 0.85, 0.75, 0.63, 0.52, 0.42, 0.98, 0.95, 0.88, 0.78, 0.67, 0.55, 0.45, 0.36,
    0.93, 0.88, 0.80, 0.70, 0.59, 0.48, 0.39, 0.31, 0.85, 0.78, 0.70, 0.60, 0.50, 0.41, 0.33, 0.26,
    0.75, 0.67, 0.59, 0.50, 0.42, 0.34, 0.27, 0.22, 0.63, 0.55, 0.48, 0.41, 0.34, 0.28, 0.22, 0.18,
    0.52, 0.45, 0.39, 0.33, 0.27, 0.22, 0.18, 0.14, 0.42, 0.36, 0.31, 0.26, 0.22, 0.18, 0.14, 0.11,
];

#[derive(Debug, Clone)]
pub struct QuantTable {
    pub table: [u16; 64],
}

impl QuantTable {
    pub fn for_quality(quality: u8, is_chroma: bool) -> Self {
        let base = if is_chroma { JPEG_CHROMA } else { JPEG_LUMA };
        let q = quality.clamp(1, 100) as u32;
        let scale = if q < 50 { 5000 / q } else { 200 - q * 2 };
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

    pub fn with_csf(quality: u8, is_chroma: bool) -> Self {
        let base = if is_chroma { JPEG_CHROMA } else { JPEG_LUMA };
        let q = quality.clamp(1, 100) as u32;
        let scale = if q < 50 { 5000 / q } else { 200 - q * 2 };
        let mut table = [0u16; 64];
        for i in 0..64 {
            let csf_factor = 1.0 + (1.0 - CSF_WEIGHTS[i]) * 0.5;
            let adjusted = (base[i] as f32 * csf_factor) as u32;
            let val = (adjusted * scale + 50) / 100;
            table[i] = val.clamp(1, 255) as u16;
        }
        Self { table }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub mean: f32,
    pub variance: f32,
    pub contrast: f32,
    pub edge_density: f32,
    pub is_extreme: bool,
}

impl BlockStats {
    pub fn needs_fallback(&self) -> bool {
        self.is_extreme
            || self.variance > 5000.0
            || self.variance < 1.0
            || self.mean < 5.0
            || self.mean > 250.0
    }
}

pub struct AdaptiveQuantizer {
    base_qp: u8,
    min_qp: u8,
    max_qp: u8,
    use_csf: bool,
}

impl AdaptiveQuantizer {
    pub fn new(quality: u8) -> Self {
        let base = quality.clamp(1, 100);
        Self {
            base_qp: base,
            min_qp: base.saturating_sub(10).max(1),
            max_qp: base.saturating_add(10).min(100),
            use_csf: true,
        }
    }

    pub fn analyze_block(&self, block: &[u8], size: usize) -> BlockStats {
        let n = size * size;
        if block.len() < n {
            return BlockStats {
                is_extreme: true,
                ..Default::default()
            };
        }

        let mut sum = 0u64;
        let mut sum_sq = 0u64;
        let mut min_val = 255u8;
        let mut max_val = 0u8;

        for &p in block.iter().take(n) {
            sum += p as u64;
            sum_sq += (p as u64) * (p as u64);
            min_val = min_val.min(p);
            max_val = max_val.max(p);
        }

        let mean = (sum as f32) / (n as f32);
        let variance = (sum_sq as f32 / n as f32) - (mean * mean);
        let contrast = (max_val.saturating_sub(min_val) as f32) / 255.0;

        let edge_density = if size >= 2 {
            let mut edge_sum = 0u64;
            for y in 1..size {
                for x in 1..size {
                    let curr = block[y * size + x] as i32;
                    let left = block[y * size + x - 1] as i32;
                    let top = block[(y - 1) * size + x] as i32;
                    edge_sum += ((curr - left).abs() + (curr - top).abs()) as u64;
                }
            }
            edge_sum as f32 / ((size - 1) * (size - 1) * 510) as f32
        } else {
            0.0
        };

        BlockStats {
            mean,
            variance: variance.max(0.0),
            contrast,
            edge_density,
            is_extreme: false,
        }
    }

    pub fn compute_qp(&self, stats: &BlockStats) -> u8 {
        if stats.needs_fallback() {
            return self.base_qp;
        }

        let mut qp_adjust: i8 = 0;

        if stats.mean < 40.0 {
            qp_adjust -= 2;
        } else if stats.mean > 215.0 {
            qp_adjust -= 1;
        }

        if stats.variance < 50.0 {
            qp_adjust += 2;
        } else if stats.variance > 1500.0 {
            qp_adjust -= 2;
        }

        if stats.edge_density > 0.25 {
            qp_adjust -= 2;
        } else if stats.edge_density < 0.03 {
            qp_adjust += 1;
        }

        qp_adjust = qp_adjust.clamp(-5, 5);

        let qp =
            (self.base_qp as i16 + qp_adjust as i16).clamp(self.min_qp as i16, self.max_qp as i16);
        qp as u8
    }

    pub fn get_table(&self, qp: u8, is_chroma: bool) -> QuantTable {
        if qp < 10 || qp > 98 {
            return QuantTable::for_quality(qp, is_chroma);
        }
        if self.use_csf {
            QuantTable::with_csf(qp, is_chroma)
        } else {
            QuantTable::for_quality(qp, is_chroma)
        }
    }

    pub fn quantize(&self, block: &[i16; 64], table: &QuantTable) -> [i16; 64] {
        let mut out = [0i16; 64];
        for i in 0..64 {
            let t = table.table[i].max(1) as i32;
            out[i] = ((block[i] as i32) / t) as i16;
        }
        out
    }

    pub fn dequantize(&self, block: &[i16; 64], table: &QuantTable) -> [i16; 64] {
        let mut out = [0i16; 64];
        for i in 0..64 {
            let val = (block[i] as i32) * (table.table[i] as i32);
            out[i] = val.clamp(-32768, 32767) as i16;
        }
        out
    }

    pub fn base_quality(&self) -> u8 {
        self.base_qp
    }
}

impl Default for AdaptiveQuantizer {
    fn default() -> Self {
        Self::new(85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qp_clamping() {
        let aq = AdaptiveQuantizer::new(50);

        let normal = BlockStats {
            mean: 128.0,
            variance: 500.0,
            contrast: 0.5,
            edge_density: 0.1,
            is_extreme: false,
        };
        let qp = aq.compute_qp(&normal);
        assert!(qp >= 40 && qp <= 60, "QP {} out of expected range", qp);

        let extreme = BlockStats {
            mean: 2.0,
            variance: 10000.0,
            contrast: 1.0,
            edge_density: 0.9,
            is_extreme: false,
        };
        let qp = aq.compute_qp(&extreme);
        assert_eq!(qp, 50, "Extreme stats should fallback to base QP");
    }

    #[test]
    fn test_dequant_no_overflow() {
        let aq = AdaptiveQuantizer::new(50);
        let table = QuantTable::for_quality(50, false);

        let block = [32767i16; 64];
        let dequant = aq.dequantize(&block, &table);

        for &v in &dequant {
            let _ = v;
        }
    }
}
