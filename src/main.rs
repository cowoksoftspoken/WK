use std::path::Path;
use wk_format::{WkConverter, WkResult};

fn main() -> WkResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let input = &args[2];

    match command.as_str() {
        "to-wk" => {
            if args.len() < 4 {
                eprintln!("Error: Output file required");
                print_usage();
                std::process::exit(1);
            }
            let output = &args[3];
            convert_to_wk(input, output)?;
        }
        "from-wk" => {
            if args.len() < 4 {
                eprintln!("Error: Output file required");
                print_usage();
                std::process::exit(1);
            }
            let output = &args[3];
            convert_from_wk(input, output)?;
        }
        "info" => {
            show_info(input)?;
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn convert_to_wk(input: &str, output: &str) -> WkResult<()> {
    println!("Converting {} to WK format...", input);

    let input_path = Path::new(input);
    let converter = WkConverter::new();

    converter.to_wk(input_path, output)?;

    println!("✓ Successfully converted to {}", output);

    // Tampilkan info kompresi
    let input_size = std::fs::metadata(input_path)?.len();
    let output_size = std::fs::metadata(output)?.len();
    let ratio = (output_size as f64 / input_size as f64) * 100.0;

    println!("  Input size:  {} bytes", input_size);
    println!("  Output size: {} bytes", output_size);
    println!("  Ratio:       {:.2}%", ratio);

    Ok(())
}

fn convert_from_wk(input: &str, output: &str) -> WkResult<()> {
    println!("Converting {} from WK format...", input);

    let converter = WkConverter::new();
    converter.from_wk(input, output)?;

    println!("✓ Successfully converted to {}", output);
    Ok(())
}

fn show_info(input: &str) -> WkResult<()> {
    println!("Reading WK file: {}", input);

    let converter = WkConverter::new();
    let (img, metadata) = converter.wk_to_image(input)?;

    println!("\n=== WK File Information ===");
    println!("Dimensions: {}x{}", img.width(), img.height());
    println!("Color Type: {:?}", img.color());

    println!("\n=== Metadata ===");
    if let Some(created) = &metadata.created_at {
        println!("Created:     {}", created);
    }
    if let Some(software) = &metadata.software {
        println!("Software:    {}", software);
    }
    if let Some(author) = &metadata.author {
        println!("Author:      {}", author);
    }
    if let Some(desc) = &metadata.description {
        println!("Description: {}", desc);
    }

    if !metadata.custom_fields.is_empty() {
        println!("\n=== Custom Fields ===");
        for (key, value) in &metadata.custom_fields {
            println!("{}: {}", key, value);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("WK Image Format Converter");
    println!("\nUsage:");
    println!("  wk-converter to-wk <input> <output.wk>   - Convert to WK format");
    println!("  wk-converter from-wk <input.wk> <output> - Convert from WK format");
    println!("  wk-converter info <input.wk>             - Show WK file information");
    println!("\nSupported input formats: PNG, JPEG, WebP, HEIC, and more");
    println!("Supported output formats: PNG, JPEG, BMP, TIFF, etc.");
}
