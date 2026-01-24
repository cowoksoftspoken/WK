#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntraMode {
    DC,
    Horizontal,
    Vertical,
    DiagonalDownLeft,
    DiagonalDownRight,
    VerticalRight,
    HorizontalDown,
    VerticalLeft,
    HorizontalUp,
    Planar,
    TrueMotion,
}

impl IntraMode {
    pub const ALL: [IntraMode; 11] = [
        Self::DC,
        Self::Horizontal,
        Self::Vertical,
        Self::DiagonalDownLeft,
        Self::DiagonalDownRight,
        Self::VerticalRight,
        Self::HorizontalDown,
        Self::VerticalLeft,
        Self::HorizontalUp,
        Self::Planar,
        Self::TrueMotion,
    ];

    pub const SAFE_EDGE: [IntraMode; 3] = [Self::DC, Self::Horizontal, Self::Vertical];

    pub fn to_u8(self) -> u8 {
        match self {
            Self::DC => 0,
            Self::Horizontal => 1,
            Self::Vertical => 2,
            Self::DiagonalDownLeft => 3,
            Self::DiagonalDownRight => 4,
            Self::VerticalRight => 5,
            Self::HorizontalDown => 6,
            Self::VerticalLeft => 7,
            Self::HorizontalUp => 8,
            Self::Planar => 9,
            Self::TrueMotion => 10,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::DC),
            1 => Some(Self::Horizontal),
            2 => Some(Self::Vertical),
            3 => Some(Self::DiagonalDownLeft),
            4 => Some(Self::DiagonalDownRight),
            5 => Some(Self::VerticalRight),
            6 => Some(Self::HorizontalDown),
            7 => Some(Self::VerticalLeft),
            8 => Some(Self::HorizontalUp),
            9 => Some(Self::Planar),
            10 => Some(Self::TrueMotion),
            _ => None,
        }
    }
}

impl Default for IntraMode {
    fn default() -> Self {
        Self::DC
    }
}

pub struct IntraPredictor {
    size: usize,
}

impl IntraPredictor {
    pub fn new(size: usize) -> Self {
        Self { size: size.max(1) }
    }

    fn safe_top(&self, top: &[u8]) -> Vec<u8> {
        let mut safe = vec![128u8; self.size * 2];
        for (i, &v) in top.iter().take(self.size * 2).enumerate() {
            safe[i] = v;
        }
        safe
    }

    fn safe_left(&self, left: &[u8]) -> Vec<u8> {
        let mut safe = vec![128u8; self.size * 2];
        for (i, &v) in left.iter().take(self.size * 2).enumerate() {
            safe[i] = v;
        }
        safe
    }

    pub fn is_edge_block(bx: usize, by: usize) -> bool {
        bx == 0 || by == 0
    }
    pub fn predict(&self, mode: IntraMode, top: &[u8], left: &[u8], top_left: u8) -> Vec<u8> {
        let n = self.size;
        let mut pred = vec![128u8; n * n];
        let top = self.safe_top(top);
        let left = self.safe_left(left);

        match mode {
            IntraMode::DC => {
                let sum_top: u32 = top.iter().take(n).map(|&x| x as u32).sum();
                let sum_left: u32 = left.iter().take(n).map(|&x| x as u32).sum();
                let dc = ((sum_top + sum_left + n as u32) / (2 * n as u32)) as u8;
                pred.fill(dc);
            }
            IntraMode::Horizontal => {
                for y in 0..n {
                    let val = left[y];
                    for x in 0..n {
                        pred[y * n + x] = val;
                    }
                }
            }
            IntraMode::Vertical => {
                for y in 0..n {
                    for x in 0..n {
                        pred[y * n + x] = top[x];
                    }
                }
            }
            IntraMode::DiagonalDownLeft => {
                for y in 0..n {
                    for x in 0..n {
                        let idx = x + y + 1;
                        let a = top.get(idx).copied().unwrap_or(128);
                        let b = top.get(idx + 1).copied().unwrap_or(a);
                        pred[y * n + x] = ((a as u16 + b as u16 + 1) / 2) as u8;
                    }
                }
            }
            IntraMode::DiagonalDownRight => {
                for y in 0..n {
                    for x in 0..n {
                        let val = if x > y {
                            top.get(x - y - 1).copied().unwrap_or(128)
                        } else if x < y {
                            left.get(y - x - 1).copied().unwrap_or(128)
                        } else {
                            top_left
                        };
                        pred[y * n + x] = val;
                    }
                }
            }
            IntraMode::VerticalRight => {
                for y in 0..n {
                    for x in 0..n {
                        let idx = (x as i32) - (y as i32 / 2);
                        let val = if idx >= 0 {
                            top.get(idx as usize).copied().unwrap_or(128)
                        } else {
                            left.get((-idx - 1) as usize).copied().unwrap_or(128)
                        };
                        pred[y * n + x] = val;
                    }
                }
            }
            IntraMode::HorizontalDown => {
                for y in 0..n {
                    for x in 0..n {
                        let idx = (y as i32) - (x as i32 / 2);
                        let val = if idx >= 0 {
                            left.get(idx as usize).copied().unwrap_or(128)
                        } else {
                            top.get((-idx - 1) as usize).copied().unwrap_or(128)
                        };
                        pred[y * n + x] = val;
                    }
                }
            }
            IntraMode::VerticalLeft => {
                for y in 0..n {
                    for x in 0..n {
                        let idx = x + y / 2;
                        let a = top.get(idx).copied().unwrap_or(128);
                        let b = top.get(idx + 1).copied().unwrap_or(a);
                        pred[y * n + x] = if y % 2 == 0 {
                            a
                        } else {
                            ((a as u16 + b as u16 + 1) / 2) as u8
                        };
                    }
                }
            }
            IntraMode::HorizontalUp => {
                for y in 0..n {
                    for x in 0..n {
                        let idx = y + x / 2;
                        let a = left.get(idx).copied().unwrap_or(128);
                        let b = left.get(idx + 1).copied().unwrap_or(a);
                        pred[y * n + x] = if x % 2 == 0 {
                            a
                        } else {
                            ((a as u16 + b as u16 + 1) / 2) as u8
                        };
                    }
                }
            }
            IntraMode::Planar => {
                let tr = top.get(n - 1).copied().unwrap_or(128) as i32;
                let bl = left.get(n - 1).copied().unwrap_or(128) as i32;
                for y in 0..n {
                    for x in 0..n {
                        let t = top[x] as i32;
                        let l = left[y] as i32;
                        let h = (n - 1 - x) as i32 * l + (x + 1) as i32 * tr;
                        let v = (n - 1 - y) as i32 * t + (y + 1) as i32 * bl;
                        pred[y * n + x] = ((h + v + n as i32) / (2 * n as i32)).clamp(0, 255) as u8;
                    }
                }
            }
            IntraMode::TrueMotion => {
                for y in 0..n {
                    for x in 0..n {
                        let t = top[x] as i32;
                        let l = left[y] as i32;
                        let val = t + l - top_left as i32;
                        pred[y * n + x] = val.clamp(0, 255) as u8;
                    }
                }
            }
        }
        pred
    }

