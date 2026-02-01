pub const VP8_ZIGZAG_4X4: [usize; 16] = [0, 1, 5, 6, 2, 4, 7, 12, 3, 8, 11, 13, 9, 10, 14, 15];

pub const VP8_ZIGZAG_4X4_INVERSE: [usize; 16] =
    [0, 1, 4, 8, 5, 2, 3, 6, 9, 12, 13, 10, 7, 11, 14, 15];

pub const VP8_ZIGZAG_8X8: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

pub fn build_inverse_8x8() -> [usize; 64] {
    let mut inv = [0usize; 64];
    for (i, &v) in VP8_ZIGZAG_8X8.iter().enumerate() {
        inv[v] = i;
    }
    inv
}

pub const VP8_ZIGZAG_8X8_INVERSE: [usize; 64] = [
    0, 1, 5, 6, 14, 15, 27, 28, 2, 4, 7, 13, 16, 26, 29, 42, 3, 8, 12, 17, 25, 30, 41, 43, 9, 11,
    18, 24, 31, 40, 44, 53, 10, 19, 23, 32, 39, 45, 52, 54, 20, 22, 33, 38, 46, 51, 55, 60, 21, 34,
    37, 47, 50, 56, 59, 61, 35, 36, 48, 49, 57, 58, 62, 63,
];

pub fn coeff_index_to_band(index: usize) -> usize {
    match index {
        0 => 0,
        1..=3 => 1,
        4..=6 => 2,
        7..=10 => 3,
        11..=15 => 4,
        16..=24 => 5,
        25..=39 => 6,
        _ => 7,
    }
}

pub fn coeff_index_to_band_4x4(index: usize) -> usize {
    match index {
        0 => 0,
        1 => 1,
        2..=3 => 2,
        4..=5 => 3,
        6..=8 => 4,
        9..=11 => 5,
        12..=13 => 6,
        _ => 7,
    }
}

pub fn zigzag_scan_4x4(block: &[i16; 16]) -> [i16; 16] {
    let mut out = [0i16; 16];
    for (i, &idx) in VP8_ZIGZAG_4X4.iter().enumerate() {
        out[i] = block[idx];
    }
    out
}

pub fn zigzag_unscan_4x4(scanned: &[i16; 16]) -> [i16; 16] {
    let mut out = [0i16; 16];
    for (i, &idx) in VP8_ZIGZAG_4X4.iter().enumerate() {
        out[idx] = scanned[i];
    }
    out
}

pub fn zigzag_scan_8x8(block: &[i16; 64]) -> [i16; 64] {
    let mut out = [0i16; 64];
    for (i, &idx) in VP8_ZIGZAG_8X8.iter().enumerate() {
        out[i] = block[idx];
    }
    out
}

pub fn zigzag_unscan_8x8(scanned: &[i16; 64]) -> [i16; 64] {
    let mut out = [0i16; 64];
    for (i, &idx) in VP8_ZIGZAG_8X8.iter().enumerate() {
        out[idx] = scanned[i];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_4x4_roundtrip() {
        let block: [i16; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let scanned = zigzag_scan_4x4(&block);
        let unscanned = zigzag_unscan_4x4(&scanned);
        assert_eq!(block, unscanned);
    }

    #[test]
    fn test_zigzag_8x8_roundtrip() {
        let mut block = [0i16; 64];
        for i in 0..64 {
            block[i] = i as i16;
        }
        let scanned = zigzag_scan_8x8(&block);
        let unscanned = zigzag_unscan_8x8(&scanned);
        assert_eq!(block, unscanned);
    }

    #[test]
    fn test_band_mapping() {
        assert_eq!(coeff_index_to_band(0), 0); // DC
        assert_eq!(coeff_index_to_band(1), 1); // Low
        assert_eq!(coeff_index_to_band(5), 2); // Mid
        assert_eq!(coeff_index_to_band(10), 3);
        assert_eq!(coeff_index_to_band(50), 7); // High
    }
}
