use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferFunction {
    SDR,
    PQ,
    HLG,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorGamut {
    SRGB,
    AdobeRGB,
    DisplayP3,
    Rec2020,
    ProPhotoRGB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HDRMetadata {
    pub bit_depth: u8,
    pub transfer: TransferFunction,
    pub gamut: ColorGamut,
    pub max_cll: Option<u16>,
    pub max_fall: Option<u16>,
    pub mastering_display: Option<MasteringDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteringDisplay {
    pub red_primary: (f32, f32),
    pub green_primary: (f32, f32),
    pub blue_primary: (f32, f32),
    pub white_point: (f32, f32),
    pub max_luminance: f32,
    pub min_luminance: f32,
}

impl Default for HDRMetadata {
    fn default() -> Self {
        Self {
            bit_depth: 8,
            transfer: TransferFunction::SDR,
            gamut: ColorGamut::SRGB,
            max_cll: None,
            max_fall: None,
            mastering_display: None,
        }
    }
}

impl HDRMetadata {
    pub fn sdr() -> Self {
        Self::default()
    }

    pub fn hdr10() -> Self {
        Self {
            bit_depth: 10,
            transfer: TransferFunction::PQ,
            gamut: ColorGamut::Rec2020,
            max_cll: Some(1000),
            max_fall: Some(400),
            mastering_display: None,
        }
    }

    pub fn hlg() -> Self {
        Self {
            bit_depth: 10,
            transfer: TransferFunction::HLG,
            gamut: ColorGamut::Rec2020,
            max_cll: None,
            max_fall: None,
            mastering_display: None,
        }
    }
}

pub fn pq_eotf(v: f32) -> f32 {
    let m1 = 0.1593017578125;
    let m2 = 78.84375;
    let c1 = 0.8359375;
    let c2 = 18.8515625;
    let c3 = 18.6875;

    let v_m2 = v.powf(1.0 / m2);
    let num = (v_m2 - c1).max(0.0);
    let den = c2 - c3 * v_m2;
    (num / den).powf(1.0 / m1)
}

pub fn pq_oetf(l: f32) -> f32 {
    let m1 = 0.1593017578125;
    let m2 = 78.84375;
    let c1 = 0.8359375;
    let c2 = 18.8515625;
    let c3 = 18.6875;

    let l_m1 = l.powf(m1);
    let num = c1 + c2 * l_m1;
    let den = 1.0 + c3 * l_m1;
    (num / den).powf(m2)
}

pub fn hlg_eotf(v: f32) -> f32 {
    let a = 0.17883277;
    let b = 0.28466892;
    let c = 0.55991073;

    if v <= 0.5 {
        (v * v) / 3.0
    } else {
        ((v - c) / a + b).exp() / 12.0
    }
}

pub fn hlg_oetf(l: f32) -> f32 {
    let a = 0.17883277;
    let b = 0.28466892;
    let c = 0.55991073;

    if l <= 1.0 / 12.0 {
        (3.0 * l).sqrt()
    } else {
        a * (12.0 * l - b).ln() + c
    }
}

pub fn convert_bit_depth(value: u16, from_bits: u8, to_bits: u8) -> u16 {
    if from_bits == to_bits {
        return value;
    }
    let from_max = (1u32 << from_bits) - 1;
    let to_max = (1u32 << to_bits) - 1;
    ((value as u32 * to_max + from_max / 2) / from_max) as u16
}

pub fn expand_to_16bit(data: &[u8], bit_depth: u8) -> Vec<u16> {
    match bit_depth {
        8 => data.iter().map(|&v| (v as u16) << 8 | v as u16).collect(),
        10 => {
            let mut out = Vec::with_capacity(data.len() * 8 / 10);
            for chunk in data.chunks(5) {
                if chunk.len() == 5 {
                    let v0 = ((chunk[0] as u16) << 2) | (((chunk[4] >> 6) & 0x03) as u16);
                    let v1 = ((chunk[1] as u16) << 2) | (((chunk[4] >> 4) & 0x03) as u16);
                    let v2 = ((chunk[2] as u16) << 2) | (((chunk[4] >> 2) & 0x03) as u16);
                    let v3 = ((chunk[3] as u16) << 2) | ((chunk[4] & 0x03) as u16);
                    out.push(v0 << 6);
                    out.push(v1 << 6);
                    out.push(v2 << 6);
                    out.push(v3 << 6);
                }
            }
            out
        }
        12 => {
            let mut out = Vec::with_capacity(data.len() * 2 / 3);
            for chunk in data.chunks(3) {
                if chunk.len() == 3 {
                    let v0 = ((chunk[0] as u16) << 4) | ((chunk[2] >> 4) as u16);
                    let v1 = ((chunk[1] as u16) << 4) | ((chunk[2] & 0x0F) as u16);
                    out.push(v0 << 4);
                    out.push(v1 << 4);
                }
            }
            out
        }
        16 => {
            let mut out = Vec::with_capacity(data.len() / 2);
            for chunk in data.chunks(2) {
                if chunk.len() == 2 {
                    out.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                }
            }
            out
        }
        _ => data.iter().map(|&v| (v as u16) << 8).collect(),
    }
}

pub fn compress_to_8bit(data: &[u16], bit_depth: u8) -> Vec<u8> {
    match bit_depth {
        8 => data.iter().map(|&v| (v >> 8) as u8).collect(),
        10 | 12 | 16 => data.iter().map(|&v| (v >> 8) as u8).collect(),
        _ => data.iter().map(|&v| (v >> 8) as u8).collect(),
    }
}
