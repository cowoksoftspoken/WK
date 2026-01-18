const WK_MAGIC = [0x57, 0x4b, 0x01, 0x00];

const ColorType = {
  RGB: 0,
  RGBA: 1,
  Grayscale: 2,
  GrayscaleAlpha: 3,
};

const ColorTypeNames = {
  0: "RGB",
  1: "RGBA",
  2: "Grayscale",
  3: "Grayscale + Alpha",
};

class WkDecoder {
  constructor(arrayBuffer) {
    this.data = new Uint8Array(arrayBuffer);
    this.offset = 0;
  }

  readU8() {
    return this.data[this.offset++];
  }

  readU32() {
    const value =
      this.data[this.offset] |
      (this.data[this.offset + 1] << 8) |
      (this.data[this.offset + 2] << 16) |
      (this.data[this.offset + 3] << 24);
    this.offset += 4;
    return value >>> 0;
  }

  readBytes(length) {
    const bytes = this.data.slice(this.offset, this.offset + length);
    this.offset += length;
    return bytes;
  }

  decode() {
    const magic = [this.readU8(), this.readU8(), this.readU8(), this.readU8()];
    if (!magic.every((v, i) => v === WK_MAGIC[i])) {
      throw new Error("Invalid WK file: wrong magic number");
    }

    const width = this.readU32();
    const height = this.readU32();
    const colorType = this.readU8();
    const compression = this.readU8();
    const metadataSize = this.readU32();

    let metadata = null;
    if (metadataSize > 0) {
      const metadataBytes = this.readBytes(metadataSize);
      try {
        metadata = this.decodeMetadata(metadataBytes);
      } catch (e) {
        console.warn("Failed to decode metadata:", e);
      }
    }

    const compressedSize = this.readU32();
    const compressedData = this.readBytes(compressedSize);

    const channels = this.getChannels(colorType);
    const expectedSize = width * height * channels;
    const rawData =
      compression === 1
        ? this.rleDecompress(compressedData, expectedSize)
        : compressedData;

    return {
      width,
      height,
      colorType,
      compression,
      metadata,
      rawData,
      channels,
    };
  }

  getChannels(colorType) {
    switch (colorType) {
      case ColorType.RGB:
        return 3;
      case ColorType.RGBA:
        return 4;
      case ColorType.Grayscale:
        return 1;
      case ColorType.GrayscaleAlpha:
        return 2;
      default:
        throw new Error("Unknown color type: " + colorType);
    }
  }

  rleDecompress(data, expectedSize) {
    const result = new Uint8Array(expectedSize);
    let resultIdx = 0;
    let i = 0;

    while (i < data.length && resultIdx < expectedSize) {
      const header = data[i++];

      if (header & 0x80) {
        const rawCount = header & 0x7f;
        const count = rawCount === 0 ? 128 : rawCount;
        if (i >= data.length) {
          throw new Error("Unexpected end of RLE data");
        }
        const value = data[i++];

        for (let j = 0; j < count; j++) {
          result[resultIdx++] = value;
        }
      } else {
        const count = header;
        if (count === 0) continue;

        if (i + count > data.length) {
          throw new Error(
            `Invalid literal count: ${count} bytes needed, ${data.length - i} available`,
          );
        }

        for (let j = 0; j < count; j++) {
          result[resultIdx++] = data[i++];
        }
      }
    }

    if (resultIdx !== expectedSize) {
      throw new Error(
        `Decompression size mismatch: expected ${expectedSize}, got ${resultIdx}`,
      );
    }

    return result;
  }

  decodeMetadata(bytes) {
    const text = new TextDecoder().decode(bytes);
    try {
      return JSON.parse(text);
    } catch (e) {
      return { raw: text };
    }
  }
}

