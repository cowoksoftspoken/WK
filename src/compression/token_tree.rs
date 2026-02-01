use super::probability_tables::{BlockType, CoeffContext, CoeffProbabilities};
use super::vp8_coder::{RangeDecoder, RangeEncoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoeffToken {
    EOB,
    Zero,
    One,
    Two,
    Three,
    Four,
    Cat1,
    Cat2,
    Cat3,
    Cat4,
    Cat5,
    Cat6,
}

impl CoeffToken {
    pub fn from_value(abs_val: u16) -> Self {
        match abs_val {
            0 => CoeffToken::Zero,
            1 => CoeffToken::One,
            2 => CoeffToken::Two,
            3 => CoeffToken::Three,
            4 => CoeffToken::Four,
            5..=6 => CoeffToken::Cat1,
            7..=10 => CoeffToken::Cat2,
            11..=18 => CoeffToken::Cat3,
            19..=34 => CoeffToken::Cat4,
            35..=66 => CoeffToken::Cat5,
            _ => CoeffToken::Cat6,
        }
    }

    pub fn base_value(&self) -> u16 {
        match self {
            CoeffToken::EOB => 0,
            CoeffToken::Zero => 0,
            CoeffToken::One => 1,
            CoeffToken::Two => 2,
            CoeffToken::Three => 3,
            CoeffToken::Four => 4,
            CoeffToken::Cat1 => 5,
            CoeffToken::Cat2 => 7,
            CoeffToken::Cat3 => 11,
            CoeffToken::Cat4 => 19,
            CoeffToken::Cat5 => 35,
            CoeffToken::Cat6 => 67,
        }
    }

    pub fn extra_bits(&self) -> u8 {
        match self {
            CoeffToken::EOB
            | CoeffToken::Zero
            | CoeffToken::One
            | CoeffToken::Two
            | CoeffToken::Three
            | CoeffToken::Four => 0,
            CoeffToken::Cat1 => 1,
            CoeffToken::Cat2 => 2,
            CoeffToken::Cat3 => 3,
            CoeffToken::Cat4 => 4,
            CoeffToken::Cat5 => 5,
            CoeffToken::Cat6 => 11,
        }
    }
}

const CAT_PROBS: &[&[u8]] = &[
    &[159],                                                   // Cat1
    &[165, 145],                                              // Cat2
    &[173, 148, 140],                                         // Cat3
    &[176, 155, 140, 135],                                    // Cat4
    &[180, 157, 141, 134, 130],                               // Cat5
    &[254, 254, 243, 230, 196, 177, 153, 140, 133, 130, 129], // Cat6
];

pub struct TokenEncoder<'a> {
    encoder: &'a mut RangeEncoder,
    probs: &'a CoeffProbabilities,
}

impl<'a> TokenEncoder<'a> {
    pub fn new(encoder: &'a mut RangeEncoder, probs: &'a CoeffProbabilities) -> Self {
        Self { encoder, probs }
    }

    fn encode_token(
        &mut self,
        token: CoeffToken,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
    ) {
        let _prob_base = self.probs.get(block_type, band, context, 0);

        match token {
            CoeffToken::EOB => {
                self.encoder.encode(true, 252);
            }
            CoeffToken::Zero => {
                self.encoder.encode(false, 252);
                self.encoder.encode(false, 180);
            }
            CoeffToken::One => {
                self.encoder.encode(false, 252);
                self.encoder.encode(true, 180);
                self.encoder.encode(false, 165);
            }
            CoeffToken::Two => {
                self.encoder.encode(false, 252);
                self.encoder.encode(true, 180);
                self.encoder.encode(true, 165);
                self.encoder.encode(false, 145);
            }
            CoeffToken::Three => {
                self.encoder.encode(false, 252);
                self.encoder.encode(true, 180);
                self.encoder.encode(true, 165);
                self.encoder.encode(true, 145);
                self.encoder.encode(false, 140);
            }
            CoeffToken::Four => {
                self.encoder.encode(false, 252);
                self.encoder.encode(true, 180);
                self.encoder.encode(true, 165);
                self.encoder.encode(true, 145);
                self.encoder.encode(true, 140);
                self.encoder.encode(false, 135);
            }
            CoeffToken::Cat1
            | CoeffToken::Cat2
            | CoeffToken::Cat3
            | CoeffToken::Cat4
            | CoeffToken::Cat5
            | CoeffToken::Cat6 => {
                self.encoder.encode(false, 252);
                self.encoder.encode(true, 180);
                self.encoder.encode(true, 165);
                self.encoder.encode(true, 145);
                self.encoder.encode(true, 140);
                self.encoder.encode(true, 135);

                let cat_idx = match token {
                    CoeffToken::Cat1 => 0,
                    CoeffToken::Cat2 => 1,
                    CoeffToken::Cat3 => 2,
                    CoeffToken::Cat4 => 3,
                    CoeffToken::Cat5 => 4,
                    CoeffToken::Cat6 => 5,
                    _ => 0,
                };

                #[allow(unused)]
                for i in 0..cat_idx {
                    self.encoder.encode(true, 128);
                }
                if cat_idx < 5 {
                    self.encoder.encode(false, 128);
                }
            }
        }
    }

