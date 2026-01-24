#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::{DecodedImage, WkDecoder};

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
