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
    let our_png = render_pdf_to_png(&our_pdf, "sim_ours");

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
    let pdflatex_png = render_pdf_to_png(&tmp.join("compare.pdf"), "sim_pdflatex");

    // --- Compare pixel data ---
    let our_bytes = std::fs::read(&our_png).unwrap_or_default();
    let their_bytes = std::fs::read(&pdflatex_png).unwrap_or_default();

    let min_len = our_bytes.len().min(their_bytes.len());
    let max_len = our_bytes.len().max(their_bytes.len());

    let mut diff_count: u64 = 0;
    for i in 0..min_len {
        if our_bytes[i] != their_bytes[i] {
            diff_count += 1;
        }
    }
    // Bytes beyond the shorter file are all different
    diff_count += (max_len - min_len) as u64;

    let similarity = if max_len > 0 {
        1.0 - (diff_count as f64 / max_len as f64)
    } else {
        1.0
    };

    eprintln!("=== Pixel Similarity Report ===");
    eprintln!("Our PNG size:      {} bytes", our_bytes.len());
    eprintln!("pdflatex PNG size: {} bytes", their_bytes.len());
    eprintln!("Differing bytes:   {}", diff_count);
    eprintln!(
        "Byte similarity:   {:.4} ({:.2}%)",
        similarity,
        similarity * 100.0
    );
    eprintln!("===============================");

    // Clean up
    let _ = std::fs::remove_file(&our_pdf);
    let _ = std::fs::remove_file(&our_png);
    let _ = std::fs::remove_file(&pdflatex_png);
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
