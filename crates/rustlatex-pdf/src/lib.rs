//! `rustlatex-pdf` — PDF backend
//!
//! This crate takes the typeset pages produced by `rustlatex-engine` and
//! emits PDF output. The goal is to produce PDF output that is semantically
//! equivalent to pdflatex output, and eventually binary-identical.
//!
//! Currently a stub — future milestones will implement actual PDF generation.

use rustlatex_engine::Page;

/// PDF generation result.
#[derive(Debug)]
pub struct PdfOutput {
    /// The raw PDF bytes.
    pub bytes: Vec<u8>,
}

/// The PDF writer converts typeset pages into PDF bytes.
#[derive(Default)]
pub struct PdfWriter;

impl PdfWriter {
    /// Create a new PDF writer.
    pub fn new() -> Self {
        PdfWriter
    }

    /// Write pages to PDF.
    ///
    /// This is currently a stub that returns a minimal placeholder.
    pub fn write(&self, pages: &[Page]) -> PdfOutput {
        // Stub: return a comment describing the pages
        let content = format!(
            "%% RustLaTex PDF stub — {} page(s)\n",
            pages.len()
        );
        PdfOutput {
            bytes: content.into_bytes(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_engine::Page;

    #[test]
    fn test_pdf_writer_stub() {
        let pages = vec![Page { number: 1, content: "test".to_string() }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
        let text = String::from_utf8(output.bytes).unwrap();
        assert!(text.contains("RustLaTex"));
    }
}
