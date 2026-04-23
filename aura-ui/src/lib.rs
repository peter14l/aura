// aura-ui/src/lib.rs

slint::include_modules!();

pub use MainUI;
pub use TabNode;

pub fn create_ui() -> MainUI {
    MainUI::new().expect("Failed to initialize MainUI")
}

pub fn extract_dominant_color(img_bytes: &[u8]) -> Option<slint::Color> {
    use image::GenericImageView;
    let img = image::load_from_memory(img_bytes).ok()?;
    let (width, height) = img.dimensions();

    let mut r = 0u64;
    let mut g = 0u64;
    let mut b = 0u64;
    let mut count = 0u64;

    // Sample pixels (every 4th to save time)
    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            let pixel = img.get_pixel(x, y);
            if pixel[3] > 128 {
                // Only opaque pixels
                r += pixel[0] as u64;
                g += pixel[1] as u64;
                b += pixel[2] as u64;
                count += 1;
            }
        }
    }

    if count == 0 {
        return None;
    }

    Some(slint::Color::from_rgb_u8(
        (r / count) as u8,
        (g / count) as u8,
        (b / count) as u8,
    ))
}
