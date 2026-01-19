const WK_MAGIC_V2 = "WK2.0\x00\x00\x00";
const WK_MAGIC_V1 = [0x57, 0x4b, 0x01, 0x00];

const ChunkTypes = {
  IHDR: "IHDR",
  ICCP: "ICCP",
  EXIF: "EXIF",
  XMP: "XMP\x00",
  THUM: "THUM",
  ANIM: "ANIM",
  IDAT: "IDAT",
  IDLS: "IDLS",
  FRMD: "FRMD",
  CUST: "CUST",
  IEND: "IEND",
};

const ColorTypes = {
  0: { name: "Grayscale", channels: 1 },
  1: { name: "Grayscale+Alpha", channels: 2 },
  2: { name: "RGB", channels: 3 },
  3: { name: "RGBA", channels: 4 },
  4: { name: "YUV420", channels: 3 },
  5: { name: "YUV444", channels: 3 },
};

const CompressionModes = {
  0: "Lossless",
  1: "Lossy",
  2: "Mixed",
};

class WkDecoderV2 {
  constructor(arrayBuffer) {
    this.data = new Uint8Array(arrayBuffer);
    this.view = new DataView(arrayBuffer);
    this.offset = 0;
  }

  readU8() {
    return this.data[this.offset++];
  }

  readU16LE() {
    const v = this.view.getUint16(this.offset, true);
    this.offset += 2;
    return v;
  }

  readU32LE() {
    const v = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return v;
  }

  readBytes(n) {
    const bytes = this.data.slice(this.offset, this.offset + n);
    this.offset += n;
    return bytes;
  }

  readString(n) {
    const bytes = this.readBytes(n);
    return String.fromCharCode(...bytes);
  }

  verifyMagic() {
    const magic = this.readString(8);
    if (magic !== WK_MAGIC_V2) {
      const v1Magic = [this.data[0], this.data[1], this.data[2], this.data[3]];
      if (v1Magic.every((v, i) => v === WK_MAGIC_V1[i])) {
        throw new Error(
          "This is a WK v1 file. Please use an older viewer or re-encode.",
        );
      }
      throw new Error("Invalid WK file: wrong magic number");
    }
  }

  readChunk() {
    const typeBytes = this.readString(4);
    const size = this.readU32LE();
    const data = this.readBytes(size);
    const crc = this.readU32LE();
    return { type: typeBytes, size, data, crc };
  }

  decodeHeader(data) {
    const view = new DataView(data.buffer, data.byteOffset, data.length);
    return {
      width: view.getUint32(0, true),
      height: view.getUint32(4, true),
      colorType: data[8],
      compressionMode: data[9],
      quality: data[10],
      flags: data[11],
      bitDepth: data[12],
      hasAlpha: (data[11] & 0x01) !== 0,
      hasAnimation: (data[11] & 0x02) !== 0,
    };
  }

  decode() {
    this.verifyMagic();

    const chunks = [];
    while (this.offset < this.data.length) {
      const chunk = this.readChunk();
      chunks.push(chunk);
      if (chunk.type === ChunkTypes.IEND) break;
    }

    const ihdrChunk = chunks.find((c) => c.type === ChunkTypes.IHDR);
    if (!ihdrChunk) throw new Error("Missing IHDR chunk");

    const header = this.decodeHeader(ihdrChunk.data);

    const dataChunk = chunks.find(
      (c) => c.type === ChunkTypes.IDAT || c.type === ChunkTypes.IDLS,
    );
    if (!dataChunk) throw new Error("Missing image data chunk");

    const isLossy = dataChunk.type === ChunkTypes.IDLS;
    const colorInfo = ColorTypes[header.colorType] || ColorTypes[2];
    const channels = colorInfo.channels;

    let rawData;
    if (isLossy) {
      rawData = this.decompressLossy(
        dataChunk.data,
        header.width,
        header.height,
        channels,
      );
    } else {
      rawData = this.decompressLossless(
        dataChunk.data,
        header.width,
        header.height,
        channels,
      );
    }

    return {
      width: header.width,
      height: header.height,
      colorType: header.colorType,
      colorTypeName: colorInfo.name,
      compression: header.compressionMode,
      compressionName: CompressionModes[header.compressionMode],
      quality: header.quality,
      hasAlpha: header.hasAlpha,
      channels: channels,
      rawData: rawData,
      chunks: chunks,
    };
  }

