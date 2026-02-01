#[derive(Debug, Clone, Copy)]
pub struct DeblockConfig {
    pub strength: u8,
    pub sharpness: u8,
    pub edge_threshold: u8,
    pub q_scale: u8,
    pub simple: bool,
    pub enabled: bool, // Skip deblocking at very high quality
}

impl DeblockConfig {
    pub fn from_quality(quality: u8) -> Self {
        // At very high quality (97+), skip deblocking entirely to preserve sharpness
        if quality >= 97 {
            return Self {
                strength: 0,
                sharpness: 7,
                edge_threshold: 5,
                q_scale: 4,
                simple: true,
                enabled: false,
            };
        }

        // Reduced strength values for high quality to prevent blur
        let strength = if quality >= 95 {
            3 // Was 8 - minimal filtering
        } else if quality >= 90 {
            6 // Was 12 - light filtering
        } else if quality >= 85 {
            10 // Was 18 - moderate filtering
        } else if quality >= 80 {
            18 // Was 25
        } else if quality >= 70 {
            28 // Was 35
        } else {
            40 // Was 45
        };

        // Higher edge threshold = more selective (only filter strong edges)
        let edge_threshold = if quality >= 90 {
            10 // Was 15 - more selective
        } else if quality >= 80 {
            15 // Was 20
        } else {
            20 // Was 25
        };

        // Higher sharpness = less filtering
        let sharpness = if quality >= 90 {
            6 // Was 4 - preserve more sharpness
        } else if quality >= 80 {
            5 // Was 3
        } else {
            3 // Was 2
        };

        Self {
            strength,
            sharpness,
            edge_threshold,
            q_scale: 4,
            simple: quality >= 92,
            enabled: true,
        }
    }
}

impl Default for DeblockConfig {
    fn default() -> Self {
        Self::from_quality(85)
    }
}

#[derive(Clone, Copy)]
pub enum EdgeDir {
    Vertical,
    Horizontal,
}

struct Samples {
    p2: i32,
    p1: i32,
    p0: i32,
    q0: i32,
    q1: i32,
    q2: i32,
}

pub struct Plane<'a> {
    pub data: &'a mut [u8],
    pub w: usize,
    pub h: usize,
    pub stride: usize,
}

pub struct DeblockingFilter {
    config: DeblockConfig,
}

impl DeblockingFilter {
    pub fn new(config: DeblockConfig) -> Self {
        Self { config }
    }

    fn apply_sharpness(level: i32, sharpness: u8) -> i32 {
        let s = sharpness.min(7) as i32;
        ((level * (8 - s)) / 8).max(1)
    }

    fn should_filter(s: &Samples, edge_th: u8, level: i32) -> bool {
        let boundary = (s.p0 - s.q0).abs();
        let grad_p = (s.p1 - s.p0).abs();
        let grad_q = (s.q1 - s.q0).abs();
        let grad_in = grad_p.max(grad_q);

        (boundary as u8) <= edge_th || (boundary < level * 4 && grad_in < level * 2)
    }

    fn is_flat(s: &Samples, level: i32) -> bool {
        let diffs = [
            (s.p2 - s.p1).abs(),
            (s.p1 - s.p0).abs(),
            (s.q1 - s.q0).abs(),
            (s.q2 - s.q1).abs(),
        ];
        diffs.into_iter().max().unwrap_or(0) < level
    }

    fn is_strong_flat(s: &Samples, level: i32) -> bool {
        Self::is_flat(s, level) && (s.p0 - s.q0).abs() < (level / 2).max(1)
    }

