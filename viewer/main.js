const WK_MAGIC = [0x57, 0x4b, 0x33, 0x2e, 0x30, 0x00, 0x00, 0x00];

class WKDecoder {
  constructor(buffer) {
    this.data = new Uint8Array(buffer);
    this.pos = 0;
  }

  readU8() {
    return this.data[this.pos++];
  }
  readU16() {
    return this.readU8() | (this.readU8() << 8);
  }
  readU32() {
    return this.readU16() | (this.readU16() << 16);
  }

  readBytes(n) {
    const bytes = this.data.slice(this.pos, this.pos + n);
    this.pos += n;
    return bytes;
  }

  readString(n) {
    return String.fromCharCode(...this.readBytes(n));
  }

  decode() {
    for (let i = 0; i < 8; i++) {
      if (this.data[i] !== WK_MAGIC[i]) {
        throw new Error("Invalid WK v3.1.1 file (magic mismatch)");
      }
    }
    this.pos = 8;

    const chunks = [];
    while (this.pos < this.data.length - 4) {
      const typeStr = this.readString(4);
      const length = this.readU32();
      const data = this.readBytes(length);
      const crc = this.readU32();
      chunks.push({ type: typeStr, length, data, crc });

      if (typeStr === "IEND") break;
    }

    let header = null;
    let imageData = null;
    let isLossy = false;

    for (const chunk of chunks) {
      if (chunk.type === "IHDR") {
        header = this.parseHeader(chunk.data);
      } else if (chunk.type === "IDAT") {
        imageData = chunk.data;
        isLossy = false;
      } else if (chunk.type === "IDLS") {
        imageData = chunk.data;
        isLossy = true;
      }
    }

    if (!header) {
      throw new Error("Missing IHDR chunk");
    }

    return { header, imageData, chunks, isLossy };
  }

  parseHeader(data) {
    const view = new DataView(data.buffer, data.byteOffset, data.length);
    const colorTypes = ["Grayscale", "GrayscaleAlpha", "RGB", "RGBA"];
    const compressionModes = ["Lossless", "Lossy"];

    return {
      width: view.getUint32(0, true),
      height: view.getUint32(4, true),
      colorType: colorTypes[data[8]] || `Unknown(${data[8]})`,
      compression: compressionModes[data[9]] || `Unknown(${data[9]})`,
      quality: data[10],
      hasAlpha: data[11] === 1,
      hasAnimation: data[12] === 1,
      bitDepth: data[13] || 8,
    };
  }
}

class App {
  constructor() {
    this.canvas = document.getElementById("canvas");
    this.ctx = this.canvas.getContext("2d");
    this.currentFile = null;
    this.init();
  }

  init() {
    const dropzone = document.getElementById("dropzone");
    const fileInput = document.getElementById("fileInput");

    dropzone.addEventListener("click", () => fileInput.click());
    dropzone.addEventListener("dragover", (e) => {
      e.preventDefault();
      dropzone.classList.add("drag-over");
    });
    dropzone.addEventListener("dragleave", () => {
      dropzone.classList.remove("drag-over");
    });
    dropzone.addEventListener("drop", (e) => {
      e.preventDefault();
      dropzone.classList.remove("drag-over");
      const file = e.dataTransfer.files[0];
      if (file) this.loadFile(file);
    });
    fileInput.addEventListener("change", (e) => {
      const file = e.target.files[0];
      if (file) this.loadFile(file);
    });

    document
      .getElementById("downloadPng")
      .addEventListener("click", () => this.downloadPng());
    document
      .getElementById("clearBtn")
      .addEventListener("click", () => this.clear());
  }

  async loadFile(file) {
    this.showLoading(true);
    this.currentFile = file;

    try {
      if (file.name.endsWith(".wk")) {
        await this.loadWkFile(file);
      } else {
        await this.loadImageFile(file);
      }
      this.showToast("File loaded successfully", "success");
    } catch (error) {
      console.error(error);
      this.showToast(error.message, "error");
    } finally {
      this.showLoading(false);
    }
  }

  async loadWkFile(file) {
    const buffer = await file.arrayBuffer();

    // Try WASM decoder first
    try {
      const { loadWasm, decodeWkImage, createImageData } =
        await import("./wasm_loader.js");
      await loadWasm();

      const decoded = await decodeWkImage(buffer);

      this.updateInfo({
        size: `${decoded.width} × ${decoded.height}`,
        format: "WK v3.1.1",
        quality: decoded.quality ? `Q${decoded.quality}` : "Lossless",
        mode: decoded.compression,
      });

      this.canvas.width = decoded.width;
      this.canvas.height = decoded.height;

      const imgData = this.ctx.createImageData(decoded.width, decoded.height);
      imgData.data.set(decoded.pixels);
      this.ctx.putImageData(imgData, 0, 0);

      document.getElementById("dropzone").style.display = "none";
      document.getElementById("canvasWrapper").style.display = "flex";

      this.drawHistogram(imgData.data);
      this.setStatus(
        `WK Decoded: ${decoded.width}×${decoded.height} ${decoded.compression}`,
      );
      return;
    } catch (wasmError) {
      console.warn(
        "WASM decode failed, falling back to header-only:",
        wasmError,
      );
    }

    const decoder = new WKDecoder(buffer);
    const { header, isLossy, chunks } = decoder.decode();

    this.updateInfo({
      size: `${header.width} × ${header.height}`,
      format: "WK v3.1.1",
      quality:
        header.compression === "Lossless" ? "Lossless" : `Q${header.quality}`,
      mode: header.compression,
    });

    this.canvas.width = header.width;
    this.canvas.height = header.height;

    const imgData = this.ctx.createImageData(header.width, header.height);

    this.drawPlaceholder(imgData, header);
    this.ctx.putImageData(imgData, 0, 0);

    this.drawWkInfo(header, file.size, chunks.length, isLossy);

    document.getElementById("dropzone").style.display = "none";
    document.getElementById("canvasWrapper").style.display = "flex";

    this.drawHistogram(imgData.data);
    this.setStatus(
      `WK Info: ${header.width}×${header.height} ${header.compression}`,
    );
  }

