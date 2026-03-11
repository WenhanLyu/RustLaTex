//! GhostScript-based integration tests for the RustLaTex pipeline.
//!
//! These tests compile each example .tex file through the full pipeline,
//! write the PDF to a temp file, invoke `gs` to render it to a PNG image,
//! and verify the PNG is non-empty (>1000 bytes).
//!
//! Tests skip gracefully if the `gs` binary is not available.

use rustlatex_engine::Engine;
use rustlatex_parser::Parser;
use rustlatex_pdf::PdfWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

// ===== Helper Functions =====

/// Check if the GhostScript binary is available on this system.
fn gs_available() -> bool {
    let candidates = [
        "/opt/homebrew/bin/gs",
        "/usr/bin/gs",
        "/usr/local/bin/gs",
        "gs",
    ];
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return true;
        }
        // Also try running it via PATH
        if Command::new(candidate).arg("--version").output().is_ok() {
            return true;
        }
    }
    false
}

/// Find the GhostScript binary path.
fn gs_path() -> String {
    let candidates = ["/opt/homebrew/bin/gs", "/usr/bin/gs", "/usr/local/bin/gs"];
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    // Fall back to PATH lookup
    "gs".to_string()
}

/// Run the full pipeline on a LaTeX source string and return PDF bytes.
fn compile(source: &str) -> Vec<u8> {
    let mut parser = Parser::new(source);
    let ast = parser.parse();
    let engine = Engine::new(ast);
    let pages = engine.typeset();
    let writer = PdfWriter::new();
    let pdf = writer.write(&pages);
    pdf.bytes
}

/// Compile LaTeX source to PDF and write to a unique temp file. Returns the PDF path.
fn compile_to_tempfile(source: &str, test_name: &str) -> PathBuf {
    let pdf_bytes = compile(source);
    let temp_dir = std::env::temp_dir();
    let pdf_path = temp_dir.join(format!("rustlatex_gs_test_{}.pdf", test_name));
    std::fs::write(&pdf_path, &pdf_bytes).expect("failed to write PDF temp file");
    pdf_path
}

/// Invoke GhostScript to render a PDF to a PNG. Returns the PNG output path.
fn render_with_gs(pdf_path: &Path, test_name: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let png_path = temp_dir.join(format!("rustlatex_gs_test_{}.png", test_name));

    // Remove any existing output file
    let _ = std::fs::remove_file(&png_path);

    let gs = gs_path();
    let status = Command::new(&gs)
        .args([
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=pngalpha",
            "-r72",
            &format!("-sOutputFile={}", png_path.display()),
            pdf_path.to_str().expect("PDF path is not valid UTF-8"),
        ])
        .output()
        .expect("failed to invoke gs");

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        panic!(
            "gs failed for {}: exit={:?}\nstderr: {}",
            pdf_path.display(),
            status.status.code(),
            stderr
        );
    }

    png_path
}

/// Compile source, render with gs, verify PNG, and clean up.
fn run_gs_test(source: &str, test_name: &str) {
    if !gs_available() {
        eprintln!("Skipping {}: gs not available", test_name);
        return;
    }

    let pdf_path = compile_to_tempfile(source, test_name);
    let png_path = render_with_gs(&pdf_path, test_name);

    // Verify the PNG exists and is >1000 bytes
    assert!(
        png_path.exists(),
        "PNG output file does not exist: {}",
        png_path.display()
    );
    let png_size = std::fs::metadata(&png_path)
        .expect("failed to stat PNG file")
        .len();
    assert!(
        png_size > 1000,
        "PNG file is too small ({} bytes) for test {}; expected >1000 bytes",
        png_size,
        test_name
    );

    // Clean up temp files
    let _ = std::fs::remove_file(&pdf_path);
    let _ = std::fs::remove_file(&png_path);
}

// ===== Example File Tests =====

#[test]
fn test_gs_example_hello_tex() {
    // Read actual examples/hello.tex
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/hello.tex");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    run_gs_test(&source, "hello_tex");
}

#[test]
fn test_gs_example_lists_tex() {
    // Read actual examples/lists.tex
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/lists.tex");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    run_gs_test(&source, "lists_tex");
}

