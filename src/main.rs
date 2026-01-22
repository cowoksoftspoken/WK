use colored::Colorize;
use wk_format::metadata::exif::ExifBuilder;
use wk_format::metadata::icc::IccProfile;
use wk_format::metadata::xmp::XmpBuilder;
use wk_format::{WkDecoder, WkEncoder, WkMetadata, WkResult};

fn main() -> WkResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let input = &args[2];

    match command.as_str() {
        "to-wk" | "encode" => {
            if args.len() < 4 {
                eprintln!("{} Output file required", "Error:".red().bold());
                std::process::exit(1);
            }
            let output = &args[3];
            let quality = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(85);
            encode_image(input, output, quality)?;
        }
        "from-wk" | "decode" => {
            if args.len() < 4 {
                eprintln!("{} Output file required", "Error:".red().bold());
                std::process::exit(1);
            }
            let output = &args[3];
            decode_image(input, output)?;
        }
        "info" => {
            show_info(input)?;
        }
        "lossless" => {
            if args.len() < 4 {
                eprintln!("{} Output file required", "Error:".red().bold());
                std::process::exit(1);
            }
            let output = &args[3];
            encode_lossless(input, output)?;
        }
        "benchmark" => {
            if args.len() < 4 {
                eprintln!("{} Output directory required", "Error:".red().bold());
                std::process::exit(1);
            }
            let output_dir = &args[3];
            benchmark(input, output_dir)?;
        }
        _ => {
            eprintln!("{} Unknown command: {}", "Error:".red().bold(), command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn encode_image(input: &str, output: &str, quality: u8) -> WkResult<()> {
    println!(
        "{} {} → {} (quality: {})",
        "Encoding".cyan().bold(),
        input.yellow(),
        output.green(),
        quality.to_string().magenta()
    );

    let img = image::open(input)?;

    let exif = ExifBuilder::new().software("WK Image Format v3.0").build();

    let xmp = XmpBuilder::new().creator_tool("WK Converter v3.0").build();

    let metadata = WkMetadata::new()
        .with_exif(exif)
        .with_xmp(xmp)
        .with_icc(IccProfile::srgb());

    let encoder = WkEncoder::lossy(quality).with_metadata(metadata);

    let mut file = std::fs::File::create(output)?;
    encoder.encode(&img, &mut file)?;

    let input_size = std::fs::metadata(input)?.len();
    let output_size = std::fs::metadata(output)?.len();
    let ratio = output_size as f64 / input_size as f64 * 100.0;

    println!("{}", "✓ Encoded successfully!".green().bold());
    println!(
        "  {} {} bytes",
        "Input: ".dimmed(),
        input_size.to_string().white()
    );
    println!(
        "  {} {} bytes",
        "Output:".dimmed(),
        output_size.to_string().white()
    );
    println!(
        "  {} {}%",
        "Ratio: ".dimmed(),
        format!("{:.1}", ratio).cyan()
    );

    Ok(())
}

fn encode_lossless(input: &str, output: &str) -> WkResult<()> {
    println!(
        "{} {} → {} {}",
        "Encoding".cyan().bold(),
        input.yellow(),
        output.green(),
        "(lossless)".magenta()
    );

    let img = image::open(input)?;
    let encoder = WkEncoder::lossless();

    let mut file = std::fs::File::create(output)?;
    encoder.encode(&img, &mut file)?;

    let input_size = std::fs::metadata(input)?.len();
    let output_size = std::fs::metadata(output)?.len();

    println!("{}", "✓ Encoded successfully!".green().bold());
    println!(
        "  {} {} bytes",
        "Input: ".dimmed(),
        input_size.to_string().white()
    );
    println!(
        "  {} {} bytes",
        "Output:".dimmed(),
        output_size.to_string().white()
    );

    Ok(())
}

fn decode_image(input: &str, output: &str) -> WkResult<()> {
    println!(
        "{} {} → {}",
        "Decoding".cyan().bold(),
        input.yellow(),
        output.green()
    );

    let file = std::fs::File::open(input)?;
    let decoder = WkDecoder::new();
    let decoded = decoder.decode(std::io::BufReader::new(file))?;

    decoded.image.save(output)?;

    println!("{}", "✓ Decoded successfully!".green().bold());
    println!(
        "  {} {}x{}",
        "Dimensions:".dimmed(),
        decoded.header.width.to_string().white(),
        decoded.header.height.to_string().white()
    );
    println!(
        "  {} {:?}",
        "Color Type:".dimmed(),
        format!("{:?}", decoded.header.color_type).cyan()
    );
    println!(
        "  {} {:?}",
        "Mode:      ".dimmed(),
        format!("{:?}", decoded.header.compression_mode).magenta()
    );

    Ok(())
}

fn show_info(input: &str) -> WkResult<()> {
    let file = std::fs::File::open(input)?;
    let decoder = WkDecoder::new();
    let decoded = decoder.decode(std::io::BufReader::new(file))?;

    println!();
    println!("{}", "═══ WK Image Information ═══".cyan().bold());
    println!("{} {}", "Version:".dimmed(), "3.0".green());
    println!(
        "{} {}x{}",
        "Dimensions:".dimmed(),
        decoded.header.width.to_string().white(),
        decoded.header.height.to_string().white()
    );
    println!(
        "{} {:?}",
        "Color Type:".dimmed(),
        format!("{:?}", decoded.header.color_type).cyan()
    );
    println!(
        "{} {:?}",
        "Compression:".dimmed(),
        format!("{:?}", decoded.header.compression_mode).magenta()
    );
    println!(
        "{} {}",
        "Quality:".dimmed(),
        decoded.header.quality.to_string().yellow()
    );
    println!(
        "{} {}",
        "Has Alpha:".dimmed(),
        if decoded.header.has_alpha {
            "Yes".green()
        } else {
            "No".red()
        }
    );
    println!(
        "{} {}",
        "Bit Depth:".dimmed(),
        decoded.header.bit_depth.to_string().white()
    );

    if let Some(ref icc) = decoded.metadata.icc_profile {
        println!();
        println!("{}", "═══ Color Profile ═══".cyan().bold());
        println!(
            "{} {:?}",
            "Color Space:".dimmed(),
            format!("{:?}", icc.color_space).green()
        );
        println!("{} {}", "Profile:".dimmed(), icc.profile_name.white());
    }

    if let Some(ref exif) = decoded.metadata.exif {
        println!();
        println!("{}", "═══ EXIF Data ═══".cyan().bold());
        if let Some(make) = exif.camera_make() {
            println!("{} {}", "Camera Make:".dimmed(), make.white());
        }
        if let Some(model) = exif.camera_model() {
            println!("{} {}", "Camera Model:".dimmed(), model.white());
        }
        if let Some(iso) = exif.iso() {
            println!("{} {}", "ISO:".dimmed(), iso.to_string().yellow());
        }
        if let Some(aperture) = exif.aperture() {
            println!(
                "{} f/{:.1}",
                "Aperture:".dimmed(),
                format!("{:.1}", aperture).cyan()
            );
        }
        if let Some(focal) = exif.focal_length() {
            println!(
                "{} {:.0}mm",
                "Focal Length:".dimmed(),
                format!("{:.0}", focal).magenta()
            );
        }
    }

    if let Some(ref xmp) = decoded.metadata.xmp {
        println!();
        println!("{}", "═══ XMP Data ═══".cyan().bold());
        if let Some(ref title) = xmp.title {
            println!("{} {}", "Title:".dimmed(), title.white());
        }
        if let Some(ref desc) = xmp.description {
            println!("{} {}", "Description:".dimmed(), desc.white());
        }
        if !xmp.creator.is_empty() {
            println!(
                "{} {}",
                "Creators:".dimmed(),
                xmp.creator.join(", ").white()
            );
        }
        if let Some(rating) = xmp.rating {
            let stars = "★".repeat(rating as usize);
            let empty = "☆".repeat(5 - rating as usize);
            println!(
                "{} {}{}",
                "Rating:".dimmed(),
                stars.yellow(),
                empty.dimmed()
            );
        }
    }

    let custom = &decoded.metadata.custom;
    if custom.author.is_some() || !custom.fields.is_empty() {
        println!();
        println!("{}", "═══ Custom Metadata ═══".cyan().bold());
        if let Some(ref author) = custom.author {
            println!("{} {}", "Author:".dimmed(), author.white());
        }
        if let Some(ref software) = custom.software {
            println!("{} {}", "Software:".dimmed(), software.white());
        }
        for (key, value) in custom.iter() {
            println!("{}: {:?}", key.dimmed(), format!("{:?}", value).white());
        }
    }

    println!();

    Ok(())
}

fn benchmark(input: &str, output_dir: &str) -> WkResult<()> {
    std::fs::create_dir_all(output_dir)?;

    let img = image::open(input)?;
    let raw_size = img.width() as usize * img.height() as usize * 3;

    println!();
    println!("{}", "═══ WK Format Benchmark ═══".cyan().bold());
    println!(
        "{} {} ({}x{}, {} bytes raw)",
        "Input:".dimmed(),
        input.yellow(),
        img.width(),
        img.height(),
        raw_size
    );
    println!();

    let qualities = [100, 95, 90, 85, 75, 50, 25];

    println!(
        "{:>6} {:>10} {:>10} {:>8} {:>10} {:>10}",
        "Q".white().bold(),
        "Mode".white().bold(),
        "Size".white().bold(),
        "Ratio".white().bold(),
        "Encode".white().bold(),
        "Decode".white().bold()
    );
    println!("{}", "─".repeat(60).dimmed());

    for q in qualities {
        let output_path = format!("{}/q{}.wk", output_dir, q);
        let encoder = if q == 100 {
            WkEncoder::lossless()
        } else {
            WkEncoder::lossy(q)
        };

        let start = std::time::Instant::now();
        let encoded = encoder.encode_to_vec(&img)?;
        let encode_time = start.elapsed();

        std::fs::write(&output_path, &encoded)?;

        let decoder = WkDecoder::new();
        let start = std::time::Instant::now();
        let _ = decoder.decode(encoded.as_slice())?;
        let decode_time = start.elapsed();

        let ratio = encoded.len() as f64 / raw_size as f64 * 100.0;
        let mode = if q == 100 { "lossless" } else { "lossy" };
        let mode_colored = if q == 100 {
            mode.green()
        } else {
            mode.yellow()
        };

        println!(
            "{:>6} {:>10} {:>10} {:>7}% {:>9}ms {:>9}ms",
            q.to_string().cyan(),
            mode_colored,
            encoded.len().to_string().white(),
            format!("{:.1}", ratio).magenta(),
            format!("{:.2}", encode_time.as_secs_f64() * 1000.0).dimmed(),
            format!("{:.2}", decode_time.as_secs_f64() * 1000.0).dimmed()
        );
    }

    println!();
    println!("{}", "✓ Benchmark complete!".green().bold());

    Ok(())
}

fn print_usage() {
    println!();
    println!(
        "{} {}",
        "WK Image Format Converter".cyan().bold(),
        "v3.0".green()
    );
    println!();
    println!("{}", "USAGE:".yellow().bold());
    println!(
        "  {} {} <input> <output.wk> [quality]",
        "wkconverter".white(),
        "encode".green()
    );
    println!(
        "  {} {} <input> <output.wk>",
        "wkconverter".white(),
        "lossless".green()
    );
    println!(
        "  {} {} <input.wk> <output>",
        "wkconverter".white(),
        "decode".green()
    );
    println!("  {} {} <input.wk>", "wkconverter".white(), "info".green());
    println!(
        "  {} {} <input> <output_dir>",
        "wkconverter".white(),
        "benchmark".green()
    );
    println!();
    println!("{}", "OPTIONS:".yellow().bold());
    println!(
        "  {} 1-100 (default: 85, 100 = lossless)",
        "Quality:".dimmed()
    );
    println!();
    println!("{}", "EXAMPLES:".yellow().bold());
    println!("  {} photo.jpg photo.wk 85", "wkconverter encode".cyan());
    println!("  {} art.png art.wk", "wkconverter lossless".cyan());
    println!("  {} image.wk image.png", "wkconverter decode".cyan());
    println!("  {} image.wk", "wkconverter info".cyan());
    println!();
}