    pub fn select_best_mode(
        &self,
        block: &[u8],
        top: &[u8],
        left: &[u8],
        top_left: u8,
    ) -> (IntraMode, u64) {
        self.select_best_mode_edge(block, top, left, top_left, false, false)
    }

    pub fn select_best_mode_edge(
        &self,
        block: &[u8],
        top: &[u8],
        left: &[u8],
        top_left: u8,
        is_first_row: bool,
        is_first_col: bool,
    ) -> (IntraMode, u64) {
        let candidates = if is_first_row || is_first_col {
            &IntraMode::SAFE_EDGE[..]
        } else {
            &IntraMode::ALL[..]
        };

        let mut best_mode = IntraMode::DC;
        let mut best_sad = u64::MAX;

        for &mode in candidates {
            let pred = self.predict(mode, top, left, top_left);
            let sad: u64 = block
                .iter()
                .zip(pred.iter())
                .map(|(&a, &b)| (a as i32 - b as i32).unsigned_abs() as u64)
                .sum();
            if sad < best_sad {
                best_sad = sad;
                best_mode = mode;
            }
        }
        (best_mode, best_sad)
    }

    pub fn compute_residual(&self, block: &[u8], prediction: &[u8]) -> Vec<i16> {
        block
            .iter()
            .zip(prediction.iter())
            .map(|(&a, &b)| a as i16 - b as i16)
            .collect()
    }

    pub fn reconstruct(&self, prediction: &[u8], residual: &[i16]) -> Vec<u8> {
        prediction
            .iter()
            .zip(residual.iter())
            .map(|(&p, &r)| (p as i32 + r as i32).clamp(0, 255) as u8)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_prediction() {
        let pred = IntraPredictor::new(8);
        let top = vec![100u8; 8];
        let left = vec![100u8; 8];
        let result = pred.predict(IntraMode::DC, &top, &left, 100);
        assert!(result.iter().all(|&v| v == 100));
    }

    #[test]
    fn test_edge_block_uses_safe_modes() {
        let pred = IntraPredictor::new(8);
        let block = vec![128u8; 64];
        let top = vec![128u8; 8];
        let left = vec![128u8; 8];

        let (mode, _) = pred.select_best_mode_edge(&block, &top, &left, 128, true, false);
        assert!(
            IntraMode::SAFE_EDGE.contains(&mode),
            "Edge block should use safe mode"
        );
    }

    #[test]
    fn test_residual_roundtrip() {
        let pred = IntraPredictor::new(8);
        let block: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
        let prediction = vec![128u8; 64];

        let residual = pred.compute_residual(&block, &prediction);
        let reconstructed = pred.reconstruct(&prediction, &residual);

        assert_eq!(
            block, reconstructed,
            "Residual roundtrip should be lossless"
        );
    }
}
