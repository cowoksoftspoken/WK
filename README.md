<h1 align="center">WK Image Format</h1>

<p align="center">
  <img src="https://img.shields.io/badge/WK-Image%20Format-blueviolet?style=for-the-badge&logo=rust" alt="WK Format"/>
  <img src="https://img.shields.io/badge/version-2.0.0-blue?style=for-the-badge" alt="Version"/>
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License"/>
</p>

<p align="center">
  <strong>A production-grade image format with Predictive Compression and Adaptive Block Quantization</strong>
</p>

---

## ✨ Features

| Feature                       | Description                                                   |
| ----------------------------- | ------------------------------------------------------------- |
| 🎯 **Lossy & Lossless**       | Both compression modes with quality control (1-100)           |
| 🔮 **Predictive Compression** | 5 filters (None, Sub, Up, Average, Paeth) with auto-selection |
| 📦 **8x8 DCT Transform**      | JPEG-compatible discrete cosine transform                     |
| ⚡ **Adaptive Quantization**  | Quality-dependent compression with perceptual weighting       |
| 🔐 **CRC32 Integrity**        | Per-chunk data verification                                   |
| 📸 **EXIF Metadata**          | Camera info, GPS, ISO, aperture, focal length                 |
| 🎨 **ICC Color Profiles**     | sRGB, Adobe RGB, Display P3, ProPhoto RGB, Rec.2020           |
| 📝 **XMP Metadata**           | Title, description, creators, ratings, subjects               |
| 🎬 **Animation Support**      | Frame delay, blend modes, dispose modes                       |
| 🔌 **Extensible**             | Chunk-based format for future additions                       |

## 📦 Installation

### From Source

```bash
git clone https://github.com/cowoksoftspoken/WK.git
cd WK
cargo build --release
```

### With Viewer (GUI)

```bash
cargo build --release --features viewer
```

## 🚀 Usage

### CLI Commands

```bash
# Encode image to WK (lossy)
wkconverter encode input.jpg output.wk 85

# Encode lossless
wkconverter lossless input.png output.wk

# Decode WK to image
wkconverter decode input.wk output.png

# View file information
wkconverter info input.wk

# Run benchmark
wkconverter benchmark input.jpg ./output_dir
```

### GUI Viewer

```bash
./target/release/wkviewer
```

- Drag & drop any image (PNG, JPEG, WebP, BMP, GIF, TIFF)
- Convert to WK format with quality slider
- View file metadata and compression info

### Rust Library

```rust
use wk_format::{WkEncoder, WkDecoder, WkMetadata};
use wk_format::metadata::exif::ExifBuilder;
use wk_format::metadata::icc::IccProfile;

// Encode with metadata
let exif = ExifBuilder::new()
    .make("Canon")
    .model("EOS R5")
    .iso(800)
    .build();

let metadata = WkMetadata::new()
    .with_exif(exif)
    .with_icc(IccProfile::srgb());

let encoder = WkEncoder::lossy(85).with_metadata(metadata);
let encoded = encoder.encode_to_vec(&image)?;

// Decode
let decoder = WkDecoder::new();
let decoded = decoder.decode(&encoded[..])?;
println!("{}x{}", decoded.image.width(), decoded.image.height());
```

## 🔧 Technical Details

### File Structure

```
┌─────────────────────────────────────┐
│ Magic Number: "WK2.0\x00\x00\x00"   │  8 bytes
├─────────────────────────────────────┤
│ IHDR Chunk (Image Header)           │
│ ├─ Width, Height                    │
│ ├─ Color Type, Compression Mode     │
│ └─ Quality, Flags, Bit Depth        │
├─────────────────────────────────────┤
│ ICCP Chunk (ICC Profile) [optional] │
├─────────────────────────────────────┤
│ EXIF Chunk (EXIF Data) [optional]   │
├─────────────────────────────────────┤
│ XMP Chunk (XMP Data) [optional]     │
├─────────────────────────────────────┤
│ IDAT/IDLS Chunk (Image Data)        │
│ ├─ Quantization Tables (lossy)      │
│ └─ Compressed Coefficients          │
├─────────────────────────────────────┤
│ IEND Chunk (End Marker)             │
└─────────────────────────────────────┘
```

