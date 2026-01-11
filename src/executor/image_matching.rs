//! # Image Matching Algorithms
//!
//! This module provides image template matching and comparison functions
//! for the image recognition features.
//!
//! ## Key Functions
//! - [`find_template_in_image`]: Find a template image within a screen capture
//! - [`compare_images`]: Calculate similarity score between two images
//!
//! ## Algorithm Details
//!
//! ### Template Matching
//! Uses multi-scale search to handle Retina displays:
//! 1. Exact match at 1.0x scale
//! 2. Downscaled match at 0.5x scale for Retina screens
//!
//! Optimizations:
//! - Center pixel early rejection
//! - Parallel search using Rayon
//! - Alpha channel masking (pixels with alpha < 128 are ignored)
//!
//! ### Image Comparison
//! - Per-channel tolerance matching
//! - Sampling for large images (max 10000 samples)
//!
//! ## Dependencies
//! - `image`: Image processing
//! - `rayon`: Parallel iteration

use image::RgbaImage;
use rayon::prelude::*;

/// Find a template image within a screen image using multi-scale matching.
///
/// # Arguments
/// * `screen` - The screen capture to search in
/// * `template` - The template image to find
/// * `tolerance` - Per-channel color difference tolerance (0-255)
/// * `region_x`, `region_y` - Top-left corner of search region
/// * `region_w`, `region_h` - Size of search region
///
/// # Returns
/// `(x, y, found)` - Center coordinates of found template (in logical pixels) and success flag
///
/// # Multi-Scale Handling
/// For Retina displays (screen width > 2000), the function:
/// 1. Searches at native resolution first
/// 2. Falls back to 0.5x downscaled search
/// 3. Returns coordinates in logical (non-Retina) space
pub fn find_template_in_image(
    screen: &RgbaImage,
    template: &RgbaImage,
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

    // Helper function for parallel search on a specific screen buffer
    let search_on_buffer = |search_screen: &RgbaImage, scale: u32| -> Option<(i64, i64)> {
        let scr_w = search_screen.width();
        let scr_h = search_screen.height();

        // Adjust region for current scale
        let r_x = region_x / scale;
        let r_y = region_y / scale;
        let r_w = region_w / scale;
        let r_h = region_h / scale;

        let end_x = (r_x + r_w).min(scr_w).saturating_sub(tpl_w);
        let end_y = (r_y + r_h).min(scr_h).saturating_sub(tpl_h);

        if r_x > end_x || r_y > end_y {
            return None;
        }

        let found = (r_y..=end_y).into_par_iter().find_map_first(|sy| {
            for sx in r_x..=end_x {
                // Check if this position matches
                let mut matches = true;
                // Optimization: Check center pixel first
                let center_pixel = search_screen.get_pixel(sx + tpl_w / 2, sy + tpl_h / 2);
                let tpl_center = template.get_pixel(tpl_w / 2, tpl_h / 2);
                if tpl_center[3] >= 128 {
                    let dr = (center_pixel[0] as i32 - tpl_center[0] as i32).abs();
                    let dg = (center_pixel[1] as i32 - tpl_center[1] as i32).abs();
                    let db = (center_pixel[2] as i32 - tpl_center[2] as i32).abs();
                    if dr > tolerance || dg > tolerance || db > tolerance {
                        continue;
                    }
                }

                // Full check
                for ty in 0..tpl_h {
                    for tx in 0..tpl_w {
                        let tpl_pixel = template.get_pixel(tx, ty);
                        if tpl_pixel[3] < 128 {
                            continue;
                        }

                        let scr_pixel = search_screen.get_pixel(sx + tx, sy + ty);
                        let dr = (scr_pixel[0] as i32 - tpl_pixel[0] as i32).abs();
                        let dg = (scr_pixel[1] as i32 - tpl_pixel[1] as i32).abs();
                        let db = (scr_pixel[2] as i32 - tpl_pixel[2] as i32).abs();

                        if dr > tolerance || dg > tolerance || db > tolerance {
                            matches = false;
                            break;
                        }
                    }
                    if !matches {
                        break;
                    }
                }

                if matches {
                    return Some((sx, sy));
                }
            }
            None
        });

        found.map(|(x, y)| {
            let final_x = (x * scale) as i64 + (tpl_w * scale / 2) as i64;
            let final_y = (y * scale) as i64 + (tpl_h * scale / 2) as i64;
            (final_x, final_y)
        })
    };

    // Pass 1: Try exact match (Scale 1x)
    if let Some((x, y)) = search_on_buffer(screen, 1) {
        // If screen > 2000, we are in Physical pixels.
        // But output should be Logical for UI/Enigo.
        // If Pass 1 matched on Retina, it means Template was also Physical (xcap).
        // So we must divide result by 2 to get Logical.
        if screen.width() > 2000 {
            return (x / 2, y / 2, true);
        }
        return (x, y, true);
    }

    // Pass 2: Try Downscaled match (Scale 2x)
    // Cases handled:
    // - Retina screen (@2x), Template @1x (e.g. from screencapture)
    if screen.width() > 2000 {
        // Downscale screen by 2x
        let new_w = screen.width() / 2;
        let new_h = screen.height() / 2;
        let downscaled = image::imageops::resize(
            screen,
            new_w,
            new_h,
            image::imageops::FilterType::Triangle,
        );

        if let Some((x, y)) = search_on_buffer(&downscaled, 2) {
            return (x, y, true);
        }
    }

    (0, 0, false)
}

/// Compare two images and return similarity score (0.0 - 1.0).
///
/// # Arguments
/// * `img1`, `img2` - Images to compare (must have same dimensions)
/// * `tolerance` - Per-channel difference tolerance for matching
///
/// # Returns
/// Similarity score from 0.0 (completely different) to 1.0 (identical within tolerance)
///
/// # Performance
/// For large images, uses sampling (max ~10000 pixels checked)
pub fn compare_images(img1: &RgbaImage, img2: &RgbaImage, tolerance: i32) -> f64 {
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
