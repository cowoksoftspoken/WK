use super::probability_tables::CoeffContext;

pub struct ContextModel {
    left_nonzero: Vec<bool>,
    top_nonzero: Vec<bool>,
    // block_width: usize,
    // block_height: usize,
}

impl ContextModel {
    pub fn new(width_blocks: usize, height_blocks: usize) -> Self {
        Self {
            left_nonzero: vec![false; height_blocks],
            top_nonzero: vec![false; width_blocks],
        }
    }

    pub fn reset(&mut self) {
        self.left_nonzero.fill(false);
        self.top_nonzero.fill(false);
    }

    pub fn get_context(&self, bx: usize, by: usize) -> CoeffContext {
        let left = if bx > 0 { self.left_nonzero[by] } else { false };
        let top = if by > 0 { self.top_nonzero[bx] } else { false };

        match (left, top) {
            (false, false) => CoeffContext::Zero,
            (true, false) | (false, true) => CoeffContext::One,
            (true, true) => CoeffContext::TwoPlus,
        }
    }

    pub fn update(&mut self, bx: usize, by: usize, has_nonzero: bool) {
        if by < self.left_nonzero.len() {
            self.left_nonzero[by] = has_nonzero;
        }
        if bx < self.top_nonzero.len() {
            self.top_nonzero[bx] = has_nonzero;
        }
    }

    pub fn get_coeff_context(prev_coeff: i16) -> CoeffContext {
        CoeffContext::from_prev_nonzero(prev_coeff)
    }
}

pub struct BlockContext {
    prev_coeff: i16,
    nonzero_count: u8,
}

impl BlockContext {
    pub fn new() -> Self {
        Self {
            prev_coeff: 0,
            nonzero_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.prev_coeff = 0;
        self.nonzero_count = 0;
    }

    pub fn get_context(&self) -> CoeffContext {
        CoeffContext::from_prev_nonzero(self.prev_coeff)
    }

    pub fn update(&mut self, coeff: i16) {
        self.prev_coeff = coeff;
        if coeff != 0 {
            self.nonzero_count += 1;
        }
    }

    pub fn has_nonzero(&self) -> bool {
        self.nonzero_count > 0
    }
}

impl Default for BlockContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_model() {
        let mut ctx = ContextModel::new(4, 4);

        assert_eq!(ctx.get_context(0, 0), CoeffContext::Zero);

        ctx.update(0, 0, true);
        assert_eq!(ctx.get_context(1, 0), CoeffContext::One);
        assert_eq!(ctx.get_context(0, 1), CoeffContext::One);

        ctx.update(1, 0, true);
        ctx.update(0, 1, true);
        assert_eq!(ctx.get_context(1, 1), CoeffContext::TwoPlus);
    }

    #[test]
    fn test_block_context() {
        let mut ctx = BlockContext::new();
        assert_eq!(ctx.get_context(), CoeffContext::Zero);

        ctx.update(5);
        assert_eq!(ctx.get_context(), CoeffContext::TwoPlus);

        ctx.update(1);
        assert_eq!(ctx.get_context(), CoeffContext::One);

        ctx.update(0);
        assert_eq!(ctx.get_context(), CoeffContext::Zero);
    }
}