#[test]
fn test_gs_example_math_tex() {
    // Read actual examples/math.tex
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/math.tex");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    run_gs_test(&source, "math_tex");
}

#[test]
fn test_gs_example_sections_tex() {
    // Read actual examples/sections.tex
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/sections.tex");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    run_gs_test(&source, "sections_tex");
}

#[test]
fn test_gs_example_simple_tex() {
    // Read actual examples/simple.tex
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/simple.tex");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    run_gs_test(&source, "simple_tex");
}

// ===== Inline Source Tests =====

#[test]
fn test_gs_inline_hello_world() {
    let source = r"\documentclass{article}
\begin{document}
Hello, world!
\end{document}";
    run_gs_test(source, "inline_hello_world");
}

#[test]
fn test_gs_inline_math_display() {
    let source = r"\documentclass{article}
\begin{document}
The formula is:
\[E = mc^2\]
This is Einstein's famous equation.
\end{document}";
    run_gs_test(source, "inline_math_display");
}

#[test]
fn test_gs_inline_math_inline() {
    let source = r"\documentclass{article}
\begin{document}
The Pythagorean theorem states $a^2 + b^2 = c^2$.
\end{document}";
    run_gs_test(source, "inline_math_inline");
}

#[test]
fn test_gs_inline_sections() {
    let source = r"\documentclass{article}
\begin{document}
\section{Introduction}
An introduction section.
\section{Body}
The main body content.
\subsection{Details}
Some details here.
\section{Conclusion}
Wrapping up.
\end{document}";
    run_gs_test(source, "inline_sections");
}

#[test]
fn test_gs_inline_itemize_list() {
    let source = r"\documentclass{article}
\begin{document}
\begin{itemize}
  \item First item
  \item Second item
  \item Third item
\end{itemize}
\end{document}";
    run_gs_test(source, "inline_itemize_list");
}

#[test]
fn test_gs_inline_enumerate_list() {
    let source = r"\documentclass{article}
\begin{document}
\begin{enumerate}
  \item Step one
  \item Step two
  \item Step three
\end{enumerate}
\end{document}";
    run_gs_test(source, "inline_enumerate_list");
}

#[test]
fn test_gs_inline_mixed_content() {
    let source = r"\documentclass{article}
\begin{document}
\section{Math and Lists}
Here is some math: $a + b = c$.
\begin{itemize}
  \item Item one with $x = 1$
  \item Item two with $y = 2$
\end{itemize}
\end{document}";
    run_gs_test(source, "inline_mixed_content");
}

#[test]
fn test_gs_inline_multiple_paragraphs() {
    let source = r"\documentclass{article}
\begin{document}
First paragraph has some text in it.

Second paragraph follows after a blank line.

Third paragraph concludes the document.
\end{document}";
    run_gs_test(source, "inline_multiple_paragraphs");
}

#[test]
fn test_gs_inline_text_formatting() {
    let source = r"\documentclass{article}
\begin{document}
This has \textbf{bold text} and \textit{italic text} and \emph{emphasized text}.
\end{document}";
    run_gs_test(source, "inline_text_formatting");
}

#[test]
fn test_gs_inline_subsection_structure() {
    let source = r"\documentclass{article}
\begin{document}
\section{Main Section}
Main content here.
\subsection{Sub Section}
Sub content here.
\subsubsection{Sub Sub Section}
Deep content here.
\end{document}";
    run_gs_test(source, "inline_subsection_structure");
}

// ===== PDF Validity Test (without gs) =====

/// Verify that compile_to_tempfile produces a valid PDF even without gs.
#[test]
fn test_compile_to_tempfile_produces_valid_pdf() {
    let source = r"\documentclass{article}
\begin{document}
Test document for tempfile helper.
\end{document}";
    let pdf_path = compile_to_tempfile(source, "validity_check");
    assert!(pdf_path.exists(), "PDF temp file should exist");
    let bytes = std::fs::read(&pdf_path).expect("failed to read PDF temp file");
    assert!(!bytes.is_empty(), "PDF bytes must not be empty");
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF must start with %PDF- header"
    );
    let _ = std::fs::remove_file(&pdf_path);
}
