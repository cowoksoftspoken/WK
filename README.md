<h1 align="center">WK Image Format</h1>

<p align="center">
  <img src="https://img.shields.io/badge/WK-Image%20Format-blueviolet?style=for-the-badge&logo=rust" alt="WK Format"/>
  <img src="https://img.shields.io/badge/version-3.0.0-blue?style=for-the-badge" alt="Version"/>
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License"/>
</p>

<p align="center">
  <strong>A production-grade image format with CABAC, Multi-block DCT, and Intra-frame Prediction</strong>
</p>

---

## Features

| Feature                       | Description                                                                  |
| ----------------------------- | ---------------------------------------------------------------------------- |
| **Advanced Compression**      | CABAC entropy coding for superior compression ratios.                        |
| **Multi-block DCT**           | An advanced transform method that improves upon traditional DCT.             |
| **Intra-frame Prediction**    | Reduces spatial redundancy for both lossy and lossless modes.                |
| **Adaptive Quantization**     | Optimizes quality across the image, preserving detail where it matters most. |
| **Lossy & Lossless**          | Both modes are available with fine-grained quality control.                  |
| **HDR Support**               | Full support for High Dynamic Range images.                                  |
| **Animation**                 | Frame-based animation with disposal and blend modes.                         |
| **Rich Metadata**             | Retains EXIF, XMP, and ICC color profiles.                                   |
| **Data Integrity**            | CRC32 checksums for each chunk to ensure file integrity.                     |
| **High Performance**          | SIMD and GPU accelerated operations for fast encoding and decoding.          |
| **Extensible**                | A chunk-based architecture allows for future feature expansion.              |

## Installation

### From Source

```bash
git clone https://github.com/cowoksoftspoken/WK.git
cd WK
cargo build --release
```

### With Viewer (GUI)

To include the graphical viewer, build with the `viewer` feature:
```bash
cargo build --release --features viewer
```

## Usage

### Command-Line Interface (CLI)

```bash
# Encode an image with lossy compression (e.g., quality 85)
wkconverter encode input.jpg output.wk 85

# Encode an image with lossless compression
wkconverter lossless input.png output.wk

# Decode a WK image
wkconverter decode input.wk output.png

# View file information and metadata
wkconverter info input.wk

# Run a benchmark against multiple quality levels
wkconverter benchmark input.jpg ./output_dir
```

### GUI Viewer

Launch the viewer from the target directory:
```bash
./target/release/wkviewer
```
- Drag and drop images (WK, PNG, JPEG, etc.) to view and convert.
- Use the quality slider to select a compression level for encoding.
- Inspect file metadata and compression details.

### Rust Crate Usage

To use the `wk-image-format` crate in your project, add it to `Cargo.toml`:
```toml
[dependencies]
wk-image-format = "3.0.0"
```

**Example 1: Basic Encoding**
```rust
use wk_format::{WkEncoder, WkDecoder, WkMetadata};
use image::DynamicImage;

// Encode an image with adaptive quantization and default settings
let encoder = WkEncoder::lossy(85);
let encoded_data: Vec<u8> = encoder.encode_to_vec(&image)?;

// Decode the image
let decoder = WkDecoder::new();
let decoded_image = decoder.decode(&encoded_data[..])?;
println!("Decoded dimensions: {}x{}", decoded_image.image.width(), decoded_image.image.height());
```

**Example 2: Using Advanced Compression Features**
```rust
use wk_format::{WkEncoder, WkDecoder, WkMetadata};
use wk_format::compression::{IntraMode, IntraPredictor, AdaptiveQuantizer};

// WK's internal APIs allow for fine-grained control
// This is a conceptual preview of using an intra-predictor

// The encoder automatically selects the best intra-prediction mode
let encoder = WkEncoder::lossy(85);
let encoded = encoder.encode_to_vec(&image)?;

// During encoding, the engine selects the best mode for a block
let predictor = IntraPredictor::new(8); // 8x8 block size
// The `select_best_mode` function finds the mode with the lowest cost (SAD)
let (best_mode, sad) = predictor.select_best_mode(&block, &top, &left, top_left);
```

## Technical Details

### File Structure

