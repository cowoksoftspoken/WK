use image::{DynamicImage, RgbImage};
use wk_format::{WkConverter, WkDecoder, WkEncoder, WkMetadata, WkResult};

fn main() -> WkResult<()> {
    println!("WK Image Format Example\n");

    example_create_and_encode()?;
    example_convert_png_to_wk()?;
    example_read_wk_info()?;
    example_custom_metadata()?;

    println!("\n✓ All examples completed successfully!");
    Ok(())
}

fn example_create_and_encode() -> WkResult<()> {
    println!("1. Creating and encoding image to WK format...");

    let img = DynamicImage::ImageRgb8(RgbImage::from_fn(256, 256, |x, y| {
        image::Rgb([x as u8, y as u8, 128])
    }));

    let mut metadata = WkMetadata::new();
    metadata.author = Some("WK Example".to_string());
    metadata.description = Some("Generated gradient image".to_string());

    let encoder = WkEncoder::new().with_metadata(metadata);
    let mut file = std::fs::File::create("example_gradient.wk")?;
    encoder.encode(&img, &mut file)?;

    println!("   ✓ Saved to example_gradient.wk");
    Ok(())
}

fn example_convert_png_to_wk() -> WkResult<()> {
    println!("\n2. Converting PNG to WK format...");

    let img = DynamicImage::ImageRgb8(RgbImage::from_fn(100, 100, |x, y| {
        image::Rgb([255 - (x * 2) as u8, (y * 2) as u8, 128])
    }));
    img.save("sample.png")?;

    let converter = WkConverter::new();
    converter.to_wk("sample.png", "sample.wk")?;
    let png_size = std::fs::metadata("sample.png")?.len();
    let wk_size = std::fs::metadata("sample.wk")?.len();

    println!("   PNG size: {} bytes", png_size);
    println!("   WK size:  {} bytes", wk_size);
    println!(
        "   Ratio:    {:.2}%",
        (wk_size as f64 / png_size as f64) * 100.0
    );

    Ok(())
}

fn example_read_wk_info() -> WkResult<()> {
    println!("\n3. Reading WK file information...");

    let converter = WkConverter::new();
    let (img, metadata) = converter.wk_to_image("example_gradient.wk")?;

    println!("   Image: {}x{}", img.width(), img.height());
    println!("   Color: {:?}", img.color());

    if let Some(author) = &metadata.author {
        println!("   Author: {}", author);
    }
    if let Some(desc) = &metadata.description {
        println!("   Description: {}", desc);
    }

    Ok(())
}

fn example_custom_metadata() -> WkResult<()> {
    println!("\n4. Using custom metadata fields...");

    let img = DynamicImage::ImageRgb8(RgbImage::from_fn(50, 50, |_, _| image::Rgb([255, 0, 0])));

    let mut metadata = WkMetadata::new();
    metadata.author = Some("Custom Example".to_string());
    metadata.add_custom_field("version".to_string(), "1.0".to_string());
    metadata.add_custom_field("project".to_string(), "WK Format Demo".to_string());
    metadata.add_custom_field("color".to_string(), "red".to_string());

    let converter = WkConverter::new();
    converter.image_to_wk(&img, "custom_metadata.wk", Some(metadata))?;

    let (_, metadata) = converter.wk_to_image("custom_metadata.wk")?;
    println!("   Custom fields:");
    for (key, value) in &metadata.custom_fields {
        println!("     {}: {}", key, value);
    }

    Ok(())
}
