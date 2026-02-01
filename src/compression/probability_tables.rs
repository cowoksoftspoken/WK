pub const NUM_BLOCK_TYPES: usize = 4;
pub const NUM_COEFF_BANDS: usize = 8;
pub const NUM_PREV_COEFF_CONTEXTS: usize = 3;
pub const NUM_ENTROPY_NODES: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Y1 = 0,
    Y2 = 1,
    UV = 2,
    Y1AC = 3,
}

impl BlockType {
    pub fn from_usize(v: usize) -> Self {
        match v {
            0 => BlockType::Y1,
            1 => BlockType::Y2,
            2 => BlockType::UV,
            _ => BlockType::Y1AC,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoeffContext {
    Zero = 0,
    One = 1,
    TwoPlus = 2,
}

impl CoeffContext {
    pub fn from_prev_nonzero(prev_nonzero: i16) -> Self {
        match prev_nonzero.abs() {
            0 => CoeffContext::Zero,
            1 => CoeffContext::One,
            _ => CoeffContext::TwoPlus,
        }
    }
}

pub const DEFAULT_COEFF_PROBS: [[[[u8; NUM_ENTROPY_NODES]; NUM_PREV_COEFF_CONTEXTS];
    NUM_COEFF_BANDS]; NUM_BLOCK_TYPES] = [
    // BlockType::Y1
    [
        // Band 0 (DC)
        [
            [128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128],
            [128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128],
            [128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 1 (Low freq)
        [
            [176, 128, 128, 167, 128, 128, 128, 128, 128, 128, 128],
            [167, 128, 128, 154, 128, 128, 128, 128, 128, 128, 128],
            [154, 128, 128, 141, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 2
        [
            [186, 128, 128, 177, 128, 128, 128, 128, 128, 128, 128],
            [177, 128, 128, 166, 128, 128, 128, 128, 128, 128, 128],
            [166, 128, 128, 154, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 3
        [
            [190, 128, 128, 183, 128, 128, 128, 128, 128, 128, 128],
            [183, 128, 128, 175, 128, 128, 128, 128, 128, 128, 128],
            [175, 128, 128, 166, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 4
        [
            [193, 128, 128, 188, 128, 128, 128, 128, 128, 128, 128],
            [188, 128, 128, 182, 128, 128, 128, 128, 128, 128, 128],
            [182, 128, 128, 175, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 5
        [
            [195, 128, 128, 191, 128, 128, 128, 128, 128, 128, 128],
            [191, 128, 128, 186, 128, 128, 128, 128, 128, 128, 128],
            [186, 128, 128, 181, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 6
        [
            [197, 128, 128, 194, 128, 128, 128, 128, 128, 128, 128],
            [194, 128, 128, 190, 128, 128, 128, 128, 128, 128, 128],
            [190, 128, 128, 185, 128, 128, 128, 128, 128, 128, 128],
        ],
        // Band 7 (High freq)
        [
            [199, 128, 128, 196, 128, 128, 128, 128, 128, 128, 128],
            [196, 128, 128, 193, 128, 128, 128, 128, 128, 128, 128],
            [193, 128, 128, 189, 128, 128, 128, 128, 128, 128, 128],
        ],
    ],
    // BlockType::Y2 (same as Y1 for simplicity)
    [[[128; NUM_ENTROPY_NODES]; NUM_PREV_COEFF_CONTEXTS]; NUM_COEFF_BANDS],
    // BlockType::UV
    [[[128; NUM_ENTROPY_NODES]; NUM_PREV_COEFF_CONTEXTS]; NUM_COEFF_BANDS],
    // BlockType::Y1AC
    [[[128; NUM_ENTROPY_NODES]; NUM_PREV_COEFF_CONTEXTS]; NUM_COEFF_BANDS],
];

pub struct CoeffProbabilities {
    probs: [[[[u8; NUM_ENTROPY_NODES]; NUM_PREV_COEFF_CONTEXTS]; NUM_COEFF_BANDS]; NUM_BLOCK_TYPES],
}

impl CoeffProbabilities {
    pub fn new() -> Self {
        Self {
            probs: DEFAULT_COEFF_PROBS,
        }
    }

    pub fn get(
        &self,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
        node: usize,
    ) -> u8 {
        let bt = block_type as usize;
        let b = band.min(NUM_COEFF_BANDS - 1);
        let c = context as usize;
        let n = node.min(NUM_ENTROPY_NODES - 1);
        self.probs[bt][b][c][n]
    }

    pub fn set(
        &mut self,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
        node: usize,
        prob: u8,
    ) {
        let bt = block_type as usize;
        let b = band.min(NUM_COEFF_BANDS - 1);
        let c = context as usize;
        let n = node.min(NUM_ENTROPY_NODES - 1);
        self.probs[bt][b][c][n] = prob;
    }

    pub fn reset(&mut self) {
        self.probs = DEFAULT_COEFF_PROBS;
    }
}

impl Default for CoeffProbabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coeff_context() {
        assert_eq!(CoeffContext::from_prev_nonzero(0), CoeffContext::Zero);
        assert_eq!(CoeffContext::from_prev_nonzero(1), CoeffContext::One);
        assert_eq!(CoeffContext::from_prev_nonzero(-1), CoeffContext::One);
        assert_eq!(CoeffContext::from_prev_nonzero(5), CoeffContext::TwoPlus);
    }

    #[test]
    fn test_prob_get_set() {
        let mut probs = CoeffProbabilities::new();
        let bt = BlockType::Y1;
        probs.set(bt, 0, CoeffContext::Zero, 0, 200);
        assert_eq!(probs.get(bt, 0, CoeffContext::Zero, 0), 200);
    }

    #[test]
    fn test_default_probs_range() {
        let probs = CoeffProbabilities::new();
        for bt in 0..NUM_BLOCK_TYPES {
            for band in 0..NUM_COEFF_BANDS {
                for ctx in 0..NUM_PREV_COEFF_CONTEXTS {
                    for node in 0..NUM_ENTROPY_NODES {
                        let p = probs.probs[bt][band][ctx][node];
                        assert!(p >= 1 && p <= 254, "Prob out of range: {}", p);
                    }
                }
            }
        }
    }
}
