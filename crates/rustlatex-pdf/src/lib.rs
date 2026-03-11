//! `rustlatex-pdf` — PDF backend
//!
//! This crate takes the typeset pages produced by `rustlatex-engine` and
//! emits PDF output. It produces valid PDF files using the `pdf-writer` crate
//! with Base-14 Helvetica font on A4 pages.

use pdf_writer::{Content, Name, Pdf, Rect, Ref, Str};
use rustlatex_engine::{BoxNode, Page as EnginePage};

/// PDF generation result.
#[derive(Debug)]
pub struct PdfOutput {
    /// The raw PDF bytes.
    pub bytes: Vec<u8>,
}

/// The PDF writer converts typeset pages into PDF bytes.
#[derive(Default)]
pub struct PdfWriter;

/// Escape a string for use as a PDF string literal.
/// Escapes backslashes and parentheses.
fn pdf_escape(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'\\' => {
                out.push(b'\\');
                out.push(b'\\');
            }
            b'(' => {
                out.push(b'\\');
                out.push(b'(');
            }
            b')' => {
                out.push(b'\\');
                out.push(b')');
            }
            _ => out.push(b),
        }
    }
    out
}

impl PdfWriter {
    /// Create a new PDF writer.
    pub fn new() -> Self {
        PdfWriter
    }

    /// Write pages to PDF.
    ///
    /// Produces a valid PDF document with:
    /// - Document catalog and page tree
    /// - A4 page size (595 × 842 pt), 50pt margins
    /// - Base-14 Helvetica font at 10pt
    /// - Text rendered from box_lines, top-to-bottom
    pub fn write(&self, pages: &[EnginePage]) -> PdfOutput {
        // If no pages, produce a valid PDF with zero pages
        let page_count = if pages.is_empty() { 0 } else { pages.len() };

        // Allocate Ref IDs:
        // 1 = catalog
        // 2 = page tree
        // 3 = font (Helvetica Type1)
        // For each page i (0-indexed):
        //   4 + i*2     = page object
        //   4 + i*2 + 1 = content stream
        let catalog_id = Ref::new(1);
        let page_tree_id = Ref::new(2);
        let font_id = Ref::new(3);

        let mut pdf = Pdf::new();

        // Document catalog
        pdf.catalog(catalog_id).pages(page_tree_id);

        // Collect page Refs
        let page_refs: Vec<Ref> = (0..page_count)
            .map(|i| Ref::new((4 + i * 2) as i32))
            .collect();

        // Page tree
        pdf.pages(page_tree_id)
            .kids(page_refs.iter().copied())
            .count(page_count as i32);

        // Font: Helvetica (Base-14, no embedding needed)
        pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

        // A4 dimensions
        let media_box = Rect::new(0.0, 0.0, 595.0, 842.0);

        // Margins
        let margin_left: f32 = 50.0;
        let margin_top: f32 = 50.0;
        let font_size: f32 = 10.0;
        let line_height: f32 = 14.0;

        // Starting y position: page height - top margin = 842 - 50 = 792
        let start_y: f32 = 842.0 - margin_top;

        for (i, page) in pages.iter().enumerate() {
            let page_id = Ref::new((4 + i * 2) as i32);
            let content_id = Ref::new((4 + i * 2 + 1) as i32);

            // Build content stream
            let mut content = Content::new();
            content.begin_text();
            content.set_font(Name(b"F1"), font_size);

            let mut current_y = start_y;

            for line in &page.box_lines {
                let mut current_x = margin_left;

                // Position at the start of this line
                content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);

                for node in line {
                    match node {
                        BoxNode::Text { text, width } => {
                            // Move to current_x position using Td relative positioning
                            // We already set the text matrix, so use show
                            let escaped = pdf_escape(text);
                            content.show(Str(&escaped));
                            current_x += *width as f32;
                        }
                        BoxNode::Glue { natural, .. } => {
                            // Advance x by natural width - emit a space
                            // Use text matrix repositioning for the next text node
                            current_x += *natural as f32;
                            // Re-position for the next text element
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        BoxNode::Kern { amount } => {
                            current_x += *amount as f32;
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        _ => {
                            // HBox, VBox, Penalty — skip for now
                        }
                    }
                }

                current_y -= line_height;
            }

            content.end_text();
            let content_bytes = content.finish();

            // Write content stream
            pdf.stream(content_id, &content_bytes);

            // Write page object
            {
                let mut page_writer = pdf.page(page_id);
                page_writer.parent(page_tree_id);
                page_writer.media_box(media_box);
                page_writer.contents(content_id);
                let mut resources = page_writer.resources();
                resources.fonts().pair(Name(b"F1"), font_id);
            }
        }

        let bytes = pdf.finish();
        PdfOutput { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_engine::{BoxNode, Page as EnginePage};

    #[test]
    fn test_pdf_header_starts_with_pdf() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(output.bytes.len() >= 4);
        assert_eq!(&output.bytes[0..5], b"%PDF-");
    }

    #[test]
    fn test_pdf_non_empty_output() {
        let pages = vec![EnginePage {
            number: 1,
            content: "Hello world".to_string(),
            box_lines: vec![vec![
                BoxNode::Text {
                    text: "Hello".to_string(),
                    width: 25.0,
                },
                BoxNode::Glue {
                    natural: 3.33,
                    stretch: 1.67,
                    shrink: 1.11,
                },
                BoxNode::Text {
                    text: "world".to_string(),
                    width: 24.76,
                },
            ]],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
        // Should be a reasonably sized PDF (at least a few hundred bytes)
        assert!(output.bytes.len() > 100);
    }

    #[test]
    fn test_pdf_single_page() {
        let pages = vec![EnginePage {
            number: 1,
            content: "page one".to_string(),
            box_lines: vec![vec![BoxNode::Text {
                text: "Hello".to_string(),
                width: 25.0,
            }]],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Verify it's valid PDF structure
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(text.contains("%PDF-"));
        // Should contain /Count 1 for one page
        assert!(text.contains("/Count 1"));
    }

    #[test]
    fn test_pdf_two_pages() {
        let pages = vec![
            EnginePage {
                number: 1,
                content: "page one".to_string(),
                box_lines: vec![vec![BoxNode::Text {
                    text: "First".to_string(),
                    width: 20.0,
                }]],
            },
            EnginePage {
                number: 2,
                content: "page two".to_string(),
                box_lines: vec![vec![BoxNode::Text {
                    text: "Second".to_string(),
                    width: 30.0,
                }]],
            },
        ];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
        let text = String::from_utf8_lossy(&output.bytes);
        // Should contain /Count 2 for two pages
        assert!(text.contains("/Count 2"));
    }

    #[test]
    fn test_pdf_empty_pages_slice() {
        let pages: Vec<EnginePage> = vec![];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Should not panic and produce a valid PDF
        assert!(!output.bytes.is_empty());
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(text.contains("%PDF-"));
        assert!(text.contains("/Count 0"));
    }

    #[test]
    fn test_pdf_escape_parentheses() {
        let escaped = pdf_escape("hello (world)");
        assert_eq!(escaped, b"hello \\(world\\)");
    }

    #[test]
    fn test_pdf_escape_backslash() {
        let escaped = pdf_escape("path\\to\\file");
        assert_eq!(escaped, b"path\\\\to\\\\file");
    }

    #[test]
    fn test_pdf_contains_helvetica() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![vec![BoxNode::Text {
                text: "test".to_string(),
                width: 20.0,
            }]],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(
            text.contains("Helvetica"),
            "PDF should reference Helvetica font"
        );
    }
}
