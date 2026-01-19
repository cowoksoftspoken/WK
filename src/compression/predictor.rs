use crate::error::WkResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PredictorType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

impl PredictorType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Sub,
            2 => Self::Up,
            3 => Self::Average,
            4 => Self::Paeth,
            _ => Self::None,
        }
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

pub fn apply_predictor(
    data: &[u8],
    width: usize,
    height: usize,
    channels: usize,
    predictor: PredictorType,
) -> Vec<u8> {
    let stride = width * channels;
    let mut filtered = vec![0u8; data.len() + height];
    let mut out_idx = 0;

    for y in 0..height {
        filtered[out_idx] = predictor as u8;
        out_idx += 1;

        for x in 0..stride {
            let idx = y * stride + x;
            let raw = data[idx];

            let left = if x >= channels {
                data[idx - channels]
            } else {
                0
            };
            let up = if y > 0 { data[idx - stride] } else { 0 };
            let up_left = if x >= channels && y > 0 {
                data[idx - stride - channels]
            } else {
                0
            };

            let prediction = match predictor {
                PredictorType::None => 0,
                PredictorType::Sub => left,
                PredictorType::Up => up,
                PredictorType::Average => ((left as u16 + up as u16) / 2) as u8,
                PredictorType::Paeth => paeth_predictor(left, up, up_left),
            };

            filtered[out_idx] = raw.wrapping_sub(prediction);
            out_idx += 1;
        }
    }

    filtered
}

pub fn reverse_predictor(
    filtered: &[u8],
    width: usize,
    height: usize,
    channels: usize,
) -> WkResult<Vec<u8>> {
    let stride = width * channels;
    let mut data = vec![0u8; width * height * channels];
    let mut in_idx = 0;

    for y in 0..height {
        let predictor = PredictorType::from_u8(filtered[in_idx]);
        in_idx += 1;

        for x in 0..stride {
            let idx = y * stride + x;
            let delta = filtered[in_idx];
            in_idx += 1;

            let left = if x >= channels {
                data[idx - channels]
            } else {
                0
            };
            let up = if y > 0 { data[idx - stride] } else { 0 };
            let up_left = if x >= channels && y > 0 {
                data[idx - stride - channels]
            } else {
                0
            };

            let prediction = match predictor {
                PredictorType::None => 0,
                PredictorType::Sub => left,
                PredictorType::Up => up,
                PredictorType::Average => ((left as u16 + up as u16) / 2) as u8,
                PredictorType::Paeth => paeth_predictor(left, up, up_left),
            };

            data[idx] = delta.wrapping_add(prediction);
        }
    }

    Ok(data)
}

pub fn select_optimal_predictor(
    row: &[u8],
    prev_row: Option<&[u8]>,
    channels: usize,
) -> PredictorType {
    let predictors = [
        PredictorType::None,
        PredictorType::Sub,
        PredictorType::Up,
        PredictorType::Average,
        PredictorType::Paeth,
    ];

    let mut best = PredictorType::None;
    let mut best_score = usize::MAX;

    for &predictor in &predictors {
        let mut score = 0usize;

        for (x, &raw) in row.iter().enumerate() {
            let left = if x >= channels { row[x - channels] } else { 0 };
            let up = prev_row.map(|r| r[x]).unwrap_or(0);
            let up_left = if x >= channels {
                prev_row.map(|r| r[x - channels]).unwrap_or(0)
            } else {
                0
            };

            let prediction = match predictor {
                PredictorType::None => 0,
                PredictorType::Sub => left,
                PredictorType::Up => up,
                PredictorType::Average => ((left as u16 + up as u16) / 2) as u8,
                PredictorType::Paeth => paeth_predictor(left, up, up_left),
            };

            let delta = raw.wrapping_sub(prediction);
            let abs_delta = if delta > 127 {
                256 - delta as usize
            } else {
                delta as usize
            };
            score += abs_delta;
        }

        if score < best_score {
            best_score = score;
            best = predictor;
        }
    }

    best
}

pub fn apply_optimal_predictor(
    data: &[u8],
    width: usize,
    height: usize,
    channels: usize,
) -> Vec<u8> {
    let stride = width * channels;
    let mut filtered = vec![0u8; data.len() + height];
    let mut out_idx = 0;

    for y in 0..height {
        let row_start = y * stride;
        let row = &data[row_start..row_start + stride];
        let prev_row = if y > 0 {
            Some(&data[(y - 1) * stride..(y - 1) * stride + stride])
        } else {
            None
        };

        let predictor = select_optimal_predictor(row, prev_row, channels);
        filtered[out_idx] = predictor as u8;
        out_idx += 1;

        for x in 0..stride {
            let raw = row[x];
            let left = if x >= channels { row[x - channels] } else { 0 };
            let up = prev_row.map(|r| r[x]).unwrap_or(0);
            let up_left = if x >= channels {
                prev_row.map(|r| r[x - channels]).unwrap_or(0)
            } else {
                0
            };

            let prediction = match predictor {
                PredictorType::None => 0,
                PredictorType::Sub => left,
                PredictorType::Up => up,
                PredictorType::Average => ((left as u16 + up as u16) / 2) as u8,
                PredictorType::Paeth => paeth_predictor(left, up, up_left),
            };

            filtered[out_idx] = raw.wrapping_sub(prediction);
            out_idx += 1;
        }
    }

    filtered
}
