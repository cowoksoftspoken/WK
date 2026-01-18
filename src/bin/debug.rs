use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wk-debug <file.wk>");
        std::process::exit(1);
    }

    let path = &args[1];
    let mut file = std::fs::File::open(path)?;

    println!("=== WK File Debug Info ===\n");

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    println!(
        "Magic: {:02X?} ({})",
        magic,
        String::from_utf8_lossy(&magic)
    );

    let width = file.read_u32::<LittleEndian>()?;
    let height = file.read_u32::<LittleEndian>()?;
    let color_type = file.read_u8()?;
    let compression = file.read_u8()?;
    let metadata_size = file.read_u32::<LittleEndian>()?;

    println!("\n=== Header ===");
    println!("Width: {}", width);
    println!("Height: {}", height);
    println!(
        "Color Type: {} ({})",
        color_type,
        match color_type {
            0 => "RGB",
            1 => "RGBA",
            2 => "Grayscale",
            3 => "GrayscaleAlpha",
            _ => "Unknown",
        }
    );
    println!(
        "Compression: {} ({})",
        compression,
        if compression == 1 { "RLE" } else { "None" }
    );
    println!("Metadata Size: {} bytes", metadata_size);

    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    file.read_exact(&mut metadata_bytes)?;
    println!(
        "\nMetadata (first 100 bytes): {:02X?}...",
        &metadata_bytes[..metadata_bytes.len().min(100)]
    );

    let compressed_size = file.read_u32::<LittleEndian>()?;
    println!("\n=== Compression Info ===");
    println!("Compressed Size: {} bytes", compressed_size);

    let channels = match color_type {
        0 => 3,
        1 => 4,
        2 => 1,
        3 => 2,
        _ => 0,
    };
    let expected_size = width * height * channels;
    println!("Expected Uncompressed: {} bytes", expected_size);
    println!(
        "Compression Ratio: {:.2}%",
        (compressed_size as f64 / expected_size as f64) * 100.0
    );

    let mut compressed_data = vec![0u8; compressed_size as usize];
    file.read_exact(&mut compressed_data)?;

    println!("\n=== Compressed Data Analysis ===");
    println!(
        "First 50 bytes: {:02X?}",
        &compressed_data[..compressed_data.len().min(50)]
    );

    let mut i = 0;
    let mut rle_runs = 0;
    let mut literal_runs = 0;
    let mut total_rle_bytes = 0;
    let mut total_literal_bytes = 0;

    while i < compressed_data.len() {
        let header = compressed_data[i];
        i += 1;

        if header & 0x80 != 0 {
            let count = (header & 0x7F) as usize;
            if i < compressed_data.len() {
                i += 1;
                rle_runs += 1;
                total_rle_bytes += count;
            }
        } else {
            let count = header as usize;
            if count > 0 && i + count <= compressed_data.len() {
                i += count;
                literal_runs += 1;
                total_literal_bytes += count;
            } else if count > 0 {
                println!(
                    "\n⚠️  ERROR: Literal at pos {} wants {} bytes, only {} available",
                    i - 1,
                    count,
                    compressed_data.len() - i
                );
                break;
            }
        }
    }

    println!("\nRLE Runs: {}", rle_runs);
    println!("Literal Runs: {}", literal_runs);
    println!("Total RLE bytes (uncompressed): {}", total_rle_bytes);
    println!("Total Literal bytes: {}", total_literal_bytes);
    println!(
        "Total decoded should be: {}",
        total_rle_bytes + total_literal_bytes
    );
    println!("Expected: {}", expected_size);

    if total_rle_bytes + total_literal_bytes != expected_size as usize {
        println!("\n⚠️  WARNING: Decoded size mismatch!");
        println!(
            "   Difference: {} bytes",
            (expected_size as i64) - ((total_rle_bytes + total_literal_bytes) as i64)
        );
    }

    Ok(())
}
