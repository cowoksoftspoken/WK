const WK_MAGIC = [0x57, 0x4b, 0x33, 0x2e, 0x30, 0x00, 0x00, 0x00];

let wasm = null;
let currentImageData = null;
let currentWkData = null;
let currentFileName = null;

async function initWasm() {
  try {
    const module = await import("./wk_format.js");
    await module.default();
    wasm = module;
    wasm.init_panic_hook();
    console.log("WASM loaded");
    return true;
  } catch (e) {
    console.error("WASM init failed:", e);
    return false;
  }
}

class App {
  constructor() {
    this.canvas = document.getElementById("canvas");
    this.ctx = this.canvas.getContext("2d");
    this.init();
  }

  init() {
    const dropzone = document.getElementById("dropzone");
    const fileInput = document.getElementById("fileInput");
    const qualitySlider = document.getElementById("qualitySlider");
    const qualityValue = document.getElementById("qualityValue");

    dropzone.addEventListener("click", () => fileInput.click());
    dropzone.addEventListener("dragover", (e) => {
      e.preventDefault();
      dropzone.classList.add("drag-over");
    });
    dropzone.addEventListener("dragleave", () =>
      dropzone.classList.remove("drag-over"),
    );
    dropzone.addEventListener("drop", (e) => {
      e.preventDefault();
      dropzone.classList.remove("drag-over");
      if (e.dataTransfer.files[0]) this.loadFile(e.dataTransfer.files[0]);
    });
    fileInput.addEventListener("change", (e) => {
      if (e.target.files[0]) this.loadFile(e.target.files[0]);
    });

    qualitySlider.addEventListener("input", (e) => {
      qualityValue.textContent = e.target.value;
    });

    document
      .getElementById("encodeBtn")
      .addEventListener("click", () => this.encodeToWk());
    document
      .getElementById("downloadPng")
      .addEventListener("click", () => this.downloadPng());
    document
      .getElementById("downloadWk")
      .addEventListener("click", () => this.downloadWk());
    document
      .getElementById("clearBtn")
      .addEventListener("click", () => this.clear());

    document.querySelectorAll(".nav-btn").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        document
          .querySelectorAll(".nav-btn")
          .forEach((b) => b.classList.remove("active"));
        e.target.classList.add("active");
      });
    });
  }

  showLoading(show, text = "Processing...") {
    document.getElementById("loading").style.display = show ? "flex" : "none";
    document.getElementById("loadingText").textContent = text;
  }

  showToast(message, type = "info") {
    const toast = document.getElementById("toast");
    toast.textContent = message;
    toast.className = `toast ${type} show`;
    setTimeout(() => toast.classList.remove("show"), 3000);
  }

  updateStatus(text) {
    document.getElementById("status").textContent = text;
  }

  async loadFile(file) {
    this.showLoading(true, "Loading...");
    currentFileName = file.name.replace(/\.[^/.]+$/, "");

    try {
      if (file.name.endsWith(".wk")) {
        await this.loadWkFile(file);
      } else {
        await this.loadImageFile(file);
      }
      this.showToast("File loaded", "success");
    } catch (error) {
      console.error(error);
      this.showToast(error.message, "error");
    } finally {
      this.showLoading(false);
    }
  }

  async loadWkFile(file) {
    const buffer = await file.arrayBuffer();
    const data = new Uint8Array(buffer);
    currentWkData = data;

    if (!wasm) await initWasm();

    if (wasm) {
      try {
        const image = wasm.decode_wk(data);
        this.displayImage(image.get_pixels(), image.width, image.height);
        this.updateInfo({
          width: image.width,
          height: image.height,
          colorType: image.color_type,
          compression: image.compression,
          quality: image.quality,
          fileSize: data.length,
        });
        document.getElementById("downloadPng").disabled = false;
        document.getElementById("downloadWk").disabled = false;
        document.getElementById("encodeBtn").disabled = true;
        this.updateStatus("WK file decoded");
        return;
      } catch (e) {
        console.error("WASM decode failed:", e);
      }
    }

    throw new Error("Failed to decode WK file");
  }

  async loadImageFile(file) {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.canvas.width = img.width;
        this.canvas.height = img.height;
        this.ctx.drawImage(img, 0, 0);

        currentImageData = this.ctx.getImageData(0, 0, img.width, img.height);
        currentWkData = null;

        document.getElementById("dropzone").style.display = "none";
        document.getElementById("canvasWrapper").style.display = "flex";

        this.updateInfo({
          width: img.width,
          height: img.height,
          colorType: "RGBA",
          compression: "Source",
          quality: "-",
          fileSize: file.size,
        });

        document.getElementById("encodeBtn").disabled = false;
        document.getElementById("downloadPng").disabled = false;
        document.getElementById("downloadWk").disabled = true;
        this.updateStatus("Image loaded - ready to encode");
        resolve();
      };
      img.onerror = () => reject(new Error("Failed to load image"));
      img.src = URL.createObjectURL(file);
    });
  }

  displayImage(pixels, width, height) {
    this.canvas.width = width;
    this.canvas.height = height;
    const imageData = new ImageData(
      new Uint8ClampedArray(pixels),
      width,
      height,
    );
    this.ctx.putImageData(imageData, 0, 0);
    currentImageData = imageData;

    document.getElementById("dropzone").style.display = "none";
    document.getElementById("canvasWrapper").style.display = "flex";
  }

  updateInfo(info) {
    document.getElementById("imgSize").textContent =
      `${info.width}×${info.height}`;
    document.getElementById("imgFormat").textContent = info.colorType;
    document.getElementById("imgQuality").textContent = info.quality;
    document.getElementById("imgFileSize").textContent = this.formatSize(
      info.fileSize,
    );
    document.getElementById("imgMode").textContent = info.compression;
  }

  formatSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(2) + " MB";
  }

  async encodeToWk() {
    if (!currentImageData) {
      this.showToast("No image loaded", "error");
      return;
    }

    if (!wasm) {
      const loaded = await initWasm();
      if (!loaded) {
        this.showToast("WASM not available", "error");
        return;
      }
    }

    this.showLoading(true, "Encoding to WK...");
    this.updateStatus("Encoding...");

    try {
      await new Promise((r) => setTimeout(r, 50));

      const quality = parseInt(document.getElementById("qualitySlider").value);
      const pixels = new Uint8Array(currentImageData.data);

      currentWkData = wasm.encode_wk(
        pixels,
        this.canvas.width,
        this.canvas.height,
        quality,
      );

      this.updateInfo({
        width: this.canvas.width,
        height: this.canvas.height,
        colorType: "RGBA",
        compression: "Lossy",
        quality: quality,
        fileSize: currentWkData.length,
      });

      document.getElementById("downloadWk").disabled = false;
      this.updateStatus(`Encoded: ${this.formatSize(currentWkData.length)}`);
      this.showToast(
        `Encoded to WK (${this.formatSize(currentWkData.length)})`,
        "success",
      );
    } catch (e) {
      console.error("Encode failed:", e);
      this.showToast("Encode failed: " + e, "error");
      this.updateStatus("Encode failed");
    } finally {
      this.showLoading(false);
    }
  }

  downloadPng() {
    if (!currentImageData) {
      this.showToast("No image to download", "error");
      return;
    }

    this.canvas.toBlob((blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = (currentFileName || "image") + ".png";
      a.click();
      URL.revokeObjectURL(url);
      this.showToast("PNG downloaded", "success");
    }, "image/png");
  }

  downloadWk() {
    if (!currentWkData) {
      this.showToast("No WK data to download", "error");
      return;
    }

    const blob = new Blob([currentWkData], {
      type: "application/octet-stream",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (currentFileName || "image") + ".wk";
    a.click();
    URL.revokeObjectURL(url);
    this.showToast("WK file downloaded", "success");
  }

  clear() {
    currentImageData = null;
    currentWkData = null;
    currentFileName = null;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    document.getElementById("dropzone").style.display = "flex";
    document.getElementById("canvasWrapper").style.display = "none";
    document.getElementById("encodeBtn").disabled = true;
    document.getElementById("downloadPng").disabled = true;
    document.getElementById("downloadWk").disabled = true;
    document.getElementById("imgSize").textContent = "-";
    document.getElementById("imgFormat").textContent = "-";
    document.getElementById("imgQuality").textContent = "-";
    document.getElementById("imgFileSize").textContent = "-";
    document.getElementById("imgMode").textContent = "-";
    this.updateStatus("Ready");
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  await initWasm();
  new App();
});
