#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    RGB,
    YCbCr601,
    YCbCr709,
    YCbCr2020,
    YCbCrFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSubsampling {
    YUV444,
    YUV420,
    YUV422,
}

pub fn rgb_to_ycbcr(r: u8, g: u8, b: u8, space: ColorSpace) -> (u8, u8, u8) {
    let (r, g, b) = (r as f32, g as f32, b as f32);
    let (y, cb, cr) = match space {
        ColorSpace::RGB => return (r as u8, g as u8, b as u8),
        ColorSpace::YCbCrFull => {
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let cb = 128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b;
            let cr = 128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b;
            return (
                y.round().clamp(0.0, 255.0) as u8,
                cb.round().clamp(0.0, 255.0) as u8,
                cr.round().clamp(0.0, 255.0) as u8,
            );
        }
        ColorSpace::YCbCr601 => {
            let y = 16.0 + 65.481 * r / 255.0 + 128.553 * g / 255.0 + 24.966 * b / 255.0;
            let cb = 128.0 - 37.797 * r / 255.0 - 74.203 * g / 255.0 + 112.0 * b / 255.0;
            let cr = 128.0 + 112.0 * r / 255.0 - 93.786 * g / 255.0 - 18.214 * b / 255.0;
            (y, cb, cr)
        }
        ColorSpace::YCbCr709 => {
            let y = 16.0 + 46.742 * r / 255.0 + 157.243 * g / 255.0 + 15.874 * b / 255.0;
            let cb = 128.0 - 25.765 * r / 255.0 - 86.674 * g / 255.0 + 112.439 * b / 255.0;
            let cr = 128.0 + 112.439 * r / 255.0 - 102.129 * g / 255.0 - 10.310 * b / 255.0;
            (y, cb, cr)
        }
        ColorSpace::YCbCr2020 => {
            let y = 16.0 + 46.559 * r / 255.0 + 156.629 * g / 255.0 + 16.812 * b / 255.0;
            let cb = 128.0 - 25.494 * r / 255.0 - 85.723 * g / 255.0 + 111.217 * b / 255.0;
            let cr = 128.0 + 111.217 * r / 255.0 - 101.370 * g / 255.0 - 9.847 * b / 255.0;
            (y, cb, cr)
        }
    };
    (
        y.round().clamp(16.0, 235.0) as u8,
        cb.round().clamp(16.0, 240.0) as u8,
        cr.round().clamp(16.0, 240.0) as u8,
    )
}

pub fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8, space: ColorSpace) -> (u8, u8, u8) {
    let (y, cb, cr) = (y as f32, cb as f32, cr as f32);
    let (r, g, b) = match space {
        ColorSpace::RGB => return (y as u8, cb as u8, cr as u8),
        ColorSpace::YCbCrFull => {
            let cb1 = cb - 128.0;
            let cr1 = cr - 128.0;
            let r = y + 1.402 * cr1;
            let g = y - 0.344136 * cb1 - 0.714136 * cr1;
            let b = y + 1.772 * cb1;
            return (
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
            );
        }
        ColorSpace::YCbCr601 => {
            let y1 = (y - 16.0) * 255.0 / 219.0;
            let cb1 = (cb - 128.0) * 255.0 / 224.0;
            let cr1 = (cr - 128.0) * 255.0 / 224.0;
            let r = y1 + 1.402 * cr1;
            let g = y1 - 0.344136 * cb1 - 0.714136 * cr1;
            let b = y1 + 1.772 * cb1;
            (r, g, b)
        }
        ColorSpace::YCbCr709 => {
            let y1 = (y - 16.0) * 255.0 / 219.0;
            let cb1 = (cb - 128.0) * 255.0 / 224.0;
            let cr1 = (cr - 128.0) * 255.0 / 224.0;
            let r = y1 + 1.5748 * cr1;
            let g = y1 - 0.1873 * cb1 - 0.4681 * cr1;
            let b = y1 + 1.8556 * cb1;
            (r, g, b)
        }
        ColorSpace::YCbCr2020 => {
            let y1 = (y - 16.0) * 255.0 / 219.0;
            let cb1 = (cb - 128.0) * 255.0 / 224.0;
            let cr1 = (cr - 128.0) * 255.0 / 224.0;
            let r = y1 + 1.4746 * cr1;
            let g = y1 - 0.1646 * cb1 - 0.5714 * cr1;
            let b = y1 + 1.8814 * cb1;
            (r, g, b)
        }
    };
    (
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    )
}

pub fn convert_rgb_to_ycbcr_image(
    data: &[u8],
    width: usize,
    height: usize,
    channels: usize,
    space: ColorSpace,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let n = width * height;
    let mut y = Vec::with_capacity(n);
    let mut cb = Vec::with_capacity(n);
    let mut cr = Vec::with_capacity(n);
    for i in 0..n {
        let offset = i * channels;
        let r = data[offset];
        let g = data[offset + 1];
        let b = data[offset + 2];
        // let (yv, cbv, crv) = rgb_to_ycbcr(rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2], space);
        let (yv, cbv, crv) = rgb_to_ycbcr(r, g, b, space);
        y.push(yv);
        cb.push(cbv);
        cr.push(crv);
    }
    (y, cb, cr)
}

pub fn convert_ycbcr_to_rgb_image(
    y: &[u8],
    cb: &[u8],
    cr: &[u8],
    width: usize,
    height: usize,
    channels: usize,
    space: ColorSpace,
) -> Vec<u8> {
    let n = width * height;
    let mut rgb = Vec::with_capacity(n * channels);
    for i in 0..n {
        let (r, g, b) = ycbcr_to_rgb(y[i], cb[i], cr[i], space);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);

        if channels == 4 {
            rgb.push(255);
        }
    }
    rgb
}

pub fn downsample_420(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let w2 = (width + 1) / 2;
    let h2 = (height + 1) / 2;
    let mut out = Vec::with_capacity(w2 * h2);
    for y in 0..h2 {
        for x in 0..w2 {
            let y0 = y * 2;
            let x0 = x * 2;
            let mut sum = 0u32;
            let mut count = 0u32;
            for dy in 0..2 {
                for dx in 0..2 {
                    let yy = (y0 + dy).min(height - 1);
                    let xx = (x0 + dx).min(width - 1);
                    sum += data[yy * width + xx] as u32;
                    count += 1;
                }
            }
            out.push(((sum + count / 2) / count) as u8);
        }
    }
    out
}

pub fn upsample_420(
    data: &[u8],
    small_w: usize,
    small_h: usize,
    full_w: usize,
    full_h: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; full_w * full_h];
    for y in 0..full_h {
        for x in 0..full_w {
            let sx = x / 2;
            let sy = y / 2;
            let fx = (x % 2) as f32 * 0.5;
            let fy = (y % 2) as f32 * 0.5;
            let x0 = sx.min(small_w - 1);
            let x1 = (sx + 1).min(small_w - 1);
            let y0 = sy.min(small_h - 1);
            let y1 = (sy + 1).min(small_h - 1);
            let v00 = data[y0 * small_w + x0] as f32;
            let v10 = data[y0 * small_w + x1] as f32;
            let v01 = data[y1 * small_w + x0] as f32;
            let v11 = data[y1 * small_w + x1] as f32;
            let v0 = v00 * (1.0 - fx) + v10 * fx;
            let v1 = v01 * (1.0 - fx) + v11 * fx;
            let v = v0 * (1.0 - fy) + v1 * fy;
            out[y * full_w + x] = v.round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}
