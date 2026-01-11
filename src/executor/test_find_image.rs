
    use super::*;
    use image::GenericImageView;

    pub fn run_verification_test() {
        println!("Starting FindImage self-test...");

        // 1. Capture screen
        let monitors = xcap::Monitor::all().unwrap();
        let monitor = monitors.first().unwrap();
        let screen = monitor.capture_image().unwrap();
        let screen = image::RgbaImage::from_raw(
            screen.width(),
            screen.height(),
            screen.into_raw(),
        ).unwrap();

        println!("Screen captured: {}x{}", screen.width(), screen.height());

        // 2. Crop a region (e.g., center 100x100)
        // IMPORTANT: Ensure coordinates are EVEN for Retina (2x) downscaling tests.
        // If we crop at an odd coordinate, the downscaled grid (aligned to 0) will have a half-pixel phase shift
        // relative to the cropped template, causing mismatch.
        let mut crop_x = screen.width() / 2;
        let mut crop_y = screen.height() / 2;
        
        if crop_x % 2 != 0 { crop_x -= 1; }
        if crop_y % 2 != 0 { crop_y -= 1; }
        
        let crop_w = 200;
        let crop_h = 100;
        
        let template = image::imageops::crop_imm(&screen, crop_x, crop_y, crop_w, crop_h).to_image();
        println!("Template created from screen: {}x{} at ({},{})", crop_w, crop_h, crop_x, crop_y);

        // Case A: Exact Match (Same resolution)
        println!("Testing Case A: Exact Match...");
        let start = std::time::Instant::now();
        let (found_x, found_y, found) = Interpreter::find_template_in_image(
            &screen, 
            &template, 
            10, // low tolerance
            0, 0, screen.width(), screen.height()
        );
        println!("Case A Result: Found={} at ({},{}) took {:.2}s", found, found_x, found_y, start.elapsed().as_secs_f64());
        
        // DEBUG: Print first pixel difference if not found
        if !found {
            println!("DEBUG: Checking pixel difference at expected ({},{})...", crop_x, crop_y);
            let tpl_p = template.get_pixel(0, 0);
            let scr_p = screen.get_pixel(crop_x, crop_y);
            println!("Template(0,0): {:?}", tpl_p);
            println!("Screen({},{}): {:?}", crop_x, crop_y, scr_p);
            
            // Check why it failed
            let tol = 10;
            let dr = (scr_p[0] as i32 - tpl_p[0] as i32).abs();
            let dg = (scr_p[1] as i32 - tpl_p[1] as i32).abs();
            let db = (scr_p[2] as i32 - tpl_p[2] as i32).abs();
            println!("Diff: R={} G={} B={} (Tol={})", dr, dg, db, tol);
            
            // Allow alpha check
            println!("Template Alpha: {}, Screen Alpha: {}", tpl_p[3], scr_p[3]);
        }

        assert!(found, "Failed to find exact match of template in screen");
        if found && (found_x - (crop_x as i64 + crop_w as i64 / 2)).abs() < 5 && (found_y - (crop_y as i64 + crop_h as i64 / 2)).abs() < 5  {
             println!("✅ Case A PASSED (Exact Match)");
        } else {
             println!("❌ Case A FAILED - Expected Exact Match");
             println!("   Expected Center: ({}, {})", crop_x + crop_w / 2, crop_y + crop_h / 2);
             println!("   Actual Found:    ({}, {})", found_x, found_y);
        }
        
        // Assert Case A must pass
        if !found {
            // DEBUG block already printed above
            panic!("Case A Failed");
        }

        // Case B: Retina Mismatch Simulation
        // If screen is > 2000px wide, we assume it's Retina (@2x).
        // Let's create a "logical" template by downscaling our crop by 2x.
        // This simulates a template captured by `screencapture` (@1x) but screen is `xcap` (@2x).
        if screen.width() > 2000 {
            println!("Testing Case B: Retina Mismatch (Template @1x, Screen @2x)...");
            // NOTE: Resizing a crop is not exactly the same as cropping a resize, so pixel match won't be perfect.
            // We use a lenient tolerance here.
            let small_template = image::imageops::resize(
                &template,
                crop_w / 2,
                crop_h / 2,
                image::imageops::FilterType::Triangle
            );
            println!("Simulated logical template: {}x{}", small_template.width(), small_template.height());

            let start = std::time::Instant::now();
            let (found_x, found_y, found) = Interpreter::find_template_in_image(
                &screen, 
                &small_template, 
                40, // Reduced tolerance to avoid false positives
                0, 0, screen.width(), screen.height()
            );
            println!("Case B Result: Found={} at ({},{}) took {:.2}s", found, found_x, found_y, start.elapsed().as_secs_f64());

            // Note: found_x/y should be in physical pixels (Center of original crop)
            let expected_x = crop_x + crop_w / 2;
            let expected_y = crop_y + crop_h / 2;
            
            if found && (found_x - expected_x as i64).abs() < 20 {
                println!("✅ Case B PASSED (Retina handled correctly)");
            } else {
                 println!("❌ Case B FAILED");
                 if found {
                     println!("   Expected Center: ({}, {})", expected_x, expected_y);
                     println!("   Actual Found:    ({}, {})", found_x, found_y);
                 }
            }
        } else {
            println!("Skipping Retina test (screen width < 2000px)");
        }
    }
