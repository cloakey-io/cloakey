use image::{GenericImageView, ImageEncoder};
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logo_path = Path::new("CloaKey logo no bg.png");
    if !logo_path.exists() {
        return Err(format!(
            "Could not find '{}' in the workspace root.",
            logo_path.display()
        )
        .into());
    }

    println!("Loading custom logo: {}", logo_path.display());
    let img = image::open(logo_path)?;
    let (w, h) = img.dimensions();
    println!("Original dimensions: {}x{}", w, h);

    // Create assets directory if missing
    std::fs::create_dir_all("assets")?;

    // 1. Generate 32x32 green tray icon (normal / unlocked)
    // The original logo has a green shield key design, so we just resize it
    println!("Generating 32x32 system tray active icon (icon.png)...");
    let tray_unlocked = img.resize(32, 32, image::imageops::FilterType::Lanczos3);
    tray_unlocked.save("assets/icon.png")?;

    // 2. Generate 32x32 red tray icon (locked / keyboard/mouse locked)
    // We convert it to RGBA and shift the hue/tint to red
    println!("Generating 32x32 system tray locked icon (icon_locked.png)...");
    let mut locked_rgba = img.to_rgba8();
    for pixel in locked_rgba.pixels_mut() {
        let alpha = pixel[3];
        if alpha > 0 {
            // Convert to grayscale intensity to preserve details
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            let intensity = 0.299 * r + 0.587 * g + 0.114 * b;

            // Map intensity to a strong red color
            pixel[0] = (intensity * 1.1).min(255.0) as u8; // High Red
            pixel[1] = (intensity * 0.1) as u8; // Minimal Green
            pixel[2] = (intensity * 0.1) as u8; // Minimal Blue
        }
    }
    let locked_img = image::DynamicImage::ImageRgba8(locked_rgba);
    let tray_locked = locked_img.resize(32, 32, image::imageops::FilterType::Lanczos3);
    tray_locked.save("assets/icon_locked.png")?;

    // 3. Generate multi-size executable icon (icon.ico)
    // We'll write 16, 32, 48, 64, 128, 256 sizes into the ICO format
    println!("Generating multi-resolution application icon (icon.ico)...");
    let file = File::create("assets/icon.ico")?;
    let encoder = image::codecs::ico::IcoEncoder::new(file);

    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    encoder.write_image(&rgba, 256, 256, image::ColorType::Rgba8.into())?;

    // 4. Generate Rust array for TUI display (34x14 characters target)
    println!("Generating Rust array logo for TUI (crates/cloakey-cli/src/logo_data.rs)...");
    let tui_w = 34;
    let tui_h = 14;
    let tui_img = img.resize(tui_w, tui_h, image::imageops::FilterType::Lanczos3);
    let (actual_w, actual_h) = tui_img.dimensions();
    let tui_rgba = tui_img.to_rgba8();

    let mut rust_code = String::new();
    rust_code.push_str("/// Auto-generated logo pixel data from CloaKey logo no bg.png\n");
    rust_code.push_str(&format!("pub const LOGO_WIDTH: usize = {};\n", actual_w));
    rust_code.push_str(&format!("pub const LOGO_HEIGHT: usize = {};\n\n", actual_h));
    rust_code
        .push_str("pub const LOGO_PIXELS: [[Option<(u8, u8, u8)>; LOGO_WIDTH]; LOGO_HEIGHT] = [\n");

    for y in 0..actual_h {
        rust_code.push_str("    [\n        ");
        for x in 0..actual_w {
            let pixel = tui_rgba.get_pixel(x, y);
            let alpha = pixel[3];
            if alpha < 35 {
                rust_code.push_str("None, ");
            } else {
                rust_code.push_str(&format!(
                    "Some(({}, {}, {})), ",
                    pixel[0], pixel[1], pixel[2]
                ));
            }
        }
        rust_code.push_str("\n    ],\n");
    }
    rust_code.push_str("];\n");

    std::fs::write("crates/cloakey-cli/src/logo_data.rs", rust_code)?;

    println!("✓ All icon and TUI logo assets successfully generated!");
    Ok(())
}
