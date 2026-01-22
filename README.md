<h1 align="center">WK Image Format</h1>

<p align="center">
  <img src="https://img.shields.io/badge/WK-Image%20Format-blueviolet?style=for-the-badge&logo=rust" alt="WK Format"/>
  <img src="https://img.shields.io/badge/version-3.0.0-blue?style=for-the-badge" alt="Version"/>
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License"/>
</p>

<p align="center">
  <strong>Production-grade image format with CABAC, Multi-block DCT, HDR, and Animation support</strong>
</p>

---

## ✨ Features

### Compression Engine

| Feature                      | Description                                |
| ---------------------------- | ------------------------------------------ |
| 🎯 **CABAC**                 | Context-Adaptive Binary Arithmetic Coding  |
| 🔮 **Intra-Prediction**      | 11 modes (DC, Angular, Planar, TrueMotion) |
| 📦 **Multi-block DCT**       | 8×8, 16×16, 32×32 block sizes              |
| ⚡ **Adaptive Quantization** | CSF-weighted perceptual optimization       |
| 🎨 **Color Space**           | YCbCr (BT.601, BT.709, BT.2020)            |
| 📊 **Chroma Subsampling**    | 4:2:0, 4:4:4 support                       |

### HDR & Wide Gamut

| Feature                   | Description                   |
| ------------------------- | ----------------------------- |
| 🌈 **Bit Depth**          | 8, 10, 12, 16-bit support     |
| ☀️ **Transfer Functions** | PQ (HDR10), HLG               |
| 🖥️ **Color Gamuts**       | sRGB, Adobe RGB, P3, Rec.2020 |

### Animation

| Feature                  | Description                            |
| ------------------------ | -------------------------------------- |
| 🎬 **Frame Types**       | I-frames (keyframes), P-frames (delta) |
| 🔄 **Motion Estimation** | Diamond, Hexagon, Three-Step search    |
| ⏱️ **Temporal RDO**      | Optimized keyframe placement           |

### Performance

| Feature          | Description                            |
| ---------------- | -------------------------------------- |
| 🚀 **SIMD**      | SSE4.2 / AVX2 acceleration             |
| 🔧 **Parallel**  | Tile-based multi-threaded encoding     |
| 📡 **Streaming** | Progressive decode with resync markers |

## 📦 Installation

```bash
git clone https://github.com/cowoksoftspoken/WK.git
cd WK
cargo build --release
```

### With Viewer

```bash
cargo build --release --features viewer
```

## 🚀 Usage

### CLI

```bash
# Encode (lossy)
wkconverter encode input.jpg output.wk 85

# Encode (lossless)
wkconverter lossless input.png output.wk

# Decode
wkconverter decode input.wk output.png

# Info
wkconverter info input.wk

# Benchmark
wkconverter benchmark input.jpg ./output/
```

### Viewer

```bash
./target/release/wkviewer
```

Features:

- 🔍 Zoom/Pan (mouse wheel + drag)
- 📈 RGB Histogram
- ⏱️ Decode time metrics
- 🔄 Convert any image to WK
- ⚙️ Advanced compression options

### Library

```rust
use wk_format::{WkEncoder, WkDecoder, WkMetadata};
use wk_format::compression::{IntraMode, IntraPredictor, AdaptiveQuantizer};

// Encode with adaptive quantization
let encoder = WkEncoder::lossy(85);
let encoded = encoder.encode_to_vec(&image)?;

// Use intra-prediction
let predictor = IntraPredictor::new(8);
let (best_mode, sad) = predictor.select_best_mode(&block, &top, &left, top_left);
```

## 🔧 Technical Details

### File Structure

```
┌─────────────────────────────────────┐
│ Magic: "WK3.0\x00\x00\x00"          │
├─────────────────────────────────────┤
│ IHDR (Header)                       │
│ ├─ Dimensions, Color Type           │
│ ├─ Compression Mode, Quality        │
│ └─ HDR Metadata                     │
├─────────────────────────────────────┤
│ ICCP (ICC Profile)                  │
├─────────────────────────────────────┤
│ IDAT (Image Data)                   │
│ ├─ Quantization Tables              │
│ ├─ Intra-Prediction Modes           │
│ └─ CABAC Encoded Coefficients       │
├─────────────────────────────────────┤
│ IEND (End)                          │
└─────────────────────────────────────┘
```

### Compression Pipeline

**Lossless:**

```
Image → Predictor (optimal) → Huffman → Output
```

**Lossy:**

```
Image → YCbCr → Intra-Pred → DCT → Quantize (CSF) → CABAC → Output
```

## 📁 Project Structure

```
src/
├── compression/
│   ├── multi_dct.rs      # Multi-block DCT (8×8, 16×16)
│   ├── intra_prediction.rs # 11 prediction modes
│   ├── cabac.rs          # Arithmetic coding
│   ├── adaptive_quant.rs # CSF-weighted quantization
│   ├── color.rs          # YCbCr conversion
│   └── simd.rs           # SSE4/AVX2 acceleration
├── format/
│   ├── hdr.rs            # HDR/PQ/HLG support
│   └── progressive.rs    # Tiling & streaming
├── animation/
│   └── motion.rs         # Motion estimation
└── bin/
    └── viewer.rs         # GUI with histogram
```

## 📜 License

MIT License

## 👨‍💻 Author

**Inggrit Setya Budi** ([@cowoksoftspoken](https://github.com/cowoksoftspoken))
