//! # Image Matching Algorithms
//!
//! This module provides image template matching and comparison functions
//! for the image recognition features.
//!
//! ## Key Functions
//! - [`find_template_in_image`]: Find a template image within a screen capture using NCC
//! - [`compare_images`]: Calculate similarity score between two images
//!
//! ## Algorithm Details
//!
//! ### Template Matching (NCC - Normalized Cross-Correlation)
//! Uses `imageproc::template_matching` for robust matching:
//! - Invariant to brightness changes
//! - Handles compression artifacts
//! - Returns confidence score (0.0 to 1.0)
//!
//! ### Image Comparison
//! - Per-channel tolerance matching
//! - Sampling for large images (max 10000 samples)
//!
//! ## Dependencies
//! - `image`: Image processing
//! - `imageproc`: Template matching with NCC

use image::{GrayImage, RgbaImage};
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};

/// Find a template image within a screen region using Normalized Cross-Correlation.
///
/// # Arguments
/// * `screen` - The screen capture to search in
/// * `template` - The template image to find
/// * `tolerance` - Matching threshold (0-255). Lower = stricter match required.
///                 Converted to NCC threshold: threshold = 1.0 - (tolerance / 255.0)
/// * `region_x`, `region_y` - Top-left corner of search region (in screen pixels)
/// * `region_w`, `region_h` - Size of search region
///
/// # Returns
/// `(x, y, found)` - Center coordinates of found template and success flag
///
/// # Algorithm
/// 1. Crop screen to specified region
/// 2. Convert both images to grayscale
/// 3. Apply NCC template matching
/// 4. Find maximum correlation value
/// 5. Compare against threshold derived from tolerance
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

    // Clamp region to screen bounds
    let scr_w = screen.width();
    let scr_h = screen.height();
    
    let actual_x = region_x.min(scr_w.saturating_sub(1));
    let actual_y = region_y.min(scr_h.saturating_sub(1));
    let actual_w = region_w.min(scr_w.saturating_sub(actual_x));
    let actual_h = region_h.min(scr_h.saturating_sub(actual_y));

    // Ensure region is large enough for template
    if actual_w < tpl_w || actual_h < tpl_h {
        return (0, 0, false);
    }

    // Crop screen to region
    let region = image::imageops::crop_imm(screen, actual_x, actual_y, actual_w, actual_h).to_image();

    // Convert to grayscale for NCC matching
    let screen_gray: GrayImage = image::imageops::grayscale(&region);
    let template_gray: GrayImage = image::imageops::grayscale(template);

    // Perform NCC template matching
    let result = match_template(
        &screen_gray,
        &template_gray,
        MatchTemplateMethod::CrossCorrelationNormalized,
    );

    // Find the maximum correlation value
    let extremes = find_extremes(&result);

    // Convert tolerance (0-255) to NCC threshold (0.0-1.0)
    // tolerance 0 = threshold 1.0 (perfect match required)
    // tolerance 255 = threshold 0.0 (any match accepted)
    // For practical use: tolerance 10 → threshold ~0.96
    let tolerance_clamped = tolerance.clamp(0, 255) as f32;
    let threshold = 1.0 - (tolerance_clamped / 255.0);

    // NCC returns values in [0, 1] for CrossCorrelationNormalized
    // Higher value = better match
    if extremes.max_value >= threshold {
        let (local_x, local_y) = extremes.max_value_location;
        
        // Convert back to screen coordinates (add region offset and template center)
        let center_x = actual_x as i64 + local_x as i64 + (tpl_w / 2) as i64;
        let center_y = actual_y as i64 + local_y as i64 + (tpl_h / 2) as i64;
        
        (center_x, center_y, true)
    } else {
        (0, 0, false)
    }
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