  decompressLossless(data, width, height, channels) {
    const huffDecoded = this.huffmanDecode(data);
    return this.reversePredictor(huffDecoded, width, height, channels);
  }

  huffmanDecode(data) {
    const view = new DataView(data.buffer, data.byteOffset, data.length);
    let offset = 0;

    const freq = new Uint32Array(256);
    for (let i = 0; i < 256; i++) {
      freq[i] = view.getUint32(offset, true);
      offset += 4;
    }

    const originalLen = view.getUint32(offset, true);
    offset += 4;
    const compressedLen = view.getUint32(offset, true);
    offset += 4;
    const compressed = data.slice(offset, offset + compressedLen);

    const tree = this.buildHuffmanTree(freq);
    if (!tree) return new Uint8Array(0);

    const output = new Uint8Array(originalLen);
    let outIdx = 0;
    let node = tree;

    outer: for (const byte of compressed) {
      for (let bit = 7; bit >= 0; bit--) {
        const b = (byte >> bit) & 1;
        node = b === 0 ? node.left : node.right;

        if (node.symbol !== null) {
          output[outIdx++] = node.symbol;
          if (outIdx >= originalLen) break outer;
          node = tree;
        }
      }
    }

    return output;
  }

  buildHuffmanTree(freq) {
    const nodes = [];
    for (let i = 0; i < 256; i++) {
      if (freq[i] > 0) {
        nodes.push({ symbol: i, freq: freq[i], left: null, right: null });
      }
    }
    if (nodes.length === 0) return null;
    if (nodes.length === 1) return nodes[0];

    while (nodes.length > 1) {
      nodes.sort((a, b) => b.freq - a.freq);
      const right = nodes.pop();
      const left = nodes.pop();
      nodes.push({ symbol: null, freq: left.freq + right.freq, left, right });
    }
    return nodes[0];
  }

  reversePredictor(filtered, width, height, channels) {
    const stride = width * channels;
    const data = new Uint8Array(width * height * channels);
    let inIdx = 0;

    for (let y = 0; y < height; y++) {
      const predictor = filtered[inIdx++];

      for (let x = 0; x < stride; x++) {
        const idx = y * stride + x;
        const delta = filtered[inIdx++];

        const left = x >= channels ? data[idx - channels] : 0;
        const up = y > 0 ? data[idx - stride] : 0;
        const upLeft =
          x >= channels && y > 0 ? data[idx - stride - channels] : 0;

        let prediction;
        switch (predictor) {
          case 0:
            prediction = 0;
            break;
          case 1:
            prediction = left;
            break;
          case 2:
            prediction = up;
            break;
          case 3:
            prediction = (left + up) >> 1;
            break;
          case 4:
            prediction = this.paeth(left, up, upLeft);
            break;
          default:
            prediction = 0;
        }

        data[idx] = (delta + prediction) & 0xff;
      }
    }

    return data;
  }

  paeth(a, b, c) {
    const p = a + b - c;
    const pa = Math.abs(p - a);
    const pb = Math.abs(p - b);
    const pc = Math.abs(p - c);
    if (pa <= pb && pa <= pc) return a;
    if (pb <= pc) return b;
    return c;
  }

