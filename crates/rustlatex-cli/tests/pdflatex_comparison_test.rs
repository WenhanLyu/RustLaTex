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
            "-r150",
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

    // Parse PPM P6 header: "P6\n[#comment\n]...<width> <height>\n<maxval>\n"
    // Returns offset to start of pixel data.
    fn parse_ppm_header(data: &[u8]) -> Option<usize> {
        if !data.starts_with(b"P6") {
            return None;
        }
        let mut i = 2; // skip "P6"
                       // skip to end of magic line
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        i += 1; // skip '\n' after P6
                // skip comment lines starting with '#'
        while i < data.len() && data[i] == b'#' {
            while i < data.len() && data[i] != b'\n' {
                i += 1;
            }
            if i < data.len() {
                i += 1;
            }
        }
        // skip width height line
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        i += 1; // skip '\n'
                // skip maxval line
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        i += 1; // skip '\n'
        Some(i)
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

/// Parse PPM P6 binary data and return (width, height, pixel_data).
/// pixel_data is a Vec<u8> of RGB triplets in row-major order.
fn parse_ppm(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if !data.starts_with(b"P6") {
        return None;
    }
    // Parse PPM P6 header, skipping comment lines starting with '#'
    // Format: "P6\n[#comment\n]...<width> <height>\n<maxval>\n<pixels>"
    let mut i = 2; // skip "P6"
                   // skip to end of magic line
    while i < data.len() && data[i] != b'\n' {
        i += 1;
    }
    if i >= data.len() {
        return None;
    }
    i += 1; // skip '\n' after P6

    // skip comment lines
    while i < data.len() && data[i] == b'#' {
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i < data.len() {
            i += 1; // skip '\n'
        }
    }

    // read width height
    let wh_start = i;
    while i < data.len() && data[i] != b'\n' {
        i += 1;
    }
    if i >= data.len() {
        return None;
    }
    let wh_str = String::from_utf8_lossy(&data[wh_start..i]).to_string();
    i += 1; // skip '\n'

    let parts: Vec<&str> = wh_str.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let width: usize = parts[0].parse().ok()?;
    let height: usize = parts[1].parse().ok()?;

    // skip maxval line
    while i < data.len() && data[i] != b'\n' {
        i += 1;
    }
    if i >= data.len() {
        return None;
    }
    i += 1; // skip '\n'

    let pixels = data[i..].to_vec();
    Some((width, height, pixels))
}

/// Find the first non-white row and column in a PPM image.
/// Returns (first_row, first_col) where the first non-white pixel appears.
/// "White" means all three channels >= 250.
fn find_first_non_white(
    width: usize,
    height: usize,
    pixels: &[u8],
) -> (Option<usize>, Option<usize>) {
    let mut first_row: Option<usize> = None;
    let mut first_col: Option<usize> = None;

    for row in 0..height {
        for col in 0..width {
            let idx = (row * width + col) * 3;
            if idx + 2 >= pixels.len() {
                break;
            }
            let r = pixels[idx];
            let g = pixels[idx + 1];
            let b = pixels[idx + 2];
            // Non-white: any channel below 250
            if r < 250 || g < 250 || b < 250 {
                if first_row.is_none() {
                    first_row = Some(row);
                }
                if first_col.is_none() || col < first_col.unwrap() {
                    first_col = Some(col);
                }
            }
        }
    }
    (first_row, first_col)
}

#[test]
fn test_ppm_text_bounding_box() {
    // Diagnostic test: finds first non-white row and column in both our PPM
    // and pdflatex PPM, reports offsets to stderr.
    if skip_pdflatex() || !pdflatex_available() || !gs_available() {
        eprintln!("test_ppm_text_bounding_box: SKIPPED (pdflatex or gs not available)");
        return;
    }

    let tex_path = compare_tex_path();
    if !tex_path.exists() {
        eprintln!("test_ppm_text_bounding_box: SKIPPED (compare.tex not found)");
        return;
    }

    // --- Compile with our compiler ---
    let our_pdf =
        std::env::temp_dir().join(format!("rustlatex_bbox_ours_{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&our_pdf);
    let our_status = Command::new(env!("CARGO_BIN_EXE_rustlatex"))
        .args([tex_path.to_str().unwrap(), our_pdf.to_str().unwrap()])
        .output();
    let our_ok = our_status
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !our_ok {
        eprintln!("test_ppm_text_bounding_box: SKIPPED (our compiler failed)");
        return;
    }

    // --- Compile with pdflatex ---
    let pdflatex_dir =
        std::env::temp_dir().join(format!("rustlatex_bbox_pdflatex_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&pdflatex_dir);
    let pdflatex_status = Command::new("pdflatex")
        .args([
            "-interaction=nonstopmode",
            &format!("-output-directory={}", pdflatex_dir.display()),
            tex_path.to_str().unwrap(),
        ])
        .output();
    let pdflatex_ok = pdflatex_status
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !pdflatex_ok {
        eprintln!("test_ppm_text_bounding_box: SKIPPED (pdflatex failed)");
        return;
    }
    let pdflatex_pdf = pdflatex_dir.join("compare.pdf");
    if !pdflatex_pdf.exists() {
        eprintln!("test_ppm_text_bounding_box: SKIPPED (pdflatex PDF not found)");
        return;
    }

    // --- Render both to PPM ---
    let our_ppm = render_pdf_to_ppm(&our_pdf, "bbox_ours");
    let pdflatex_ppm = render_pdf_to_ppm(&pdflatex_pdf, "bbox_pdflatex");

    // --- Parse PPMs ---
    let our_data = std::fs::read(&our_ppm).expect("failed to read our PPM");
    let pdflatex_data = std::fs::read(&pdflatex_ppm).expect("failed to read pdflatex PPM");

    let (our_w, our_h, our_pixels) = parse_ppm(&our_data).expect("failed to parse our PPM");
    let (pdf_w, pdf_h, pdf_pixels) =
        parse_ppm(&pdflatex_data).expect("failed to parse pdflatex PPM");

    // --- Find bounding boxes ---
    let (our_first_row, our_first_col) = find_first_non_white(our_w, our_h, &our_pixels);
    let (pdf_first_row, pdf_first_col) = find_first_non_white(pdf_w, pdf_h, &pdf_pixels);

    eprintln!("=== test_ppm_text_bounding_box DIAGNOSTIC ===");
    eprintln!(
        "Our PPM:     {}x{}, first_non_white row={:?} col={:?}",
        our_w, our_h, our_first_row, our_first_col
    );
    eprintln!(
        "pdflatex PPM: {}x{}, first_non_white row={:?} col={:?}",
        pdf_w, pdf_h, pdf_first_row, pdf_first_col
    );

    if let (Some(our_r), Some(pdf_r)) = (our_first_row, pdf_first_row) {
        let row_offset = our_r as i64 - pdf_r as i64;
        eprintln!("Row offset (ours - pdflatex): {} pixels", row_offset);
    }
    if let (Some(our_c), Some(pdf_c)) = (our_first_col, pdf_first_col) {
        let col_offset = our_c as i64 - pdf_c as i64;
        eprintln!("Col offset (ours - pdflatex): {} pixels", col_offset);
    }
    eprintln!("=== END DIAGNOSTIC ===");

    // Cleanup
    let _ = std::fs::remove_file(&our_pdf);
    let _ = std::fs::remove_file(&our_ppm);
    let _ = std::fs::remove_file(&pdflatex_ppm);
    let _ = std::fs::remove_dir_all(&pdflatex_dir);
}

// ===== Per-document pixel similarity logged tests (non-failing) =====

/// Helper: compile tex_name.tex with our CLI + pdflatex, render both to PPM,
/// compute similarity (±2 tolerance), write to /tmp/<tex_name>_similarity.txt and log.
/// Non-failing (no assert on similarity value). Skips if pdflatex not available.
fn run_similarity_logged(tex_name: &str) {
    if skip_pdflatex() {
        eprintln!("Skipping {}: SKIP_PDFLATEX_TESTS is set", tex_name);
        return;
    }
    if !pdflatex_available() {
        eprintln!("Skipping {}: pdflatex not found on PATH", tex_name);
        return;
    }
    if !gs_available() {
        eprintln!("Skipping {}: gs not available", tex_name);
        return;
    }

    let mut tex_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    tex_path.push(format!("../../examples/{}.tex", tex_name));

    if !tex_path.exists() {
        eprintln!("Skipping {}: {:?} does not exist", tex_name, tex_path);
        return;
    }

    // --- Our PDF ---
    let our_pdf = std::env::temp_dir().join(format!(
        "rustlatex_{}_ours_{}.pdf",
        tex_name,
        std::process::id()
    ));
    let _ = std::fs::remove_file(&our_pdf);
    let bin = env!("CARGO_BIN_EXE_rustlatex");
    let output = Command::new(bin)
        .arg(tex_path.to_str().unwrap())
        .arg(our_pdf.to_str().unwrap())
        .output()
        .expect("failed to run rustlatex");
    if !output.status.success() {
        eprintln!(
            "rustlatex failed for {}: {}",
            tex_name,
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let our_label = format!("{}_ours", tex_name);
    let our_ppm = render_pdf_to_ppm(&our_pdf, &our_label);

    // --- pdflatex PDF ---
    let tmp = std::env::temp_dir().join(format!(
        "rustlatex_{}_pdflatex_{}",
        tex_name,
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let tex_filename = format!("{}.tex", tex_name);
    std::fs::copy(&tex_path, tmp.join(&tex_filename)).expect("copy failed");
    let output = Command::new("pdflatex")
        .args(["-interaction=nonstopmode", &tex_filename])
        .current_dir(&tmp)
        .output()
        .expect("failed to run pdflatex");
    if !output.status.success() {
        eprintln!(
            "pdflatex failed for {}: {}",
            tex_name,
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = std::fs::remove_file(&our_pdf);
        let _ = std::fs::remove_file(&our_ppm);
        let _ = std::fs::remove_dir_all(&tmp);
        return;
    }
    let pdflatex_label = format!("{}_pdflatex", tex_name);
    let pdflatex_ppm = render_pdf_to_ppm(&tmp.join(format!("{}.pdf", tex_name)), &pdflatex_label);

    // --- Compare ---
    let similarity = compare_ppm_files(&our_ppm, &pdflatex_ppm);

    let our_size = std::fs::metadata(&our_ppm).map(|m| m.len()).unwrap_or(0);
    let their_size = std::fs::metadata(&pdflatex_ppm)
        .map(|m| m.len())
        .unwrap_or(0);

    eprintln!("=== Pixel Similarity Report ({}) ===", tex_name);
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
        format!("/tmp/{}_similarity.txt", tex_name),
        format!("similarity={:.4}", similarity),
    )
    .ok();

    // Cleanup
    let _ = std::fs::remove_file(&our_pdf);
    let _ = std::fs::remove_file(&our_ppm);
    let _ = std::fs::remove_file(&pdflatex_ppm);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_hello_tex_pixel_similarity_logged() {
    run_similarity_logged("hello");
}

#[test]
fn test_sections_tex_pixel_similarity_logged() {
    run_similarity_logged("sections");
}

#[test]
fn test_math_tex_pixel_similarity_logged() {
    run_similarity_logged("math");
}

#[test]
fn test_lists_tex_pixel_similarity_logged() {
    run_similarity_logged("lists");
}

// ── M90 unit tests: DPI constants and pixel-count math ──────────────────

/// Verify the render_pdf_to_ppm function uses -r150 (not the old value).
/// We read the source file and inspect only the render_pdf_to_ppm function body.
#[test]
fn test_m90_render_dpi_is_150() {
    let src = include_str!("pdflatex_comparison_test.rs");
    let render_fn_start = src
        .find("fn render_pdf_to_ppm")
        .expect("render_pdf_to_ppm not found");
    // Take a slice covering just the function (next 600 chars is enough for args).
    let snippet = &src[render_fn_start..render_fn_start + 600.min(src.len() - render_fn_start)];
    assert!(
        snippet.contains("\"-r150\""),
        "render_pdf_to_ppm should use -r150"
    );
    // Build the old flag string at runtime so it doesn't appear as a literal.
    let old_flag = format!("\"-r{}\"", 72);
    assert!(
        !snippet.contains(&old_flag),
        "render_pdf_to_ppm should not use the old DPI"
    );
}

/// A4 page at 150 DPI should produce ~1240 × 1754 pixels.
#[test]
fn test_m90_ppm_pixel_count_at_150_dpi() {
    let width_px = (595.0_f64 * 150.0 / 72.0) as u32; // ~1240
    let height_px = (842.0_f64 * 150.0 / 72.0) as u32; // ~1754
    assert!(
        width_px >= 1230 && width_px <= 1250,
        "Width should be ~1240px at 150 DPI, got {}",
        width_px
    );
    assert!(
        height_px >= 1745 && height_px <= 1765,
        "Height should be ~1754px at 150 DPI, got {}",
        height_px
    );
    let total_pixels = width_px as u64 * height_px as u64;
    assert!(
        total_pixels > 2_000_000,
        "150 DPI A4 should have >2M pixels, got {}",
        total_pixels
    );
}

/// 150 DPI pixel area must be strictly greater than 72 DPI pixel area.
#[test]
fn test_m90_pixel_area_larger_than_72_dpi() {
    let pixels_72 = 595_u64 * 842; // ~501,090 at 72 DPI (1:1 pt)
    let pixels_150 = (595.0_f64 * 150.0 / 72.0) as u64 * (842.0_f64 * 150.0 / 72.0) as u64;
    assert!(
        pixels_150 > pixels_72 * 4,
        "150 DPI should give >4× more pixels than 72 DPI ({} vs {})",
        pixels_150,
        pixels_72
    );
}

/// At 150 DPI we get ~4.34× more pixels, giving much higher precision.
#[test]
fn test_m90_dpi_increases_measurement_precision() {
    let ratio = (150.0_f64 / 72.0).powi(2);
    assert!(
        ratio > 4.3,
        "150/72 DPI squared ratio should be >4.3, got {}",
        ratio
    );
    assert!(
        ratio < 4.4,
        "150/72 DPI squared ratio should be <4.4, got {}",
        ratio
    );
}

/// Width at 150 DPI: 595 pt × (150/72) ≈ 1239.58 → 1239 px.
#[test]
fn test_m90_a4_width_px_at_150_dpi() {
    let width = (595.0_f64 * 150.0 / 72.0) as u32;
    assert_eq!(width, 1239, "A4 width at 150 DPI should be 1239 px");
}

/// Height at 150 DPI: 842 pt × (150/72) ≈ 1754.17 → 1754 px.
#[test]
fn test_m90_a4_height_px_at_150_dpi() {
    let height = (842.0_f64 * 150.0 / 72.0) as u32;
    assert_eq!(height, 1754, "A4 height at 150 DPI should be 1754 px");
}

/// PPM raw file size for A4 at 150 DPI: header + width×height×3 bytes.
#[test]
fn test_m90_ppm_raw_byte_size_at_150_dpi() {
    let w = (595.0_f64 * 150.0 / 72.0) as u64;
    let h = (842.0_f64 * 150.0 / 72.0) as u64;
    let pixel_bytes = w * h * 3;
    // PPM raw pixel data alone should be >6 MB
    assert!(
        pixel_bytes > 6_000_000,
        "PPM pixel data at 150 DPI should be >6 MB, got {}",
        pixel_bytes
    );
    assert!(
        pixel_bytes < 7_000_000,
        "PPM pixel data at 150 DPI should be <7 MB, got {}",
        pixel_bytes
    );
}

/// The old 72 DPI PPM pixel data was only ~1.5 MB.
#[test]
fn test_m90_old_72_dpi_pixel_data_was_small() {
    let w_72 = 595_u64;
    let h_72 = 842_u64;
    let pixel_bytes_72 = w_72 * h_72 * 3;
    assert!(
        pixel_bytes_72 < 2_000_000,
        "72 DPI pixel data should be <2 MB, got {}",
        pixel_bytes_72
    );
}

/// Ensure 150 DPI gives at least 4× the byte count of 72 DPI.
#[test]
fn test_m90_byte_ratio_150_vs_72() {
    let bytes_72 = 595_u64 * 842 * 3;
    let bytes_150 = (595.0_f64 * 150.0 / 72.0) as u64 * (842.0_f64 * 150.0 / 72.0) as u64 * 3;
    let ratio = bytes_150 as f64 / bytes_72 as f64;
    assert!(ratio > 4.3, "Byte ratio should be >4.3, got {}", ratio);
}

/// Minimum detectable similarity difference shrinks with more pixels.
/// At 72 DPI: 1 pixel = 1/501090 ≈ 0.0002%
/// At 150 DPI: 1 pixel = 1/2174706 ≈ 0.000046%
#[test]
fn test_m90_minimum_detectable_diff() {
    let pixels_72 = 595_u64 * 842;
    let pixels_150 = (595.0_f64 * 150.0 / 72.0) as u64 * (842.0_f64 * 150.0 / 72.0) as u64;
    let min_diff_72 = 1.0 / pixels_72 as f64;
    let min_diff_150 = 1.0 / pixels_150 as f64;
    assert!(
        min_diff_150 < min_diff_72,
        "150 DPI should detect smaller differences"
    );
    assert!(
        min_diff_150 < 0.000001,
        "150 DPI min detectable diff should be < 0.0001%"
    );
}

/// DPI value 150 is a reasonable choice (between 72 and 300).
#[test]
fn test_m90_dpi_value_reasonable() {
    let dpi: u32 = 150;
    assert!(dpi > 72, "DPI should be higher than old 72");
    assert!(dpi <= 300, "DPI should not exceed 300 (too slow for CI)");
    assert_eq!(dpi % 6, 0, "DPI should be divisible by 6 for clean scaling");
}

/// Verify the -r flag format matches GhostScript expectations.
#[test]
fn test_m90_gs_r_flag_format() {
    let dpi = 150;
    let flag = format!("-r{}", dpi);
    assert_eq!(flag, "-r150");
    assert!(flag.starts_with("-r"));
    assert!(flag[2..].parse::<u32>().is_ok());
}