  drawPlaceholder(imgData, header) {
    const w = header.width;
    const h = header.height;

    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const i = (y * w + x) * 4;
        const checker = (Math.floor(x / 16) + Math.floor(y / 16)) % 2 === 0;
        const base = checker ? 45 : 55;
        imgData.data[i] = base;
        imgData.data[i + 1] = base;
        imgData.data[i + 2] = base + 10;
        imgData.data[i + 3] = 255;
      }
    }
  }

  drawWkInfo(header, fileSize, chunkCount, isLossy) {
    this.ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
    this.ctx.fillRect(10, 10, 280, 140);

    this.ctx.fillStyle = "#6366f1";
    this.ctx.font = "bold 16px Inter, sans-serif";
    this.ctx.fillText("WK v3.0 File Info", 20, 35);

    this.ctx.fillStyle = "#f1f5f9";
    this.ctx.font = "14px Inter, sans-serif";
    this.ctx.fillText(`Dimensions: ${header.width} × ${header.height}`, 20, 60);
    this.ctx.fillText(`Color: ${header.colorType}`, 20, 80);
    this.ctx.fillText(
      `Mode: ${isLossy ? "Lossy" : "Lossless"} (Q${header.quality})`,
      20,
      100,
    );
    this.ctx.fillText(`Chunks: ${chunkCount}`, 20, 120);
    this.ctx.fillText(`Size: ${(fileSize / 1024).toFixed(1)} KB`, 20, 140);

    this.ctx.fillStyle = "#f59e0b";
    this.ctx.font = "12px Inter, sans-serif";
    this.ctx.fillText("Use wkconverter CLI for full decode", 20, 160);
  }

  async loadImageFile(file) {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.canvas.width = img.width;
        this.canvas.height = img.height;
        this.ctx.drawImage(img, 0, 0);

        this.updateInfo({
          size: `${img.width} × ${img.height}`,
          format: file.type.split("/")[1]?.toUpperCase() || "Unknown",
          quality: "-",
          mode: "Original",
        });

        document.getElementById("dropzone").style.display = "none";
        document.getElementById("canvasWrapper").style.display = "flex";

        const imgData = this.ctx.getImageData(0, 0, img.width, img.height);
        this.drawHistogram(imgData.data);
        this.setStatus(`Loaded: ${file.name}`);
        resolve();
      };
      img.onerror = () => reject(new Error("Failed to load image"));
      img.src = URL.createObjectURL(file);
    });
  }

  updateInfo(info) {
    document.getElementById("imgSize").textContent = info.size;
    document.getElementById("imgFormat").textContent = info.format;
    document.getElementById("imgQuality").textContent = info.quality;
    document.getElementById("imgMode").textContent = info.mode;
  }

  drawHistogram(data) {
    const canvas = document.getElementById("histogram");
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    const r = new Array(256).fill(0);
    const g = new Array(256).fill(0);
    const b = new Array(256).fill(0);

    for (let i = 0; i < data.length; i += 4) {
      r[data[i]]++;
      g[data[i + 1]]++;
      b[data[i + 2]]++;
    }

    const max = Math.max(...r, ...g, ...b);
    ctx.fillStyle = "#1a1a25";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    for (let i = 0; i < 256; i++) {
      const x = (i / 256) * canvas.width;
      const rh = (r[i] / max) * canvas.height;
      const gh = (g[i] / max) * canvas.height;
      const bh = (b[i] / max) * canvas.height;

      ctx.fillStyle = "rgba(239, 68, 68, 0.5)";
      ctx.fillRect(x, canvas.height - rh, 1, rh);
      ctx.fillStyle = "rgba(34, 197, 94, 0.5)";
      ctx.fillRect(x, canvas.height - gh, 1, gh);
      ctx.fillStyle = "rgba(59, 130, 246, 0.5)";
      ctx.fillRect(x, canvas.height - bh, 1, bh);
    }
  }

  downloadPng() {
    if (!this.currentFile?.name.endsWith(".wk")) {
      if (!this.canvas) {
        this.showToast("No image loaded", "error");
        return;
      }
      const link = document.createElement("a");
      link.download = (this.currentFile?.name || "image").replace(
        /\.\w+$/,
        ".png",
      );
      link.href = this.canvas.toDataURL("image/png");
      link.click();
      this.showToast("PNG downloaded", "success");
    } else {
      this.showToast("Use wkconverter CLI to convert WK files", "error");
    }
  }

  clear() {
    document.getElementById("dropzone").style.display = "flex";
    document.getElementById("canvasWrapper").style.display = "none";
    this.currentFile = null;
    this.updateInfo({ size: "-", format: "-", quality: "-", mode: "-" });
    this.setStatus("Ready");
  }

  showLoading(show) {
    const el = document.getElementById("loading");
    if (el) el.style.display = show ? "flex" : "none";
  }

  setStatus(text) {
    const el = document.getElementById("status");
    if (el) el.textContent = text;
  }

  showToast(message, type = "") {
    const toast = document.getElementById("toast");
    if (!toast) return;
    toast.textContent = message;
    toast.className = `toast show ${type}`;
    setTimeout(() => toast.classList.remove("show"), 3000);
  }
}

document.addEventListener("DOMContentLoaded", () => new App());
