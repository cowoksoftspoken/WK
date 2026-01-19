use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExifTag {
    Make,
    Model,
    Software,
    DateTime,
    DateTimeOriginal,
    ExposureTime,
    FNumber,
    ISOSpeedRatings,
    FocalLength,
    FocalLengthIn35mm,
    LensModel,
    Artist,
    Copyright,
    ImageDescription,
    Orientation,
    XResolution,
    YResolution,
    GPSLatitude,
    GPSLongitude,
    GPSAltitude,
    ImageWidth,
    ImageHeight,
    WhiteBalance,
    Flash,
    MeteringMode,
    ExposureProgram,
    ExposureBiasValue,
    ColorSpace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExifValue {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Rational(u32, u32),
    SRational(i32, i32),
    Bytes(Vec<u8>),
}

impl ExifValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ExifValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            ExifValue::Int(v) => Some(*v),
            ExifValue::UInt(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ExifValue::Float(v) => Some(*v),
            ExifValue::Rational(n, d) if *d != 0 => Some(*n as f64 / *d as f64),
            ExifValue::SRational(n, d) if *d != 0 => Some(*n as f64 / *d as f64),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExifData {
    pub tags: HashMap<ExifTag, ExifValue>,
}

impl ExifData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, tag: ExifTag, value: ExifValue) {
        self.tags.insert(tag, value);
    }

    pub fn get(&self, tag: ExifTag) -> Option<&ExifValue> {
        self.tags.get(&tag)
    }

    pub fn set_string(&mut self, tag: ExifTag, value: impl Into<String>) {
        self.set(tag, ExifValue::String(value.into()));
    }

    pub fn set_int(&mut self, tag: ExifTag, value: i64) {
        self.set(tag, ExifValue::Int(value));
    }

    pub fn set_float(&mut self, tag: ExifTag, value: f64) {
        self.set(tag, ExifValue::Float(value));
    }

    pub fn set_rational(&mut self, tag: ExifTag, numerator: u32, denominator: u32) {
        self.set(tag, ExifValue::Rational(numerator, denominator));
    }

    pub fn camera_make(&self) -> Option<&str> {
        self.get(ExifTag::Make).and_then(|v| v.as_string())
    }

    pub fn camera_model(&self) -> Option<&str> {
        self.get(ExifTag::Model).and_then(|v| v.as_string())
    }

    pub fn date_time(&self) -> Option<&str> {
        self.get(ExifTag::DateTime).and_then(|v| v.as_string())
    }

    pub fn iso(&self) -> Option<i64> {
        self.get(ExifTag::ISOSpeedRatings).and_then(|v| v.as_int())
    }

    pub fn focal_length(&self) -> Option<f64> {
        self.get(ExifTag::FocalLength).and_then(|v| v.as_float())
    }

    pub fn aperture(&self) -> Option<f64> {
        self.get(ExifTag::FNumber).and_then(|v| v.as_float())
    }

    pub fn exposure_time(&self) -> Option<f64> {
        self.get(ExifTag::ExposureTime).and_then(|v| v.as_float())
    }

    pub fn gps_coordinates(&self) -> Option<(f64, f64)> {
        let lat = self.get(ExifTag::GPSLatitude).and_then(|v| v.as_float())?;
        let lon = self.get(ExifTag::GPSLongitude).and_then(|v| v.as_float())?;
        Some((lat, lon))
    }

    pub fn orientation(&self) -> Option<i64> {
        self.get(ExifTag::Orientation).and_then(|v| v.as_int())
    }
}

impl ExifData {
    pub fn builder() -> ExifBuilder {
        ExifBuilder::new()
    }
}

pub struct ExifBuilder {
    data: ExifData,
}

impl ExifBuilder {
    pub fn new() -> Self {
        Self {
            data: ExifData::new(),
        }
    }

    pub fn make(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::Make, value);
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::Model, value);
        self
    }

    pub fn software(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::Software, value);
        self
    }

    pub fn date_time(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::DateTime, value);
        self
    }

    pub fn iso(mut self, value: i64) -> Self {
        self.data.set_int(ExifTag::ISOSpeedRatings, value);
        self
    }

    pub fn focal_length(mut self, mm: f64) -> Self {
        self.data.set_float(ExifTag::FocalLength, mm);
        self
    }

    pub fn aperture(mut self, f_number: f64) -> Self {
        self.data.set_float(ExifTag::FNumber, f_number);
        self
    }

    pub fn exposure(mut self, seconds: f64) -> Self {
        self.data.set_float(ExifTag::ExposureTime, seconds);
        self
    }

    pub fn gps(mut self, lat: f64, lon: f64) -> Self {
        self.data.set_float(ExifTag::GPSLatitude, lat);
        self.data.set_float(ExifTag::GPSLongitude, lon);
        self
    }

    pub fn artist(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::Artist, value);
        self
    }

    pub fn copyright(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::Copyright, value);
        self
    }

    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.data.set_string(ExifTag::ImageDescription, value);
        self
    }

    pub fn orientation(mut self, value: i64) -> Self {
        self.data.set_int(ExifTag::Orientation, value);
        self
    }

    pub fn build(self) -> ExifData {
        self.data
    }
}

impl Default for ExifBuilder {
    fn default() -> Self {
        Self::new()
    }
}
