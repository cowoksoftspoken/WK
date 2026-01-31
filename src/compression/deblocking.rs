#[derive(Debug, Clone, Copy)]
pub struct DeblockConfig {
    pub strength: u8,
    pub sharpness: u8,
    pub simple: bool,
}

impl DeblockConfig {
    pub fn from_quality(quality: u8) -> Self {
        let strength = if quality >= 90 {
            10
        } else if quality >= 80 {
            20
        } else if quality >= 70 {
            30
        } else {
            40
        };

        Self {
            strength,
            sharpness: 2,
            simple: quality >= 85,
        }
    }
}

impl Default for DeblockConfig {
    fn default() -> Self {
        Self::from_quality(85)
    }
}

pub struct DeblockingFilter {
    config: DeblockConfig,
}

impl DeblockingFilter {
    pub fn new(config: DeblockConfig) -> Self {
        Self { config }
    }

    fn filter_limit(&self) -> i32 {
        let s = self.config.strength as i32;
        let limit = if self.config.sharpness == 0 {
            s
        } else {
            let shift = (self.config.sharpness + 3) >> 2;
            (s >> shift).max(1)
        };
        limit.min(63)
    }

    fn clamp_pixel(val: i32) -> u8 {
        val.clamp(0, 255) as u8
    }
    fn filter_value(p1: i32, p0: i32, q0: i32, q1: i32, limit: i32) -> i32 {
        let a = (p1 - q1).clamp(-128, 127);
        let b = 3 * (q0 - p0) + a;

        let diff = (p0 - q0).abs();
        if diff > limit {
            return 0;
        }
        let adj = (b + 4) >> 3;
        adj.clamp(-limit, limit)
    }

    fn simple_h_filter(&self, data: &mut [u8], stride: usize, x: usize, y: usize, limit: i32) {
        if y < 1 || y >= data.len() / stride {
            return;
        }

        let p0_idx = (y - 1) * stride + x;
        let q0_idx = y * stride + x;

        if p0_idx >= data.len() || q0_idx >= data.len() {
            return;
        }

        let p0 = data[p0_idx] as i32;
        let q0 = data[q0_idx] as i32;

        let diff = (p0 - q0).abs();
        if diff <= limit {
            let adj = ((q0 - p0 + 4) >> 3).clamp(-limit, limit);
            data[p0_idx] = Self::clamp_pixel(p0 + adj);
            data[q0_idx] = Self::clamp_pixel(q0 - adj);
        }
    }

    fn simple_v_filter(&self, data: &mut [u8], stride: usize, x: usize, y: usize, limit: i32) {
        if x < 1 || x >= stride {
            return;
        }

        let idx = y * stride + x;
        if idx < 1 || idx >= data.len() {
            return;
        }

        let p0 = data[idx - 1] as i32;
        let q0 = data[idx] as i32;

        let diff = (p0 - q0).abs();
        if diff <= limit {
            let adj = ((q0 - p0 + 4) >> 3).clamp(-limit, limit);
            data[idx - 1] = Self::clamp_pixel(p0 + adj);
            data[idx] = Self::clamp_pixel(q0 - adj);
        }
    }

    fn normal_h_filter(&self, data: &mut [u8], stride: usize, x: usize, y: usize, limit: i32) {
        if y < 2 || y + 1 >= data.len() / stride {
            return;
        }

        let p1_idx = (y - 2) * stride + x;
        let p0_idx = (y - 1) * stride + x;
        let q0_idx = y * stride + x;
        let q1_idx = (y + 1) * stride + x;

        if q1_idx >= data.len() {
            return;
        }

        let p1 = data[p1_idx] as i32;
        let p0 = data[p0_idx] as i32;
        let q0 = data[q0_idx] as i32;
        let q1 = data[q1_idx] as i32;

        let adj = Self::filter_value(p1, p0, q0, q1, limit);
        if adj != 0 {
            data[p0_idx] = Self::clamp_pixel(p0 + adj);
            data[q0_idx] = Self::clamp_pixel(q0 - adj);
        }
    }

    fn normal_v_filter(&self, data: &mut [u8], stride: usize, x: usize, y: usize, limit: i32) {
        if x < 2 || x + 1 >= stride {
            return;
        }

        let row_start = y * stride;
        if row_start + x + 1 >= data.len() {
            return;
        }

        let p1 = data[row_start + x - 2] as i32;
        let p0 = data[row_start + x - 1] as i32;
        let q0 = data[row_start + x] as i32;
        let q1 = data[row_start + x + 1] as i32;

        let adj = Self::filter_value(p1, p0, q0, q1, limit);
        if adj != 0 {
            data[row_start + x - 1] = Self::clamp_pixel(p0 + adj);
            data[row_start + x] = Self::clamp_pixel(q0 - adj);
        }
    }

    pub fn apply(&self, data: &mut [u8], width: usize, height: usize, block_size: usize) {
        if self.config.strength == 0 {
            return;
        }

        let limit = self.filter_limit();
        let stride = width;

        for by in 1..(height / block_size) {
            let y = by * block_size;
            for x in 0..width {
                if self.config.simple {
                    self.simple_h_filter(data, stride, x, y, limit);
                } else {
                    self.normal_h_filter(data, stride, x, y, limit);
                }
            }
        }

        for bx in 1..(width / block_size) {
            let x = bx * block_size;
            for y in 0..height {
                if self.config.simple {
                    self.simple_v_filter(data, stride, x, y, limit);
                } else {
                    self.normal_v_filter(data, stride, x, y, limit);
                }
            }
        }
    }

    pub fn apply_channel(&self, channel: &mut [u8], width: usize, height: usize) {
        self.apply(channel, width, height, 8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deblock_config_from_quality() {
        let config_low = DeblockConfig::from_quality(50);
        let config_high = DeblockConfig::from_quality(95);

        assert!(config_low.strength > config_high.strength);
        assert!(config_high.simple);
    }

    #[test]
    fn test_deblock_filter_apply() {
        let config = DeblockConfig::from_quality(75);
        let filter = DeblockingFilter::new(config);

        let mut data = vec![128u8; 16 * 16];

        for y in 0..8 {
            for x in 0..16 {
                data[y * 16 + x] = 100;
            }
        }
        for y in 8..16 {
            for x in 0..16 {
                data[y * 16 + x] = 150;
            }
        }

        let original = data.clone();
        filter.apply(&mut data, 16, 16, 8);

        let boundary_diff_before = (original[7 * 16] as i32 - original[8 * 16] as i32).abs();
        let boundary_diff_after = (data[7 * 16] as i32 - data[8 * 16] as i32).abs();

        assert!(
            boundary_diff_after <= boundary_diff_before,
            "Filter should reduce boundary discontinuity"
        );
    }
}
