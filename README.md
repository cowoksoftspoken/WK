<h1 align="center">
  WK Image Format
</h1>

<p align="center">
  <img src="https://img.shields.io/badge/WK-Image%20Format-blueviolet?style=for-the-badge&logo=rust" alt="WK Format"/>
  <img src="https://img.shields.io/badge/version-3.1.1-blue?style=for-the-badge" alt="Version"/>
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License"/>
  <img src="https://img.shields.io/badge/rust-1.70+-orange?style=for-the-badge&logo=rust" alt="Rust"/>
</p>

<p align="center">
  <strong>A modern image format with efficient compression, HDR support, animation capabilities, and WebAssembly for browser integration</strong>
</p>

---

## What is WK?

**WK** is a next-generation image format designed to provide efficient compression while maintaining high visual quality. It combines proven techniques from modern video codecs (like CABAC from H.264/HEVC and intra-prediction from VP8/VP9) with a flexible chunk-based file structure.

### Why Another Image Format?

| Problem                                              | WK Solution                                                           |
| ---------------------------------------------------- | --------------------------------------------------------------------- |
| JPEG is lossy but has limited compression efficiency | WK uses CABAC entropy coding + DCT for better compression ratios      |
| PNG is lossless but produces large files             | WK offers both lossy and lossless modes with better compression       |
| WebP has good compression but limited tooling        | WK is built in Rust with native CLI, desktop viewer, and WASM support |
| Most formats lack HDR support                        | WK supports 10/12/16-bit depth with PQ and HLG transfer functions     |
| Existing formats don't work well in browsers         | WK compiles to WebAssembly (187KB) for full browser decoding          |

### Key Design Goals

1. **Efficient Compression** - Competitive with WebP/AVIF while being simpler to implement
2. **High Quality** - Preserve visual fidelity using perceptually-optimized quantization
3. **Cross-Platform** - Works on desktop (Rust), web (WASM), with CLI tools
4. **Extensible** - Chunk-based format allows future extensions without breaking compatibility
5. **Open Source** - MIT licensed, fully documented

---

## Features

### Compression Engine

WK uses a multi-stage compression pipeline inspired by modern video codecs:

| Component             | What It Does                                               | Why It Matters                                                      |
| --------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------- |
| **DCT Transform**     | Converts 8×8 pixel blocks to frequency domain              | Separates image into components that can be quantized independently |
| **Intra-Prediction**  | Predicts pixel values from neighboring blocks (11 modes)   | Reduces data to encode by exploiting spatial redundancy             |
| **Quantization**      | Reduces precision of DCT coefficients based on JPEG tables | Controls quality/size tradeoff, removes imperceptible details       |
| **Exp-Golomb Coding** | Variable-length encoding for coefficient values            | Efficiently encodes small values (common in quantized data)         |
| **Zlib Compression**  | Final compression layer using DEFLATE algorithm            | Further reduces file size (typically 30-50% reduction)              |

### Color & HDR Support

| Feature                | Specification                         |
| ---------------------- | ------------------------------------- |
| **Bit Depth**          | 8, 10, 12, 16-bit per channel         |
| **Color Spaces**       | sRGB, Adobe RGB, Display P3, Rec.2020 |
| **Transfer Functions** | Gamma, PQ (HDR10), HLG                |
| **Chroma Subsampling** | 4:4:4 (full), 4:2:0 (reduced)         |
| **ICC Profiles**       | Embedded profile support              |

### Animation

WK supports animated images using frame-based encoding:

- **I-frames (Keyframes)**: Complete images, used as reference
- **P-frames (Delta frames)**: Only differences from previous frame
- **Motion Estimation**: Diamond, Hexagon, Three-Step search algorithms
- **Temporal Optimization**: Intelligent keyframe placement

### Performance

| Optimization        | Implementation                          |
| ------------------- | --------------------------------------- |
| **SIMD**            | SSE4.2 and AVX2 intrinsics for DCT/IDCT |
| **Multi-threading** | Rayon-based parallel block processing   |
| **WebAssembly**     | 187KB WASM module for browser decoding  |
| **Streaming**       | Progressive decode with resync markers  |

---

## Installation

### Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **wasm-pack** (optional) - For WebAssembly builds

### Build from Source

```bash
# Clone the repository
git clone https://github.com/cowoksoftspoken/WK.git
cd WK

# Build release binaries
cargo build --release

# Binaries are located at:
# ./target/release/wkconverter    (CLI tool)
# ./target/release/wkviewer       (requires --features viewer)
```

### Build Desktop Viewer

The desktop viewer provides a GUI for viewing and converting images:

```bash
cargo build --release --features viewer
```

### Build WebAssembly Module

