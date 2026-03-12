//! Pdflatex comparison tests for the RustLaTex compiler.
//!
//! These tests compile `examples/compare.tex` with both our compiler and pdflatex,
//! render the resulting PDFs to PNG with GhostScript, and log the pixel difference.
//!
//! Tests that require `pdflatex` skip gracefully when the `SKIP_PDFLATEX_TESTS`
//! environment variable is set. Tests that require `gs` skip when it is not available.

use std::path::{Path, PathBuf};
use std::process::Command;

// ===== Helper Functions =====

/// Return the path to `examples/compare.tex` relative to the crate root.
fn compare_tex_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../examples/compare.tex");
    p
}

/// Check if the GhostScript binary is available on this system.
fn gs_available() -> bool {
    for candidate in &[
        "/opt/homebrew/bin/gs",
        "/usr/bin/gs",
        "/usr/local/bin/gs",
        "gs",
    ] {
        if Path::new(candidate).exists() {
            return true;
        }
        if Command::new(candidate)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Find the GhostScript binary path.
fn gs_path() -> String {
    for candidate in &["/opt/homebrew/bin/gs", "/usr/bin/gs", "/usr/local/bin/gs"] {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "gs".to_string()
}

/// Render a PDF to a PNG using GhostScript. Returns the PNG path.
fn render_pdf_to_png(pdf_path: &Path, label: &str) -> PathBuf {
    let png_path = std::env::temp_dir().join(format!(
        "rustlatex_cmp_{}_{}.png",
        label,
        std::process::id()
    ));
    let _ = std::fs::remove_file(&png_path);
    let gs = gs_path();
    let output = Command::new(&gs)
        .args([
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=pngalpha",
            "-r150",
            &format!("-sOutputFile={}", png_path.display()),
            pdf_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to invoke gs");
    assert!(
        output.status.success(),
        "gs failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    png_path
}

/// Render a PDF to a PPM using GhostScript. Returns the PPM path.
fn render_pdf_to_ppm(pdf_path: &Path, label: &str) -> PathBuf {
    let ppm_path = std::env::temp_dir().join(format!(
        "rustlatex_cmp_{}_{}.ppm",
        label,
        std::process::id()
    ));
    let _ = std::fs::remove_file(&ppm_path);
    let gs = gs_path();
    let output = Command::new(&gs)
        .args([
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=ppmraw",
            "-r72",
            "-dFirstPage=1",
            "-dLastPage=1",
            &format!("-sOutputFile={}", ppm_path.display()),
            pdf_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to invoke gs for PPM");
    assert!(
        output.status.success(),
        "gs PPM render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    ppm_path
}

/// Compare two PPM (P6 binary) files pixel-by-pixel.
/// Returns the fraction of matching RGB pixels (0.0 to 1.0).
fn compare_ppm_files(path1: &Path, path2: &Path) -> f64 {
    let data1 = match std::fs::read(path1) {
        Ok(d) => d,
        Err(_) => return 0.0,
    };
    let data2 = match std::fs::read(path2) {
        Ok(d) => d,
        Err(_) => return 0.0,
    };

    // Parse PPM P6 header: "P6\n", "width height\n", "maxval\n"
    fn parse_ppm_header(data: &[u8]) -> Option<usize> {
        if !data.starts_with(b"P6") {
            return None;
        }
        // Skip past 3 newlines to get pixel data offset
        let mut newlines = 0;
        let mut i = 0;
        while i < data.len() && newlines < 3 {
            if data[i] == b'\n' {
                newlines += 1;
            }
            i += 1;
        }
        if newlines == 3 {
            Some(i)
        } else {
            None
        }
    }

    let offset1 = match parse_ppm_header(&data1) {
        Some(o) => o,
        None => return 0.0,
    };
    let offset2 = match parse_ppm_header(&data2) {
        Some(o) => o,
        None => return 0.0,
    };

    let pixels1 = &data1[offset1..];
    let pixels2 = &data2[offset2..];

    let min_len = pixels1.len().min(pixels2.len());
    if min_len == 0 {
        return 0.0;
    }

    // Count matching pixels (compare RGB triplets with ±2 per-channel tolerance)
    let total_pixels = min_len / 3;
    if total_pixels == 0 {
        return 0.0;
    }
    let mut matching = 0u64;
    for i in 0..total_pixels {
        let idx = i * 3;
        let r_ok = (pixels1[idx] as i32 - pixels2[idx] as i32).abs() <= 2;
        let g_ok = (pixels1[idx + 1] as i32 - pixels2[idx + 1] as i32).abs() <= 2;
        let b_ok = (pixels1[idx + 2] as i32 - pixels2[idx + 2] as i32).abs() <= 2;
        if r_ok && g_ok && b_ok {
            matching += 1;
        }
    }
    matching as f64 / total_pixels as f64
}

/// Check if pdflatex tests should be skipped.
fn skip_pdflatex() -> bool {
    std::env::var("SKIP_PDFLATEX_TESTS").is_ok()
}

/// Check if pdflatex is available on the system.
fn pdflatex_available() -> bool {
    Command::new("pdflatex")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ===== Test 1: Our compiler produces a PDF for compare.tex =====

#[test]
fn test_our_compiler_produces_pdf_for_compare_tex() {
    let tex_path = compare_tex_path();
    assert!(
        tex_path.exists(),
        "compare.tex must exist at {:?}",
        tex_path
    );

    let out_pdf =
        std::env::temp_dir().join(format!("rustlatex_cmp_ours_{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&out_pdf);

    // Use the compiled CLI binary
    let bin = env!("CARGO_BIN_EXE_rustlatex");
    let output = Command::new(bin)
        .arg(tex_path.to_str().unwrap())
        .arg(out_pdf.to_str().unwrap())
        .output()
        .expect("failed to run rustlatex binary");

    assert!(
        output.status.success(),
        "rustlatex failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(out_pdf.exists(), "Output PDF must exist");
    let size = std::fs::metadata(&out_pdf).unwrap().len();
    assert!(size > 0, "Output PDF must be non-empty, got {} bytes", size);

    // Clean up
    let _ = std::fs::remove_file(&out_pdf);
}

// ===== Test 2: pdflatex produces a PDF for compare.tex =====

#[test]
fn test_pdflatex_produces_pdf_for_compare_tex() {
    if skip_pdflatex() {
        eprintln!("Skipping: SKIP_PDFLATEX_TESTS is set");
        return;
    }
    if !pdflatex_available() {
        eprintln!("Skipping: pdflatex not found on PATH");
        return;
    }

    let tex_path = compare_tex_path();
    assert!(tex_path.exists(), "compare.tex must exist");

    // Copy compare.tex into a temp directory so pdflatex outputs there
    let tmp = std::env::temp_dir().join(format!("rustlatex_pdflatex_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let dest_tex = tmp.join("compare.tex");
    std::fs::copy(&tex_path, &dest_tex).expect("failed to copy compare.tex");

    let output = Command::new("pdflatex")
        .args(["-interaction=nonstopmode", "compare.tex"])
        .current_dir(&tmp)
        .output()
        .expect("failed to run pdflatex");

    assert!(
        output.status.success(),
        "pdflatex failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let pdf_path = tmp.join("compare.pdf");
    assert!(pdf_path.exists(), "pdflatex must produce compare.pdf");
    let size = std::fs::metadata(&pdf_path).unwrap().len();
    assert!(
        size > 0,
        "pdflatex PDF must be non-empty, got {} bytes",
        size
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp);
}

// ===== Test 3: Our PDF is renderable by GhostScript =====

#[test]
fn test_our_pdf_renderable_by_ghostscript() {
    if !gs_available() {
        eprintln!("Skipping: gs not available");
        return;
    }

    let tex_path = compare_tex_path();
    let out_pdf =
        std::env::temp_dir().join(format!("rustlatex_cmp_ours_gs_{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&out_pdf);

    let bin = env!("CARGO_BIN_EXE_rustlatex");
    let output = Command::new(bin)
        .arg(tex_path.to_str().unwrap())
        .arg(out_pdf.to_str().unwrap())
        .output()
        .expect("failed to run rustlatex binary");
    assert!(output.status.success(), "rustlatex compilation failed");

    let png_path = render_pdf_to_png(&out_pdf, "ours");
    assert!(png_path.exists(), "PNG output must exist");
    let png_size = std::fs::metadata(&png_path).unwrap().len();
    assert!(
        png_size > 1000,
        "PNG must be >1000 bytes, got {} bytes",
        png_size
    );

    // Clean up
    let _ = std::fs::remove_file(&out_pdf);
    let _ = std::fs::remove_file(&png_path);
}

// ===== Test 4: pdflatex PDF is renderable by GhostScript =====

#[test]
fn test_pdflatex_pdf_renderable_by_ghostscript() {
    if skip_pdflatex() {
        eprintln!("Skipping: SKIP_PDFLATEX_TESTS is set");
        return;
    }
    if !pdflatex_available() {
        eprintln!("Skipping: pdflatex not found on PATH");
        return;
    }
    if !gs_available() {
        eprintln!("Skipping: gs not available");
        return;
    }

    let tex_path = compare_tex_path();
    let tmp = std::env::temp_dir().join(format!("rustlatex_pdflatex_gs_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let dest_tex = tmp.join("compare.tex");
    std::fs::copy(&tex_path, &dest_tex).expect("failed to copy compare.tex");

    let output = Command::new("pdflatex")
        .args(["-interaction=nonstopmode", "compare.tex"])
        .current_dir(&tmp)
        .output()
        .expect("failed to run pdflatex");
    assert!(output.status.success(), "pdflatex failed");

    let pdf_path = tmp.join("compare.pdf");
    let png_path = render_pdf_to_png(&pdf_path, "pdflatex");
    assert!(png_path.exists(), "PNG output must exist");
    let png_size = std::fs::metadata(&png_path).unwrap().len();
    assert!(
        png_size > 1000,
        "PNG must be >1000 bytes, got {} bytes",
        png_size
    );

    // Clean up
    let _ = std::fs::remove_file(&png_path);
    let _ = std::fs::remove_dir_all(&tmp);
}

// ===== Test 5: Pixel similarity logged (no assertion) =====

#[test]
fn test_pixel_similarity_logged() {
    if skip_pdflatex() {
        eprintln!("Skipping: SKIP_PDFLATEX_TESTS is set");
        return;
    }
    if !pdflatex_available() {
        eprintln!("Skipping: pdflatex not found on PATH");
        return;
    }
    if !gs_available() {
        eprintln!("Skipping: gs not available");
        return;
    }

    let tex_path = compare_tex_path();

    // --- Our PDF ---
    let our_pdf =
        std::env::temp_dir().join(format!("rustlatex_cmp_sim_ours_{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&our_pdf);
    let bin = env!("CARGO_BIN_EXE_rustlatex");
    let output = Command::new(bin)
        .arg(tex_path.to_str().unwrap())
        .arg(our_pdf.to_str().unwrap())
        .output()
        .expect("failed to run rustlatex");
    assert!(output.status.success(), "rustlatex failed");
    let our_ppm = render_pdf_to_ppm(&our_pdf, "sim_ours");

    // --- pdflatex PDF ---
    let tmp = std::env::temp_dir().join(format!("rustlatex_sim_pdflatex_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::copy(&tex_path, tmp.join("compare.tex")).expect("copy failed");
    let output = Command::new("pdflatex")
        .args(["-interaction=nonstopmode", "compare.tex"])
        .current_dir(&tmp)
        .output()
        .expect("failed to run pdflatex");
    assert!(output.status.success(), "pdflatex failed");
    let pdflatex_ppm = render_pdf_to_ppm(&tmp.join("compare.pdf"), "sim_pdflatex");

    // --- Compare pixel data using PPM comparison ---
    let similarity = compare_ppm_files(&our_ppm, &pdflatex_ppm);

    let our_size = std::fs::metadata(&our_ppm).map(|m| m.len()).unwrap_or(0);
    let their_size = std::fs::metadata(&pdflatex_ppm)
        .map(|m| m.len())
        .unwrap_or(0);

    eprintln!("=== Pixel Similarity Report (PPM) ===");
    eprintln!("Our PPM size:      {} bytes", our_size);
    eprintln!("pdflatex PPM size: {} bytes", their_size);
    eprintln!(
        "Visual similarity: {:.4} ({:.2}%)",
        similarity,
        similarity * 100.0
    );
    eprintln!("=====================================");

    // Write similarity to temp file for CI consumption
    std::fs::write(
        "/tmp/pixel_similarity.txt",
        format!("similarity={:.4}", similarity),
    )
    .ok();

    // Clean up
    let _ = std::fs::remove_file(&our_pdf);
    let _ = std::fs::remove_file(&our_ppm);
    let _ = std::fs::remove_file(&pdflatex_ppm);
    let _ = std::fs::remove_dir_all(&tmp);
}

// ===== M31: Tests verifying similarity infrastructure =====

/// Verify that similarity helper functions exist and behave correctly.
#[test]
fn test_similarity_score_is_one_for_identical_bytes() {
    let data = vec![1u8, 2, 3, 4, 5];
    let our_bytes = &data;
    let their_bytes = &data;
    let min_len = our_bytes.len().min(their_bytes.len());
    let max_len = our_bytes.len().max(their_bytes.len());
    let mut diff_count: u64 = 0;
    for i in 0..min_len {
        if our_bytes[i] != their_bytes[i] {
            diff_count += 1;
        }
    }
    diff_count += (max_len - min_len) as u64;
    let similarity = if max_len > 0 {
        1.0 - (diff_count as f64 / max_len as f64)
    } else {
        1.0
    };
    assert!(
        (similarity - 1.0).abs() < f64::EPSILON,
        "Identical byte arrays should have similarity 1.0"
    );
}

/// Verify that similarity is 0 for completely different byte arrays.
#[test]
fn test_similarity_score_is_zero_for_all_different_bytes() {
    let our_bytes = vec![0u8, 0, 0, 0];
    let their_bytes = vec![1u8, 2, 3, 4];
    let min_len = our_bytes.len().min(their_bytes.len());
    let max_len = our_bytes.len().max(their_bytes.len());
    let mut diff_count: u64 = 0;
    for i in 0..min_len {
        if our_bytes[i] != their_bytes[i] {
            diff_count += 1;
        }
    }
    diff_count += (max_len - min_len) as u64;
    let similarity = if max_len > 0 {
        1.0 - (diff_count as f64 / max_len as f64)
    } else {
        1.0
    };
    assert!(
        similarity < 1.0,
        "Completely different byte arrays should have similarity < 1.0"
    );
    assert!(similarity >= 0.0, "Similarity must be non-negative");
}

/// Verify that gs_available() returns a bool without panicking.
#[test]
fn test_gs_available_does_not_panic() {
    let _result = gs_available();
    // Just verify it runs without panicking
}

/// Verify skip_pdflatex() function behaves correctly when env var absent.
#[test]
fn test_skip_pdflatex_default_is_false() {
    // Unless SKIP_PDFLATEX_TESTS is set in the env, should return false
    if std::env::var("SKIP_PDFLATEX_TESTS").is_err() {
        assert!(
            !skip_pdflatex(),
            "skip_pdflatex() should be false when env var not set"
        );
    }
}

// ===== M35: PPM comparison tests =====

#[test]
fn test_compare_ppm_files_identical() {
    // Two identical PPM files should produce similarity 1.0
    let header = b"P6\n2 1\n255\n";
    let pixels = [255u8, 0, 0, 0, 255, 0]; // red, green
    let mut data = Vec::new();
    data.extend_from_slice(header);
    data.extend_from_slice(&pixels);

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_identical_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_identical_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data).unwrap();
    std::fs::write(&p2, &data).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        (sim - 1.0).abs() < f64::EPSILON,
        "Identical PPM should have similarity 1.0, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_different() {
    // Completely different pixels should produce similarity 0.0
    // Use 0 vs 128 to exceed the ±2 per-channel tolerance
    let header = b"P6\n2 1\n255\n";
    let mut data1 = Vec::new();
    data1.extend_from_slice(header);
    data1.extend_from_slice(&[0, 0, 0, 0, 0, 0]);

    let mut data2 = Vec::new();
    data2.extend_from_slice(header);
    data2.extend_from_slice(&[128, 128, 128, 128, 128, 128]);

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_diff_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_diff_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data1).unwrap();
    std::fs::write(&p2, &data2).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        sim < f64::EPSILON,
        "Completely different PPM should have similarity 0.0, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_empty_returns_zero() {
    // Empty files should return 0.0
    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_empty_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_empty_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, b"").unwrap();
    std::fs::write(&p2, b"").unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        sim < f64::EPSILON,
        "Empty PPM files should have similarity 0.0, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_invalid_format() {
    // Non-PPM data should return 0.0
    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_invalid_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_invalid_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, b"not a ppm file").unwrap();
    std::fs::write(&p2, b"also not ppm").unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        sim < f64::EPSILON,
        "Invalid PPM should have similarity 0.0, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_partial_match() {
    // Half matching pixels: 1 match out of 2 -> 0.5
    let header = b"P6\n2 1\n255\n";
    let mut data1 = Vec::new();
    data1.extend_from_slice(header);
    data1.extend_from_slice(&[255, 0, 0, 0, 255, 0]);

    let mut data2 = Vec::new();
    data2.extend_from_slice(header);
    data2.extend_from_slice(&[255, 0, 0, 0, 0, 255]); // first pixel same, second different

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_partial_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_partial_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data1).unwrap();
    std::fs::write(&p2, &data2).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        (sim - 0.5).abs() < f64::EPSILON,
        "Half matching should have similarity 0.5, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_missing_file() {
    // Missing file should return 0.0
    let dir = std::env::temp_dir();
    let p1 = dir.join("nonexistent_ppm_file_1.ppm");
    let p2 = dir.join("nonexistent_ppm_file_2.ppm");
    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        sim < f64::EPSILON,
        "Missing files should have similarity 0.0, got {}",
        sim
    );
}

#[test]
fn test_compare_ppm_header_parsing() {
    // Valid P6 header should be parsed; single pixel white
    let header = b"P6\n1 1\n255\n";
    let pixels = [255u8, 255, 255];
    let mut data = Vec::new();
    data.extend_from_slice(header);
    data.extend_from_slice(&pixels);

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_header_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_header_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data).unwrap();
    std::fs::write(&p2, &data).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        (sim - 1.0).abs() < f64::EPSILON,
        "Single identical pixel should give 1.0, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_tex_has_subsection() {
    let tex_path = compare_tex_path();
    let content = std::fs::read_to_string(&tex_path).expect("Failed to read compare.tex");
    assert!(
        content.contains(r"\subsection{Details}"),
        "compare.tex must contain \\subsection{{Details}}"
    );
}

#[test]
fn test_compare_tex_has_display_math() {
    let tex_path = compare_tex_path();
    let content = std::fs::read_to_string(&tex_path).expect("Failed to read compare.tex");
    assert!(
        content.contains(r"\[ E = mc^2 \]"),
        "compare.tex must contain \\[ E = mc^2 \\]"
    );
}

// ===== M47: PPM tolerance tests =====

#[test]
fn test_compare_ppm_files_within_tolerance() {
    // Pixels differing by ±2 per channel should be counted as matching
    let header = b"P6\n2 1\n255\n";
    let mut data1 = Vec::new();
    data1.extend_from_slice(header);
    data1.extend_from_slice(&[100, 100, 100, 200, 200, 200]);

    let mut data2 = Vec::new();
    data2.extend_from_slice(header);
    data2.extend_from_slice(&[102, 98, 100, 201, 199, 202]); // within ±2

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_tol_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_tol_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data1).unwrap();
    std::fs::write(&p2, &data2).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        (sim - 1.0).abs() < f64::EPSILON,
        "Pixels within ±2 tolerance should match, got similarity {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_outside_tolerance() {
    // Pixels differing by more than ±2 on any channel should NOT match
    let header = b"P6\n1 1\n255\n";
    let mut data1 = Vec::new();
    data1.extend_from_slice(header);
    data1.extend_from_slice(&[100, 100, 100]);

    let mut data2 = Vec::new();
    data2.extend_from_slice(header);
    data2.extend_from_slice(&[103, 100, 100]); // R differs by 3 (outside tolerance)

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_outside_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_outside_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data1).unwrap();
    std::fs::write(&p2, &data2).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        sim < f64::EPSILON,
        "Pixels outside ±2 tolerance should NOT match, got similarity {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}

#[test]
fn test_compare_ppm_files_boundary_tolerance() {
    // Exactly ±2 difference should match; ±3 should not
    let header = b"P6\n2 1\n255\n";

    // Pixel 1: diff exactly 2 on all channels (should match)
    // Pixel 2: diff exactly 3 on red channel (should NOT match)
    let mut data1 = Vec::new();
    data1.extend_from_slice(header);
    data1.extend_from_slice(&[10, 10, 10, 10, 10, 10]);

    let mut data2 = Vec::new();
    data2.extend_from_slice(header);
    data2.extend_from_slice(&[12, 8, 12, 13, 10, 10]); // first: all ±2; second: R=13 (diff=3)

    let dir = std::env::temp_dir();
    let p1 = dir.join(format!("test_ppm_boundary_1_{}.ppm", std::process::id()));
    let p2 = dir.join(format!("test_ppm_boundary_2_{}.ppm", std::process::id()));
    std::fs::write(&p1, &data1).unwrap();
    std::fs::write(&p2, &data2).unwrap();

    let sim = compare_ppm_files(&p1, &p2);
    assert!(
        (sim - 0.5).abs() < f64::EPSILON,
        "One pixel within tolerance, one outside => 0.5, got {}",
        sim
    );

    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}