    fn encode_extra(&mut self, token: CoeffToken, value: u16) {
        let extra = value - token.base_value();
        let num_bits = token.extra_bits();

        if num_bits == 0 {
            return;
        }

        let cat_idx = match token {
            CoeffToken::Cat1 => 0,
            CoeffToken::Cat2 => 1,
            CoeffToken::Cat3 => 2,
            CoeffToken::Cat4 => 3,
            CoeffToken::Cat5 => 4,
            CoeffToken::Cat6 => 5,
            _ => return,
        };

        let probs = CAT_PROBS[cat_idx];
        for i in 0..num_bits as usize {
            let bit = ((extra >> (num_bits as usize - 1 - i)) & 1) != 0;
            let prob = if i < probs.len() { probs[i] } else { 128 };
            self.encoder.encode(bit, prob as u32);
        }
    }

    pub fn encode_coeff(
        &mut self,
        value: i16,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
    ) {
        let abs_val = value.unsigned_abs();
        let token = CoeffToken::from_value(abs_val);

        self.encode_token(token, block_type, band, context);

        if token != CoeffToken::Zero && token != CoeffToken::EOB {
            self.encode_extra(token, abs_val);
            self.encoder.encode_bit(value < 0);
        }
    }

    pub fn encode_eob(&mut self, block_type: BlockType, band: usize, context: CoeffContext) {
        self.encode_token(CoeffToken::EOB, block_type, band, context);
    }
}

pub struct TokenDecoder<'a> {
    decoder: &'a mut RangeDecoder,
    probs: &'a CoeffProbabilities,
}

impl<'a> TokenDecoder<'a> {
    pub fn new(decoder: &'a mut RangeDecoder, probs: &'a CoeffProbabilities) -> Self {
        Self { decoder, probs }
    }

    fn decode_token(
        &mut self,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
    ) -> CoeffToken {
        let _prob_base = self.probs.get(block_type, band, context, 0);

        if self.decoder.decode(252) {
            return CoeffToken::EOB;
        }

        if !self.decoder.decode(180) {
            return CoeffToken::Zero;
        }

        if !self.decoder.decode(165) {
            return CoeffToken::One;
        }

        if !self.decoder.decode(145) {
            return CoeffToken::Two;
        }

        if !self.decoder.decode(140) {
            return CoeffToken::Three;
        }

        if !self.decoder.decode(135) {
            return CoeffToken::Four;
        }

        let mut cat = 0;
        while cat < 5 && self.decoder.decode(128) {
            cat += 1;
        }

        match cat {
            0 => CoeffToken::Cat1,
            1 => CoeffToken::Cat2,
            2 => CoeffToken::Cat3,
            3 => CoeffToken::Cat4,
            4 => CoeffToken::Cat5,
            _ => CoeffToken::Cat6,
        }
    }

    fn decode_extra(&mut self, token: CoeffToken) -> u16 {
        let num_bits = token.extra_bits();
        if num_bits == 0 {
            return 0;
        }

        let cat_idx = match token {
            CoeffToken::Cat1 => 0,
            CoeffToken::Cat2 => 1,
            CoeffToken::Cat3 => 2,
            CoeffToken::Cat4 => 3,
            CoeffToken::Cat5 => 4,
            CoeffToken::Cat6 => 5,
            _ => return 0,
        };

        let probs = CAT_PROBS[cat_idx];
        let mut extra = 0u16;
        for i in 0..num_bits as usize {
            let prob = if i < probs.len() { probs[i] } else { 128 };
            let bit = self.decoder.decode(prob as u32);
            extra = (extra << 1) | (bit as u16);
        }
        extra
    }

    pub fn decode_coeff(
        &mut self,
        block_type: BlockType,
        band: usize,
        context: CoeffContext,
    ) -> Option<i16> {
        let token = self.decode_token(block_type, band, context);

        if token == CoeffToken::EOB {
            return None;
        }

        if token == CoeffToken::Zero {
            return Some(0);
        }

        let base = token.base_value();
        let extra = self.decode_extra(token);
        let abs_val = base + extra;
        let sign = self.decoder.decode_bit();

        Some(if sign {
            -(abs_val as i16)
        } else {
            abs_val as i16
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_from_value() {
        assert_eq!(CoeffToken::from_value(0), CoeffToken::Zero);
        assert_eq!(CoeffToken::from_value(1), CoeffToken::One);
        assert_eq!(CoeffToken::from_value(4), CoeffToken::Four);
        assert_eq!(CoeffToken::from_value(5), CoeffToken::Cat1);
        assert_eq!(CoeffToken::from_value(100), CoeffToken::Cat6);
    }

    #[test]
    fn test_coeff_roundtrip() {
        let probs = CoeffProbabilities::new();
        let test_values: [i16; 10] = [0, 1, -1, 2, 4, 5, 10, 20, 50, -100];

        for &val in &test_values {
            let mut encoder = RangeEncoder::new();
            {
                let mut tok_enc = TokenEncoder::new(&mut encoder, &probs);
                tok_enc.encode_coeff(val, BlockType::Y1, 0, CoeffContext::Zero);
            }
            let data = encoder.finish();

            let mut decoder = RangeDecoder::new(data);
            let mut tok_dec = TokenDecoder::new(&mut decoder, &probs);
            let decoded = tok_dec.decode_coeff(BlockType::Y1, 0, CoeffContext::Zero);

            assert_eq!(decoded, Some(val), "Failed for value {}", val);
        }
    }
}
