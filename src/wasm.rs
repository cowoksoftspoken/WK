#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::{DecodedImage, WkDecoder, WkEncoder};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WkWasmDecoder {
    data: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WkWasmDecoder {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn decode(&self) -> Result<WkWasmImage, JsValue> {
        let cursor = std::io::Cursor::new(&self.data);
        let decoder = WkDecoder::new();

        match decoder.decode(cursor) {
            Ok(decoded) => Ok(WkWasmImage::from_decoded(decoded)),
            Err(e) => Err(JsValue::from_str(&format!("Decode error: {}", e))),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WkWasmImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    color_type: String,
    compression: String,
    quality: u8,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WkWasmImage {
    fn from_decoded(decoded: DecodedImage) -> Self {
        let rgba = decoded.image.to_rgba8();
        Self {
            width: decoded.header.width,
            height: decoded.header.height,
            pixels: rgba.into_raw(),
            color_type: format!("{:?}", decoded.header.color_type),
            compression: format!("{:?}", decoded.header.compression_mode),
            quality: decoded.header.quality,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter)]
    pub fn color_type(&self) -> String {
        self.color_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn compression(&self) -> String {
        self.compression.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> u8 {
        self.quality
    }

    pub fn get_pixels(&self) -> Vec<u8> {
        self.pixels.clone()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn decode_wk(data: &[u8]) -> Result<WkWasmImage, JsValue> {
    init_panic_hook();
    let decoder = WkWasmDecoder::new(data);
    decoder.decode()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn encode_wk(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, JsValue> {
    init_panic_hook();
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_data.to_vec())
            .ok_or_else(|| JsValue::from_str("Invalid image dimensions"))?;

    let dynamic_img = image::DynamicImage::ImageRgba8(img);
    let encoder = WkEncoder::new().with_quality(quality);

    let mut output = Vec::new();
    encoder
        .encode(&dynamic_img, &mut output)
        .map_err(|e| JsValue::from_str(&format!("Encode error: {}", e)))?;

    Ok(output)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WkFileInfo {
    width: u32,
    height: u32,
    color_type: String,
    compression: String,
    quality: u8,
    file_size: usize,
    has_alpha: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WkFileInfo {
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
    #[wasm_bindgen(getter)]
    pub fn color_type(&self) -> String {
        self.color_type.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn compression(&self) -> String {
        self.compression.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> u8 {
        self.quality
    }
    #[wasm_bindgen(getter)]
    pub fn file_size(&self) -> usize {
        self.file_size
    }
    #[wasm_bindgen(getter)]
    pub fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_file_info(data: &[u8]) -> Result<WkFileInfo, JsValue> {
    init_panic_hook();
    let cursor = std::io::Cursor::new(data);
    let decoder = WkDecoder::new();

    match decoder.decode(cursor) {
        Ok(decoded) => Ok(WkFileInfo {
            width: decoded.header.width,
            height: decoded.header.height,
            color_type: format!("{:?}", decoded.header.color_type),
            compression: format!("{:?}", decoded.header.compression_mode),
            quality: decoded.header.quality,
            file_size: data.len(),
            has_alpha: decoded.header.has_alpha,
        }),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}
