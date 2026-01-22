#[derive(Debug, Clone)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data_offset: u64,
    pub data_size: u32,
}

#[derive(Debug, Clone)]
pub struct TileGrid {
    pub tile_width: u32,
    pub tile_height: u32,
    pub cols: u32,
    pub rows: u32,
    pub tiles: Vec<Tile>,
}

impl TileGrid {
    pub fn new(image_width: u32, image_height: u32, tile_size: u32) -> Self {
        let cols = (image_width + tile_size - 1) / tile_size;
        let rows = (image_height + tile_size - 1) / tile_size;
        let mut tiles = Vec::with_capacity((cols * rows) as usize);

        for row in 0..rows {
            for col in 0..cols {
                let x = col * tile_size;
                let y = row * tile_size;
                let w = (image_width - x).min(tile_size);
                let h = (image_height - y).min(tile_size);
                tiles.push(Tile {
                    x,
                    y,
                    width: w,
                    height: h,
                    data_offset: 0,
                    data_size: 0,
                });
            }
        }

        Self {
            tile_width: tile_size,
            tile_height: tile_size,
            cols,
            rows,
            tiles,
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        let col = x / self.tile_width;
        let row = y / self.tile_height;
        if col < self.cols && row < self.rows {
            Some(&self.tiles[(row * self.cols + col) as usize])
        } else {
            None
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

pub struct ScanOrder {
    order: Vec<usize>,
}

impl ScanOrder {
    pub fn sequential(count: usize) -> Self {
        Self {
            order: (0..count).collect(),
        }
    }

    pub fn dc_first_8x8() -> Self {
        let mut order = Vec::with_capacity(64);
        order.push(0);
        for i in 1..64 {
            order.push(i);
        }
        Self { order }
    }

    pub fn progressive_8x8() -> Self {
        let dc = [0usize];
        let ac_low: [usize; 15] = [1, 2, 3, 8, 9, 10, 16, 17, 18, 24, 25, 32, 33, 40, 48];
        let ac_high: Vec<usize> = (0..64)
            .filter(|&i| i != 0 && !ac_low.contains(&i))
            .collect();

        let mut order = Vec::with_capacity(64);
        order.extend_from_slice(&dc);
        order.extend_from_slice(&ac_low);
        order.extend_from_slice(&ac_high);
        Self { order }
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.order.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanPass {
    DC,
    ACLow,
    ACHigh,
    All,
}

pub fn reorder_coefficients(coeffs: &[i16; 64], pass: ScanPass) -> Vec<i16> {
    match pass {
        ScanPass::DC => vec![coeffs[0]],
        ScanPass::ACLow => {
            let low_indices = [1, 2, 3, 8, 9, 10, 16, 17, 18, 24, 25, 32, 33, 40, 48];
            low_indices.iter().map(|&i| coeffs[i]).collect()
        }
        ScanPass::ACHigh => (0..64)
            .filter(|&i| {
                i != 0 && ![1, 2, 3, 8, 9, 10, 16, 17, 18, 24, 25, 32, 33, 40, 48].contains(&i)
            })
            .map(|i| coeffs[i])
            .collect(),
        ScanPass::All => coeffs.to_vec(),
    }
}

pub fn merge_progressive_coefficients(dc: &[i16], ac_low: &[i16], ac_high: &[i16]) -> [i16; 64] {
    let mut coeffs = [0i16; 64];
    if !dc.is_empty() {
        coeffs[0] = dc[0];
    }

    let low_indices = [1, 2, 3, 8, 9, 10, 16, 17, 18, 24, 25, 32, 33, 40, 48];
    for (i, &idx) in low_indices.iter().enumerate() {
        if i < ac_low.len() {
            coeffs[idx] = ac_low[i];
        }
    }

    let high_indices: Vec<usize> = (0..64)
        .filter(|&i| i != 0 && !low_indices.contains(&i))
        .collect();
    for (i, &idx) in high_indices.iter().enumerate() {
        if i < ac_high.len() {
            coeffs[idx] = ac_high[i];
        }
    }

    coeffs
}

pub const RESYNC_MARKER: [u8; 4] = [0xFF, 0xD0, 0x00, 0x00];

pub fn insert_resync_marker(data: &mut Vec<u8>, interval: usize) {
    let mut positions = Vec::new();
    let mut i = interval;
    while i < data.len() {
        positions.push(i);
        i += interval + 4;
    }
    for &pos in positions.iter().rev() {
        if pos <= data.len() {
            data.splice(pos..pos, RESYNC_MARKER.iter().copied());
        }
    }
}

pub fn find_resync_marker(data: &[u8], start: usize) -> Option<usize> {
    for i in start..data.len().saturating_sub(3) {
        if data[i..i + 4] == RESYNC_MARKER {
            return Some(i);
        }
    }
    None
}
