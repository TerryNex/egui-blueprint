//! Image recognition helpers for template matching and image comparison.

/// Find a template image within a screen image using simple template matching.
/// Uses grid sampling for performance and tolerance for fuzzy matching.
///
/// # Arguments
/// * `screen` - The screen capture image to search in
/// * `template` - The template image to search for
/// * `tolerance` - Color tolerance for matching (0-255)
/// * `region_x`, `region_y`, `region_w`, `region_h` - Search region bounds
///
/// # Returns
/// Tuple of (x, y, found) where x,y is the top-left corner of the match
pub fn find_template_in_image(
    screen: &image::RgbaImage,
    template: &image::RgbaImage,
    tolerance: i32,
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
) -> (i64, i64, bool) {
    let tpl_w = template.width();
    let tpl_h = template.height();

    if tpl_w == 0 || tpl_h == 0 {
        return (0, 0, false);
    }

    let end_x = (region_x + region_w)
        .min(screen.width())
        .saturating_sub(tpl_w);
    let end_y = (region_y + region_h)
        .min(screen.height())
        .saturating_sub(tpl_h);

    // Grid sample step for performance (check every Nth pixel of template)
    let sample_step = 4u32.max((tpl_w * tpl_h / 100).max(1));

    for sy in region_y..=end_y {
        for sx in region_x..=end_x {
            let mut matches = true;
            let mut checked = 0u32;

            // Sample points in template
            'check: for ty in (0..tpl_h).step_by(sample_step as usize) {
                for tx in (0..tpl_w).step_by(sample_step as usize) {
                    let tpl_pixel = template.get_pixel(tx, ty);
                    // Skip transparent pixels in template
                    if tpl_pixel[3] < 128 {
                        continue;
                    }

                    let scr_pixel = screen.get_pixel(sx + tx, sy + ty);
                    let dr = (scr_pixel[0] as i32 - tpl_pixel[0] as i32).abs();
                    let dg = (scr_pixel[1] as i32 - tpl_pixel[1] as i32).abs();
                    let db = (scr_pixel[2] as i32 - tpl_pixel[2] as i32).abs();

                    if dr > tolerance || dg > tolerance || db > tolerance {
                        matches = false;
                        break 'check;
                    }
                    checked += 1;
                }
            }

            // Require at least some pixels checked
            if matches && checked > 5 {
                return (sx as i64, sy as i64, true);
            }
        }
    }

    (0, 0, false)
}

/// Compare two images and return similarity score (0.0 - 1.0), considering tolerance.
///
/// # Arguments
/// * `img1`, `img2` - Images to compare (must be same size)
/// * `tolerance` - Color tolerance per channel (0-255)
///
/// # Returns
/// Similarity score from 0.0 (no match) to 1.0 (identical)
pub fn compare_images(img1: &image::RgbaImage, img2: &image::RgbaImage, tolerance: i32) -> f64 {
    // If sizes don't match, return 0
    if img1.width() != img2.width() || img1.height() != img2.height() {
        return 0.0;
    }

    let total_pixels = (img1.width() * img1.height()) as f64;
    if total_pixels == 0.0 {
        return 0.0;
    }

    let mut matching_pixels = 0u64;

    // Sample for performance on large images
    let sample_step = 1u32.max((total_pixels as u32 / 10000).max(1));
    let mut sampled = 0u64;

    for y in (0..img1.height()).step_by(sample_step as usize) {
        for x in (0..img1.width()).step_by(sample_step as usize) {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2.get_pixel(x, y);

            let dr = (p1[0] as i32 - p2[0] as i32).abs();
            let dg = (p1[1] as i32 - p2[1] as i32).abs();
            let db = (p1[2] as i32 - p2[2] as i32).abs();

            if dr <= tolerance && dg <= tolerance && db <= tolerance {
                matching_pixels += 1;
            }
            sampled += 1;
        }
    }

    if sampled == 0 {
        return 0.0;
    }

    (matching_pixels as f64) / (sampled as f64)
}