    fn gather_samples_v(
        data: &[u8],
        stride: usize,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Option<Samples> {
        if x < 3 || x + 2 >= w || y >= h {
            return None;
        }
        let row = y * stride;
        Some(Samples {
            p2: data[row + x - 3] as i32,
            p1: data[row + x - 2] as i32,
            p0: data[row + x - 1] as i32,
            q0: data[row + x] as i32,
            q1: data[row + x + 1] as i32,
            q2: data[row + x + 2] as i32,
        })
    }

    fn gather_samples_h(
        data: &[u8],
        stride: usize,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Option<Samples> {
        if y < 3 || y + 2 >= h || x >= w {
            return None;
        }
        let at = |yy: usize| data[yy * stride + x] as i32;
        Some(Samples {
            p2: at(y - 3),
            p1: at(y - 2),
            p0: at(y - 1),
            q0: at(y),
            q1: at(y + 1),
            q2: at(y + 2),
        })
    }

    fn apply_simple_v(data: &mut [u8], stride: usize, x: usize, y: usize, level: i32) {
        let row = y * stride;
        let p0 = data[row + x - 1] as i32;
        let q0 = data[row + x] as i32;
        let mut delta = ((q0 - p0) * 3) / 8;
        delta = delta.clamp(-level, level);
        data[row + x - 1] = (p0 + delta).clamp(0, 255) as u8;
        data[row + x] = (q0 - delta).clamp(0, 255) as u8;
    }

    fn apply_simple_h(data: &mut [u8], stride: usize, x: usize, y: usize, level: i32) {
        let p0 = data[(y - 1) * stride + x] as i32;
        let q0 = data[y * stride + x] as i32;
        let mut delta = ((q0 - p0) * 3) / 8;
        delta = delta.clamp(-level, level);
        data[(y - 1) * stride + x] = (p0 + delta).clamp(0, 255) as u8;
        data[y * stride + x] = (q0 - delta).clamp(0, 255) as u8;
    }

    fn apply_normal_v(data: &mut [u8], stride: usize, x: usize, y: usize, level: i32) {
        let row = y * stride;
        let p1 = data[row + x - 2] as i32;
        let p0 = data[row + x - 1] as i32;
        let q0 = data[row + x] as i32;
        let q1 = data[row + x + 1] as i32;

        let a = (p1 - q1).clamp(-128, 127);
        let b = 3 * (q0 - p0) + a;
        let delta = ((b + 4) >> 3).clamp(-level, level);

        data[row + x - 1] = (p0 + delta).clamp(0, 255) as u8;
        data[row + x] = (q0 - delta).clamp(0, 255) as u8;

        let delta2 = (delta + 1) >> 1;
        data[row + x - 2] = (p1 + delta2).clamp(0, 255) as u8;
        data[row + x + 1] = (q1 - delta2).clamp(0, 255) as u8;
    }

    fn apply_normal_h(data: &mut [u8], stride: usize, x: usize, y: usize, level: i32) {
        let p1 = data[(y - 2) * stride + x] as i32;
        let p0 = data[(y - 1) * stride + x] as i32;
        let q0 = data[y * stride + x] as i32;
        let q1 = data[(y + 1) * stride + x] as i32;

        let a = (p1 - q1).clamp(-128, 127);
        let b = 3 * (q0 - p0) + a;
        let delta = ((b + 4) >> 3).clamp(-level, level);

        data[(y - 1) * stride + x] = (p0 + delta).clamp(0, 255) as u8;
        data[y * stride + x] = (q0 - delta).clamp(0, 255) as u8;

        let delta2 = (delta + 1) >> 1;
        data[(y - 2) * stride + x] = (p1 + delta2).clamp(0, 255) as u8;
        data[(y + 1) * stride + x] = (q1 - delta2).clamp(0, 255) as u8;
    }

    fn apply_strong_v(data: &mut [u8], stride: usize, x: usize, y: usize, _level: i32) {
        let row = y * stride;
        let p2 = data[row + x - 3] as i32;
        let p1 = data[row + x - 2] as i32;
        let p0 = data[row + x - 1] as i32;
        let q0 = data[row + x] as i32;
        let q1 = data[row + x + 1] as i32;
        let q2 = data[row + x + 2] as i32;

        let new_p1 = (p2 + p1 * 2 + p0 * 2 + q0 + 3) / 6;
        let new_p0 = (p1 + p0 * 2 + q0 * 2 + q1 + 3) / 6;
        let new_q0 = (p0 + q0 * 2 + q1 * 2 + q2 + 3) / 6;
        let new_q1 = (q0 + q1 * 2 + q2 * 2 + q2 + 3) / 6;

        data[row + x - 2] = new_p1.clamp(0, 255) as u8;
        data[row + x - 1] = new_p0.clamp(0, 255) as u8;
        data[row + x] = new_q0.clamp(0, 255) as u8;
        data[row + x + 1] = new_q1.clamp(0, 255) as u8;
    }

    fn apply_strong_h(data: &mut [u8], stride: usize, x: usize, y: usize, _level: i32) {
        let at = |yy: usize| data[yy * stride + x] as i32;
        let p2 = at(y - 3);
        let p1 = at(y - 2);
        let p0 = at(y - 1);
        let q0 = at(y);
        let q1 = at(y + 1);
        let q2 = at(y + 2);

        let new_p1 = (p2 + p1 * 2 + p0 * 2 + q0 + 3) / 6;
        let new_p0 = (p1 + p0 * 2 + q0 * 2 + q1 + 3) / 6;
        let new_q0 = (p0 + q0 * 2 + q1 * 2 + q2 + 3) / 6;
        let new_q1 = (q0 + q1 * 2 + q2 * 2 + q2 + 3) / 6;

        data[(y - 2) * stride + x] = new_p1.clamp(0, 255) as u8;
        data[(y - 1) * stride + x] = new_p0.clamp(0, 255) as u8;
        data[y * stride + x] = new_q0.clamp(0, 255) as u8;
        data[(y + 1) * stride + x] = new_q1.clamp(0, 255) as u8;
    }

    fn filter_edge_v(
        &self,
        data: &mut [u8],
        stride: usize,
        edge_x: usize,
        y_start: usize,
        len: usize,
        w: usize,
        h: usize,
        level: i32,
        chroma: bool,
    ) {
        let eff_level = if chroma { (level * 3) / 4 } else { level };
        if eff_level <= 0 {
            return;
        }

        for i in 0..len {
            let y = y_start + i;
            if y >= h {
                break;
            }

            if let Some(s) = Self::gather_samples_v(data, stride, edge_x, y, w, h) {
                if !Self::should_filter(&s, self.config.edge_threshold, eff_level) {
                    continue;
                }

                if self.config.simple {
                    Self::apply_simple_v(data, stride, edge_x, y, eff_level);
                } else if Self::is_strong_flat(&s, eff_level) {
                    Self::apply_strong_v(data, stride, edge_x, y, eff_level);
                } else if Self::is_flat(&s, eff_level) {
                    Self::apply_normal_v(data, stride, edge_x, y, eff_level);
                } else {
                    Self::apply_simple_v(data, stride, edge_x, y, eff_level);
                }
            }
        }
    }

    fn filter_edge_h(
        &self,
        data: &mut [u8],
        stride: usize,
        edge_y: usize,
        x_start: usize,
        len: usize,
        w: usize,
        h: usize,
        level: i32,
        chroma: bool,
    ) {
        let eff_level = if chroma { (level * 3) / 4 } else { level };
        if eff_level <= 0 {
            return;
        }

        for i in 0..len {
            let x = x_start + i;
            if x >= w {
                break;
            }

            if let Some(s) = Self::gather_samples_h(data, stride, x, edge_y, w, h) {
                if !Self::should_filter(&s, self.config.edge_threshold, eff_level) {
                    continue;
                }

                if self.config.simple {
                    Self::apply_simple_h(data, stride, x, edge_y, eff_level);
                } else if Self::is_strong_flat(&s, eff_level) {
                    Self::apply_strong_h(data, stride, x, edge_y, eff_level);
                } else if Self::is_flat(&s, eff_level) {
                    Self::apply_normal_h(data, stride, x, edge_y, eff_level);
                } else {
                    Self::apply_simple_h(data, stride, x, edge_y, eff_level);
                }
            }
        }
    }

    pub fn deblock_plane(
        &self,
        data: &mut [u8],
        w: usize,
        h: usize,
        stride: usize,
        block: usize,
        chroma: bool,
    ) {
        if self.config.strength == 0 {
            return;
        }

        let mut level = self.config.strength as i32;
        level = Self::apply_sharpness(level, self.config.sharpness);

        let bw = (w + block - 1) / block;
        let bh = (h + block - 1) / block;

        for by in 0..bh {
            for bx in 1..bw {
                let x = bx * block;
                let y0 = by * block;
                self.filter_edge_v(data, stride, x, y0, block, w, h, level, chroma);
            }
        }

        for by in 1..bh {
            for bx in 0..bw {
                let y = by * block;
                let x0 = bx * block;
                self.filter_edge_h(data, stride, y, x0, block, w, h, level, chroma);
            }
        }
    }

    pub fn apply(&self, data: &mut [u8], width: usize, height: usize, block_size: usize) {
        if !self.config.enabled {
            return;
        }
        self.deblock_plane(data, width, height, width, block_size, false);
    }

    pub fn apply_chroma(&self, data: &mut [u8], width: usize, height: usize, block_size: usize) {
        if !self.config.enabled {
            return;
        }
        self.deblock_plane(data, width, height, width, block_size, true);
    }

    pub fn apply_channel(&self, channel: &mut [u8], width: usize, height: usize) {
        if !self.config.enabled {
            return;
        }
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
    fn test_should_filter() {
        let flat_samples = Samples {
            p2: 100,
            p1: 100,
            p0: 100,
            q0: 110,
            q1: 110,
            q2: 110,
        };
        assert!(DeblockingFilter::should_filter(&flat_samples, 20, 10));

        let edge_samples = Samples {
            p2: 50,
            p1: 80,
            p0: 100,
            q0: 200,
            q1: 220,
            q2: 250,
        };
        assert!(!DeblockingFilter::should_filter(&edge_samples, 15, 10));
    }

    #[test]
    fn test_is_flat() {
        let flat = Samples {
            p2: 100,
            p1: 101,
            p0: 102,
            q0: 103,
            q1: 104,
            q2: 105,
        };
        assert!(DeblockingFilter::is_flat(&flat, 10));

        let not_flat = Samples {
            p2: 100,
            p1: 120,
            p0: 140,
            q0: 160,
            q1: 180,
            q2: 200,
        };
        assert!(!DeblockingFilter::is_flat(&not_flat, 10));
    }
}