To use WK in web browsers:

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build WASM package
wasm-pack build --target web --features wasm

# Output files in pkg/ directory:
# - wk_format.js        (JavaScript bindings)
# - wk_format_bg.wasm   (WASM binary, ~187KB)
# - wk_format.d.ts      (TypeScript definitions)
```

---

## Usage

### Command Line Interface

The `wkconverter` CLI provides all encoding/decoding functionality:

#### Encode (Lossy Compression)

```bash
# Basic usage: encode <input> <output> <quality>
# Quality range: 1-100 (higher = better quality, larger file)

wkconverter encode photo.jpg photo.wk 85

# Examples with different quality levels:
wkconverter encode input.png output.wk 95  # High quality
wkconverter encode input.png output.wk 75  # Balanced
wkconverter encode input.png output.wk 50  # Smaller file
```

**How Quality Affects File Size (for 6KB source JPEG):**

| Quality | Output Size | Compression Ratio |
| ------- | ----------- | ----------------- |
| Q78     | 18.5KB      | 3.05x             |
| Q80     | 19.8KB      | 3.26x             |
| Q85     | 22.5KB      | 3.70x             |
| Q90     | ~25KB       | ~4.1x             |

#### Encode (Lossless)

```bash
# Lossless compression preserves every pixel exactly
wkconverter lossless input.png output.wk
```

#### Decode

```bash
# Decode WK file to common formats
wkconverter decode input.wk output.png
wkconverter decode input.wk output.jpg
wkconverter decode input.wk output.webp
```

#### View File Information

```bash
wkconverter info input.wk

# Output shows:
# - Dimensions
# - Color type
# - Compression mode
# - Quality setting
# - File size
```

#### Benchmark

```bash
# Run compression benchmark
wkconverter benchmark input.jpg ./output/
```

### Desktop Viewer

The WK Viewer provides a full-featured GUI application:

```bash
./target/release/wkviewer
```

**Features:**

- **Zoom & Pan** - Mouse wheel zoom, drag to pan
- **Histogram** - Real-time RGB histogram display
- **Metrics** - Decode time and compression stats
- **Convert** - Convert any image to WK format
- **Options** - Configure CABAC, intra-prediction, adaptive quantization
- **Batch** - Process multiple files at once

### Web Viewer

The web viewer uses WebAssembly for full client-side decoding:

```bash
# Start a local web server
cd viewer
python -m http.server 8080

# Open in browser: http://localhost:8080
```

**Features:**

- Drag & drop WK files
- Full pixel decoding via WASM (no server processing)
- Display image info (dimensions, quality, compression mode)
- Convert and download files

---

## WebAssembly Integration

### How It Works

WK compiles to WebAssembly, allowing browsers to decode WK files natively without server-side processing:

1. **wasm-pack** compiles Rust code to WASM binary
2. **wasm-bindgen** generates JavaScript bindings
3. Browser loads 187KB WASM module
4. JavaScript calls Rust decoder functions directly

### Integration Example

```javascript
// Import the WASM module
import init, { decode_wk } from "./wk_format.js";

