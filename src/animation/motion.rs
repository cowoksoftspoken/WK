#[derive(Debug, Clone, Copy)]
pub struct MotionVector {
    pub x: i16,
    pub y: i16,
}

impl MotionVector {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SearchPattern {
    FullSearch,
    Diamond,
    Hexagon,
    ThreeStep,
}

pub struct MotionEstimator {
    search_range: i16,
    pattern: SearchPattern,
    subpixel: bool,
}

impl MotionEstimator {
    pub fn new(search_range: i16) -> Self {
        Self {
            search_range,
            pattern: SearchPattern::Diamond,
            subpixel: true,
        }
    }

    pub fn estimate(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
    ) -> MotionVector {
        match self.pattern {
            SearchPattern::FullSearch => self.full_search(
                current, reference, width, height, block_x, block_y, block_size,
            ),
            SearchPattern::Diamond => self.diamond_search(
                current, reference, width, height, block_x, block_y, block_size,
            ),
            SearchPattern::Hexagon => self.hexagon_search(
                current, reference, width, height, block_x, block_y, block_size,
            ),
            SearchPattern::ThreeStep => self.three_step_search(
                current, reference, width, height, block_x, block_y, block_size,
            ),
        }
    }

    fn full_search(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
    ) -> MotionVector {
        let mut best_mv = MotionVector::zero();
        let mut best_sad = u64::MAX;

        for dy in -self.search_range..=self.search_range {
            for dx in -self.search_range..=self.search_range {
                let sad = self.compute_sad(
                    current, reference, width, height, block_x, block_y, block_size, dx, dy,
                );
                if sad < best_sad {
                    best_sad = sad;
                    best_mv = MotionVector::new(dx, dy);
                }
            }
        }

        if self.subpixel {
            best_mv = self.refine_subpixel(
                current, reference, width, height, block_x, block_y, block_size, best_mv,
            );
        }

        best_mv
    }

    fn diamond_search(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
    ) -> MotionVector {
        let ldsp: [(i16, i16); 9] = [
            (0, 0),
            (-2, 0),
            (2, 0),
            (0, -2),
            (0, 2),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ];
        let sdsp: [(i16, i16); 5] = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];

        let mut cx = 0i16;
        let mut cy = 0i16;
        let mut best_sad = u64::MAX;

        for _ in 0..16 {
            let mut step_best = (cx, cy);
            let mut step_sad = best_sad;

            for &(dx, dy) in &ldsp {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx.abs() > self.search_range || ny.abs() > self.search_range {
                    continue;
                }
                let sad = self.compute_sad(
                    current, reference, width, height, block_x, block_y, block_size, nx, ny,
                );
                if sad < step_sad {
                    step_sad = sad;
                    step_best = (nx, ny);
                }
            }

            if step_best == (cx, cy) {
                break;
            }
            cx = step_best.0;
            cy = step_best.1;
            best_sad = step_sad;
        }

        for &(dx, dy) in &sdsp {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx.abs() > self.search_range || ny.abs() > self.search_range {
                continue;
            }
            let sad = self.compute_sad(
                current, reference, width, height, block_x, block_y, block_size, nx, ny,
            );
            if sad < best_sad {
                best_sad = sad;
                cx = nx;
                cy = ny;
            }
        }

        MotionVector::new(cx, cy)
    }

    fn hexagon_search(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
    ) -> MotionVector {
        let hex: [(i16, i16); 7] = [(0, 0), (-2, 0), (2, 0), (-1, -2), (1, -2), (-1, 2), (1, 2)];
        let square: [(i16, i16); 9] = [
            (0, 0),
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ];

        let mut cx = 0i16;
        let mut cy = 0i16;
        let mut best_sad = self.compute_sad(
            current, reference, width, height, block_x, block_y, block_size, 0, 0,
        );

        for _ in 0..16 {
            let mut found = false;
            for &(dx, dy) in &hex[1..] {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx.abs() > self.search_range || ny.abs() > self.search_range {
                    continue;
                }
                let sad = self.compute_sad(
                    current, reference, width, height, block_x, block_y, block_size, nx, ny,
                );
                if sad < best_sad {
                    best_sad = sad;
                    cx = nx;
                    cy = ny;
                    found = true;
                }
            }
            if !found {
                break;
            }
        }

        for &(dx, dy) in &square {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx.abs() > self.search_range || ny.abs() > self.search_range {
                continue;
            }
            let sad = self.compute_sad(
                current, reference, width, height, block_x, block_y, block_size, nx, ny,
            );
            if sad < best_sad {
                best_sad = sad;
                cx = nx;
                cy = ny;
            }
        }

        MotionVector::new(cx, cy)
    }

    fn three_step_search(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
    ) -> MotionVector {
        let mut step = self.search_range / 2;
        let mut cx = 0i16;
        let mut cy = 0i16;
        let mut best_sad = self.compute_sad(
            current, reference, width, height, block_x, block_y, block_size, 0, 0,
        );

        while step >= 1 {
            for dy in [-step, 0, step] {
                for dx in [-step, 0, step] {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx.abs() > self.search_range || ny.abs() > self.search_range {
                        continue;
                    }
                    let sad = self.compute_sad(
                        current, reference, width, height, block_x, block_y, block_size, nx, ny,
                    );
                    if sad < best_sad {
                        best_sad = sad;
                        cx = nx;
                        cy = ny;
                    }
                }
            }
            step /= 2;
        }

        MotionVector::new(cx, cy)
    }

    fn compute_sad(
        &self,
        current: &[u8],
        reference: &[u8],
        width: usize,
        height: usize,
        block_x: usize,
        block_y: usize,
        block_size: usize,
        dx: i16,
        dy: i16,
    ) -> u64 {
        let mut sad = 0u64;
        for y in 0..block_size {
            for x in 0..block_size {
                let cx = block_x + x;
                let cy = block_y + y;
                let rx =
                    (block_x as i32 + x as i32 + dx as i32).clamp(0, width as i32 - 1) as usize;
                let ry =
                    (block_y as i32 + y as i32 + dy as i32).clamp(0, height as i32 - 1) as usize;

                if cx < width && cy < height {
                    let c = current[cy * width + cx] as i32;
                    let r = reference[ry * width + rx] as i32;
                    sad += (c - r).unsigned_abs() as u64;
                }
            }
        }
        sad
    }

    fn refine_subpixel(
        &self,
        _current: &[u8],
        _reference: &[u8],
        _width: usize,
        _height: usize,
        _block_x: usize,
        _block_y: usize,
        _block_size: usize,
        mv: MotionVector,
    ) -> MotionVector {
        mv
    }
}

pub fn apply_motion_compensation(
    reference: &[u8],
    width: usize,
    height: usize,
    mvs: &[MotionVector],
    block_size: usize,
) -> Vec<u8> {
    let block_w = (width + block_size - 1) / block_size;
    let block_h = (height + block_size - 1) / block_size;
    let mut output = vec![0u8; width * height];

    for by in 0..block_h {
        for bx in 0..block_w {
            let mv = mvs[by * block_w + bx];
            for y in 0..block_size {
                for x in 0..block_size {
                    let px = bx * block_size + x;
                    let py = by * block_size + y;
                    if px >= width || py >= height {
                        continue;
                    }

                    let rx = (px as i32 + mv.x as i32).clamp(0, width as i32 - 1) as usize;
                    let ry = (py as i32 + mv.y as i32).clamp(0, height as i32 - 1) as usize;

                    output[py * width + px] = reference[ry * width + rx];
                }
            }
        }
    }

    output
}
