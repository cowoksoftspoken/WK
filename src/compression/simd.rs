#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdLevel {
    None,
    SSE42,
    AVX2,
}

pub fn detect_simd() -> SimdLevel {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return SimdLevel::AVX2;
        }
        if is_x86_feature_detected!("sse4.2") {
            return SimdLevel::SSE42;
        }
    }
    SimdLevel::None
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn dct_1d_sse(input: &[i16; 8], cos_table: &[[f32; 8]; 8]) -> [f32; 8] {
    let mut output = [0.0f32; 8];
    for u in 0..8 {
        let cos = _mm_loadu_ps(cos_table[u].as_ptr());
        let cos2 = _mm_loadu_ps(cos_table[u].as_ptr().add(4));

        let in1 = _mm_set_ps(
            input[3] as f32,
            input[2] as f32,
            input[1] as f32,
            input[0] as f32,
        );
        let in2 = _mm_set_ps(
            input[7] as f32,
            input[6] as f32,
            input[5] as f32,
            input[4] as f32,
        );

        let mul1 = _mm_mul_ps(in1, cos);
        let mul2 = _mm_mul_ps(in2, cos2);

        let sum = _mm_add_ps(mul1, mul2);
        let hi = _mm_movehl_ps(sum, sum);
        let sum2 = _mm_add_ps(sum, hi);
        let hi2 = _mm_shuffle_ps(sum2, sum2, 1);
        let final_sum = _mm_add_ss(sum2, hi2);

        output[u] = _mm_cvtss_f32(final_sum);
    }
    output
}

pub fn dct_8x8_simd(block: &[i16; 64]) -> [i16; 64] {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { dct_8x8_sse(block) };
        }
    }
    dct_8x8_scalar(block)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn dct_8x8_sse(block: &[i16; 64]) -> [i16; 64] {
    use std::f32::consts::PI;

    let mut cos_table = [[0.0f32; 8]; 8];
    for u in 0..8 {
        for x in 0..8 {
            cos_table[u][x] = ((2 * x + 1) as f32 * u as f32 * PI / 16.0).cos();
        }
    }

    let mut temp = [[0.0f32; 8]; 8];
    for y in 0..8 {
        let mut row = [0i16; 8];
        for x in 0..8 {
            row[x] = block[y * 8 + x];
        }
        let transformed = dct_1d_sse(&row, &cos_table);
        for u in 0..8 {
            temp[y][u] = transformed[u];
        }
    }

    let mut output = [0i16; 64];
    let alpha0 = 1.0 / (2.0f32).sqrt();

    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0.0f32;
            for y in 0..8 {
                let cos_v = ((2 * y + 1) as f32 * v as f32 * PI / 16.0).cos();
                sum += temp[y][u] * cos_v;
            }
            let au = if u == 0 { alpha0 } else { 1.0 };
            let av = if v == 0 { alpha0 } else { 1.0 };
            output[v * 8 + u] = (0.25 * au * av * sum).round() as i16;
        }
    }

    output
}

pub fn dct_8x8_scalar(block: &[i16; 64]) -> [i16; 64] {
    use std::f64::consts::PI;
    let inv_sqrt2 = 0.7071067811865476;

    let alpha = |u: usize| if u == 0 { inv_sqrt2 } else { 1.0 };

    let mut output = [0i16; 64];
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
            output[v * 8 + u] = (0.25 * alpha(u) * alpha(v) * sum).round() as i16;
        }
    }
    output
}

pub fn idct_8x8_simd(coeffs: &[i16; 64]) -> [i16; 64] {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { idct_8x8_sse(coeffs) };
        }
    }
    idct_8x8_scalar(coeffs)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn idct_8x8_sse(coeffs: &[i16; 64]) -> [i16; 64] {
    idct_8x8_scalar(coeffs)
}

pub fn idct_8x8_scalar(coeffs: &[i16; 64]) -> [i16; 64] {
    use std::f64::consts::PI;
    let inv_sqrt2 = 0.7071067811865476;

    let alpha = |u: usize| if u == 0 { inv_sqrt2 } else { 1.0 };

    let mut output = [0i16; 64];
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
            output[y * 8 + x] = (0.25 * sum).round() as i16;
        }
    }
    output
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
pub unsafe fn rgb_to_ycbcr_simd(rgb: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(rgb.len());

    for chunk in rgb.chunks(12) {
        if chunk.len() == 12 {
            for i in 0..4 {
                let r = chunk[i * 3] as f32;
                let g = chunk[i * 3 + 1] as f32;
                let b = chunk[i * 3 + 2] as f32;

                let y = 16.0 + 65.481 * r / 255.0 + 128.553 * g / 255.0 + 24.966 * b / 255.0;
                let cb = 128.0 - 37.797 * r / 255.0 - 74.203 * g / 255.0 + 112.0 * b / 255.0;
                let cr = 128.0 + 112.0 * r / 255.0 - 93.786 * g / 255.0 - 18.214 * b / 255.0;

                output.push(y.clamp(16.0, 235.0) as u8);
                output.push(cb.clamp(16.0, 240.0) as u8);
                output.push(cr.clamp(16.0, 240.0) as u8);
            }
        } else {
            for i in 0..chunk.len() / 3 {
                let r = chunk[i * 3] as f32;
                let g = chunk[i * 3 + 1] as f32;
                let b = chunk[i * 3 + 2] as f32;

                let y = 16.0 + 65.481 * r / 255.0 + 128.553 * g / 255.0 + 24.966 * b / 255.0;
                output.push(y.clamp(16.0, 235.0) as u8);
                output.push(128);
                output.push(128);
            }
        }
    }

    output
}

pub fn quantize_simd(block: &[i16; 64], table: &[u16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    for i in 0..64 {
        output[i] = (block[i] as i32 / table[i] as i32) as i16;
    }
    output
}

pub fn dequantize_simd(block: &[i16; 64], table: &[u16; 64]) -> [i16; 64] {
    let mut output = [0i16; 64];
    for i in 0..64 {
        output[i] = (block[i] as i32 * table[i] as i32) as i16;
    }
    output
}