  decompressLossy(data, width, height, channels) {
    const lumaTable = new Uint16Array(64);
    const chromaTable = new Uint16Array(64);

    for (let i = 0; i < 64; i++) {
      lumaTable[i] = data[i * 2] | (data[i * 2 + 1] << 8);
    }
    for (let i = 0; i < 64; i++) {
      chromaTable[i] = data[128 + i * 2] | (data[128 + i * 2 + 1] << 8);
    }

    const compressed = data.slice(256);
    const coeffs = this.decodeRleHuffman(compressed);

    const blockWidth = Math.ceil(width / 8);
    const blockHeight = Math.ceil(height / 8);
    const blocksPerChannel = blockWidth * blockHeight;
    const paddedW = blockWidth * 8;
    const paddedH = blockHeight * 8;

    const output = new Uint8Array(width * height * channels);

    for (let ch = 0; ch < channels; ch++) {
      const isChroma = ch > 0 && channels >= 3;
      const table = isChroma ? chromaTable : lumaTable;
      const channelOffset = ch * blocksPerChannel * 64;
      const padded = new Uint8Array(paddedW * paddedH);

      for (let by = 0; by < blockHeight; by++) {
        for (let bx = 0; bx < blockWidth; bx++) {
          const blockIdx = by * blockWidth + bx;
          const coeffStart = channelOffset + blockIdx * 64;

          if (coeffStart + 64 > coeffs.length) continue;

          const scanned = coeffs.slice(coeffStart, coeffStart + 64);
          const zigzagged = this.zigzagUnscan(scanned);

          const dequantized = new Int16Array(64);
          for (let i = 0; i < 64; i++) {
            dequantized[i] = zigzagged[i] * table[i];
          }

          const block = this.idct8x8(dequantized);

          for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
              const px = bx * 8 + x;
              const py = by * 8 + y;
              if (px < paddedW && py < paddedH) {
                const val = Math.max(0, Math.min(255, block[y * 8 + x] + 128));
                padded[py * paddedW + px] = val;
              }
            }
          }
        }
      }

      for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
          output[(y * width + x) * channels + ch] = padded[y * paddedW + x];
        }
      }
    }

    return output;
  }

  decodeRleHuffman(data) {
    const rle = this.huffmanDecode(data);
    const output = [];
    let i = 0;

    while (i < rle.length) {
      switch (rle[i]) {
        case 0:
          if (i + 1 >= rle.length) {
            i++;
            break;
          }
          const count = rle[i + 1];
          for (let j = 0; j < count; j++) output.push(0);
          i += 2;
          break;
        case 1:
          if (i + 1 >= rle.length) {
            i++;
            break;
          }
          const b = rle[i + 1];
          const mag1 = b & 0x7f;
          const sign1 = (b >> 7) & 1;
          output.push(sign1 === 1 ? -mag1 : mag1);
          i += 2;
          break;
        case 2:
          if (i + 2 >= rle.length) {
            i++;
            break;
          }
          const low = rle[i + 1];
          const high = rle[i + 2] & 0x7f;
          const mag2 = low | (high << 8);
          const sign2 = (rle[i + 2] >> 7) & 1;
          output.push(sign2 === 1 ? -mag2 : mag2);
          i += 3;
          break;
        default:
          i++;
      }
    }

    return new Int16Array(output);
  }

  idct8x8(coeffs) {
    const output = new Int16Array(64);
    const c = (u) => (u === 0 ? 1 / Math.SQRT2 : 1);

    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        let sum = 0;
        for (let v = 0; v < 8; v++) {
          for (let u = 0; u < 8; u++) {
            const coeff = coeffs[v * 8 + u];
            const cosU = Math.cos(((2 * x + 1) * u * Math.PI) / 16);
            const cosV = Math.cos(((2 * y + 1) * v * Math.PI) / 16);
            sum += c(u) * c(v) * coeff * cosU * cosV;
          }
        }
        output[y * 8 + x] = Math.round(0.25 * sum);
      }
    }

    return output;
  }

  zigzagUnscan(scanned) {
    const order = [
      0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33,
      40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43,
      36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53,
      60, 61, 54, 47, 55, 62, 63,
    ];
    const output = new Int16Array(64);
    for (let i = 0; i < 64; i++) {
      output[order[i]] = scanned[i];
    }
    return output;
  }
}

function renderToCanvas(imageData, canvas) {
  const { width, height, rawData, channels } = imageData;
  canvas.width = width;
  canvas.height = height;

  const ctx = canvas.getContext("2d");
  const imgData = ctx.createImageData(width, height);

  for (let i = 0; i < width * height; i++) {
    const srcIdx = i * channels;
    const dstIdx = i * 4;

    if (channels === 1) {
      imgData.data[dstIdx] = rawData[srcIdx];
      imgData.data[dstIdx + 1] = rawData[srcIdx];
      imgData.data[dstIdx + 2] = rawData[srcIdx];
      imgData.data[dstIdx + 3] = 255;
    } else if (channels === 2) {
      imgData.data[dstIdx] = rawData[srcIdx];
      imgData.data[dstIdx + 1] = rawData[srcIdx];
      imgData.data[dstIdx + 2] = rawData[srcIdx];
      imgData.data[dstIdx + 3] = rawData[srcIdx + 1];
    } else if (channels === 3) {
      imgData.data[dstIdx] = rawData[srcIdx];
      imgData.data[dstIdx + 1] = rawData[srcIdx + 1];
      imgData.data[dstIdx + 2] = rawData[srcIdx + 2];
      imgData.data[dstIdx + 3] = 255;
    } else if (channels === 4) {
      imgData.data[dstIdx] = rawData[srcIdx];
      imgData.data[dstIdx + 1] = rawData[srcIdx + 1];
      imgData.data[dstIdx + 2] = rawData[srcIdx + 2];
      imgData.data[dstIdx + 3] = rawData[srcIdx + 3];
    }
  }

  ctx.putImageData(imgData, 0, 0);
}

