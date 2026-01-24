import init, { decode_wk } from "./wk_format.js";

let wasmLoaded = false;

export async function loadWasm() {
  if (!wasmLoaded) {
    await init();
    wasmLoaded = true;
    console.log("WK WASM decoder loaded");
  }
}

export async function decodeWkImage(arrayBuffer) {
  await loadWasm();

  try {
    const data = new Uint8Array(arrayBuffer);
    const image = decode_wk(data);

    return {
      width: image.width,
      height: image.height,
      pixels: image.get_pixels(),
      colorType: image.color_type,
      compression: image.compression,
      quality: image.quality,
    };
  } catch (error) {
    console.error("WASM decode failed:", error);
    throw error;
  }
}

export function createImageData(decoded, canvas) {
  const ctx = canvas.getContext("2d");
  canvas.width = decoded.width;
  canvas.height = decoded.height;

  const imageData = ctx.createImageData(decoded.width, decoded.height);
  imageData.data.set(decoded.pixels);
  ctx.putImageData(imageData, 0, 0);

  return imageData;
}
