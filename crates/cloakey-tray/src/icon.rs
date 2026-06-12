use std::path::Path;
use image::GenericImageView;
use tray_icon::Icon;

/// Load a system tray icon from a PNG file path.
pub fn load_icon_from_path<P: AsRef<Path>>(path: P) -> Result<Icon, String> {
    let img = image::open(path).map_err(|e| format!("Image open error: {}", e))?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    Icon::from_rgba(rgba, width, height).map_err(|e| format!("Icon creation error: {}", e))
}

/// Create a solid-color circular icon dynamically in memory.
///
/// Useful as a fallback if the PNG assets are not available.
/// - Unlocked -> Green `[0, 180, 0, 255]`
/// - Locked   -> Red `[220, 0, 0, 255]`
/// - Ghost    -> Purple `[120, 0, 180, 255]`
pub fn create_default_icon(color: [u8; 4]) -> Icon {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            
            // Draw a circular dot
            let dx = (x as f32) - 15.5;
            let dy = (y as f32) - 15.5;
            let dist_sq = dx * dx + dy * dy;
            
            // Antialiased circle edge
            if dist_sq <= 12.5 * 12.5 {
                rgba[idx] = color[0];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[2];
                rgba[idx + 3] = color[3];
            } else if dist_sq <= 14.5 * 14.5 {
                // Fade out at edge
                let alpha = ((14.5 - dist_sq.sqrt()) / 2.0 * 255.0) as u8;
                rgba[idx] = color[0];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[2];
                rgba[idx + 3] = alpha;
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }
    
    Icon::from_rgba(rgba, width, height).expect("Failed to create fallback icon")
}