async function loadWkImage(file) {
  // Initialize WASM (do this once)
  await init();

  // Read file as ArrayBuffer
  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);

  // Decode using WASM
  const image = decode_wk(data);

  // Access decoded data
  console.log("Dimensions:", image.width, "x", image.height);
  console.log("Quality:", image.quality);
  console.log("Compression:", image.compression);

  // Get RGBA pixel data for canvas
  const pixels = image.get_pixels();

  // Draw to canvas
  const canvas = document.getElementById("canvas");
  const ctx = canvas.getContext("2d");
  canvas.width = image.width;
  canvas.height = image.height;

  const imageData = ctx.createImageData(image.width, image.height);
  imageData.data.set(pixels);
  ctx.putImageData(imageData, 0, 0);
}
```

### Serving WASM Files

WASM files require proper MIME types. Configure your web server:

```nginx
# Nginx
types {
    application/wasm wasm;
}
```

```apache
# Apache
AddType application/wasm .wasm
```

---

## File Format Specification

### Overview

WK uses a chunk-based format similar to PNG, making it easy to parse and extend:

```
┌─────────────────────────────────────────┐
│ Magic Number: "WK3.0\x00\x00\x00"       │ 8 bytes
├─────────────────────────────────────────┤
│ Chunk 1: IHDR (Image Header)            │
│ ├─ Type: 4 bytes ("IHDR")               │
│ ├─ Length: 4 bytes (little-endian)      │
│ ├─ Data: Variable                       │
│ └─ CRC32: 4 bytes                       │
├─────────────────────────────────────────┤
│ Chunk 2: ICCP (ICC Profile) [optional]  │
├─────────────────────────────────────────┤
│ Chunk 3: IDAT or IDLS (Image Data)      │
│ ├─ IDAT: Lossless compressed data       │
│ └─ IDLS: Lossy compressed data          │
├─────────────────────────────────────────┤
│ Chunk N: IEND (End Marker)              │
└─────────────────────────────────────────┘
```

### IHDR (Image Header) Structure

| Field         | Size    | Description                        |
| ------------- | ------- | ---------------------------------- |
| Width         | 4 bytes | Image width in pixels              |
| Height        | 4 bytes | Image height in pixels             |
| Color Type    | 1 byte  | 0=Gray, 1=GrayAlpha, 2=RGB, 3=RGBA |
| Compression   | 1 byte  | 0=Lossless, 1=Lossy                |
| Quality       | 1 byte  | 1-100 for lossy mode               |
| Has Alpha     | 1 byte  | Alpha channel present              |
| Has Animation | 1 byte  | Animated image                     |
| Bit Depth     | 1 byte  | Bits per channel (8/10/12/16)      |

### IDLS (Lossy Data) Structure

```
┌──────────────────────────────────────┐
│ Flags (3 bytes)                      │
│ ├─ use_cabac: 1 byte                 │
│ ├─ use_intra: 1 byte                 │
│ └─ use_adaptive: 1 byte              │
├──────────────────────────────────────┤
│ Luma Quant Table (128 bytes)         │
│ └─ 64 × u16 values                   │
├──────────────────────────────────────┤
│ Chroma Quant Table (128 bytes)       │
│ └─ 64 × u16 values                   │
├──────────────────────────────────────┤
│ Compressed Length (4 bytes)          │
├──────────────────────────────────────┤
│ Zlib Compressed Data                 │
│ ├─ Intra-prediction modes            │
│ ├─ Block QP values                   │
│ └─ Encoded coefficients              │
└──────────────────────────────────────┘
```

---

## Project Structure

```
WK/
├── src/
│   ├── lib.rs                    # Library entry point, public API
│   ├── main.rs                   # CLI application (wkconverter)
│   ├── wasm.rs                   # WebAssembly bindings
│   │
│   ├── compression/              # Compression engine
│   │   ├── engine.rs             # Main encode/decode orchestration
│   │   ├── cabac.rs              # Exp-Golomb + bit-level encoding
│   │   ├── dct.rs                # 8×8 DCT/IDCT transforms
│   │   ├── intra_prediction.rs   # 11 prediction modes
│   │   ├── adaptive_quant.rs     # JPEG-based quantization tables
│   │   ├── quantizer.rs          # Coefficient quantization
│   │   ├── entropy.rs            # Huffman encoding (lossless)
│   │   ├── predictor.rs          # Pixel predictors (lossless)
│   │   ├── color.rs              # RGB ↔ YCbCr conversion
│   │   └── simd.rs               # SIMD-optimized functions
│   │
│   ├── format/                   # File format handling
│   │   ├── header.rs             # IHDR structure
│   │   ├── chunk.rs              # Chunk read/write
│   │   ├── hdr.rs                # HDR metadata, PQ/HLG
│   │   └── progressive.rs        # Progressive/streaming decode
│   │
│   ├── animation/                # Animation support
│   │   ├── mod.rs                # Animation types
│   │   └── motion.rs             # Motion estimation algorithms
│   │
│   └── bin/
│       ├── viewer.rs             # Desktop GUI (egui/eframe)
│       └── debug.rs              # Debug utilities
│
├── viewer/                       # Web viewer application
│   ├── index.html                # Main HTML page
│   ├── main.js                   # JavaScript application
│   ├── styles.css                # Styling
│   ├── wasm_loader.js            # WASM integration helper
│   ├── wk_format.js              # WASM bindings (from pkg/)
│   └── wk_format_bg.wasm         # WASM binary (from pkg/)
│
├── pkg/                          # WASM package (generated by wasm-pack)
├── tests/                        # Test files and outputs
├── Cargo.toml                    # Rust dependencies and features
└── README.md                     # This file
```

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test compression
cargo test format

# Current test status: 14 tests passing
```

---

## Build Features

Configure build with Cargo features:

```toml
[features]
default = []
simd = []           # Enable SIMD optimizations
gpu = ["wgpu"]      # GPU acceleration (experimental)
viewer = [...]      # Desktop GUI viewer
wasm = [...]        # WebAssembly support
```

```bash
# Build with specific features
cargo build --release --features "viewer"
cargo build --release --features "wasm"
cargo build --release --features "simd,viewer"
```

---

## License

MIT License - See [LICENSE](LICENSE) for details.

---

## Author

**Inggrit Setya Budi** ([@cowoksoftspoken](https://github.com/cowoksoftspoken))
