//! # Image Matching Algorithms
//!
//! This module provides image template matching and comparison functions
//! for the image recognition features.
//!
//! ## Supported Algorithms
//! - **NCC** (Normalized Cross-Correlation): Most accurate, slower
//! - **SSD** (Sum of Squared Differences): Fastest, less robust
//! - **SSDNorm** (Normalized SSD): Balanced speed and accuracy
//!
//! ## DPI Scaling
//! - User provides coordinates in logical pixels
//! - Screen capture is in physical pixels
//! - Scale factor is auto-detected (physical_width / logical_width)
//! - Output coordinates are converted back to logical pixels

use image::{GrayImage, RgbaImage};
use imageproc::template_matching::{find_extremes, match_template_parallel, MatchTemplateMethod};

/// Matching algorithm selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchingAlgorithm {
    /// Normalized Cross-Correlation - Most accurate, handles brightness changes
    NCC,
    /// Sum of Squared Differences - Fastest, best for exact matches
    SSD,
    /// Normalized Sum of Squared Differences - Balanced
    SSDNorm,
}

impl MatchingAlgorithm {
    /// Parse algorithm from string (for node input)
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SSD" => MatchingAlgorithm::SSD,
            "SSDNORM" | "SSD_NORM" => MatchingAlgorithm::SSDNorm,
            _ => MatchingAlgorithm::NCC, // Default
        }
    }
    
    /// Convert to imageproc MatchTemplateMethod
    fn to_method(self) -> MatchTemplateMethod {
        match self {
            MatchingAlgorithm::NCC => MatchTemplateMethod::CrossCorrelationNormalized,
            MatchingAlgorithm::SSD => MatchTemplateMethod::SumOfSquaredErrors,
            MatchingAlgorithm::SSDNorm => MatchTemplateMethod::SumOfSquaredErrorsNormalized,
        }
    }
    
    /// Whether to use max (NCC) or min (SSD) for best match
    fn use_max(self) -> bool {
        match self {
            MatchingAlgorithm::NCC => true,  // Higher = better
            MatchingAlgorithm::SSD | MatchingAlgorithm::SSDNorm => false, // Lower = better
        }
    }
}

/// Find a template image within a screen region.
///
/// # Arguments
/// * `screen` - The screen capture (in physical pixels)
/// * `template` - The template image to find (in physical pixels from xcap)
/// * `tolerance` - Matching threshold (1-100). 100 = strict, 1 = loose.
/// * `region_x`, `region_y` - Top-left corner of search region (in LOGICAL pixels)
/// * `region_w`, `region_h` - Size of search region (in LOGICAL pixels)
/// * `scale_factor` - DPI scale factor (e.g., 2.0 for Retina displays)
/// * `algorithm` - Matching algorithm to use
///
/// # Returns
/// `(x, y, found)` - Center coordinates of found template (in LOGICAL pixels) and success flag
pub fn find_template_in_image(
    screen: &RgbaImage,
    template: &RgbaImage,
    tolerance: i32,
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
    scale_factor: f32,
    algorithm: MatchingAlgorithm,
) -> (i64, i64, bool) {
    let tpl_w = template.width();
    let tpl_h = template.height();

    if tpl_w == 0 || tpl_h == 0 {
        return (0, 0, false);
    }

    // Convert logical region to physical pixels
    let phys_x = (region_x as f32 * scale_factor) as u32;
    let phys_y = (region_y as f32 * scale_factor) as u32;
    let phys_w = (region_w as f32 * scale_factor) as u32;
    let phys_h = (region_h as f32 * scale_factor) as u32;

    // Clamp region to screen bounds
    let scr_w = screen.width();
    let scr_h = screen.height();
    
    let actual_x = phys_x.min(scr_w.saturating_sub(1));
    let actual_y = phys_y.min(scr_h.saturating_sub(1));
    let actual_w = phys_w.min(scr_w.saturating_sub(actual_x));
    let actual_h = phys_h.min(scr_h.saturating_sub(actual_y));

    // Ensure region is large enough for template
    if actual_w < tpl_w || actual_h < tpl_h {
        return (0, 0, false);
    }

    // Crop screen to region
    let region = image::imageops::crop_imm(screen, actual_x, actual_y, actual_w, actual_h).to_image();

    // Convert to grayscale for faster matching
    let screen_gray: GrayImage = image::imageops::grayscale(&region);
    let template_gray: GrayImage = image::imageops::grayscale(template);

    // Use PARALLEL template matching with selected algorithm
    let result = match_template_parallel(
        &screen_gray,
        &template_gray,
        algorithm.to_method(),
    );

    // Find extremes
    let extremes = find_extremes(&result);

    // Convert tolerance (1-100) to threshold
    let tolerance_clamped = tolerance.clamp(1, 100) as f32;
    
    // Check if match is good enough based on algorithm type
    let (matched, best_x, best_y) = if algorithm.use_max() {
        // NCC: Higher value = better match (range 0.0 to 1.0)
        let threshold = tolerance_clamped / 100.0;
        let matched = extremes.max_value >= threshold;
        (matched, extremes.max_value_location.0, extremes.max_value_location.1)
    } else {
        // SSD: Lower value = better match
        // For SSD, tolerance maps inversely: high tolerance = accept higher errors
        // Typical SSD values depend on image size, so we use a relative threshold
        let max_possible_error = (tpl_w * tpl_h) as f32 * 255.0 * 255.0;
        let threshold = max_possible_error * (1.0 - tolerance_clamped / 100.0) * 0.1;
        let matched = extremes.min_value <= threshold;
        (matched, extremes.min_value_location.0, extremes.min_value_location.1)
    };

    if matched {
        // Calculate center in physical pixels
        let phys_center_x = actual_x as f32 + best_x as f32 + (tpl_w as f32 / 2.0);
        let phys_center_y = actual_y as f32 + best_y as f32 + (tpl_h as f32 / 2.0);
        
        // Convert back to logical pixels
        let logical_x = (phys_center_x / scale_factor) as i64;
        let logical_y = (phys_center_y / scale_factor) as i64;
        
        (logical_x, logical_y, true)
    } else {
        (0, 0, false)
    }
}

/// Compare two images and return similarity score (0.0 - 1.0).
pub fn compare_images(img1: &RgbaImage, img2: &RgbaImage, tolerance: i32) -> f64 {
    if img1.width() != img2.width() || img1.height() != img2.height() {
        return 0.0;
    }

    let total_pixels = (img1.width() * img1.height()) as f64;
    if total_pixels == 0.0 {
        return 0.0;
    }

    let mut matching_pixels = 0u64;
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

    if sampled == 0 { 0.0 } else { (matching_pixels as f64) / (sampled as f64) }
}
