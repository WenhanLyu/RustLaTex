//! CLI error handling integration tests.
//!
//! These tests verify that the `rustlatex` CLI binary exits with non-zero
//! status and prints a descriptive error message for invalid inputs.

use std::io::Write;
use std::process::Command;

/// Returns the path to the compiled rustlatex binary.
fn binary_path() -> std::path::PathBuf {
    // The binary is built to target/debug/rustlatex
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/rustlatex");
    path
}

/// Run the CLI binary with the given args, return (exit_code, stdout, stderr)
fn run_cli(args: &[&str]) -> (i32, String, String) {
    let bin = binary_path();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .expect("failed to execute rustlatex binary");
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (exit_code, stdout, stderr)
}

/// Write content to a temp file, return the path.
fn write_temp_file(name: &str, content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).expect("failed to create temp file");
    f.write_all(content.as_bytes())
        .expect("failed to write temp file");
    path
}

// ===== Error Cases =====

#[test]
fn test_cli_no_args_exits_nonzero() {
    let (exit_code, _stdout, stderr) = run_cli(&[]);
    assert_ne!(
        exit_code, 0,
        "CLI should exit with non-zero when no args given"
    );
    assert!(
        stderr.contains("Error") || stderr.contains("Usage"),
        "stderr should describe the error, got: {}",
        stderr
    );
}

#[test]
fn test_cli_missing_file_exits_nonzero() {
    let (exit_code, _stdout, stderr) = run_cli(&["/nonexistent/path/to/file.tex"]);
    assert_ne!(
        exit_code, 0,
        "CLI should exit with non-zero for missing file"
    );
    assert!(
        stderr.contains("Error") || stderr.contains("cannot read"),
        "stderr should describe the error, got: {}",
        stderr
    );
}

#[test]
fn test_cli_empty_file_exits_nonzero() {
    let path = write_temp_file("cli_test_empty.tex", "");
    let path_str = path.to_str().unwrap();
    let (exit_code, _stdout, stderr) = run_cli(&[path_str]);
    assert_ne!(exit_code, 0, "CLI should exit with non-zero for empty file");
    assert!(
        stderr.contains("Error") || stderr.contains("empty"),
        "stderr should describe the error, got: {}",
        stderr
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_cli_whitespace_only_file_exits_nonzero() {
    let path = write_temp_file("cli_test_whitespace.tex", "   \n\t\n   ");
    let path_str = path.to_str().unwrap();
    let (exit_code, _stdout, stderr) = run_cli(&[path_str]);
    assert_ne!(
        exit_code, 0,
        "CLI should exit with non-zero for whitespace-only file"
    );
    assert!(
        stderr.contains("Error") || stderr.contains("empty"),
        "stderr should describe the error, got: {}",
        stderr
    );
    let _ = std::fs::remove_file(&path);
}

// ===== Success Cases (to ensure we haven't broken good paths) =====

#[test]
fn test_cli_valid_file_exits_zero() {
    let content = r"\documentclass{article}
\begin{document}
Hello world.
\end{document}";
    let path = write_temp_file("cli_test_valid.tex", content);
    let path_str = path.to_str().unwrap();
    let (exit_code, _stdout, _stderr) = run_cli(&[path_str]);
    assert_eq!(exit_code, 0, "CLI should exit with 0 for a valid file");
    // Clean up generated pdf
    let pdf_path = std::env::temp_dir().join("cli_test_valid.pdf");
    let _ = std::fs::remove_file(&pdf_path);
    let _ = std::fs::remove_file(&path);
}
