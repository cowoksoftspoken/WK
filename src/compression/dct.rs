use std::f64::consts::PI;

const INV_SQRT_2: f64 = 0.7071067811865475;

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

pub fn dct_8x8_fast(block: &[i16; 64]) -> [i16; 64] {
    dct_8x8(block)
}

pub fn idct_8x8_fast(coeffs: &[i16; 64]) -> [i16; 64] {
    idct_8x8(coeffs)
}

pub const ZIGZAG_ORDER: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

pub fn zigzag_scan(block: &[i16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    for (i, &idx) in ZIGZAG_ORDER.iter().enumerate() {
        output[i] = block[idx];
    }
    output
}

pub fn zigzag_unscan(scanned: &[i16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    for (i, &idx) in ZIGZAG_ORDER.iter().enumerate() {
        output[idx] = scanned[i];
    }
    output
}
