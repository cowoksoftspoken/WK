use std::f64::consts::PI;

const INV_SQRT_2: f64 = 0.7071067811865476;

fn alpha(u: usize) -> f64 {
    if u == 0 {
        INV_SQRT_2
    } else {
        1.0
    }
}

pub fn dct_8x8(block: &[i16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    let mut temp = [0.0f64; 64];
    for v in 0..8 {
        for u in 0..8 {
            let mut sum = 0.0;
            for y in 0..8 {
                for x in 0..8 {
                    let pixel = block[y * 8 + x] as f64;
                    let cos_u = ((2 * x + 1) as f64 * u as f64 * PI / 16.0).cos();
                    let cos_v = ((2 * y + 1) as f64 * v as f64 * PI / 16.0).cos();
                    sum += pixel * cos_u * cos_v;
                }
            }
            temp[v * 8 + u] = 0.25 * alpha(u) * alpha(v) * sum;
        }
    }
    for i in 0..64 {
        output[i] = temp[i].round() as i16;
    }
    output
}

pub fn idct_8x8(coeffs: &[i16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    let mut temp = [0.0f64; 64];
    for y in 0..8 {
        for x in 0..8 {
            let mut sum = 0.0;
            for v in 0..8 {
                for u in 0..8 {
                    let coeff = coeffs[v * 8 + u] as f64;
                    let cos_u = ((2 * x + 1) as f64 * u as f64 * PI / 16.0).cos();
                    let cos_v = ((2 * y + 1) as f64 * v as f64 * PI / 16.0).cos();
                    sum += alpha(u) * alpha(v) * coeff * cos_u * cos_v;
                }
            }
            temp[y * 8 + x] = 0.25 * sum;
        }
    }
    for i in 0..64 {
        output[i] = temp[i].round() as i16;
    }
    output
}

pub fn dct_16x16(block: &[i16; 256]) -> [i16; 256] {
    let mut output = [0i16; 256];
    let mut temp = [0.0f64; 256];
    for v in 0..16 {
        for u in 0..16 {
            let mut sum = 0.0;
            for y in 0..16 {
                for x in 0..16 {
                    let pixel = block[y * 16 + x] as f64;
                    let cos_u = ((2 * x + 1) as f64 * u as f64 * PI / 32.0).cos();
                    let cos_v = ((2 * y + 1) as f64 * v as f64 * PI / 32.0).cos();
                    sum += pixel * cos_u * cos_v;
                }
            }
            temp[v * 16 + u] = 0.25 * alpha(u) * alpha(v) * sum;
        }
    }
    for i in 0..256 {
        output[i] = temp[i].round() as i16;
    }
    output
}

pub fn idct_16x16(coeffs: &[i16; 256]) -> [i16; 256] {
    let mut output = [0i16; 256];
    let mut temp = [0.0f64; 256];
    for y in 0..16 {
        for x in 0..16 {
            let mut sum = 0.0;
            for v in 0..16 {
                for u in 0..16 {
                    let coeff = coeffs[v * 16 + u] as f64;
                    let cos_u = ((2 * x + 1) as f64 * u as f64 * PI / 32.0).cos();
                    let cos_v = ((2 * y + 1) as f64 * v as f64 * PI / 32.0).cos();
                    sum += alpha(u) * alpha(v) * coeff * cos_u * cos_v;
                }
            }
            temp[y * 16 + x] = 0.25 * sum;
        }
    }
    for i in 0..256 {
        output[i] = temp[i].round() as i16;
    }
    output
}

const INT_DCT_8_MATRIX: [[i32; 8]; 8] = [
    [64, 64, 64, 64, 64, 64, 64, 64],
    [89, 75, 50, 18, -18, -50, -75, -89],
    [83, 36, -36, -83, -83, -36, 36, 83],
    [75, -18, -89, -50, 50, 89, 18, -75],
    [64, -64, -64, 64, 64, -64, -64, 64],
    [50, -89, 18, 75, -75, -18, 89, -50],
    [36, -83, 83, -36, -36, 83, -83, 36],
    [18, -50, 75, -89, 89, -75, 50, -18],
];

pub fn int_dct_8x8(block: &[i16; 64]) -> [i32; 64] {
    let mut temp = [0i32; 64];
    for y in 0..8 {
        for u in 0..8 {
            let mut sum = 0i32;
            for x in 0..8 {
                sum += block[y * 8 + x] as i32 * INT_DCT_8_MATRIX[u][x];
            }
            temp[y * 8 + u] = sum;
        }
    }
    let mut output = [0i32; 64];
    for x in 0..8 {
        for v in 0..8 {
            let mut sum = 0i64;
            for y in 0..8 {
                sum += temp[y * 8 + x] as i64 * INT_DCT_8_MATRIX[v][y] as i64;
            }
            output[v * 8 + x] = ((sum + 2048) >> 12) as i32;
        }
    }
    output
}

pub fn int_idct_8x8(coeffs: &[i32; 64]) -> [i16; 64] {
    let mut temp = [0i32; 64];
    for y in 0..8 {
        for x in 0..8 {
            let mut sum = 0i64;
            for u in 0..8 {
                sum += coeffs[y * 8 + u] as i64 * INT_DCT_8_MATRIX[u][x] as i64;
            }
            temp[y * 8 + x] = ((sum + 32) >> 6) as i32;
        }
    }
    let mut output = [0i16; 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0i64;
            for v in 0..8 {
                sum += temp[v * 8 + x] as i64 * INT_DCT_8_MATRIX[v][y] as i64;
            }
            output[y * 8 + x] = ((sum + 32) >> 6) as i16;
        }
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSize {
    B8x8,
    B16x16,
    B32x32,
}

impl BlockSize {
    pub fn size(&self) -> usize {
        match self {
            Self::B8x8 => 8,
            Self::B16x16 => 16,
            Self::B32x32 => 32,
        }
    }
    pub fn coeffs(&self) -> usize {
        let s = self.size();
        s * s
    }
}

pub const ZIGZAG_8X8: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

pub fn zigzag_scan_8x8(block: &[i16; 64]) -> [i16; 64] {
    let mut out = [0i16; 64];
    for (i, &idx) in ZIGZAG_8X8.iter().enumerate() {
        out[i] = block[idx];
    }
    out
}

pub fn zigzag_unscan_8x8(scanned: &[i16; 64]) -> [i16; 64] {
    let mut out = [0i16; 64];
    for (i, &idx) in ZIGZAG_8X8.iter().enumerate() {
        out[idx] = scanned[i];
    }
    out
}

pub fn dct_8x8_fast(block: &[i16; 64]) -> [i16; 64] {
    dct_8x8(block)
}
pub fn idct_8x8_fast(coeffs: &[i16; 64]) -> [i16; 64] {
    idct_8x8(coeffs)
}
pub fn zigzag_scan(block: &[i16; 64]) -> [i16; 64] {
    zigzag_scan_8x8(block)
}
pub fn zigzag_unscan(scanned: &[i16; 64]) -> [i16; 64] {
    zigzag_unscan_8x8(scanned)
}
