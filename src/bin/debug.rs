use std::io::BufReader;
use wk_format::{WkDecoder, WkResult};

fn main() -> WkResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wkdebug <file.wk>");
        std::process::exit(1);
    }

    let path = &args[1];
    let file = std::fs::File::open(path)?;

    println!("<--- WK v2.0 File Debug --->\n");
    println!("File: {}", path);

    let file_size = std::fs::metadata(path)?.len();
    println!("Size: {} bytes", file_size);

    let decoder = WkDecoder::new();
    let decoded = decoder.decode(BufReader::new(file))?;

    println!("\n<--- Header --->");
    println!("Width: {}", decoded.header.width);
    println!("Height: {}", decoded.header.height);
    println!("Color Type: {:?}", decoded.header.color_type);
    println!("Compression: {:?}", decoded.header.compression_mode);
    println!("Quality: {}", decoded.header.quality);
    println!("Has Alpha: {}", decoded.header.has_alpha);
    println!("Has Animation: {}", decoded.header.has_animation);
    println!("Bit Depth: {}", decoded.header.bit_depth);

    let raw_size = decoded.header.width as usize
        * decoded.header.height as usize
        * decoded.header.color_type.channels() as usize;
    let ratio = file_size as f64 / raw_size as f64 * 100.0;
    println!("\n<--- Compression Stats --->");
    println!("Raw size: {} bytes", raw_size);
    println!("Compressed: {} bytes", file_size);
    println!("Ratio: {:.1}%", ratio);

    if let Some(ref icc) = decoded.metadata.icc_profile {
        println!("\n<--- ICC Profile --->");
        println!("Color Space: {:?}", icc.color_space);
        println!("Profile: {}", icc.profile_name);
        println!("Rendering Intent: {:?}", icc.rendering_intent);
    }

    if let Some(ref exif) = decoded.metadata.exif {
        println!("\n<--- EXIF --->");
        if let Some(make) = exif.camera_make() {
            println!("Make: {}", make);
        }
        if let Some(model) = exif.camera_model() {
            println!("Model: {}", model);
        }
        if let Some(date) = exif.date_time() {
            println!("Date: {}", date);
        }
        if let Some(iso) = exif.iso() {
            println!("ISO: {}", iso);
        }
        if let Some(ap) = exif.aperture() {
            println!("Aperture: f/{:.1}", ap);
        }
        if let Some(fl) = exif.focal_length() {
            println!("Focal Length: {:.0}mm", fl);
        }
    }

    if let Some(ref xmp) = decoded.metadata.xmp {
        println!("\n<--- XMP --->");
        if let Some(ref title) = xmp.title {
            println!("Title: {}", title);
        }
        if let Some(ref desc) = xmp.description {
            println!("Description: {}", desc);
        }
        if !xmp.creator.is_empty() {
            println!("Creators: {}", xmp.creator.join(", "));
        }
        if !xmp.subject.is_empty() {
            println!("Subjects: {}", xmp.subject.join(", "));
        }
        if let Some(rating) = xmp.rating {
            println!("Rating: {}/5", rating);
        }
    }

    let custom = &decoded.metadata.custom;
    if custom.author.is_some() || !custom.fields.is_empty() {
        println!("\n<--- Custom --->");
        if let Some(ref author) = custom.author {
            println!("Author: {}", author);
        }
        if let Some(ref software) = custom.software {
            println!("Software: {}", software);
        }
        if let Some(ref desc) = custom.description {
            println!("Description: {}", desc);
        }
        for (key, value) in custom.iter() {
            println!("{}: {:?}", key, value);
        }
    }

    println!("\n✓ File parsed successfully");

    Ok(())
}