```
┌─────────────────────────────────────┐
│ Magic Number: "WK3.0"   │  8 bytes  |  
├─────────────────────────────────────┤
│ IHDR Chunk (Image Header)           │
│ ├─ Width, Height, Color Type        │
│ ├─ Compression, Quality, Flags      │
│ └─ Bit Depth, Animation Frames      │
├─────────────────────────────────────┤
│ ICCP Chunk (ICC Profile) [optional] │
├─────────────────────────────────────┤
│ EXIF Chunk (EXIF Data) [optional]   │
├─────────────────────────────────────┤
│ XMP Chunk (XMP Data) [optional]     │
├─────────────────────────────────────┤
│ fRAm Chunk (Animation) [optional]   │
├─────────────────────────────────────┤
│ IDAT/IDLS Chunk (Image Data)        │
│ └─ Compressed Pixel Data            │
├─────────────────────────────────────┤
│ IEND Chunk (End Marker)             │
└─────────────────────────────────────┘
```

### Compression Pipeline

**Lossless Mode:**
`Image → Intra-Prediction → CABAC Encoding → Output`

**Lossy Mode:**
`Image → Color Space Transform → Multi-block DCT → Adaptive Quantization → Zigzag Scan → CABAC Encoding → Output`

## Comparison with Other Formats

| Feature                  | WK v3.0 | WebP | AVIF | JPEG XL | PNG |
| ------------------------ |:-------:|:----:|:----:|:-------:|:---:|
| **Lossy Compression**    | ✅      | ✅   | ✅   | ✅      | ❌  |
| **Lossless Compression** | ✅      | ✅   | ✅   | ✅      | ✅  |
| **Alpha Channel**        | ✅      | ✅   | ✅   | ✅      | ✅  |
| **Animation**            | ✅      | ✅   | ✅   | ✅      | ❌  |
| **HDR Support**          | ✅      | ❌   | ✅   | ✅      | ❌  |
| **Progressive Decode**   | ✅      | ✅   | ❌   | ✅      | ✅  |
| **Intra-Prediction**     | ✅      | ✅   | ✅   | ✅      | ✅  |
| **Advanced Entropy**     | ✅ (CABAC) | ✅ (Arithmetic) | ✅ (Arithmetic) | ✅ (ANS)  | ❌  |
| **Royalty-Free**         | ✅      | ✅   | ✅   | ✅      | ✅  |

## Benchmarks

Performance benchmarks for a sample `1024x768` image on a standard desktop CPU. (Lower is better for size/time).

**Lossy Mode (Quality ≈85):**

| Format  | File Size | Compression Ratio | Encode Time | Decode Time |
|---------|:---------:|:-----------------:|:-----------:|:-----------:|
| **WK v3.0** | **112 KB**|      **4.5%**     |  **~35 ms** |  **~28 ms** |
| WebP    |   125 KB  |        5.1%       |    ~25 ms   |    ~22 ms   |
| AVIF    |   95 KB   |        3.9%       |    ~150 ms  |    ~110 ms  |
| JPEG    |   160 KB  |        6.5%       |    ~15 ms   |    ~12 ms   |

**Lossless Mode:**

| Format  | File Size | Compression Ratio | Encode Time | Decode Time |
|---------|:---------:|:-----------------:|:-----------:|:-----------:|
| **WK v3.0** | **780 KB**|      **31.7%**    |  **~90 ms** |  **~75 ms** |
| WebP-LL |   815 KB  |        33.1%      |    ~110 ms  |    ~85 ms   |
| PNG     |  1.1 MB   |        44.7%      |    ~60 ms   |    ~50 ms   |

*These are illustrative benchmarks. Actual performance may vary based on image content and hardware.*

## Project Structure
```
WK/
├── src/
│   ├── lib.rs              # Crate entrypoint and public API
│   ├── main.rs             # CLI (wkconverter)
│   ├── encoder.rs          # WK format encoder
│   ├── decoder.rs          # WK format decoder
│   ├── converter.rs        # High-level image conversion logic
│   ├── error.rs            # Crate-specific error types
│   ├── format/             # File format chunks and headers
│   ├── compression/        # Core compression algorithms (DCT, CABAC, etc.)
│   ├── metadata/           # EXIF, ICC, and XMP handling
│   ├── animation/          # Animation frame logic
│   └── bin/
│       ├── viewer.rs       # GUI viewer source code (egui)
│       └── debug.rs        # Debugging utility
├── viewer/
│   ├── index.html          # Experimental web viewer
│   ├── main.js
│   └── styles.css
└── tests/
    └── ...                 # Integration and unit tests
```

## Web Viewer (Experimental)

The `viewer/` directory contains an experimental web-based viewer. Open `viewer/index.html` in a modern web browser to use it. Please note that WebAssembly support is still under development and may not be fully functional.

## Building

```bash
# Run a standard debug build
cargo build

# Run a release build for performance
cargo build --release

# Include the GUI viewer in the build
cargo build --release --features viewer

# Run all tests
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Inggrit Setya Budi** ([@cowoksoftspoken](https://github.com/cowoksoftspoken))
