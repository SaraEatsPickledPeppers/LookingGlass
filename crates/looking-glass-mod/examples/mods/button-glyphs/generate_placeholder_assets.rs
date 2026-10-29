//! Generate placeholder PNG assets for button glyphs mod
//! Run with: cargo run --example generate_placeholder_assets

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new("crates/looking-glass-mod/examples/mods/button-glyphs/assets/textures/ui");

    let platforms = ["ps", "xbox", "switch"];
    let buttons = [
        ("south", "A / Cross / B"),
        ("east", "B / Circle / A"),
        ("west", "X / Square / Y"),
        ("north", "Y / Triangle / X"),
        ("l1", "L1 / LB / L"),
        ("r1", "R1 / RB / R"),
        ("l2", "L2 / LT / ZL"),
        ("r2", "R2 / RT / ZR"),
        ("select", "Select / Back / -"),
        ("start", "Start / Start / +"),
        ("l3", "L3 / LS / LS"),
        ("r3", "R3 / RS / RS"),
        ("dpad", "D-Pad / D-Pad / D-Pad"),
    ];

    // Simple 64x64 colored PNG with button label
    for platform in &platforms {
        for (button, label) in &buttons {
            let filename = format!("button_{}_{}.png", button, platform);
            let filepath = base_dir.join(&filename);

            // Create a simple 64x64 PNG with a solid color
            let img = create_placeholder_png(64, 64, platform, label)?;
            fs::write(&filepath, &img)?;
            println!("Created: {}", filepath.display());
        }
    }

    println!("All placeholder assets generated!");
    Ok(())
}

fn create_placeholder_png(width: u32, height: u32, platform: &str, label: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use image::{Rgba, RgbaImage};

    let mut img = RgbaImage::new(width, height);

    // Platform-specific colors
    let base_color = match platform {
        "ps" => Rgba([0, 120, 255, 255]),      // PlayStation blue
        "xbox" => Rgba([16, 124, 16, 255]),    // Xbox green
        "switch" => Rgba([220, 20, 60, 255]),  // Switch red
        _ => Rgba([128, 128, 128, 255]),
    };

    // Fill background
    for y in 0..height {
        for x in 0..width {
            let mut color = base_color;
            // Add a border
            if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
                color = Rgba([255, 255, 255, 255]);
            }
            // Add a subtle gradient
            let gradient = (x as f32 / width as f32 * 0.3) + (y as f32 / height as f32 * 0.2);
            color.0[0] = (color.0[0] as f32 * (1.0 - gradient)) as u8;
            color.0[1] = (color.0[1] as f32 * (1.0 - gradient)) as u8;
            color.0[2] = (color.0[2] as f32 * (1.0 - gradient)) as u8;
            img.put_pixel(x, y, color);
        }
    }

    // Encode as PNG
    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)?;

    Ok(png_data)
}