function renderToCanvas(imageData, canvas) {
  const { width, height, rawData, colorType, channels } = imageData;

  canvas.width = width;
  canvas.height = height;

  const ctx = canvas.getContext("2d");
  const imgData = ctx.createImageData(width, height);

  for (let i = 0; i < width * height; i++) {
    const srcIdx = i * channels;
    const dstIdx = i * 4;

    switch (colorType) {
      case ColorType.RGB:
        imgData.data[dstIdx] = rawData[srcIdx];
        imgData.data[dstIdx + 1] = rawData[srcIdx + 1];
        imgData.data[dstIdx + 2] = rawData[srcIdx + 2];
        imgData.data[dstIdx + 3] = 255;
        break;
      case ColorType.RGBA:
        imgData.data[dstIdx] = rawData[srcIdx];
        imgData.data[dstIdx + 1] = rawData[srcIdx + 1];
        imgData.data[dstIdx + 2] = rawData[srcIdx + 2];
        imgData.data[dstIdx + 3] = rawData[srcIdx + 3];
        break;
      case ColorType.Grayscale:
        const gray = rawData[srcIdx];
        imgData.data[dstIdx] = gray;
        imgData.data[dstIdx + 1] = gray;
        imgData.data[dstIdx + 2] = gray;
        imgData.data[dstIdx + 3] = 255;
        break;
      case ColorType.GrayscaleAlpha:
        const grayA = rawData[srcIdx];
        imgData.data[dstIdx] = grayA;
        imgData.data[dstIdx + 1] = grayA;
        imgData.data[dstIdx + 2] = grayA;
        imgData.data[dstIdx + 3] = rawData[srcIdx + 1];
        break;
    }
  }

  ctx.putImageData(imgData, 0, 0);
}

function displayInfo(imageData, fileName, fileSize) {
  document.getElementById("infoWidth").textContent = imageData.width + "px";
  document.getElementById("infoHeight").textContent = imageData.height + "px";
  document.getElementById("infoColorType").textContent =
    ColorTypeNames[imageData.colorType];
  document.getElementById("infoCompression").textContent =
    imageData.compression === 1 ? "RLE" : "None";
  document.getElementById("infoFileName").textContent = fileName;
  document.getElementById("infoFileSize").textContent = formatBytes(fileSize);

  // Display metadata
  const metadataContent = document.getElementById("metadataContent");
  metadataContent.innerHTML = "";

  if (imageData.metadata) {
    for (const [key, value] of Object.entries(imageData.metadata)) {
      if (value && key !== "raw") {
        const item = document.createElement("div");
        item.className = "info-item";
        item.innerHTML = `
                            <span class="info-label">${key}:</span>
                            <span class="info-value">${value}</span>
                        `;
        metadataContent.appendChild(item);
      }
    }
  }

  document.getElementById("infoSection").style.display = "block";
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

function showError(message) {
  const errorEl = document.getElementById("errorMessage");
  errorEl.textContent = message;
  errorEl.style.display = "block";
  setTimeout(() => (errorEl.style.display = "none"), 5000);
}

function showSuccess(message) {
  const successEl = document.getElementById("successMessage");
  successEl.textContent = message;
  successEl.style.display = "block";
  setTimeout(() => (successEl.style.display = "none"), 3000);
}

async function loadWkFile(file) {
  document.getElementById("loading").style.display = "block";
  document.getElementById("canvasContainer").style.display = "none";
  document.getElementById("actions").style.display = "none";

  try {
    const arrayBuffer = await file.arrayBuffer();
    const decoder = new WkDecoder(arrayBuffer);
    const imageData = decoder.decode();

    const canvas = document.getElementById("canvas");
    renderToCanvas(imageData, canvas);

    displayInfo(imageData, file.name, file.size);

    document.getElementById("loading").style.display = "none";
    document.getElementById("canvasContainer").style.display = "flex";
    document.getElementById("actions").style.display = "flex";

    showSuccess("WK file loaded successfully!");
  } catch (error) {
    document.getElementById("loading").style.display = "none";
    showError("Error loading WK file: " + error.message);
    console.error(error);
  }
}

const uploadArea = document.getElementById("uploadArea");
const fileInput = document.getElementById("fileInput");

uploadArea.addEventListener("click", () => fileInput.click());

fileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (file) loadWkFile(file);
});

uploadArea.addEventListener("dragover", (e) => {
  e.preventDefault();
  uploadArea.classList.add("dragging");
});

uploadArea.addEventListener("dragleave", () => {
  uploadArea.classList.remove("dragging");
});

uploadArea.addEventListener("drop", (e) => {
  e.preventDefault();
  uploadArea.classList.remove("dragging");
  const file = e.dataTransfer.files[0];
  if (file && file.name.endsWith(".wk")) {
    loadWkFile(file);
  } else {
    showError("Please drop a .wk file");
  }
});

document.getElementById("downloadPng").addEventListener("click", () => {
  const canvas = document.getElementById("canvas");
  const link = document.createElement("a");
  link.download = "wk-image.png";
  link.href = canvas.toDataURL("image/png");
  link.click();
  showSuccess("PNG downloaded successfully!");
});

document.getElementById("clearBtn").addEventListener("click", () => {
  document.getElementById("canvasContainer").style.display = "none";
  document.getElementById("actions").style.display = "none";
  document.getElementById("infoSection").style.display = "none";
  fileInput.value = "";
  showSuccess("Viewer cleared");
});