function displayInfo(imageData, fileName, fileSize) {
  document.getElementById("infoWidth").textContent = imageData.width + "px";
  document.getElementById("infoHeight").textContent = imageData.height + "px";
  document.getElementById("infoColorType").textContent =
    imageData.colorTypeName;
  document.getElementById("infoCompression").textContent =
    `${imageData.compressionName} (Q${imageData.quality})`;
  document.getElementById("infoFileName").textContent = fileName;
  document.getElementById("infoFileSize").textContent = formatBytes(fileSize);

  const metadataContent = document.getElementById("metadataContent");
  metadataContent.innerHTML = "";

  const rawSize = imageData.width * imageData.height * imageData.channels;
  const ratio = ((fileSize / rawSize) * 100).toFixed(1);

  addMetadataItem(metadataContent, "Version", "2.0");
  addMetadataItem(metadataContent, "Compression Ratio", `${ratio}%`);
  addMetadataItem(
    metadataContent,
    "Has Alpha",
    imageData.hasAlpha ? "Yes" : "No",
  );
  addMetadataItem(metadataContent, "Chunks", imageData.chunks.length);

  document.getElementById("infoSection").style.display = "block";
}

function addMetadataItem(container, label, value) {
  const item = document.createElement("div");
  item.className = "info-item";
  item.innerHTML = `<span class="info-label">${label}:</span><span class="info-value">${value}</span>`;
  container.appendChild(item);
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return (bytes / Math.pow(k, i)).toFixed(2) + " " + sizes[i];
}

function showError(message) {
  const el = document.getElementById("errorMessage");
  el.textContent = message;
  el.style.display = "block";
  setTimeout(() => (el.style.display = "none"), 5000);
}

function showSuccess(message) {
  const el = document.getElementById("successMessage");
  el.textContent = message;
  el.style.display = "block";
  setTimeout(() => (el.style.display = "none"), 3000);
}

async function loadWkFile(file) {
  document.getElementById("loading").style.display = "block";
  document.getElementById("canvasContainer").style.display = "none";
  document.getElementById("actions").style.display = "none";

  try {
    const arrayBuffer = await file.arrayBuffer();
    const decoder = new WkDecoderV2(arrayBuffer);
    const imageData = decoder.decode();

    renderToCanvas(imageData, document.getElementById("canvas"));
    displayInfo(imageData, file.name, file.size);

    document.getElementById("loading").style.display = "none";
    document.getElementById("canvasContainer").style.display = "flex";
    document.getElementById("actions").style.display = "flex";

    showSuccess("WK v2.0 file loaded successfully!");
  } catch (error) {
    document.getElementById("loading").style.display = "none";
    showError("Error: " + error.message);
    console.error(error);
  }
}

const uploadArea = document.getElementById("uploadArea");
const fileInput = document.getElementById("fileInput");

uploadArea.addEventListener("click", () => fileInput.click());
fileInput.addEventListener("change", (e) => {
  if (e.target.files[0]) loadWkFile(e.target.files[0]);
});

uploadArea.addEventListener("dragover", (e) => {
  e.preventDefault();
  uploadArea.classList.add("dragging");
});

uploadArea.addEventListener("dragleave", () =>
  uploadArea.classList.remove("dragging"),
);

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
  showSuccess("PNG downloaded!");
});

document.getElementById("clearBtn").addEventListener("click", () => {
  document.getElementById("canvasContainer").style.display = "none";
  document.getElementById("actions").style.display = "none";
  document.getElementById("infoSection").style.display = "none";
  fileInput.value = "";
  showSuccess("Cleared");
});
