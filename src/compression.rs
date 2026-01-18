use crate::error::{WkError, WkResult};

pub fn rle_compress(data: &[u8]) -> WkResult<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut compressed = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let current = data[i];
        let mut run_length = 1;

        // Max run_length is 127 to avoid encoding issue where 128 | 0x80 = 0x80
        // which would decode as count = 0x80 & 0x7F = 0
        while i + run_length < data.len() && data[i + run_length] == current && run_length < 127 {
            run_length += 1;
        }

        if run_length >= 3 {
            compressed.push((run_length as u8) | 0x80);
            compressed.push(current);
            i += run_length;
        } else {
            let literal_start = i;
            let mut literal_count = 0;

            while literal_start + literal_count < data.len() && literal_count < 127 {
                let check_pos = literal_start + literal_count;

                let mut next_run = 1;
                while check_pos + next_run < data.len()
                    && data[check_pos + next_run] == data[check_pos]
                    && next_run < 3
                {
                    next_run += 1;
                }

                if next_run >= 3 {
                    break;
                }

                literal_count += 1;
            }

            if literal_count == 0 {
                break;
            }

            compressed.push(literal_count as u8);
            compressed.extend_from_slice(&data[literal_start..literal_start + literal_count]);
            i = literal_start + literal_count;
        }
    }

    Ok(compressed)
}

pub fn rle_decompress(data: &[u8], expected_size: usize) -> WkResult<Vec<u8>> {
    let mut decompressed = Vec::with_capacity(expected_size);
    let mut i = 0;

    while i < data.len() && decompressed.len() < expected_size {
        if i >= data.len() {
            break;
        }

        let header = data[i];
        i += 1;

        if header & 0x80 != 0 {
            let count = (header & 0x7F) as usize;

            if count == 0 {
                continue;
            }

            if i >= data.len() {
                return Err(WkError::DecodingError(format!(
                    "Unexpected end of RLE data (v2) at position {}",
                    i
                )));
            }

            let value = data[i];
            i += 1;

            let actual_count = count.min(expected_size - decompressed.len());
            for _ in 0..actual_count {
                decompressed.push(value);
            }
        } else {
            let count = header as usize;

            if count == 0 {
                continue;
            }

            if i + count > data.len() {
                return Err(WkError::DecodingError(format!(
                    "Invalid literal count at position {}: {} bytes needed, {} available",
                    i - 1,
                    count,
                    data.len() - i
                )));
            }

            let actual_count = count.min(expected_size - decompressed.len());
            decompressed.extend_from_slice(&data[i..i + actual_count]);
            i += count;
        }
    }

    if decompressed.len() != expected_size {
        return Err(WkError::DecodingError(format!(
            "Size mismatch: expected {}, got {}",
            expected_size,
            decompressed.len()
        )));
    }

    Ok(decompressed)
}