### Compression Pipeline

**Lossless Mode:**

```
Image → Predictive Filter (optimal per-row) → Huffman Encoding → Output
```

**Lossy Mode:**

```
Image → 8x8 Blocks → DCT → Quantization → Zigzag → RLE → Huffman → Output
```

### Supported Color Types

| Type             | Channels | Description                |
| ---------------- | -------- | -------------------------- |
| `Grayscale`      | 1        | Single channel             |
| `GrayscaleAlpha` | 2        | Grayscale + Alpha          |
| `Rgb`            | 3        | Red, Green, Blue           |
| `Rgba`           | 4        | RGB + Alpha                |
| `Yuv420`         | 3        | YUV with 4:2:0 subsampling |
| `Yuv444`         | 3        | YUV without subsampling    |

## 📊 Benchmarks

Quality vs File Size (217x233 test image):

| Quality | Mode     | File Size | Ratio |
| ------- | -------- | --------- | ----- |
| 100     | Lossless | 86 KB     | 57%   |
| 95      | Lossy    | 46 KB     | 31%   |
| 85      | Lossy    | 26 KB     | 17%   |
| 50      | Lossy    | 15 KB     | 10%   |
| 25      | Lossy    | 8 KB      | 5%    |

## 🆚 Comparison with Other Formats

| Feature     | WK v2.0 | WebP | JPEG | PNG |
| ----------- | ------- | ---- | ---- | --- |
| Lossy       | ✅      | ✅   | ✅   | ❌  |
| Lossless    | ✅      | ✅   | ❌   | ✅  |
| Alpha       | ✅      | ✅   | ❌   | ✅  |
| Animation   | ✅      | ✅   | ❌   | ❌  |
| EXIF        | ✅      | ✅   | ✅   | ❌  |
| ICC Profile | ✅      | ✅   | ✅   | ✅  |
| XMP         | ✅      | ✅   | ✅   | ✅  |
| Extensible  | ✅      | ✅   | ❌   | ✅  |
| Open Source | ✅      | ✅   | ✅   | ✅  |

## 📁 Project Structure

```
WK/
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # CLI (wkconverter)
│   ├── encoder.rs          # WK encoder
│   ├── decoder.rs          # WK decoder
│   ├── converter.rs        # High-level converter
│   ├── error.rs            # Error types
│   ├── format/
│   │   ├── chunk.rs        # Chunk container
│   │   └── header.rs       # File header
│   ├── compression/
│   │   ├── dct.rs          # DCT/IDCT transform
│   │   ├── quantizer.rs    # Adaptive quantization
│   │   ├── predictor.rs    # Predictive filters
│   │   ├── entropy.rs      # Huffman coding
│   │   └── engine.rs       # Compression engine
│   ├── metadata/
│   │   ├── exif.rs         # EXIF support
│   │   ├── icc.rs          # ICC profiles
│   │   ├── xmp.rs          # XMP metadata
│   │   └── custom.rs       # Custom fields
│   ├── animation/
│   │   └── frame.rs        # Animation frames
│   └── bin/
│       ├── viewer.rs       # GUI viewer (egui)
│       └── debug.rs        # Debug tool
└── viewer/
    ├── index.html          # Web viewer
    ├── main.js             # JavaScript decoder
    └── styles.css          # Viewer styles
```

## 🌐 Web Viewer

Open `viewer/index.html` in a browser to view WK files without installing anything.

Features:

- Drag & drop WK files
- View image info and metadata
- Download as PNG
- Supports WK v2.0 format

## 🛠️ Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With viewer feature
cargo build --release --features viewer

# Run tests
cargo test
```

## 📜 License

MIT License - see [LICENSE](LICENSE) for details

## 👨‍💻 Author

**Inggrit Setya Budi** ([@cowoksoftspoken](https://github.com/cowoksoftspoken))
