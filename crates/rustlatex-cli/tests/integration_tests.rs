//! Integration tests for the full RustLaTex pipeline.
//!
//! These tests call the full pipeline: Lexer → Parser → Engine → PdfWriter
//! and verify the output is valid PDF bytes.

use rustlatex_engine::Engine;
use rustlatex_parser::Parser;
use rustlatex_pdf::PdfWriter;

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

/// Assert that the bytes are valid PDF output.
fn assert_valid_pdf(bytes: &[u8]) {
    assert!(!bytes.is_empty(), "PDF bytes must not be empty");
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF must start with %PDF-, got: {:?}",
        &bytes[..bytes.len().min(10)]
    );
    // PDF ends with %%EOF (possibly followed by whitespace/newline)
    let tail = &bytes[bytes.len().saturating_sub(20)..];
    let tail_str = String::from_utf8_lossy(tail);
    assert!(
        tail_str.contains("%%EOF"),
        "PDF must end with %%EOF, tail was: {:?}",
        tail_str
    );
}

// ===== Basic Pipeline Tests =====

#[test]
fn test_pipeline_hello_world() {
    let source = r"\documentclass{article}
\begin{document}
Hello, world!
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_empty_document() {
    // Minimal document with no body text
    let source = r"\documentclass{article}
\begin{document}
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_pdf_byte_length_nonzero() {
    let source = r"\documentclass{article}
\begin{document}
Test document.
\end{document}";
    let bytes = compile(source);
    assert!(bytes.len() > 0, "PDF output must have non-zero byte length");
}

#[test]
fn test_pipeline_pdf_starts_with_pdf_magic() {
    let source = "Hello world";
    let bytes = compile(source);
    assert!(bytes.starts_with(b"%PDF-"), "PDF must start with %PDF-");
}

#[test]
fn test_pipeline_pdf_ends_with_eof() {
    let source = "Hello world";
    let bytes = compile(source);
    let tail = &bytes[bytes.len().saturating_sub(20)..];
    let tail_str = String::from_utf8_lossy(tail);
    assert!(tail_str.contains("%%EOF"), "PDF must end with %%EOF");
}

// ===== Content Tests =====

#[test]
fn test_pipeline_simple_text() {
    let source = "The quick brown fox jumps over the lazy dog.";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_section_headers() {
    let source = r"\documentclass{article}
\begin{document}
\section{Introduction}
This is the introduction.
\section{Conclusion}
This is the conclusion.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_math_mode_inline() {
    let source = r"\documentclass{article}
\begin{document}
The formula $x^2 + y^2 = z^2$ is Pythagorean.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_math_mode_display() {
    let source = r"\documentclass{article}
\begin{document}
The formula is:
\[E = mc^2\]
This is Einstein's famous equation.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_itemize_list() {
    let source = r"\documentclass{article}
\begin{document}
\begin{itemize}
  \item First item
  \item Second item
  \item Third item
\end{itemize}
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_multiple_paragraphs() {
    let source = r"\documentclass{article}
\begin{document}
First paragraph has some text in it.

Second paragraph follows after a blank line.

Third paragraph concludes the document.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_text_formatting() {
    let source = r"\documentclass{article}
\begin{document}
This has \textbf{bold text} and \textit{italic text} and \emph{emphasized text}.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_mixed_content() {
    let source = r"\documentclass{article}
\begin{document}
\section{Math and Lists}
Here is some math: $a + b = c$.
\begin{itemize}
  \item Item one with $x = 1$
  \item Item two with $y = 2$
\end{itemize}
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_only_whitespace_is_valid_pdf() {
    // Even near-empty input should produce valid PDF
    let source = "   ";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_tex_file_simple() {
    // Matches examples/simple.tex
    let source = r"\documentclass{article}
\begin{document}
This is a simple article document.
It contains multiple sentences to test paragraph handling.
The pipeline should compile this without errors.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_tex_file_math() {
    // Matches examples/math.tex
    let source = r"\documentclass{article}
\begin{document}
Mathematics is beautiful.
The inline formula $x^2 + y^2 = z^2$ is Pythagoras.
Display math follows:
\[E = mc^2\]
Another inline formula: $\alpha + \beta = \gamma$.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_tex_file_sections() {
    // Matches examples/sections.tex
    let source = r"\documentclass{article}
\begin{document}
\section{Introduction}
This document tests section handling in the pipeline.
\section{Methods}
We describe our methods here.
\subsection{Details}
Some subsection content goes here.
\section{Conclusion}
The pipeline compiles correctly.
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_tex_file_lists() {
    // Matches examples/lists.tex
    let source = r"\documentclass{article}
\begin{document}
\section{Shopping List}
Here is an itemized list:
\begin{itemize}
  \item First item in the list
  \item Second item with more text
  \item Third item
\end{itemize}
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

// ===== Error Handling Tests =====

#[test]
fn test_pipeline_command_only_no_panic() {
    // Input with only commands and no real text — should not panic
    let source = r"\documentclass{article}\usepackage{amsmath}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}

#[test]
fn test_pipeline_deeply_nested_groups_no_panic() {
    // Deeply nested groups should not cause a stack overflow or panic
    let source = r"\documentclass{article}
\begin{document}
{{{{nested}}}}
\end{document}";
    let bytes = compile(source);
    assert_valid_pdf(&bytes);
}
